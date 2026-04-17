use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

/// A value that can be either inline or a `$ref` pointer.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum RawOrRef<T> {
    Ref {
        #[serde(rename = "$ref")]
        ref_path: String,
    },
    Value(T),
}

/// Typed enum for the values allowed inside an OpenAPI `enum` array.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum EnumValue {
    Integer(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Null,
}

pub type RawOpenApi = OpenApi;

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenApi {
    pub openapi: String,
    pub info: Info,
    #[serde(default)]
    pub paths: IndexMap<String, PathItem>,
    pub components: Option<Components>,
    #[serde(default)]
    pub servers: Vec<Server>,
    #[serde(default)]
    pub tags: Vec<Tag>,
    /// Root-level security requirements. Applied to all operations that do not
    /// declare their own `security` field. Each entry maps a security scheme
    /// name to a list of required scopes.
    #[serde(default)]
    pub security: Vec<IndexMap<String, Vec<String>>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PathItem {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub get: Option<Operation>,
    pub post: Option<Operation>,
    pub put: Option<Operation>,
    pub patch: Option<Operation>,
    pub delete: Option<Operation>,
    pub head: Option<Operation>,
    pub options: Option<Operation>,
    #[serde(default)]
    pub parameters: Vec<RawOrRef<Parameter>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub parameters: Vec<RawOrRef<Parameter>>,
    pub request_body: Option<RawOrRef<RequestBody>>,
    #[serde(default)]
    pub responses: IndexMap<String, RawOrRef<Response>>,
    pub deprecated: Option<bool>,
    /// Operation-level security requirements.
    ///
    /// - `None`       — field absent in the spec; inherit from the global `security` list.
    /// - `Some([])`   — explicitly overridden to "no security required".
    /// - `Some([…])`  — one or more security requirements; auth is required.
    ///
    /// `#[serde(default)]` is intentionally **not** applied here so that a
    /// missing field deserialises as `None` rather than `Some([])`.
    pub security: Option<Vec<IndexMap<String, Vec<String>>>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<RawOrRef<Schema>>,
    /// OpenAPI serialization style (e.g. `"form"`, `"spaceDelimited"`, `"pipeDelimited"`).
    pub style: Option<String>,
    /// Whether array/object values are exploded into repeated key-value pairs.
    /// Default for query params: `true` (form style, exploded).
    pub explode: Option<bool>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    #[serde(default)]
    pub content: IndexMap<String, MediaType>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Response {
    pub description: String,
    pub content: Option<IndexMap<String, MediaType>>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct MediaType {
    pub schema: Option<RawOrRef<Schema>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Components {
    #[serde(default)]
    pub schemas: IndexMap<String, RawOrRef<Schema>>,
    #[serde(default)]
    pub parameters: IndexMap<String, RawOrRef<Parameter>>,
    #[serde(default)]
    pub responses: IndexMap<String, RawOrRef<Response>>,
    #[serde(rename = "requestBodies", default)]
    pub request_bodies: IndexMap<String, RawOrRef<RequestBody>>,
    #[serde(rename = "securitySchemes", default)]
    pub security_schemes: IndexMap<String, SecurityScheme>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Schema {
    /// `type` field — a single string (OAS 3.0) or an array of strings (OAS 3.1).
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaTypeOrTypes>,
    pub format: Option<String>,
    pub description: Option<String>,
    /// OAS 3.0 `nullable: true`. Retained for backward compatibility; use
    /// `SchemaTypeOrTypes::contains_null()` for OAS 3.1 detection.
    pub nullable: Option<bool>,
    #[serde(default)]
    pub required: Vec<String>,
    #[serde(default)]
    pub properties: IndexMap<String, RawOrRef<Schema>>,
    pub items: Option<Box<RawOrRef<Schema>>>,
    #[serde(rename = "enum", default)]
    pub enum_values: Vec<EnumValue>,
    #[serde(rename = "allOf", default)]
    pub all_of: Vec<RawOrRef<Schema>>,
    #[serde(rename = "anyOf", default)]
    pub any_of: Vec<RawOrRef<Schema>>,
    #[serde(rename = "oneOf", default)]
    pub one_of: Vec<RawOrRef<Schema>>,
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    /// OAS 3.1: numeric exclusive lower bound (3.0 used a boolean — not supported).
    #[serde(rename = "exclusiveMinimum")]
    pub exclusive_minimum: Option<f64>,
    /// OAS 3.1: numeric exclusive upper bound (3.0 used a boolean — not supported).
    #[serde(rename = "exclusiveMaximum")]
    pub exclusive_maximum: Option<f64>,
    #[serde(rename = "minLength")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u64>,
    pub pattern: Option<String>,
    pub example: Option<serde_json::Value>,
    pub discriminator: Option<Discriminator>,
    #[serde(rename = "additionalProperties")]
    pub additional_properties: Option<Box<AdditionalProperties>>,
    pub default: Option<serde_json::Value>,
    #[serde(rename = "readOnly")]
    pub read_only: Option<bool>,
    #[serde(rename = "writeOnly")]
    pub write_only: Option<bool>,
    pub deprecated: Option<bool>,
    /// Vendor extension: human-readable label per enum value (index-aligned with `enum_values`).
    #[serde(rename = "x-enum-descriptions", default)]
    pub x_enum_descriptions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Discriminator {
    #[serde(rename = "propertyName")]
    pub property_name: String,
    #[serde(default)]
    pub mapping: IndexMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AdditionalProperties {
    Bool(bool),
    Schema(Box<RawOrRef<Schema>>),
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
    /// OpenAPI 3.1: `"null"` is a valid type name.
    Null,
}

/// OpenAPI 3.0 uses a single string for `type`; 3.1 allows an array.
/// This enum handles both via `#[serde(untagged)]`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SchemaTypeOrTypes {
    Single(SchemaType),
    Multiple(Vec<SchemaType>),
}

impl SchemaTypeOrTypes {
    /// Returns the primary type — the first non-null type in the list.
    /// For a single non-null type, returns that type.
    pub fn primary(&self) -> Option<&SchemaType> {
        match self {
            Self::Single(t) => {
                if matches!(t, SchemaType::Null) {
                    None
                } else {
                    Some(t)
                }
            }
            Self::Multiple(ts) => ts.iter().find(|t| !matches!(t, SchemaType::Null)),
        }
    }

    /// Returns true if `"null"` appears in the type list.
    pub fn contains_null(&self) -> bool {
        match self {
            Self::Single(t) => matches!(t, SchemaType::Null),
            Self::Multiple(ts) => ts.iter().any(|t| matches!(t, SchemaType::Null)),
        }
    }

    /// Returns true if the primary type matches `t`.
    pub fn is(&self, t: &SchemaType) -> bool {
        self.primary() == Some(t)
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub scheme: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub location: Option<String>,
    pub description: Option<String>,
}
