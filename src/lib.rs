pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(feature = "ssr")]
pub mod infrastructure;

pub use domain::errors::DomainError;

use adapters::ui::HomePage;
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_router::components::{Route, Router, Routes};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();
    view! {
        <Router>
            <Routes fallback=|| "Not Found">
                <Route path=leptos_router::path!("/") view=HomePage/>
            </Routes>
        </Router>
    }
}

#[cfg(feature = "ssr")]
pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    use leptos_meta::*;
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options=options.clone()/>
                <HashedStylesheet options id="leptos"/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn hydrate() {
    leptos::mount::hydrate_body(App);
}
