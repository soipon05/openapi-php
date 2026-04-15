//! Resolver: transforms a `RawOpenApi` tree into a fully resolved `ResolvedSpec` IR.
//!
//! Handles `$ref` resolution, `allOf` merging, `anyOf`/`oneOf` unions, and
//! parameter inheritance (path-level + operation-level, with operation winning).
//! Circular `$ref` chains are detected and reported as `ResolveError::CircularRef`.

use std::collections::HashSet;
use std::sync::Arc;

use anyhow::Result;
use indexmap::IndexMap;

use crate::ir::{
    ArraySchema, EnumBackingType, EnumSchema, EnumVariant, HttpMethod, ObjectSchema, PhpPrimitive,
    PrimitiveSchema, ResolvedEndpoint, ResolvedErrorResponse, ResolvedParam, ResolvedProperty,
    ResolvedRequestBody, ResolvedSchema, ResolvedSecurityScheme, ResolvedSpec, SecuritySchemeType,
    UnionSchema,
};
use crate::parser::error::ResolveError;
use crate::parser::raw::types::{
    EnumValue, OpenApi as RawOpenApi, Operation, Parameter, ParameterLocation, RawOrRef,
    RequestBody, Response, Schema, SchemaType,
};
use crate::php_utils::{escape_reserved, to_camel_case, to_pascal_case};

// ---------------------------------------------------------------------------
// Public entry-point
// ---------------------------------------------------------------------------

pub fn resolve(raw: &RawOpenApi) -> Result<ResolvedSpec> {
    let mut resolver = Resolver::new(raw);

    // Resolve every named schema in components first
    let schema_names: Vec<String> = raw
        .components
        .as_ref()
        .map(|c| c.schemas.keys().cloned().collect())
        .unwrap_or_default();

    for name in &schema_names {
        if !resolver.resolved.contains_key(name.as_str()) {
            let resolved = resolver.resolve_named_schema(name)?;
            resolver.resolved.insert(name.clone(), resolved);
        }
    }

    // Build endpoint list
    let mut endpoints = Vec::new();
    for (path, item) in &raw.paths {
        let ops: Vec<(HttpMethod, &Operation)> = [
            item.get.as_ref().map(|o| (HttpMethod::Get, o)),
            item.post.as_ref().map(|o| (HttpMethod::Post, o)),
            item.put.as_ref().map(|o| (HttpMethod::Put, o)),
            item.patch.as_ref().map(|o| (HttpMethod::Patch, o)),
            item.delete.as_ref().map(|o| (HttpMethod::Delete, o)),
            item.head.as_ref().map(|o| (HttpMethod::Head, o)),
            item.options.as_ref().map(|o| (HttpMethod::Options, o)),
        ]
        .into_iter()
        .flatten()
        .collect();

        let path_level = item.parameters.clone();
        for (method, op) in ops {
            let ep = resolver.resolve_endpoint(path, method, op, &path_level)?;
            endpoints.push(ep);
        }
    }

    let schemas = resolver.resolved;
    let base_url = raw
        .servers
        .first()
        .map(|s| s.url.clone())
        .unwrap_or_default();

    let security_schemes = raw
        .components
        .as_ref()
        .map(|c| {
            c.security_schemes
                .iter()
                .filter_map(|(scheme_name, raw_scheme)| {
                    let scheme_type = match raw_scheme.scheme_type.as_str() {
                        "http" => {
                            let s = raw_scheme.scheme.as_deref().unwrap_or("").to_lowercase();
                            Some(SecuritySchemeType::Http { scheme: s })
                        }
                        "apiKey" => {
                            let in_ = raw_scheme
                                .location
                                .as_deref()
                                .unwrap_or("header")
                                .to_string();
                            let key_name = raw_scheme.name.as_deref().unwrap_or("").to_string();
                            Some(SecuritySchemeType::ApiKey {
                                in_,
                                name: key_name,
                            })
                        }
                        _ => None,
                    }?;
                    Some(ResolvedSecurityScheme {
                        name: scheme_name.clone(),
                        scheme_type,
                    })
                })
                .collect()
        })
        .unwrap_or_default();

    Ok(ResolvedSpec {
        title: raw.info.title.clone(),
        version: raw.info.version.clone(),
        base_url,
        schemas,
        endpoints,
        security_schemes,
    })
}

