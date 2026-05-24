pub mod bruno;
pub mod postman;

use crate::domain::endpoint::HttpMethod;

pub struct ImportedCollection {
    pub name: String,
    pub description: Option<String>,
    pub endpoints: Vec<ImportedEndpoint>,
}

pub struct ImportedEndpoint {
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub status_code: u16,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_content_type: Option<String>,
    pub delay_ms: u32,
}
