use leptos::prelude::*;
use leptos_router::components::A;
use leptos_router::hooks::use_params_map;

use super::api;
use super::auth::AuthCtx;
use super::components::{
    ConfirmDelete, EmptyState, ErrorBox, Modal, Pagination, Spinner, StatusBadge,
};
use super::endpoints::EndpointList;
use crate::application::dto::collection::{
    CollectionResponse, CreateCollectionRequest, UpdateCollectionRequest,
};
use crate::application::dto::collection_share::{CreateShareRequest, UpdateShareRequest};
use crate::application::dto::group::GroupResponse;
use crate::application::dto::pagination::Paginated;
use crate::application::dto::user::UserResponse;
use crate::domain::collection::{slugify_code, CollectionStatus, CollectionVisibility};
use crate::domain::collection_share::ShareRole;

// ── Collections list ──────────────────────────────────────────────────────

#[derive(Clone)]
enum Dialog {
    Create,
    Edit(CollectionResponse),
    Delete(CollectionResponse),
}

#[component]
pub fn CollectionsPage() -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");

    let page = RwSignal::new(1u32);
    let search = RwSignal::new(String::new());
    let search_input = RwSignal::new(String::new());
    let dialog = RwSignal::<Option<Dialog>>::new(None);
    let refresh = RwSignal::new(0u32);

    let collections_data = RwSignal::new(None::<Result<Paginated<CollectionResponse>, String>>);
    let token_signal = auth.token; // Copy — keep auth available for dialog handlers

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let pg = page.get();
        let s = search.get();
        let _ = refresh.get();
        collections_data.set(None);
        leptos::task::spawn_local(async move {
            let r =
                api::list_collections(&token, pg, if s.is_empty() { None } else { Some(s) }).await;
            collections_data.set(Some(r));
        });
    });

    view! {
        <div>
            <div class="page-header">
                <h1>"Collections"</h1>
                <div class="flex-gap">
                    <ImportButton refresh=refresh />
                    <button class="btn btn-primary"
                        on:click=move |_| dialog.set(Some(Dialog::Create))
                    >"+ New Collection"</button>
                </div>
            </div>

            <div class="toolbar">
                <input
                    type="text"
                    placeholder="Search collections…"
                    style="max-width:280px"
                    prop:value=move || search_input.get()
                    on:input=move |ev| search_input.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            page.set(1);
                            search.set(search_input.get());
                        }
                    }
                />
                <button class="btn btn-secondary" on:click=move |_| {
                    page.set(1);
                    search.set(search_input.get());
                }>"Search"</button>
            </div>

            {move || match collections_data.get() {
                None => view! { <Spinner /> }.into_any(),
                Some(Err(e)) => view! { <ErrorBox msg=e /> }.into_any(),
                Some(Ok(data)) => {
                    if data.data.is_empty() {
                        view! {
                            <EmptyState icon="📁" title="No collections"
                                subtitle="Create a collection to organise your mock endpoints." />
                        }.into_any()
                    } else {
                        let meta = data.meta.clone();
                        view! {
                            <div class="table-wrap">
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Code"</th>
                                            <th>"Visibility"</th>
                                            <th>"Status"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || data.data.clone()
                                            key=|c| c.id
                                            let:c
                                        >
                                            <CollectionRow
                                                col=c.clone()
                                                on_edit=move |c| dialog.set(Some(Dialog::Edit(c)))
                                                on_delete=move |c| dialog.set(Some(Dialog::Delete(c)))
                                            />
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                            <Pagination
                                page=page
                                total=meta.total
                                limit=meta.limit
                                on_page=move |p| page.set(p)
                            />
                        }.into_any()
                    }
                }
            }}

            {move || dialog.get().map(|d| {
                let close = move || dialog.set(None);
                let done  = move || { dialog.set(None); refresh.update(|n| *n += 1); };
                match d {
                    Dialog::Create =>
                        view! { <CollectionForm on_close=close on_done=done /> }.into_any(),
                    Dialog::Edit(c) =>
                        view! { <CollectionForm col=c on_close=close on_done=done /> }.into_any(),
                    Dialog::Delete(c) => {
                        let name  = c.name.clone();
                        let id    = c.id.to_string();
                        let token = auth.token_str();
                        view! {
                            <ConfirmDelete
                                what=name
                                on_confirm=move || {
                                    let id = id.clone();
                                    let tok = token.clone();
                                    leptos::task::spawn_local(async move {
                                        let _ = api::delete_collection(&tok, &id).await;
                                    });
                                    done();
                                }
                                on_cancel=close
                            />
                        }.into_any()
                    }
                }
            })}
        </div>
    }
}

