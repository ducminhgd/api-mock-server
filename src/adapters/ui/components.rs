use leptos::prelude::*;

// ── Loading spinner ───────────────────────────────────────────────────────

#[component]
pub fn Spinner() -> impl IntoView {
    view! {
        <div class="spinner-wrap">
            <div class="spinner"></div>
        </div>
    }
}

// ── Error box ─────────────────────────────────────────────────────────────

#[component]
pub fn ErrorBox(msg: String) -> impl IntoView {
    view! {
        <div class="error-box">
            <span>"⚠ "</span>
            <span>{msg}</span>
        </div>
    }
}

// ── Empty state ───────────────────────────────────────────────────────────

#[component]
pub fn EmptyState(
    icon: &'static str,
    title: &'static str,
    #[prop(optional)] subtitle: &'static str,
) -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-icon">{icon}</div>
            <h3>{title}</h3>
            {(!subtitle.is_empty()).then(|| view! { <p>{subtitle}</p> })}
        </div>
    }
}

// ── Simple modal ──────────────────────────────────────────────────────────

#[component]
pub fn Modal(
    title: String,
    #[prop(optional)] large: bool,
    on_close: impl Fn() + Clone + Send + 'static,
    children: Children,
) -> impl IntoView {
    let cls = if large { "modal modal-lg" } else { "modal" };
    let close2 = on_close.clone();
    view! {
        <div class="modal-backdrop" on:click=move |e| {
            if e.target() == e.current_target() { close2(); }
        }>
            <div class=cls>
                <div class="modal-header">
                    <h2>{title}</h2>
                    <button class="modal-close" on:click=move |_| on_close()>"✕"</button>
                </div>
                {children()}
            </div>
        </div>
    }
}

// ── Pagination ─────────────────────────────────────────────────────────────

#[component]
pub fn Pagination(
    #[prop(into)] page: Signal<u32>,
    total: u64,
    limit: u32,
    on_page: impl Fn(u32) + Clone + Send + 'static,
) -> impl IntoView {
    let total_pages = (((total as f64) / (limit as f64)).ceil() as u32).max(1);
    let on_page_next = on_page.clone();

    view! {
        <div class="pagination">
            <span class="pagination-info">
                {format!("{total} item{}", if total == 1 { "" } else { "s" })}
            </span>
            <button class="btn btn-secondary btn-sm"
                prop:disabled={move || page.get() <= 1}
                on:click=move |_| {
                    let p = page.get();
                    if p > 1 { on_page(p - 1); }
                }
            >"← Prev"</button>
            <span class="text-muted text-sm">
                {move || format!("Page {} / {total_pages}", page.get())}
            </span>
            <button class="btn btn-secondary btn-sm"
                prop:disabled={move || page.get() >= total_pages}
                on:click=move |_| {
                    let p = page.get();
                    if p < total_pages { on_page_next(p + 1); }
                }
            >"Next →"</button>
        </div>
    }
}

// ── Status badge helpers ──────────────────────────────────────────────────

#[component]
pub fn StatusBadge(status: String) -> impl IntoView {
    let cls = format!("badge badge-{}", status.to_lowercase());
    view! { <span class=cls>{status}</span> }
}

#[component]
pub fn MethodBadge(method: String) -> impl IntoView {
    let cls = format!("badge badge-{}", method.to_lowercase());
    view! { <span class=cls>{method}</span> }
}

// ── Confirm delete dialog ─────────────────────────────────────────────────

#[component]
pub fn ConfirmDelete(
    what: String,
    on_confirm: impl Fn() + 'static,
    on_cancel: impl Fn() + 'static,
) -> impl IntoView {
    view! {
        <div class="modal-backdrop">
            <div class="modal">
                <div class="modal-header">
                    <h2>"Confirm delete"</h2>
                </div>
                <div class="modal-body">
                    <p>{format!("Delete \"{}\"? This cannot be undone.", what)}</p>
                </div>
                <div class="modal-footer">
                    <button class="btn btn-secondary" on:click=move |_| on_cancel()>"Cancel"</button>
                    <button class="btn btn-danger"    on:click=move |_| on_confirm()>"Delete"</button>
                </div>
            </div>
        </div>
    }
}
