use axum::body::Body;
use axum::extract::{Multipart, Path, Query, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::Deserialize;
use uuid::Uuid;

use crate::adapters::http::error::ApiError;
use crate::adapters::http::extractor::AuthUser;
use crate::application::dto::collection::{
    CollectionFilter, CollectionResponse, CreateCollectionRequest, TransferOwnershipRequest,
    UpdateCollectionRequest,
};
use crate::application::dto::collection_share::{
    CollectionShareResponse, CreateShareRequest, UpdateShareRequest,
};
use crate::application::dto::endpoint::{
    CreateEndpointRequest, EndpointFilter, EndpointResponse, UpdateEndpointRequest,
};
use crate::application::dto::pagination::{PageParams, Paginated};
use crate::domain::collection::{CollectionStatus, CollectionVisibility};
use crate::domain::endpoint::{EndpointStatus, HttpMethod};
use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(list).post(create))
        .route("/import", post(import_collection))
        .route("/:id", get(get_one).put(update).delete(remove))
        .route("/:id/duplicate", post(duplicate))
        .route("/:id/export", get(export_collection))
        .route("/:id/shares", get(list_shares).post(add_share))
        .route(
            "/:id/shares/:share_id",
            put(update_share).delete(remove_share),
        )
        .route("/:id/transfer", put(transfer_ownership))
        .route("/:id/endpoints", get(list_endpoints).post(create_endpoint))
        .route(
            "/:id/endpoints/:eid",
            get(get_endpoint)
                .put(update_endpoint)
                .delete(remove_endpoint),
        )
        .route("/:id/endpoints/:eid/duplicate", post(duplicate_endpoint))
}

