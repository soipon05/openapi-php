//! IR endpoint types: resolved API endpoints with fully expanded parameters and responses.

use indexmap::IndexMap;

use super::schema::ResolvedSchema;

#[derive(Debug, Clone)]
pub struct ResolvedSpec {
    pub title: String,
    pub version: String,
    pub base_url: String,
    pub schemas: IndexMap<String, ResolvedSchema>,
    pub endpoints: Vec<ResolvedEndpoint>,
    pub security_schemes: Vec<ResolvedSecurityScheme>,
}

#[derive(Debug, Clone)]
pub struct ResolvedSecurityScheme {
    /// The key name from `components/securitySchemes`, e.g. `"BearerAuth"`
    pub name: String,
    pub scheme_type: SecuritySchemeType,
}

#[derive(Debug, Clone)]
pub enum SecuritySchemeType {
    /// `type: http` — `scheme` is normalised to lowercase, e.g. `"bearer"` or `"basic"`
    Http { scheme: String },
    /// `type: apiKey`
    ApiKey {
        /// `"header"`, `"query"`, or `"cookie"`
        in_: String,
        /// The header / query / cookie name, e.g. `"X-API-Key"`
        name: String,
    },
}

#[derive(Debug, Clone)]
pub struct ResolvedEndpoint {
    pub operation_id: String,
    pub method: HttpMethod,
    pub path: String,
    pub summary: Option<String>,
    pub tags: Vec<String>,
    pub path_params: Vec<ResolvedParam>,
    pub query_params: Vec<ResolvedParam>,
    pub header_params: Vec<ResolvedParam>,
    pub request_body: Option<ResolvedRequestBody>,
    pub response: Option<ResolvedSchema>,
    pub deprecated: bool,
    pub error_responses: Vec<ResolvedErrorResponse>,
    /// `true` when the operation declares at least one non-empty security requirement.
    pub requires_auth: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedErrorResponse {
    pub status_code: u16,
    /// None when the error response has no body (e.g. 204)
    pub schema: Option<ResolvedSchema>,
}

#[derive(Debug, Clone)]
pub struct ResolvedParam {
    pub name: String,
    /// camelCase PHP name
    pub php_name: String,
    pub schema: ResolvedSchema,
    pub required: bool,
}

#[derive(Debug, Clone)]
pub struct ResolvedRequestBody {
    pub schema: ResolvedSchema,
    pub required: bool,
    pub content_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
}

impl HttpMethod {
    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }
}