// ---------------------------------------------------------------------------
// Resolver context
// ---------------------------------------------------------------------------

struct Resolver<'a> {
    raw: &'a RawOpenApi,
    resolved: IndexMap<String, ResolvedSchema>,
    in_progress: HashSet<String>,
}

impl<'a> Resolver<'a> {
    fn new(raw: &'a RawOpenApi) -> Self {
        Self {
            raw,
            resolved: IndexMap::new(),
            in_progress: HashSet::new(),
        }
    }

    // -----------------------------------------------------------------------
    // Schema resolution
    // -----------------------------------------------------------------------

    fn resolve_named_schema(&mut self, name: &str) -> Result<ResolvedSchema> {
        // Synthesise a canonical ref path for error messages when called without one.
        let synthetic = format!("#/components/schemas/{name}");
        self.resolve_named_schema_for_ref(name, &synthetic)
    }

    /// Core resolver: `name` is the bare schema name, `ref_path` is the full
    /// `$ref` string used in error messages.
    fn resolve_named_schema_for_ref(
        &mut self,
        name: &str,
        ref_path: &str,
    ) -> Result<ResolvedSchema> {
        if let Some(cached) = self.resolved.get(name) {
            return Ok(cached.clone());
        }
        if self.in_progress.contains(name) {
            return Err(ResolveError::CircularRef {
                cycle: ref_path.to_string(),
            }
            .into());
        }

        // Clone to release the immutable borrow on self before we mutate self
        let raw_ror = self
            .raw
            .components
            .as_ref()
            .and_then(|c| c.schemas.get(name))
            .ok_or_else(|| ResolveError::UnknownRef {
                ref_path: ref_path.to_string(),
                name: name.to_string(),
            })?
            .clone();

        self.in_progress.insert(name.to_string());

        let resolved = match raw_ror {
            RawOrRef::Ref {
                ref_path: inner_ref,
            } => {
                if !inner_ref.starts_with("#/components/schemas/") {
                    return Err(ResolveError::InvalidRefFormat {
                        ref_path: inner_ref,
                    }
                    .into());
                }
                let target = ref_name(&inner_ref).to_string();
                self.resolve_named_schema_for_ref(&target, &inner_ref)?
            }
            RawOrRef::Value(schema) => self.resolve_schema(&schema)?,
        };

        self.in_progress.remove(name);
        self.resolved.insert(name.to_string(), resolved.clone());
        Ok(resolved)
    }

    fn resolve_schema_or_ref(&mut self, ror: &RawOrRef<Schema>) -> Result<ResolvedSchema> {
        match ror {
            RawOrRef::Ref { ref_path } => {
                if !ref_path.starts_with("#/components/schemas/") {
                    return Err(ResolveError::InvalidRefFormat {
                        ref_path: ref_path.clone(),
                    }
                    .into());
                }
                let name = ref_name(ref_path).to_string();
                self.resolve_named_schema_for_ref(&name, ref_path)
            }
            RawOrRef::Value(schema) => {
                let schema = schema.clone(); // release borrow before mutable self
                self.resolve_schema(&schema)
            }
        }
    }

