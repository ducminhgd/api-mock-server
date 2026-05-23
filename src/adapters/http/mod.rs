// Adapters — HTTP layer (ssr only): Axum route handlers for /api/* and /mocks/*.
// Imports application use cases; never accesses infrastructure directly.
pub mod auth;
pub mod collections;
pub mod error;
pub mod extractor;
pub mod groups;
pub mod users;

use axum::Router;

use crate::infrastructure::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/collections", collections::router())
        .nest("/groups", groups::router())
        .nest("/users", users::router())
}
