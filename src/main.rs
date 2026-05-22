#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use api_mock_server::{shell, App};
    use axum::{http::StatusCode, Router};
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};

    let conf = get_configuration(None).expect("failed to load Leptos configuration");
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let api_router = Router::<leptos::config::LeptosOptions>::new()
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented") });

    let mocks_router = Router::<leptos::config::LeptosOptions>::new()
        .fallback(|| async { (StatusCode::NOT_IMPLEMENTED, "Not Implemented") });

    let app = Router::new()
        .nest("/api", api_router)
        .nest("/mocks", mocks_router)
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .unwrap_or_else(|e| panic!("failed to bind to {addr}: {e}"));
    axum::serve(listener, app)
        .await
        .expect("server exited with error");
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