#[component]
fn CollectionRow(
    col: CollectionResponse,
    on_edit: impl Fn(CollectionResponse) + 'static,
    on_delete: impl Fn(CollectionResponse) + 'static,
) -> impl IntoView {
    let c_edit = col.clone();
    let c_del = col.clone();
    let id = col.id.to_string();
    let vis = format!("{}", col.visibility).to_lowercase();
    view! {
        <tr>
            <td>
                <A href=format!("/collections/{id}")
                    attr:style="color:var(--accent);text-decoration:none;font-weight:500"
                >
                    {col.name.clone()}
                </A>
                {col.description.clone().map(|d| view! {
                    <div class="text-muted text-sm">{d}</div>
                })}
            </td>
            <td><span class="mono text-sm">{col.code.clone()}</span></td>
            <td>
                <span class=format!("badge badge-{vis}")>
                    {format!("{}", col.visibility)}
                </span>
            </td>
            <td><StatusBadge status=format!("{}", col.status) /></td>
            <td>
                <div class="td-actions">
                    <A href=format!("/collections/{id}") attr:class="btn btn-secondary btn-sm">"Open"</A>
                    <button class="btn btn-secondary btn-sm"
                        on:click=move |_| on_edit(c_edit.clone())
                    >"Edit"</button>
                    <button class="btn btn-danger btn-sm"
                        on:click=move |_| on_delete(c_del.clone())
                    >"Delete"</button>
                </div>
            </td>
        </tr>
    }
}

// ── Collection detail ─────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
enum DetailTab {
    Endpoints,
    Shares,
}