    fn resolve_schema(&mut self, schema: &Schema) -> Result<ResolvedSchema> {
        // Enum
        if !schema.enum_values.is_empty() {
            return Ok(ResolvedSchema::Enum(build_enum(schema)));
        }

        // allOf → merge into single ObjectSchema
        if !schema.all_of.is_empty() {
            return self.resolve_all_of(schema);
        }

        // anyOf / oneOf → union
        if !schema.any_of.is_empty() || !schema.one_of.is_empty() {
            return self.resolve_union(schema);
        }

        let is_object = schema
            .schema_type
            .as_ref()
            .map(|t| t.is(&SchemaType::Object))
            .unwrap_or(false)
            || (!schema.properties.is_empty() && schema.schema_type.is_none());

        if is_object {
            return self.resolve_object(schema);
        }

        if schema
            .schema_type
            .as_ref()
            .map(|t| t.is(&SchemaType::Array))
            .unwrap_or(false)
        {
            return self.resolve_array(schema);
        }

        Ok(ResolvedSchema::Primitive(build_primitive(schema)))
    }

    fn resolve_object(&mut self, schema: &Schema) -> Result<ResolvedSchema> {
        let required = schema.required.clone();
        let props: Vec<(String, RawOrRef<Schema>)> =
            schema.properties.clone().into_iter().collect();
        let mut properties = IndexMap::new();
        for (name, prop_ror) in props {
            let (nullable, description, deprecated) = if let RawOrRef::Value(s) = &prop_ror {
                (
                    is_nullable(s),
                    s.description.clone(),
                    s.deprecated.unwrap_or(false),
                )
            } else {
                (false, None, false)
            };
            let is_required = required.contains(&name);
            let prop_schema = match &prop_ror {
                RawOrRef::Ref { ref_path } => self.resolve_ref_or_inline(ref_path)?,
                _ => self.resolve_schema_or_ref(&prop_ror)?,
            };
            properties.insert(
                name,
                ResolvedProperty {
                    schema: prop_schema,
                    required: is_required,
                    nullable,
                    description,
                    deprecated,
                },
            );
        }
        Ok(ResolvedSchema::Object(ObjectSchema {
            description: schema.description.clone(),
            properties,
        }))
    }

    fn resolve_array(&mut self, schema: &Schema) -> Result<ResolvedSchema> {
        let items = if let Some(items_ror) = &schema.items {
            let ror = *items_ror.clone(); // Box<RawOrRef<Schema>> → RawOrRef<Schema>
            match ror {
                RawOrRef::Ref { ref_path } => self.resolve_ref_or_inline(&ref_path)?,
                _ => self.resolve_schema_or_ref(&ror)?,
            }
        } else {
            ResolvedSchema::Primitive(PrimitiveSchema {
                php_type: PhpPrimitive::Mixed,
                format: None,
                description: None,
                nullable: false,
                example: None,
                min_length: None,
                max_length: None,
                minimum: None,
                maximum: None,
                exclusive_minimum: None,
                exclusive_maximum: None,
                pattern: None,
            })
        };
        Ok(ResolvedSchema::Array(ArraySchema {
            items: Box::new(items),
            description: schema.description.clone(),
            nullable: is_nullable(schema),
        }))
    }

    fn resolve_all_of(&mut self, schema: &Schema) -> Result<ResolvedSchema> {
        let all_of = schema.all_of.clone();
        let mut merged: IndexMap<String, ResolvedProperty> = IndexMap::new();
        let mut description = schema.description.clone();

        for ror in &all_of {
            let resolved = self.resolve_schema_or_ref(ror)?;
            if let ResolvedSchema::Object(obj) = resolved {
                if description.is_none() {
                    description = obj.description;
                }
                merged.extend(obj.properties);
            }
        }

        // Own properties override merged ones
        let required = schema.required.clone();
        let own_props: Vec<(String, RawOrRef<Schema>)> =
            schema.properties.clone().into_iter().collect();
        for (name, prop_ror) in own_props {
            let (nullable, prop_desc, deprecated) = if let RawOrRef::Value(s) = &prop_ror {
                (
                    is_nullable(s),
                    s.description.clone(),
                    s.deprecated.unwrap_or(false),
                )
            } else {
                (false, None, false)
            };
            let is_required = required.contains(&name);
            let prop_schema = match &prop_ror {
                RawOrRef::Ref { ref_path } => self.resolve_ref_or_inline(ref_path)?,
                _ => self.resolve_schema_or_ref(&prop_ror)?,
            };
            merged.insert(
                name,
                ResolvedProperty {
                    schema: prop_schema,
                    required: is_required,
                    nullable,
                    description: prop_desc,
                    deprecated,
                },
            );
        }

        Ok(ResolvedSchema::Object(ObjectSchema {
            description,
            properties: merged,
        }))
    }

