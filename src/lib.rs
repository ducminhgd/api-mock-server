pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(feature = "ssr")]
pub mod infrastructure;

use adapters::ui::HomePage;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};

#[component]
pub fn App() -> impl IntoView {
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
                <HydrationScripts options/>
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
