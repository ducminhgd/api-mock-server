use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::endpoint::{Endpoint, EndpointStatus, HttpMethod};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointResponse {
    pub id: Uuid,
    pub collection_id: Uuid,
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub status_code: u16,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_content_type: Option<String>,
    pub delay_ms: u32,
    pub status: EndpointStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Endpoint> for EndpointResponse {
    fn from(e: Endpoint) -> Self {
        Self {
            id: e.id,
            collection_id: e.collection_id,
            name: e.name,
            method: e.method,
            path: e.path,
            status_code: e.status_code,
            response_headers: e.response_headers,
            response_body: e.response_body,
            response_content_type: e.response_content_type,
            delay_ms: e.delay_ms,
            status: e.status,
            created_at: e.created_at,
            updated_at: e.updated_at,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateEndpointRequest {
    pub name: String,
    pub method: HttpMethod,
    pub path: String,
    pub status_code: Option<u16>,
    pub delay_ms: Option<u32>,
    pub response_headers: Option<String>,
    pub response_body: Option<String>,
    pub response_content_type: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateEndpointRequest {
    pub name: Option<String>,
    pub method: Option<HttpMethod>,
    pub path: Option<String>,
    pub status_code: Option<u16>,
    pub delay_ms: Option<u32>,
    pub response_headers: Option<Option<String>>,
    pub response_body: Option<Option<String>>,
    pub response_content_type: Option<Option<String>>,
    pub status: Option<EndpointStatus>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct EndpointFilter {
    pub search: Option<String>,
    pub method: Option<HttpMethod>,
    pub status: Option<EndpointStatus>,
}
