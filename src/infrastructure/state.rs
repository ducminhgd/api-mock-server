use std::sync::Arc;

use axum::extract::FromRef;
use leptos::config::LeptosOptions;

use crate::application::services::auth::AuthService;
use crate::application::services::collections::CollectionService;
use crate::application::services::endpoints::EndpointService;
use crate::application::services::groups::GroupService;
use crate::application::services::import_export::ImportExportService;
use crate::application::services::mocks::MockService;
use crate::application::services::users::UserService;
use crate::infrastructure::auth::jwt::JwtIssuer;

#[derive(Clone)]
pub struct AppState {
    pub collections: Arc<CollectionService>,
    pub endpoints: Arc<EndpointService>,
    pub groups: Arc<GroupService>,
    pub import_export: Arc<ImportExportService>,
    pub mocks: Arc<MockService>,
    pub users: Arc<UserService>,
    pub auth: Arc<AuthService>,
    pub jwt: Arc<JwtIssuer>,
    pub leptos_options: LeptosOptions,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}
