use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

use crate::domain::errors::DomainError;

#[derive(Serialize)]
struct ErrorBody {
    code: &'static str,
    message: String,
}

pub struct ApiError(StatusCode, &'static str, String);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody { code: self.1, message: self.2 });
        (self.0, body).into_response()
    }
}

impl From<DomainError> for ApiError {
    fn from(e: DomainError) -> Self {
        match &e {
            DomainError::GroupNotFound(_) | DomainError::UserNotFound(_) => {
                ApiError(StatusCode::NOT_FOUND, "NOT_FOUND", e.to_string())
            }
            DomainError::UsernameTaken(_) | DomainError::GroupNameTaken(_) => {
                ApiError(StatusCode::CONFLICT, "CONFLICT", e.to_string())
            }
            DomainError::InvalidCredentials => {
                ApiError(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", e.to_string())
            }
            DomainError::Forbidden => {
                ApiError(StatusCode::FORBIDDEN, "FORBIDDEN", e.to_string())
            }
            DomainError::Internal(_) => {
                ApiError(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "internal server error".into())
            }
        }
    }
}
