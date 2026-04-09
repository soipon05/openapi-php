use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// ─── Root ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct OpenApi {
    pub openapi: String,
    pub info: Info,
    pub paths: Option<IndexMap<String, PathItem>>,
    pub components: Option<Components>,
    pub servers: Option<Vec<Server>>,
    pub tags: Option<Vec<Tag>>,
}

// ─── Info ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Info {
    pub title: String,
    pub version: String,
    pub description: Option<String>,
}

// ─── Server ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
}

// ─── Tag ──────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
}

// ─── Paths ────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
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
    pub parameters: Option<Vec<Parameter>>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Operation {
    pub operation_id: Option<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub tags: Option<Vec<String>>,
    pub parameters: Option<Vec<Parameter>>,
    pub request_body: Option<RequestBody>,
    pub responses: IndexMap<String, Response>,
    pub deprecated: Option<bool>,
}

// ─── Parameters ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "in")]
    pub location: ParameterLocation,
    pub description: Option<String>,
    pub required: Option<bool>,
    pub schema: Option<Schema>,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ParameterLocation {
    Query,
    Header,
    Path,
    Cookie,
}

// ─── Request / Response ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct RequestBody {
    pub description: Option<String>,
    pub required: Option<bool>,
    pub content: IndexMap<String, MediaType>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Response {
    pub description: String,
    pub content: Option<IndexMap<String, MediaType>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct MediaType {
    pub schema: Option<Schema>,
}

// ─── Components ───────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct Components {
    pub schemas: Option<IndexMap<String, Schema>>,
    pub parameters: Option<IndexMap<String, Parameter>>,
    pub responses: Option<IndexMap<String, Response>>,
    #[serde(rename = "requestBodies")]
    pub request_bodies: Option<IndexMap<String, RequestBody>>,
    #[serde(rename = "securitySchemes")]
    pub security_schemes: Option<IndexMap<String, SecurityScheme>>,
}

// ─── Schema ───────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Schema {
    #[serde(rename = "type")]
    pub schema_type: Option<SchemaType>,
    pub format: Option<String>,
    pub description: Option<String>,
    pub nullable: Option<bool>,
    pub required: Option<Vec<String>>,
    pub properties: Option<IndexMap<String, Schema>>,
    pub items: Option<Box<Schema>>,
    pub enum_values: Option<Vec<serde_json::Value>>,
    #[serde(rename = "$ref")]
    pub ref_path: Option<String>,

    // Composition
    #[serde(rename = "allOf")]
    pub all_of: Option<Vec<Schema>>,
    #[serde(rename = "anyOf")]
    pub any_of: Option<Vec<Schema>>,
    #[serde(rename = "oneOf")]
    pub one_of: Option<Vec<Schema>>,

    // Validation
    pub minimum: Option<f64>,
    pub maximum: Option<f64>,
    #[serde(rename = "minLength")]
    pub min_length: Option<u64>,
    #[serde(rename = "maxLength")]
    pub max_length: Option<u64>,
    pub pattern: Option<String>,
    pub example: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    String,
    Integer,
    Number,
    Boolean,
    Array,
    Object,
}

// ─── Security ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize, Serialize)]
pub struct SecurityScheme {
    #[serde(rename = "type")]
    pub scheme_type: String,
    pub scheme: Option<String>,
    pub name: Option<String>,
    #[serde(rename = "in")]
    pub location: Option<String>,
    pub description: Option<String>,
}
