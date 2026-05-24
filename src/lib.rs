#![recursion_limit = "512"]

pub mod adapters;
pub mod application;
pub mod domain;

#[cfg(feature = "ssr")]
pub mod infrastructure;

pub use domain::errors::DomainError;

use adapters::ui::{
    AppShell, AuthCtx, CollectionDetailPage, CollectionsPage, GroupsPage, LoginPage, Protected,
    UsersPage,
};
use leptos::prelude::*;
use leptos_meta::{provide_meta_context, Title};
use leptos_router::components::{Redirect, Route, Router, Routes};

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let auth = AuthCtx::new();
    provide_context(auth);

    view! {
        <Title text="API Mock Server"/>
        <Router>
            <Routes fallback=|| view! { <NotFound /> }>
                <Route path=leptos_router::path!("/login")      view=LoginPage />
                <Route path=leptos_router::path!("/")           view=|| view! { <Redirect path="/collections" /> } />
                <Route path=leptos_router::path!("/collections") view=|| view! {
                    <Protected>
                        <AppShell><CollectionsPage /></AppShell>
                    </Protected>
                }/>
                <Route path=leptos_router::path!("/collections/:id") view=|| view! {
                    <Protected>
                        <AppShell><CollectionDetailPage /></AppShell>
                    </Protected>
                }/>
                <Route path=leptos_router::path!("/groups") view=|| view! {
                    <Protected>
                        <AppShell><GroupsPage /></AppShell>
                    </Protected>
                }/>
                <Route path=leptos_router::path!("/users") view=|| view! {
                    <Protected>
                        <AppShell><UsersPage /></AppShell>
                    </Protected>
                }/>
            </Routes>
        </Router>
    }
}

#[component]
fn NotFound() -> impl IntoView {
    view! {
        <div style="display:flex;flex-direction:column;align-items:center;justify-content:center;min-height:100vh;gap:1rem">
            <h1 style="font-size:4rem;font-weight:700;color:var(--text-muted)">"404"</h1>
            <p style="color:var(--text-muted)">"Page not found"</p>
            <a href="/collections" style="color:var(--accent)">"← Back to Collections"</a>
        </div>
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
