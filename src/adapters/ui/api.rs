// REST API client — real impl runs only on WASM; SSR stubs return Err.
// Every function takes `token: &str` (Bearer JWT) for authenticated calls.

use crate::application::dto::collection::{
    CollectionResponse, CreateCollectionRequest, TransferOwnershipRequest, UpdateCollectionRequest,
};
use crate::application::dto::collection_share::{
    CollectionShareResponse, CreateShareRequest, UpdateShareRequest,
};
use crate::application::dto::endpoint::{
    CreateEndpointRequest, EndpointResponse, UpdateEndpointRequest,
};
use crate::application::dto::group::{CreateGroupRequest, GroupResponse, UpdateGroupRequest};
use crate::application::dto::pagination::Paginated;
use crate::application::dto::user::{
    CreateUserRequest, LoginRequest, LoginResponse, ResetPasswordResponse, UpdateUserRequest,
    UserResponse,
};

pub type ApiResult<T> = Result<T, String>;

// ── Auth ──────────────────────────────────────────────────────────────────

pub async fn login(username: String, password: String) -> ApiResult<LoginResponse> {
    post_json_anon("/api/auth/login", &LoginRequest { username, password }).await
}

// ── Groups ────────────────────────────────────────────────────────────────

pub async fn list_groups(
    token: &str,
    page: u32,
    search: Option<String>,
) -> ApiResult<Paginated<GroupResponse>> {
    let mut q = format!("?page={page}&limit=20");
    if let Some(s) = &search {
        q.push_str(&format!("&search={}", urlenc(s)));
    }
    get_json(&format!("/api/groups{q}"), token).await
}

pub async fn get_group(token: &str, id: &str) -> ApiResult<GroupResponse> {
    get_json(&format!("/api/groups/{id}"), token).await
}

pub async fn create_group(token: &str, req: CreateGroupRequest) -> ApiResult<GroupResponse> {
    post_json("/api/groups", token, &req).await
}

pub async fn update_group(
    token: &str,
    id: &str,
    req: UpdateGroupRequest,
) -> ApiResult<GroupResponse> {
    put_json(&format!("/api/groups/{id}"), token, &req).await
}

pub async fn delete_group(token: &str, id: &str) -> ApiResult<()> {
    delete_req(&format!("/api/groups/{id}"), token).await
}

// ── Users ─────────────────────────────────────────────────────────────────

pub async fn list_users(
    token: &str,
    page: u32,
    search: Option<String>,
) -> ApiResult<Paginated<UserResponse>> {
    let mut q = format!("?page={page}&limit=20");
    if let Some(s) = &search {
        q.push_str(&format!("&search={}", urlenc(s)));
    }
    get_json(&format!("/api/users{q}"), token).await
}

pub async fn create_user(token: &str, req: CreateUserRequest) -> ApiResult<UserResponse> {
    post_json("/api/users", token, &req).await
}

pub async fn update_user(token: &str, id: &str, req: UpdateUserRequest) -> ApiResult<UserResponse> {
    put_json(&format!("/api/users/{id}"), token, &req).await
}

pub async fn delete_user(token: &str, id: &str) -> ApiResult<()> {
    delete_req(&format!("/api/users/{id}"), token).await
}

pub async fn reset_password(token: &str, id: &str) -> ApiResult<ResetPasswordResponse> {
    post_json_body(&format!("/api/users/{id}/reset-password"), token, "{}").await
}

// ── Collections ───────────────────────────────────────────────────────────

pub async fn list_collections(
    token: &str,
    page: u32,
    search: Option<String>,
) -> ApiResult<Paginated<CollectionResponse>> {
    let mut q = format!("?page={page}&limit=20");
    if let Some(s) = &search {
        q.push_str(&format!("&search={}", urlenc(s)));
    }
    get_json(&format!("/api/collections{q}"), token).await
}

pub async fn get_collection(token: &str, id: &str) -> ApiResult<CollectionResponse> {
    get_json(&format!("/api/collections/{id}"), token).await
}

pub async fn create_collection(
    token: &str,
    req: CreateCollectionRequest,
) -> ApiResult<CollectionResponse> {
    post_json("/api/collections", token, &req).await
}

pub async fn update_collection(
    token: &str,
    id: &str,
    req: UpdateCollectionRequest,
) -> ApiResult<CollectionResponse> {
    put_json(&format!("/api/collections/{id}"), token, &req).await
}

pub async fn delete_collection(token: &str, id: &str) -> ApiResult<()> {
    delete_req(&format!("/api/collections/{id}"), token).await
}

pub async fn duplicate_collection(token: &str, id: &str) -> ApiResult<CollectionResponse> {
    post_json_body(&format!("/api/collections/{id}/duplicate"), token, "{}").await
}

pub async fn transfer_collection(
    token: &str,
    id: &str,
    req: TransferOwnershipRequest,
) -> ApiResult<CollectionResponse> {
    put_json(&format!("/api/collections/{id}/transfer"), token, &req).await
}

