use std::collections::HashMap;
use std::time::Duration;

use axum::extract::{Path, State};
use axum::http::{header, HeaderName, HeaderValue, Method, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::{body::Body, Router};

use crate::application::services::mocks::MockError;
use crate::domain::endpoint::HttpMethod;
use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/:collection_code", any(handle_root))
        .route("/:collection_code/*tail", any(handle_with_tail))
}

fn to_domain_method(m: &Method) -> Option<HttpMethod> {
    match m.as_str() {
        "GET" => Some(HttpMethod::Get),
        "POST" => Some(HttpMethod::Post),
        "PUT" => Some(HttpMethod::Put),
        "PATCH" => Some(HttpMethod::Patch),
        "DELETE" => Some(HttpMethod::Delete),
        "HEAD" => Some(HttpMethod::Head),
        "OPTIONS" => Some(HttpMethod::Options),
        _ => None,
    }
}

async fn handle_root(
    method: Method,
    State(state): State<AppState>,
    Path(collection_code): Path<String>,
) -> Response {
    dispatch(method, state, collection_code, "/").await
}

async fn handle_with_tail(
    method: Method,
    State(state): State<AppState>,
    Path((collection_code, tail)): Path<(String, String)>,
) -> Response {
    let path = format!("/{tail}");
    dispatch(method, state, collection_code, &path).await
}

async fn dispatch(
    method: Method,
    state: AppState,
    collection_code: String,
    path: &str,
) -> Response {
    let domain_method = match to_domain_method(&method) {
        Some(m) => m,
        None => {
            return (StatusCode::METHOD_NOT_ALLOWED, "Method not supported").into_response();
        }
    };

    match state
        .mocks
        .resolve_by_code(&collection_code, domain_method, path)
        .await
    {
        Err(MockError::CollectionNotFound) => {
            (StatusCode::NOT_FOUND, "collection not found").into_response()
        }
        Err(MockError::ServiceUnavailable) => {
            (StatusCode::SERVICE_UNAVAILABLE, "collection is inactive").into_response()
        }
        Err(MockError::NotFound) => (StatusCode::NOT_FOUND, "no matching endpoint").into_response(),
        Err(MockError::MethodNotAllowed(allowed)) => {
            let allow_value = allowed
                .iter()
                .map(|m| m.to_string())
                .collect::<Vec<_>>()
                .join(", ");
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::ALLOW, allow_value)
                .body(Body::from("method not allowed"))
                .unwrap_or_else(|_| StatusCode::METHOD_NOT_ALLOWED.into_response())
        }
        Err(MockError::Internal(_)) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        Ok(result) => {
            if result.delay_ms > 0 {
                tokio::time::sleep(Duration::from_millis(result.delay_ms as u64)).await;
            }

            let status = StatusCode::from_u16(result.status_code).unwrap_or(StatusCode::OK);

            let mut builder = Response::builder().status(status);

            // Apply custom response headers from the JSON string.
            if let Some(ref headers_json) = result.response_headers {
                if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(headers_json) {
                    for (k, v) in map {
                        if let (Ok(name), Ok(value)) = (
                            HeaderName::from_bytes(k.as_bytes()),
                            HeaderValue::from_str(&v),
                        ) {
                            builder = builder.header(name, value);
                        }
                    }
                }
            }

            // response_content_type overrides any Content-Type set above.
            if let Some(ref ct) = result.response_content_type {
                if let Ok(value) = HeaderValue::from_str(ct) {
                    builder = builder.header(header::CONTENT_TYPE, value);
                }
            }

            let body = result.response_body.unwrap_or_default();
            builder
                .body(Body::from(body))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}
