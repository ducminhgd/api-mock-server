use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::{use_location, use_navigate};

use super::auth::AuthCtx;

#[component]
pub fn AppShell(children: Children) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let navigate = use_navigate();
    let auth_for_logout = auth.clone();

    let on_logout = move |_| {
        auth_for_logout.clear();
        navigate("/login", Default::default());
    };

    view! {
        <div class="app-layout">
            <nav class="sidebar">
                <div class="sidebar-logo">
                    "API Mock " <span>"Server"</span>
                </div>
                <div class="sidebar-nav">
                    <NavLink href="/collections" label="Collections" icon="📁" />
                    <NavLink href="/groups"      label="Groups"      icon="👥" />
                    <NavLink href="/users"       label="Users"       icon="👤" />
                </div>
                <div class="sidebar-footer">
                    <div class="user-info">
                        {move || auth.username.get().map(|u| format!("Signed in as {u}"))}
                    </div>
                    <button class="btn-logout" on:click=on_logout>"Sign out"</button>
                </div>
            </nav>
            <main class="main-content">
                {children()}
            </main>
        </div>
    }
}

#[component]
fn NavLink(href: &'static str, label: &'static str, icon: &'static str) -> impl IntoView {
    let location = use_location();
    let is_active = move || location.pathname.get().starts_with(href);
    view! {
        <A href=href attr:class=move || if is_active() { "sidebar-link active" } else { "sidebar-link" }>
            <span>{icon}</span>
            <span>{label}</span>
        </A>
    }
}

// ── Auth guard ─────────────────────────────────────────────────────────────

/// Wraps children in AppShell; redirects to /login when no token is present.
#[component]
pub fn Protected(children: Children) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let navigate = use_navigate();
    let token = auth.token; // RwSignal — Copy, usable in Effect and view

    Effect::new(move |_| {
        if token.get().is_none() {
            navigate("/login", Default::default());
        }
    });

    view! { <AppShell>{children()}</AppShell> }
}