#[component]
pub fn CollectionDetailPage() -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let params = use_params_map();
    let coll_id = Signal::derive(move || params.with(|p| p.get("id").unwrap_or_default()));

    let tab = RwSignal::new(DetailTab::Endpoints);
    let refresh = RwSignal::new(0u32);
    let show_edit = RwSignal::new(false);

    let collection_data = RwSignal::new(None::<Result<CollectionResponse, String>>);
    let token_signal = auth.token; // Copy

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let cid = coll_id.get();
        let _ = refresh.get();
        collection_data.set(None);
        leptos::task::spawn_local(async move {
            let r = api::get_collection(&token, &cid).await;
            collection_data.set(Some(r));
        });
    });

    view! {
        <div>
            <div class="page-header">
                <div>
                    <a href="/collections"
                        style="font-size:.875rem;color:var(--text-muted);text-decoration:none"
                    >"← Collections"</a>
                    {move || match collection_data.get() {
                        None => view! { <h1>"Loading…"</h1> }.into_any(),
                        Some(Ok(c)) => view! { <h1 style="margin-top:.25rem">{c.name.clone()}</h1> }.into_any(),
                        Some(Err(_)) => view! { <h1>"Error"</h1> }.into_any(),
                    }}
                </div>
                <div class="flex-gap">
                    <ExportButton collection_id=coll_id />
                    <button class="btn btn-secondary"
                        on:click=move |_| show_edit.set(true)
                    >"Edit"</button>
                </div>
            </div>

            {move || {
                if let Some(Ok(c)) = collection_data.get() {
                    let vis = format!("{}", c.visibility).to_lowercase();
                    view! {
                        <div class="flex-gap" style="margin-bottom:1.25rem">
                            <span class=format!("badge badge-{vis}")>{format!("{}", c.visibility)}</span>
                            <StatusBadge status=format!("{}", c.status) />
                            <span class="mono text-sm" style="color:var(--text-muted)">{c.code.clone()}</span>
                            {c.description.map(|d| view! { <span class="text-muted text-sm">{d}</span> })}
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}

            <div class="tabs">
                <button
                    class=move || if tab.get() == DetailTab::Endpoints { "tab-btn active" } else { "tab-btn" }
                    on:click=move |_| tab.set(DetailTab::Endpoints)
                >"Endpoints"</button>
                <button
                    class=move || if tab.get() == DetailTab::Shares { "tab-btn active" } else { "tab-btn" }
                    on:click=move |_| tab.set(DetailTab::Shares)
                >"Sharing"</button>
            </div>

            {move || {
                let code = collection_data.get()
                    .and_then(|r| r.ok())
                    .map(|c| c.code)
                    .unwrap_or_else(|| coll_id.get());
                match tab.get() {
                DetailTab::Endpoints => view! {
                    <EndpointList collection_id=coll_id.get() collection_code=code />
                }.into_any(),
                DetailTab::Shares => view! {
                    <SharesPanel collection_id=coll_id.get() />
                }.into_any(),
            }
            }}

            {move || show_edit.get().then(|| {
                let close = move || show_edit.set(false);
                let done  = move || { show_edit.set(false); refresh.update(|n| *n += 1); };
                if let Some(Ok(c)) = collection_data.get() {
                    view! { <CollectionForm col=c on_close=close on_done=done /> }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            })}
        </div>
    }
}

// ── Shares panel ──────────────────────────────────────────────────────────

#[component]
fn SharesPanel(collection_id: String) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let cid = StoredValue::new(collection_id);
    let refresh = RwSignal::new(0u32);

    let shares_data = RwSignal::new(
        None::<
            Result<Vec<crate::application::dto::collection_share::CollectionShareResponse>, String>,
        >,
    );
    let all_users = RwSignal::new(Vec::<UserResponse>::new());
    let all_groups = RwSignal::new(Vec::<GroupResponse>::new());
    let token_signal = auth.token; // Copy

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let id = cid.get_value();
        let _ = refresh.get();
        shares_data.set(None);
        leptos::task::spawn_local(async move {
            let r = api::list_shares(&token, &id).await;
            shares_data.set(Some(r));
        });
    });

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        leptos::task::spawn_local(async move {
            if let Ok(us) = api::list_users_all(&token).await {
                all_users.set(us);
            }
            if let Ok(gs) = api::list_groups_all(&token).await {
                all_groups.set(gs);
            }
        });
    });

    let new_user_id = RwSignal::new(String::new());
    let new_role = RwSignal::new("viewer".to_string());
    let add_error = RwSignal::<Option<String>>::new(None);

    view! {
        <div>
            {move || match shares_data.get() {
                None => view! { <Spinner /> }.into_any(),
                Some(Err(e)) => view! { <ErrorBox msg=e /> }.into_any(),
                Some(Ok(data)) => {
                    if data.is_empty() {
                        view! {
                            <EmptyState icon="🔒" title="Not shared"
                                subtitle="Share this collection with users or groups." />
                        }.into_any()
                    } else {
                        let rows = data.clone();
                        view! {
                            <div class="table-wrap" style="margin-bottom:1.5rem">
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Shared with"</th>
                                            <th>"Role"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || rows.clone()
                                            key=|s| s.id
                                            let:s
                                        >
                                            <ShareRow
                                                share=s.clone()
                                                all_users=all_users
                                                all_groups=all_groups
                                                collection_id=cid.get_value()
                                                on_done=move || refresh.update(|n| *n += 1)
                                            />
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                        }.into_any()
                    }
                }
            }}

            <div style="max-width:520px">
                <p class="section-label">"Add share"</p>
                {move || add_error.get().map(|e| view! { <ErrorBox msg=e /> })}
                <div class="flex-gap">
                    <select
                        style="flex:1"
                        on:change=move |ev| new_user_id.set(event_target_value(&ev))
                    >
                        <option value="">
                            {move || if all_users.get().is_empty() { "Loading users…" } else { "— Select user —" }}
                        </option>
                        {move || {
                            let users = all_users.get();
                            let groups = all_groups.get();
                            users.into_iter().map(|u| {
                                let id = u.id.to_string();
                                let grp = u.group_id
                                    .and_then(|gid| groups.iter().find(|g| g.id == gid).map(|g| format!(" ({})", g.name)))
                                    .unwrap_or_default();
                                let label = format!("{}{}", u.username, grp);
                                let selected = new_user_id.get() == id;
                                view! { <option value=id selected=selected>{label}</option> }
                            }).collect_view()
                        }}
                    </select>
                    <select
                        prop:value=move || new_role.get()
                        on:change=move |ev| new_role.set(event_target_value(&ev))
                    >
                        <option value="viewer">"Viewer"</option>
                        <option value="editor">"Editor"</option>
                    </select>
                    <button class="btn btn-primary btn-sm" on:click=move |_| {
                        let uid_str = new_user_id.get();
                        if uid_str.is_empty() {
                            add_error.set(Some("Please select a user.".into()));
                            return;
                        }
                        let Ok(uid) = uid_str.parse::<uuid::Uuid>() else {
                            add_error.set(Some("Invalid user selection.".into()));
                            return;
                        };
                        let role = match new_role.get().as_str() {
                            "editor" => ShareRole::Editor,
                            _        => ShareRole::Viewer,
                        };
                        let token = auth.token_str();
                        let id = cid.get_value();
                        leptos::task::spawn_local(async move {
                            match api::add_share(&token, &id, CreateShareRequest {
                                user_id: Some(uid), group_id: None, role,
                            }).await {
                                Ok(_) => {
                                    new_user_id.set(String::new());
                                    add_error.set(None);
                                    refresh.update(|n| *n += 1);
                                }
                                Err(e) => add_error.set(Some(e)),
                            }
                        });
                    }>"Add"</button>
                </div>
            </div>
        </div>
    }
}

