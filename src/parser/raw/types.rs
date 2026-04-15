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
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<RawOrRef<Schema>>,
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
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
    pub format: Option<String>,
    pub description: Option<String>,
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
    #[serde(rename = "minLength")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u64>,
    pub pattern: Option<String>,
    pub example: Option<serde_json::Value>,
    pub discriminator: Option<Discriminator>,
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
