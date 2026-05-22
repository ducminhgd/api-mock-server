use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::adapters::http::error::ApiError;
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::application::dto::user::{
    CreateUserRequest, ResetPasswordResponse, UpdateUserRequest, UserFilter, UserResponse,
};
use crate::domain::user::UserStatus;
use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(remove))
        .route("/:id/reset-password", post(reset_password))
}

#[derive(Deserialize)]
struct ListQuery {
    search: Option<String>,
    group_id: Option<Uuid>,
    status: Option<UserStatus>,
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Paginated<UserResponse>>, ApiError> {
    let filter = UserFilter { search: q.search, group_id: q.group_id, status: q.status };
    let page = PageParams { page: q.page.unwrap_or(1), limit: q.limit.unwrap_or(20) };
    state.users.list(filter, page).await.map(Json).map_err(ApiError::from)
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<UserResponse>), ApiError> {
    let resp = state.users.create(req).await.map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<UserResponse>, ApiError> {
    state.users.get(id).await.map(Json).map_err(ApiError::from)
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    state.users.update(id, req).await.map(Json).map_err(ApiError::from)
}

async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.users.delete(id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn reset_password(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResetPasswordResponse>, ApiError> {
    state.users.reset_password(id).await.map(Json).map_err(ApiError::from)
}