#[component]
fn ShareRow(
    share: crate::application::dto::collection_share::CollectionShareResponse,
    all_users: RwSignal<Vec<UserResponse>>,
    all_groups: RwSignal<Vec<GroupResponse>>,
    collection_id: String,
    on_done: impl Fn() + Clone + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let cid = StoredValue::new(collection_id);
    let sid = StoredValue::new(share.id.to_string());
    let role = RwSignal::new(format!("{}", share.role).to_lowercase());

    let share_user_id = share.user_id;
    let share_group_id = share.group_id;
    let who = move || {
        let users = all_users.get();
        let groups = all_groups.get();
        if let Some(uid) = share_user_id {
            users
                .iter()
                .find(|u| u.id == uid)
                .map(|u| {
                    let grp = u
                        .group_id
                        .and_then(|gid| {
                            groups
                                .iter()
                                .find(|g| g.id == gid)
                                .map(|g| format!(" ({})", g.name))
                        })
                        .unwrap_or_default();
                    format!("{}{}", u.username, grp)
                })
                .unwrap_or_else(|| format!("User: {uid}"))
        } else if let Some(gid) = share_group_id {
            groups
                .iter()
                .find(|g| g.id == gid)
                .map(|g| format!("Group: {}", g.name))
                .unwrap_or_else(|| format!("Group: {gid}"))
        } else {
            "Unknown".into()
        }
    };

    let on_remove = {
        let done = on_done.clone();
        let auth = auth.clone();
        move |_| {
            let token = auth.token_str();
            let cid_v = cid.get_value();
            let sid_v = sid.get_value();
            let done = done.clone();
            leptos::task::spawn_local(async move {
                let _ = api::delete_share(&token, &cid_v, &sid_v).await;
                done();
            });
        }
    };

    let on_role_change = {
        let done = on_done;
        let auth = auth.clone();
        move |ev: leptos::ev::Event| {
            let new_role_str = event_target_value(&ev);
            role.set(new_role_str.clone());
            let parsed = match new_role_str.as_str() {
                "editor" => ShareRole::Editor,
                _ => ShareRole::Viewer,
            };
            let token = auth.token_str();
            let cid_v = cid.get_value();
            let sid_v = sid.get_value();
            let done = done.clone();
            leptos::task::spawn_local(async move {
                let _ =
                    api::update_share(&token, &cid_v, &sid_v, UpdateShareRequest { role: parsed })
                        .await;
                done();
            });
        }
    };

    view! {
        <tr>
            <td class="text-sm">{who}</td>
            <td>
                <select
                    prop:value=move || role.get()
                    on:change=on_role_change
                >
                    <option value="viewer">"Viewer"</option>
                    <option value="editor">"Editor"</option>
                </select>
            </td>
            <td>
                <button class="btn btn-danger btn-sm" on:click=on_remove>"Remove"</button>
            </td>
        </tr>
    }
}

// ── Collection form ───────────────────────────────────────────────────────

