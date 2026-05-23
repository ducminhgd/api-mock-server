use axum::extract::State;
use axum::http::StatusCode;
use axum::routing::post;
use axum::{Json, Router};

use crate::adapters::http::error::ApiError;
use crate::application::dto::user::{LoginRequest, LoginResponse};
use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/login", post(login))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> Result<(StatusCode, Json<LoginResponse>), ApiError> {
    let resp = state.auth.login(req).await.map_err(ApiError::from)?;
    Ok((StatusCode::OK, Json(resp)))
}
