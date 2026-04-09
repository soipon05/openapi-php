use indexmap::IndexMap;

use super::schema::ResolvedSchema;

#[derive(Debug, Clone)]
pub struct ResolvedSpec {
    pub title: String,
    pub version: String,
    pub base_url: String,
    pub schemas: IndexMap<String, ResolvedSchema>,
    pub endpoints: Vec<ResolvedEndpoint>,
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
    pub request_body: Option<ResolvedRequestBody>,
    pub response: Option<ResolvedSchema>,
    pub deprecated: bool,
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