    fn resolve_union(&mut self, schema: &Schema) -> Result<ResolvedSchema> {
        let variants_raw = if !schema.any_of.is_empty() {
            schema.any_of.clone()
        } else {
            schema.one_of.clone()
        };

        let mut variants = Vec::new();
        for ror in &variants_raw {
            // Preserve $ref as Ref(name) unless the target is a primitive (inline it).
            let resolved = match ror {
                RawOrRef::Ref { ref_path } => self.resolve_ref_or_inline(ref_path)?,
                _ => self.resolve_schema_or_ref(ror)?,
            };
            variants.push(resolved);
        }

        let discriminator = schema
            .discriminator
            .as_ref()
            .map(|d| d.property_name.clone());

        // mapping: discriminator value → schema name (strip $ref prefix if present)
        let discriminator_mapping = schema
            .discriminator
            .as_ref()
            .map(|d| {
                d.mapping
                    .iter()
                    .map(|(k, v)| (k.clone(), ref_name(v).to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ResolvedSchema::Union(UnionSchema {
            variants,
            discriminator,
            discriminator_mapping,
            description: schema.description.clone(),
        }))
    }

    fn resolve_ref_or_inline(&mut self, ref_path: &str) -> Result<ResolvedSchema> {
        if !ref_path.starts_with("#/components/schemas/") {
            return Err(ResolveError::InvalidRefFormat {
                ref_path: ref_path.to_string(),
            }
            .into());
        }
        let name = ref_name(ref_path);
        match self.resolve_named_schema_for_ref(name, ref_path)? {
            p @ ResolvedSchema::Primitive(_) => Ok(p),
            _ => Ok(ResolvedSchema::Ref(Arc::from(name))),
        }
    }

    // -----------------------------------------------------------------------
    // Endpoint resolution
    // -----------------------------------------------------------------------

    fn resolve_endpoint(
        &mut self,
        path: &str,
        method: HttpMethod,
        op: &Operation,
        path_level_params: &[RawOrRef<Parameter>],
    ) -> Result<ResolvedEndpoint> {
        let operation_id = op
            .operation_id
            .clone()
            .unwrap_or_else(|| derive_operation_id(&method, path));

        // Merge path-level then operation-level params (op overrides same name)
        let mut param_map: IndexMap<String, RawOrRef<Parameter>> = IndexMap::new();
        for ror in path_level_params {
            let key = self.param_name(ror)?;
            param_map.insert(key, ror.clone());
        }
        for ror in &op.parameters {
            let key = self.param_name(ror)?;
            param_map.insert(key, ror.clone());
        }

        let mut path_params = Vec::new();
        let mut query_params = Vec::new();
        let mut header_params = Vec::new();
        for ror in param_map.into_values() {
            let raw = self.get_raw_param(&ror)?;
            let location = raw.location.clone();
            let resolved = self.build_resolved_param(raw)?;
            match location {
                ParameterLocation::Path => path_params.push(resolved),
                ParameterLocation::Query => query_params.push(resolved),
                ParameterLocation::Header => header_params.push(resolved),
                _ => {} // cookie ignored for now
            }
        }

        let request_body = if let Some(rb_ror) = &op.request_body {
            let rb_ror = rb_ror.clone();
            Some(self.resolve_request_body(&rb_ror)?)
        } else {
            None
        };

        let responses = op.responses.clone();
        let response = self.resolve_success_response(&responses)?;
        let error_responses = self.resolve_error_responses(&responses)?;

        Ok(ResolvedEndpoint {
            operation_id,
            method,
            path: path.to_string(),
            summary: op.summary.clone(),
            tags: op.tags.clone(),
            path_params,
            query_params,
            header_params,
            request_body,
            response,
            deprecated: op.deprecated.unwrap_or(false),
            error_responses,
        })
    }

    fn param_name(&self, ror: &RawOrRef<Parameter>) -> Result<String> {
        match ror {
            RawOrRef::Value(p) => Ok(p.name.clone()),
            RawOrRef::Ref { ref_path } => {
                let name = ref_name(ref_path);
                let p = self
                    .raw
                    .components
                    .as_ref()
                    .and_then(|c| c.parameters.get(name))
                    .and_then(|r| {
                        if let RawOrRef::Value(p) = r {
                            Some(p)
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow::anyhow!("Parameter '{}' not found", name))?;
                Ok(p.name.clone())
            }
        }
    }

    fn get_raw_param(&self, ror: &RawOrRef<Parameter>) -> Result<Parameter> {
        match ror {
            RawOrRef::Value(p) => Ok(p.clone()),
            RawOrRef::Ref { ref_path } => {
                let name = ref_name(ref_path);
                self.raw
                    .components
                    .as_ref()
                    .and_then(|c| c.parameters.get(name))
                    .and_then(|r| {
                        if let RawOrRef::Value(p) = r {
                            Some(p.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow::anyhow!("Parameter '{}' not found", name))
            }
        }
    }

    fn build_resolved_param(&mut self, param: Parameter) -> Result<ResolvedParam> {
        let required = param
            .required
            .unwrap_or(param.location == ParameterLocation::Path);
        let php_name = escape_reserved(&to_camel_case(&param.name));

        let schema = if let Some(ror) = param.schema {
            self.resolve_schema_or_ref(&ror)?
        } else {
            mixed()
        };

        Ok(ResolvedParam {
            name: param.name,
            php_name,
            schema,
            required,
        })
    }

    fn resolve_request_body(&mut self, ror: &RawOrRef<RequestBody>) -> Result<ResolvedRequestBody> {
        let rb = match ror {
            RawOrRef::Value(r) => r.clone(),
            RawOrRef::Ref { ref_path } => {
                let name = ref_name(ref_path);
                self.raw
                    .components
                    .as_ref()
                    .and_then(|c| c.request_bodies.get(name))
                    .and_then(|r| {
                        if let RawOrRef::Value(b) = r {
                            Some(b.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow::anyhow!("RequestBody '{}' not found", name))?
            }
        };

        const PRIORITY: &[&str] = &[
            "application/json",
            "multipart/form-data",
            "application/octet-stream",
            "text/plain",
        ];

        let content_type = PRIORITY
            .iter()
            .find(|&&ct| rb.content.contains_key(ct))
            .map(|&ct| ct.to_string())
            .unwrap_or_else(|| {
                rb.content
                    .keys()
                    .next()
                    .cloned()
                    .unwrap_or_else(|| "application/json".to_string())
            });

        let schema_ror = rb
            .content
            .get(&content_type)
            .and_then(|m| m.schema.as_ref())
            .cloned();

        let schema = if let Some(ror) = schema_ror {
            // Preserve the schema name for Object/Enum/Union refs so generators can
            // derive `XxxRequest` class names. Primitive refs (e.g. `$ref: Uuid`) are
            // inlined to their native PHP type — consistent with property resolution.
            match ror {
                RawOrRef::Ref { ref_path } => self.resolve_ref_or_inline(&ref_path)?,
                _ => self.resolve_schema_or_ref(&ror)?,
            }
        } else {
            mixed()
        };

        Ok(ResolvedRequestBody {
            schema,
            required: rb.required.unwrap_or(false),
            content_type,
        })
    }

    fn resolve_error_responses(
        &mut self,
        responses: &IndexMap<String, RawOrRef<Response>>,
    ) -> Result<Vec<ResolvedErrorResponse>> {
        let success_keys: &[&str] = &["200", "201", "2xx"];
        let mut result = Vec::new();

        for (key, ror) in responses {
            // Skip success keys
            if success_keys.contains(&key.as_str()) {
                continue;
            }
            // Only process numeric status codes in the 4xx/5xx range
            let status_code = match key.parse::<u16>() {
                Ok(code) if code >= 400 => code,
                _ => continue,
            };

            let response: Response = match ror {
                RawOrRef::Value(r) => r.clone(),
                RawOrRef::Ref { ref_path } => {
                    let name = ref_name(ref_path);
                    self.raw
                        .components
                        .as_ref()
                        .and_then(|c| c.responses.get(name))
                        .and_then(|r| {
                            if let RawOrRef::Value(resp) = r {
                                Some(resp.clone())
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| anyhow::anyhow!("Response '{}' not found", name))?
                }
            };

            let schema_ror = response
                .content
                .as_ref()
                .and_then(|c| c.get("application/json"))
                .and_then(|m| m.schema.as_ref())
                .cloned();

            let schema = match schema_ror {
                // Inline primitive refs (e.g. $ref: ErrorCode when ErrorCode is type: string).
                // Preserve Ref for Object/Enum/Union so exception generators can name the class.
                Some(RawOrRef::Ref { ref_path }) => Some(self.resolve_ref_or_inline(&ref_path)?),
                Some(ror) => Some(self.resolve_schema_or_ref(&ror)?),
                None => None,
            };

            result.push(ResolvedErrorResponse {
                status_code,
                schema,
            });
        }

        Ok(result)
    }

    fn resolve_success_response(
        &mut self,
        responses: &IndexMap<String, RawOrRef<Response>>,
    ) -> Result<Option<ResolvedSchema>> {
        let ror = responses
            .get("200")
            .or_else(|| responses.get("201"))
            .or_else(|| responses.get("2xx"))
            .or_else(|| responses.get("default"));

        let Some(ror) = ror else {
            return Ok(None);
        };

        let response: Response = match ror {
            RawOrRef::Value(r) => r.clone(),
            RawOrRef::Ref { ref_path } => {
                let name = ref_name(ref_path);
                self.raw
                    .components
                    .as_ref()
                    .and_then(|c| c.responses.get(name))
                    .and_then(|r| {
                        if let RawOrRef::Value(resp) = r {
                            Some(resp.clone())
                        } else {
                            None
                        }
                    })
                    .ok_or_else(|| anyhow::anyhow!("Response '{}' not found", name))?
            }
        };

        let schema_ror = response
            .content
            .as_ref()
            .and_then(|c| c.get("application/json"))
            .and_then(|m| m.schema.as_ref())
            .cloned();

        match schema_ror {
            // Inline primitive refs; preserve Ref for Object/Enum/Union so generators
            // can derive `XxxResource` / `XxxRequest` class names from the ref name.
            Some(RawOrRef::Ref { ref_path }) => Ok(Some(self.resolve_ref_or_inline(&ref_path)?)),
            Some(ror) => Ok(Some(self.resolve_schema_or_ref(&ror)?)),
            None => Ok(None),
        }
    }
}

// ---------------------------------------------------------------------------
// Free helpers
// ---------------------------------------------------------------------------

fn ref_name(ref_path: &str) -> &str {
    ref_path.rsplit('/').next().unwrap_or(ref_path)
}

/// Returns true if the schema is nullable under either OAS 3.0 or 3.1 rules:
/// - OAS 3.0: `nullable: true`
/// - OAS 3.1: `type` array contains `"null"`
fn is_nullable(schema: &Schema) -> bool {
    if let Some(t) = &schema.schema_type
        && t.contains_null()
    {
        return true;
    }
    schema.nullable.unwrap_or(false)
}

fn mixed() -> ResolvedSchema {
    ResolvedSchema::Primitive(PrimitiveSchema {
        php_type: PhpPrimitive::Mixed,
        format: None,
        description: None,
        nullable: false,
        example: None,
        min_length: None,
        max_length: None,
        minimum: None,
        maximum: None,
        exclusive_minimum: None,
        exclusive_maximum: None,
        pattern: None,
    })
}

fn build_primitive(schema: &Schema) -> PrimitiveSchema {
    let php_type = match schema.schema_type.as_ref().and_then(|t| t.primary()) {
        Some(SchemaType::String) => match schema.format.as_deref() {
            Some("date-time") | Some("date") => PhpPrimitive::DateTime,
            _ => PhpPrimitive::String,
        },
        Some(SchemaType::Integer) => PhpPrimitive::Int,
        Some(SchemaType::Number) => PhpPrimitive::Float,
        Some(SchemaType::Boolean) => PhpPrimitive::Bool,
        _ => PhpPrimitive::Mixed,
    };
    PrimitiveSchema {
        php_type,
        format: schema.format.clone(),
        description: schema.description.clone(),
        nullable: is_nullable(schema),
        example: schema.example.clone(),
        min_length: schema.min_length,
        max_length: schema.max_length,
        minimum: schema.minimum,
        maximum: schema.maximum,
        exclusive_minimum: schema.exclusive_minimum,
        exclusive_maximum: schema.exclusive_maximum,
        pattern: schema.pattern.clone(),
    }
}

fn build_enum(schema: &Schema) -> EnumSchema {
    let all_int = schema
        .enum_values
        .iter()
        .all(|v| matches!(v, EnumValue::Integer(_) | EnumValue::Null));

    let backing_type = if all_int {
        EnumBackingType::Int
    } else {
        EnumBackingType::String
    };

    // Pair each non-null enum value with its optional description label.
    // x-enum-descriptions is index-aligned with enum_values (nulls count toward the index).
    let descriptions = &schema.x_enum_descriptions;
    let variants = schema
        .enum_values
        .iter()
        .enumerate()
        .filter_map(|(idx, v)| {
            let label = descriptions.get(idx).and_then(|s| {
                let trimmed = s.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            });
            match v {
                EnumValue::Null => None,
                EnumValue::String(s) => Some(EnumVariant {
                    name: to_safe_enum_name(s),
                    value: s.clone(),
                    label,
                }),
                EnumValue::Integer(i) => Some(EnumVariant {
                    name: format!("Value{i}"),
                    value: i.to_string(),
                    label,
                }),
                EnumValue::Float(f) => Some(EnumVariant {
                    name: format!("Value{}", *f as i64),
                    value: f.to_string(),
                    label,
                }),
                EnumValue::Bool(b) => Some(EnumVariant {
                    name: if *b { "True" } else { "False" }.to_string(),
                    value: b.to_string(),
                    label,
                }),
            }
        })
        .collect();

    EnumSchema {
        variants,
        backing_type,
        description: schema.description.clone(),
    }
}

fn to_safe_enum_name(s: &str) -> String {
    let pascal = to_pascal_case(s);
    if pascal.is_empty() {
        "Empty".to_string()
    } else if pascal.starts_with(|c: char| c.is_ascii_digit()) {
        format!("V{pascal}")
    } else {
        pascal
    }
}

fn derive_operation_id(method: &HttpMethod, path: &str) -> String {
    let method_part = method.as_str().to_lowercase();
    let path_part: String = path
        .split('/')
        .filter(|s| !s.is_empty() && !s.starts_with('{'))
        .map(to_pascal_case)
        .collect();
    format!("{method_part}{path_part}")
}
