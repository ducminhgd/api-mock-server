use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::adapters::http::error::ApiError;
use crate::application::dto::group::{
    CreateGroupRequest, GroupFilter, GroupResponse, UpdateGroupRequest,
};
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::domain::group::GroupStatus;
use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/:id", get(get_one).put(update).delete(remove))
}

#[derive(Deserialize)]
struct ListQuery {
    search: Option<String>,
    status: Option<GroupStatus>,
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Paginated<GroupResponse>>, ApiError> {
    let filter = GroupFilter {
        search: q.search,
        status: q.status,
    };
    let page = PageParams {
        page: q.page.unwrap_or(1),
        limit: q.limit.unwrap_or(20),
    };
    state
        .groups
        .list(filter, page)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn create(
    State(state): State<AppState>,
    Json(req): Json<CreateGroupRequest>,
) -> Result<(StatusCode, Json<GroupResponse>), ApiError> {
    let resp = state.groups.create(req).await.map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<GroupResponse>, ApiError> {
    state.groups.get(id).await.map(Json).map_err(ApiError::from)
}

async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateGroupRequest>,
) -> Result<Json<GroupResponse>, ApiError> {
    state
        .groups
        .update(id, req)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn remove(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state.groups.delete(id).await.map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}
