use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use super::api;
use super::components::ErrorBox;

// ── Auth context ──────────────────────────────────────────────────────────

#[derive(Clone, Debug)]
pub struct AuthCtx {
    pub token: RwSignal<Option<String>>,
    pub username: RwSignal<Option<String>>,
}

impl Default for AuthCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl AuthCtx {
    pub fn new() -> Self {
        let token = read_storage("auth_token");
        let username = read_storage("auth_username");
        Self {
            token: RwSignal::new(token),
            username: RwSignal::new(username),
        }
    }

    pub fn set(&self, token: String, username: String) {
        write_storage("auth_token", &token);
        write_storage("auth_username", &username);
        self.token.set(Some(token));
        self.username.set(Some(username));
    }

    pub fn clear(&self) {
        remove_storage("auth_token");
        remove_storage("auth_username");
        self.token.set(None);
        self.username.set(None);
    }

    pub fn token_str(&self) -> String {
        self.token.get().unwrap_or_default()
    }
}

// ── localStorage helpers (WASM only) ──────────────────────────────────────

fn read_storage(key: &str) -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    return web_sys::window()
        .and_then(|w| w.local_storage().ok().flatten())
        .and_then(|s| s.get_item(key).ok().flatten());
    #[cfg(not(target_arch = "wasm32"))]
    let _ = key;
    #[cfg(not(target_arch = "wasm32"))]
    None
}

fn write_storage(key: &str, val: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.set_item(key, val);
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (key, val);
    }
}

fn remove_storage(key: &str) {
    #[cfg(target_arch = "wasm32")]
    if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
        let _ = s.remove_item(key);
    }
    #[cfg(not(target_arch = "wasm32"))]
    let _ = key;
}

// ── Login page ────────────────────────────────────────────────────────────

#[component]
pub fn LoginPage() -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let navigate = use_navigate();

    // If a token is already present (e.g. page reload with existing session),
    // skip the login form and go straight to the app.
    let token = auth.token;
    let nav_for_effect = navigate.clone();
    Effect::new(move |_| {
        if token.get().is_some() {
            nav_for_effect("/collections", Default::default());
        }
    });

    let username = RwSignal::new(String::new());
    let password = RwSignal::new(String::new());
    let error = RwSignal::<Option<String>>::new(None);
    let pending = RwSignal::new(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let u = username.get();
        let p = password.get();
        if u.is_empty() || p.is_empty() {
            error.set(Some("Username and password are required.".into()));
            return;
        }
        error.set(None);
        pending.set(true);

        let auth = auth.clone();
        let navigate = navigate.clone();
        leptos::task::spawn_local(async move {
            match api::login(u, p).await {
                Ok(resp) => {
                    auth.set(resp.token, resp.user.username);
                    navigate("/collections", Default::default());
                }
                Err(e) => {
                    error.set(Some(e));
                    pending.set(false);
                }
            }
        });
    };

    view! {
        <div class="login-page">
            <div class="login-card">
                <h1 class="login-title">"API Mock Server"</h1>
                <p class="login-subtitle">"Sign in to continue"</p>

                {move || error.get().map(|e| view! { <ErrorBox msg=e /> })}

                <form on:submit=on_submit>
                    <div class="form-group">
                        <label class="form-label" for="username">"Username"</label>
                        <input
                            id="username"
                            type="text"
                            autocomplete="username"
                            prop:value=move || username.get()
                            on:input=move |ev| username.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label" for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            autocomplete="current-password"
                            prop:value=move || password.get()
                            on:input=move |ev| password.set(event_target_value(&ev))
                        />
                    </div>
                    <button
                        type="submit"
                        class="btn btn-primary btn-lg"
                        style="width:100%; justify-content: center; margin-top: .5rem;"
                        disabled=move || pending.get()
                    >
                        {move || if pending.get() { "Signing in…" } else { "Sign in" }}
                    </button>
                </form>
            </div>
        </div>
    }
}
