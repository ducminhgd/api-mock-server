use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::TypedHeader;
use uuid::Uuid;

use crate::adapters::http::error::ApiError;
use crate::infrastructure::state::AppState;

/// Caller identity extracted from a `Authorization: Bearer <jwt>` header.
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let app_state = AppState::from_ref(state);

        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                .map_err(|_| {
                    ApiError(
                        StatusCode::UNAUTHORIZED,
                        "UNAUTHORIZED",
                        "missing or invalid authorization header".into(),
                    )
                })?;

        let claims = app_state.jwt.verify(bearer.token()).map_err(ApiError::from)?;

        let user_id = Uuid::parse_str(&claims.sub).map_err(|_| {
            ApiError(StatusCode::UNAUTHORIZED, "UNAUTHORIZED", "invalid token subject".into())
        })?;

        Ok(AuthUser { user_id, role: claims.role })
    }
}