#[component]
fn CollectionForm(
    #[prop(optional)] col: Option<CollectionResponse>,
    on_close: impl Fn() + Clone + Send + 'static,
    on_done: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let is_edit = col.is_some();

    let name = RwSignal::new(col.as_ref().map(|c| c.name.clone()).unwrap_or_default());
    let code = RwSignal::new(col.as_ref().map(|c| c.code.clone()).unwrap_or_default());
    let code_manual = RwSignal::new(is_edit); // edit mode: don't auto-fill
    let description = RwSignal::new(
        col.as_ref()
            .and_then(|c| c.description.clone())
            .unwrap_or_default(),
    );
    let visibility = RwSignal::new(
        col.as_ref()
            .map(|c| format!("{}", c.visibility).to_lowercase())
            .unwrap_or_else(|| "private".into()),
    );
    let status = RwSignal::new(
        col.as_ref()
            .map(|c| format!("{}", c.status).to_lowercase())
            .unwrap_or_else(|| "active".into()),
    );
    let error = RwSignal::<Option<String>>::new(None);
    let pending = RwSignal::new(false);
    let cid = col.as_ref().map(|c| c.id.to_string());

    let title = if is_edit {
        "Edit Collection"
    } else {
        "New Collection"
    };
    let close = on_close.clone();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let n = name.get();
        if n.is_empty() {
            error.set(Some("Name is required.".into()));
            return;
        }
        error.set(None);
        pending.set(true);

        let token = auth.token_str();
        let c = code.get();
        let code_val = if c.is_empty() { None } else { Some(c) };
        let desc = description.get();
        let desc_val = if desc.is_empty() { None } else { Some(desc) };
        let vis = visibility.get();
        let stat = status.get();
        let id = cid.clone();
        let done = on_done.clone();

        leptos::task::spawn_local(async move {
            let vis_parsed = match vis.as_str() {
                "public" => CollectionVisibility::Public,
                _ => CollectionVisibility::Private,
            };
            let result = if let Some(id) = id {
                let stat_parsed = match stat.as_str() {
                    "inactive" => Some(CollectionStatus::Inactive),
                    _ => Some(CollectionStatus::Active),
                };
                api::update_collection(
                    &token,
                    &id,
                    UpdateCollectionRequest {
                        name: Some(n),
                        code: code_val,
                        description: Some(desc_val),
                        status: stat_parsed,
                        visibility: Some(vis_parsed),
                    },
                )
                .await
                .map(|_| ())
            } else {
                api::create_collection(
                    &token,
                    CreateCollectionRequest {
                        name: n,
                        code: code_val,
                        description: desc_val,
                        visibility: Some(vis_parsed),
                    },
                )
                .await
                .map(|_| ())
            };

            match result {
                Ok(_) => done(),
                Err(e) => {
                    error.set(Some(e));
                    pending.set(false);
                }
            }
        });
    };

    view! {
        <Modal title=title.to_string() on_close=close>
            <form on:submit=on_submit>
                <div class="modal-body">
                    {move || error.get().map(|e| view! { <ErrorBox msg=e /> })}
                    <div class="form-group">
                        <label class="form-label">"Name"</label>
                        <input type="text" required
                            prop:value=move || name.get()
                            on:input=move |ev| {
                                let v = event_target_value(&ev);
                                if !code_manual.get() {
                                    code.set(slugify_code(&v));
                                }
                                name.set(v);
                            }
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Code"</label>
                        <input type="text" required placeholder="my-api"
                            prop:value=move || code.get()
                            on:input=move |ev| {
                                code_manual.set(true);
                                code.set(event_target_value(&ev));
                            }
                        />
                        <p class="form-hint">"Unique identifier used in mock URLs: /mocks/{code}/…"</p>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Description (optional)"</label>
                        <input type="text"
                            prop:value=move || description.get()
                            on:input=move |ev| description.set(event_target_value(&ev))
                        />
                    </div>
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:.75rem">
                        <div class="form-group">
                            <label class="form-label">"Visibility"</label>
                            <select
                                prop:value=move || visibility.get()
                                on:change=move |ev| visibility.set(event_target_value(&ev))
                            >
                                <option value="private">"Private"</option>
                                <option value="public">"Public"</option>
                            </select>
                        </div>
                        {is_edit.then(|| view! {
                            <div class="form-group">
                                <label class="form-label">"Status"</label>
                                <select
                                    prop:value=move || status.get()
                                    on:change=move |ev| status.set(event_target_value(&ev))
                                >
                                    <option value="active">"Active"</option>
                                    <option value="inactive">"Inactive"</option>
                                </select>
                            </div>
                        })}
                    </div>
                </div>
                <div class="modal-footer">
                    <button type="button" class="btn btn-secondary"
                        on:click=move |_| on_close()
                    >"Cancel"</button>
                    <button type="submit" class="btn btn-primary"
                        disabled=move || pending.get()
                    >
                        {move || if pending.get() { "Saving…" } else { "Save" }}
                    </button>
                </div>
            </form>
        </Modal>
    }
}

