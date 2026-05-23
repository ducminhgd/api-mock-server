#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use api_mock_server::adapters::http as http_adapters;
    use api_mock_server::application::services::auth::AuthService;
    use api_mock_server::application::services::collections::CollectionService;
    use api_mock_server::application::services::endpoints::EndpointService;
    use api_mock_server::application::services::groups::GroupService;
    use api_mock_server::application::services::import_export::ImportExportService;
    use api_mock_server::application::services::mocks::MockService;
    use api_mock_server::application::services::users::UserService;
    use api_mock_server::infrastructure::auth::jwt::JwtIssuer;
    use api_mock_server::infrastructure::auth::password::BcryptHasher;
    use api_mock_server::infrastructure::config::Config;
    use api_mock_server::infrastructure::db;
    use api_mock_server::infrastructure::db::collection_shares::SqlxCollectionShareRepository;
    use api_mock_server::infrastructure::db::collections::SqlxCollectionRepository;
    use api_mock_server::infrastructure::db::endpoints::SqlxEndpointRepository;
    use api_mock_server::infrastructure::db::groups::SqlxGroupRepository;
    use api_mock_server::infrastructure::db::users::SqlxUserRepository;
    use api_mock_server::infrastructure::state::AppState;
    use api_mock_server::{shell, App};
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let config = Config::from_env().expect("failed to load configuration");

    let pool = db::connect(&config.database_url)
        .await
        .expect("failed to connect to database");

    db::migrate(&pool).await.expect("failed to run migrations");

    let hasher: Arc<BcryptHasher> = Arc::new(BcryptHasher::default());
    let jwt = Arc::new(JwtIssuer::new(&config.jwt_secret));

    let group_repo = Arc::new(SqlxGroupRepository::new(pool.clone()));
    let user_repo = Arc::new(SqlxUserRepository::new(pool.clone()));
    let collection_repo = Arc::new(SqlxCollectionRepository::new(pool.clone()));
    let collection_share_repo = Arc::new(SqlxCollectionShareRepository::new(pool.clone()));
    let endpoint_repo = Arc::new(SqlxEndpointRepository::new(pool.clone()));

    let conf = get_configuration(None).expect("failed to load Leptos configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let app_state = AppState {
        collections: Arc::new(CollectionService::new(
            collection_repo.clone(),
            collection_share_repo.clone(),
            endpoint_repo.clone(),
            user_repo.clone(),
        )),
        endpoints: Arc::new(EndpointService::new(
            endpoint_repo.clone(),
            collection_repo.clone(),
            collection_share_repo.clone(),
            user_repo.clone(),
        )),
        import_export: Arc::new(ImportExportService::new(
            collection_repo.clone(),
            endpoint_repo.clone(),
            collection_share_repo.clone(),
        )),
        mocks: Arc::new(MockService::new(collection_repo, endpoint_repo)),
        groups: Arc::new(GroupService::new(group_repo)),
        users: Arc::new(UserService::new(user_repo.clone(), hasher.clone())),
        auth: Arc::new(AuthService::new(user_repo, hasher, jwt.clone())),
        jwt,
        leptos_options: leptos_options.clone(),
    };

    let app = Router::new()
        .nest("/api", http_adapters::api_router())
        .nest("/mocks", http_adapters::mocks_router())
        .leptos_routes(&app_state, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(shell))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));
    axum::serve(listener, app)
        .await
        .expect("server exited with error");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