// ── Collection shares ─────────────────────────────────────────────────────

pub async fn list_shares(token: &str, cid: &str) -> ApiResult<Vec<CollectionShareResponse>> {
    get_json(&format!("/api/collections/{cid}/shares"), token).await
}

pub async fn add_share(
    token: &str,
    cid: &str,
    req: CreateShareRequest,
) -> ApiResult<CollectionShareResponse> {
    post_json(&format!("/api/collections/{cid}/shares"), token, &req).await
}

pub async fn update_share(
    token: &str,
    cid: &str,
    sid: &str,
    req: UpdateShareRequest,
) -> ApiResult<CollectionShareResponse> {
    put_json(&format!("/api/collections/{cid}/shares/{sid}"), token, &req).await
}

pub async fn delete_share(token: &str, cid: &str, sid: &str) -> ApiResult<()> {
    delete_req(&format!("/api/collections/{cid}/shares/{sid}"), token).await
}

// ── Endpoints ─────────────────────────────────────────────────────────────

pub async fn list_endpoints(
    token: &str,
    cid: &str,
    page: u32,
    search: Option<String>,
) -> ApiResult<Paginated<EndpointResponse>> {
    let mut q = format!("?page={page}&limit=20");
    if let Some(s) = &search {
        q.push_str(&format!("&search={}", urlenc(s)));
    }
    get_json(&format!("/api/collections/{cid}/endpoints{q}"), token).await
}

pub async fn create_endpoint(
    token: &str,
    cid: &str,
    req: CreateEndpointRequest,
) -> ApiResult<EndpointResponse> {
    post_json(&format!("/api/collections/{cid}/endpoints"), token, &req).await
}

pub async fn update_endpoint(
    token: &str,
    cid: &str,
    eid: &str,
    req: UpdateEndpointRequest,
) -> ApiResult<EndpointResponse> {
    put_json(
        &format!("/api/collections/{cid}/endpoints/{eid}"),
        token,
        &req,
    )
    .await
}

pub async fn delete_endpoint(token: &str, cid: &str, eid: &str) -> ApiResult<()> {
    delete_req(&format!("/api/collections/{cid}/endpoints/{eid}"), token).await
}

pub async fn duplicate_endpoint(token: &str, cid: &str, eid: &str) -> ApiResult<EndpointResponse> {
    post_json_body(
        &format!("/api/collections/{cid}/endpoints/{eid}/duplicate"),
        token,
        "{}",
    )
    .await
}

// ── HTTP helpers ──────────────────────────────────────────────────────────

#[allow(unused_variables)]
async fn get_json<T: serde::de::DeserializeOwned>(url: &str, token: &str) -> ApiResult<T> {
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::get(url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_response(resp).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

#[allow(unused_variables)]
async fn post_json_anon<T, B>(url: &str, body: &B) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize,
{
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::post(url)
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_response(resp).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

#[allow(unused_variables)]
async fn post_json<T, B>(url: &str, token: &str, body: &B) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize,
{
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_response(resp).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

#[allow(unused_variables)]
async fn post_json_body<T: serde::de::DeserializeOwned>(
    url: &str,
    token: &str,
    body: &str,
) -> ApiResult<T> {
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::post(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .body(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_response(resp).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

#[allow(unused_variables)]
async fn put_json<T, B>(url: &str, token: &str, body: &B) -> ApiResult<T>
where
    T: serde::de::DeserializeOwned,
    B: serde::Serialize,
{
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::put(url)
            .header("Authorization", &format!("Bearer {token}"))
            .header("Content-Type", "application/json")
            .json(body)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;
        parse_response(resp).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

#[allow(unused_variables)]
async fn delete_req(url: &str, token: &str) -> ApiResult<()> {
    #[cfg(target_arch = "wasm32")]
    {
        let resp = gloo_net::http::Request::delete(url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await
            .map_err(|e| e.to_string())?;
        if resp.ok() {
            Ok(())
        } else {
            Err(format!("HTTP {}", resp.status()))
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    Err("ssr-only".into())
}

// ── Internal ──────────────────────────────────────────────────────────────

fn urlenc(s: &str) -> String {
    s.chars()
        .flat_map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' || c == '~' {
                vec![c]
            } else {
                format!("%{:02X}", c as u32).chars().collect()
            }
        })
        .collect()
}

#[cfg(target_arch = "wasm32")]
async fn parse_response<T: serde::de::DeserializeOwned>(
    resp: gloo_net::http::Response,
) -> ApiResult<T> {
    if resp.ok() {
        resp.json::<T>().await.map_err(|e| e.to_string())
    } else {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        // Try to extract message from our standard error format
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&body) {
            if let Some(msg) = v["error"]["message"].as_str() {
                return Err(msg.to_owned());
            }
        }
        Err(format!("HTTP {status}"))
    }
}