// ── Import button + modal ─────────────────────────────────────────────────

#[component]
fn ImportButton(refresh: RwSignal<u32>) -> impl IntoView {
    let show_modal = RwSignal::new(false);

    view! {
        <>
            <button class="btn btn-secondary" on:click=move |_| show_modal.set(true)>
                "⬆ Import"
            </button>
            {move || show_modal.get().then(|| view! {
                <ImportModal
                    refresh=refresh
                    on_close=move || show_modal.set(false)
                />
            })}
        </>
    }
}

#[component]
fn ImportModal(
    refresh: RwSignal<u32>,
    on_close: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let filename = RwSignal::new(String::new());
    let loading = RwSignal::new(false);
    let error = RwSignal::<Option<String>>::new(None);

    // Stores the raw file bytes picked by the user (wasm only).
    let file_bytes = RwSignal::<Option<Vec<u8>>>::new(None);

    let on_file_change = move |ev: leptos::ev::Event| {
        error.set(None);
        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use wasm_bindgen_futures::JsFuture;
            let input = ev
                .target()
                .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok());
            let files = input.and_then(|i| i.files());
            let file = files.and_then(|fl| fl.get(0));
            if let Some(file) = file {
                let name = file.name();
                filename.set(name);
                leptos::task::spawn_local(async move {
                    if let Ok(ab_val) = JsFuture::from(file.array_buffer()).await {
                        let arr = js_sys::Uint8Array::new(&ab_val);
                        file_bytes.set(Some(arr.to_vec()));
                    }
                });
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        let _ = ev;
    };

    let on_close2 = on_close.clone();
    let on_import = move |_| {
        let token = auth.token_str();
        let fname = filename.get();
        let bytes = file_bytes.get();

        if fname.is_empty() || bytes.is_none() {
            error.set(Some("Please select a file first.".into()));
            return;
        }

        loading.set(true);
        error.set(None);
        let on_close3 = on_close2.clone();
        leptos::task::spawn_local(async move {
            match upload_import(token, fname, bytes.unwrap()).await {
                Ok(_) => {
                    refresh.update(|n| *n += 1);
                    on_close3();
                }
                Err(e) => {
                    loading.set(false);
                    error.set(Some(e));
                }
            }
        });
    };

    let on_close_modal = on_close.clone();
    let on_close_cancel = on_close.clone();
    view! {
        <Modal title="Import Collection".to_string() on_close=on_close_modal>
            <div class="modal-body">
                <div class="import-format-info">
                    <p class="text-sm text-muted">"Supported formats:"</p>
                    <ul class="text-sm" style="margin:.5rem 0 1rem 1.2rem">
                        <li>"Postman Collection v2.1 ("<code>".json"</code>")"</li>
                        <li>"Bruno collection ZIP ("<code>".zip"</code>")"</li>
                        <li>"Bruno single request ("<code>".bru"</code>")"</li>
                    </ul>
                </div>

                <div class="import-file-picker">
                    <label class="btn btn-secondary" style="cursor:pointer;margin-bottom:0">
                        "Choose File"
                        <input
                            type="file"
                            accept=".json,.zip,.bru"
                            style="display:none"
                            on:change=on_file_change
                        />
                    </label>
                    {move || {
                        let name = filename.get();
                        if name.is_empty() {
                            view! { <span class="text-muted text-sm" style="margin-left:.75rem">"No file selected"</span> }.into_any()
                        } else {
                            view! { <span class="text-sm" style="margin-left:.75rem">{name}</span> }.into_any()
                        }
                    }}
                </div>

                {move || error.get().map(|e| view! { <div style="margin-top:.75rem"><ErrorBox msg=e /></div> })}
            </div>
            <div class="modal-footer">
                <button class="btn btn-ghost" on:click=move |_| on_close_cancel()
                    prop:disabled=move || loading.get()
                >"Cancel"</button>
                <button class="btn btn-primary" on:click=on_import
                    prop:disabled=move || loading.get() || filename.get().is_empty()
                >
                    {move || if loading.get() { "Importing…" } else { "Import" }}
                </button>
            </div>
        </Modal>
    }
}