#[derive(Deserialize)]
struct ListQuery {
    search: Option<String>,
    status: Option<CollectionStatus>,
    visibility: Option<CollectionVisibility>,
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list(
    auth: AuthUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Paginated<CollectionResponse>>, ApiError> {
    let filter = CollectionFilter {
        search: q.search,
        status: q.status,
        visibility: q.visibility,
    };
    let page = PageParams {
        page: q.page.unwrap_or(1),
        limit: q.limit.unwrap_or(20),
    };
    state
        .collections
        .list(auth.user_id, filter, page)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn create(
    auth: AuthUser,
    State(state): State<AppState>,
    Json(req): Json<CreateCollectionRequest>,
) -> Result<(StatusCode, Json<CollectionResponse>), ApiError> {
    let resp = state
        .collections
        .create(auth.user_id, req)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_one(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<CollectionResponse>, ApiError> {
    state
        .collections
        .get(id, auth.user_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn update(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdateCollectionRequest>,
) -> Result<Json<CollectionResponse>, ApiError> {
    state
        .collections
        .update(id, auth.user_id, req)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn remove(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    state
        .collections
        .delete(id, auth.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn duplicate(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CollectionResponse>), ApiError> {
    let resp = state
        .collections
        .duplicate(id, auth.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn list_shares(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<CollectionShareResponse>>, ApiError> {
    state
        .collections
        .list_shares(id, auth.user_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn add_share(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateShareRequest>,
) -> Result<(StatusCode, Json<CollectionShareResponse>), ApiError> {
    let resp = state
        .collections
        .add_share(id, auth.user_id, req)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn update_share(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, share_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateShareRequest>,
) -> Result<Json<CollectionShareResponse>, ApiError> {
    state
        .collections
        .update_share(id, share_id, auth.user_id, req)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn remove_share(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, share_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .collections
        .remove_share(id, share_id, auth.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn transfer_ownership(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<TransferOwnershipRequest>,
) -> Result<Json<CollectionResponse>, ApiError> {
    state
        .collections
        .transfer_ownership(id, auth.user_id, req)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

// ── Endpoint handlers ─────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ListEndpointsQuery {
    search: Option<String>,
    method: Option<HttpMethod>,
    status: Option<EndpointStatus>,
    page: Option<u32>,
    limit: Option<u32>,
}

async fn list_endpoints(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ListEndpointsQuery>,
) -> Result<Json<Paginated<EndpointResponse>>, ApiError> {
    let filter = EndpointFilter {
        search: q.search,
        method: q.method,
        status: q.status,
    };
    let page = PageParams {
        page: q.page.unwrap_or(1),
        limit: q.limit.unwrap_or(20),
    };
    state
        .endpoints
        .list(id, auth.user_id, filter, page)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn create_endpoint(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(req): Json<CreateEndpointRequest>,
) -> Result<(StatusCode, Json<EndpointResponse>), ApiError> {
    let resp = state
        .endpoints
        .create(id, auth.user_id, req)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

async fn get_endpoint(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, eid)): Path<(Uuid, Uuid)>,
) -> Result<Json<EndpointResponse>, ApiError> {
    state
        .endpoints
        .get(id, eid, auth.user_id)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn update_endpoint(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, eid)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateEndpointRequest>,
) -> Result<Json<EndpointResponse>, ApiError> {
    state
        .endpoints
        .update(id, eid, auth.user_id, req)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn remove_endpoint(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, eid)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    state
        .endpoints
        .delete(id, eid, auth.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok(StatusCode::NO_CONTENT)
}

async fn duplicate_endpoint(
    auth: AuthUser,
    State(state): State<AppState>,
    Path((id, eid)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<EndpointResponse>), ApiError> {
    let resp = state
        .endpoints
        .duplicate(id, eid, auth.user_id)
        .await
        .map_err(ApiError::from)?;
    Ok((StatusCode::CREATED, Json(resp)))
}

// ── Import / Export ───────────────────────────────────────────────────────────

async fn import_collection(
    auth: AuthUser,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<(StatusCode, Json<CollectionResponse>), ApiError> {
    let mut file_bytes: Option<Vec<u8>> = None;
    let mut filename = String::new();

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| ApiError(StatusCode::BAD_REQUEST, "BAD_REQUEST", e.to_string()))?
    {
        if field.name() == Some("file") {
            filename = field.file_name().unwrap_or("upload").to_string();
            let bytes = field
                .bytes()
                .await
                .map_err(|e| ApiError(StatusCode::BAD_REQUEST, "BAD_REQUEST", e.to_string()))?;
            file_bytes = Some(bytes.to_vec());
        }
    }

    let bytes = file_bytes.ok_or_else(|| {
        ApiError(
            StatusCode::BAD_REQUEST,
            "BAD_REQUEST",
            "no file field in multipart body".into(),
        )
    })?;

    let imported = if filename.ends_with(".zip") {
        crate::application::io::bruno::parse_zip(&bytes)
            .map_err(|e| ApiError(StatusCode::UNPROCESSABLE_ENTITY, "PARSE_ERROR", e))?
    } else if filename.ends_with(".bru") {
        let content = String::from_utf8(bytes).map_err(|_| {
            ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "PARSE_ERROR",
                "file is not valid UTF-8".into(),
            )
        })?;
        let collection_name = filename.trim_end_matches(".bru");
        crate::application::io::bruno::parse_single_bru(collection_name, &content).ok_or_else(
            || {
                ApiError(
                    StatusCode::UNPROCESSABLE_ENTITY,
                    "PARSE_ERROR",
                    "failed to parse .bru file".into(),
                )
            },
        )?
    } else {
        let content = String::from_utf8(bytes).map_err(|_| {
            ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "PARSE_ERROR",
                "file is not valid UTF-8".into(),
            )
        })?;
        crate::application::io::postman::parse(&content).map_err(|e| {
            ApiError(
                StatusCode::UNPROCESSABLE_ENTITY,
                "PARSE_ERROR",
                e.to_string(),
            )
        })?
    };

    let collection = state
        .import_export
        .import(auth.user_id, imported)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CollectionResponse::from(collection)),
    ))
}

#[derive(Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn export_collection(
    auth: AuthUser,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(q): Query<ExportQuery>,
) -> Result<Response, ApiError> {
    let (collection, endpoints) = state
        .import_export
        .export(id, auth.user_id)
        .await
        .map_err(ApiError::from)?;

    let format = q.format.as_deref().unwrap_or("postman");

    match format {
        "bruno" => {
            let bytes = crate::application::io::bruno::serialize_zip(&collection, &endpoints)
                .map_err(|e| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", e))?;
            let fname = format!("{}.zip", slugify(&collection.name));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/zip")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{fname}\""),
                )
                .body(Body::from(bytes))
                .unwrap())
        }
        _ => {
            let json = crate::application::io::postman::serialize(&collection, &endpoints)
                .map_err(|e| {
                    ApiError(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "INTERNAL_ERROR",
                        e.to_string(),
                    )
                })?;
            let fname = format!("{}.postman_collection.json", slugify(&collection.name));
            Ok(Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .header(
                    "Content-Disposition",
                    format!("attachment; filename=\"{fname}\""),
                )
                .body(Body::from(json))
                .unwrap())
        }
    }
}

fn slugify(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' {
                c
            } else {
                '_'
            }
        })
        .collect()
}