#[allow(unused_variables)]
async fn upload_import(token: String, filename: String, bytes: Vec<u8>) -> Result<(), String> {
    #[cfg(target_arch = "wasm32")]
    {
        use js_sys::Uint8Array;

        let arr = Uint8Array::from(bytes.as_slice());
        let blob_parts = js_sys::Array::new();
        blob_parts.push(&arr.buffer());

        let blob =
            web_sys::Blob::new_with_u8_array_sequence(&blob_parts).map_err(|e| format!("{e:?}"))?;

        let form = web_sys::FormData::new().map_err(|e| format!("{e:?}"))?;
        form.append_with_blob_and_filename("file", &blob, &filename)
            .map_err(|e| format!("{e:?}"))?;

        let resp = gloo_net::http::Request::post("/api/collections/import")
            .header("Authorization", &format!("Bearer {token}"))
            .body(form)
            .map_err(|e| e.to_string())?
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if resp.ok() {
            return Ok(());
        }

        // Try to extract a meaningful error message from the JSON body.
        let status = resp.status();
        let msg = if let Ok(body) = resp.text().await {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&body) {
                val.get("message")
                    .or_else(|| val.get("error"))
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or(body)
            } else {
                body
            }
        } else {
            String::new()
        };

        let detail = if msg.is_empty() {
            format!("Import failed (HTTP {status})")
        } else {
            format!("Import failed: {msg}")
        };
        Err(detail)
    }
    #[cfg(not(target_arch = "wasm32"))]
    Ok(())
}

// ── Export button ─────────────────────────────────────────────────────────

#[component]
fn ExportButton(collection_id: Signal<String>) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let format = RwSignal::new("postman".to_string());

    let on_export = move |_| {
        let token = auth.token_str();
        let id = collection_id.get();
        let fmt = format.get();
        leptos::task::spawn_local(async move {
            trigger_download(token, id, fmt).await;
        });
    };

    view! {
        <div class="flex-gap">
            <select
                prop:value=move || format.get()
                on:change=move |ev| format.set(event_target_value(&ev))
                style="width:auto"
            >
                <option value="postman">"Postman"</option>
                <option value="bruno">"Bruno"</option>
            </select>
            <button class="btn btn-secondary" on:click=on_export>"⬇ Export"</button>
        </div>
    }
}

#[allow(unused_variables)]
async fn trigger_download(token: String, collection_id: String, format: String) {
    #[cfg(target_arch = "wasm32")]
    {
        let url = format!("/api/collections/{collection_id}/export?format={format}");
        let resp = gloo_net::http::Request::get(&url)
            .header("Authorization", &format!("Bearer {token}"))
            .send()
            .await;
        if let Ok(resp) = resp {
            if resp.ok() {
                let filename = resp
                    .headers()
                    .get("content-disposition")
                    .and_then(|h| {
                        h.split("filename=")
                            .nth(1)
                            .map(|s| s.trim_matches('"').to_string())
                    })
                    .unwrap_or_else(|| format!("collection.{format}"));

                if let Ok(bytes) = resp.binary().await {
                    download_bytes(&bytes, &filename);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn download_bytes(bytes: &[u8], filename: &str) {
    use js_sys::Uint8Array;

    let array = Uint8Array::from(bytes);
    let blob_parts = js_sys::Array::new();
    blob_parts.push(&array.buffer());

    if let Ok(blob) = web_sys::Blob::new_with_u8_array_sequence(&blob_parts) {
        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
            if let Some(window) = web_sys::window() {
                if let Some(doc) = window.document() {
                    if let Ok(el) = doc.create_element("a") {
                        use wasm_bindgen::JsCast;
                        if let Ok(a) = el.dyn_into::<web_sys::HtmlAnchorElement>() {
                            a.set_href(&url);
                            a.set_download(filename);
                            if let Some(body) = doc.body() {
                                let _ = body.append_child(&a);
                                a.click();
                                let _ = body.remove_child(&a);
                            }
                        }
                    }
                }
            }
            let _ = web_sys::Url::revoke_object_url(&url);
        }
    }
}
