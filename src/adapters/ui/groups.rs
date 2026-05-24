use leptos::prelude::*;

use super::api;
use super::auth::AuthCtx;
use super::components::{
    ConfirmDelete, EmptyState, ErrorBox, Modal, Pagination, Spinner, StatusBadge,
};
use crate::application::dto::group::{CreateGroupRequest, GroupResponse, UpdateGroupRequest};
use crate::application::dto::pagination::Paginated;

#[derive(Clone)]
enum Dialog {
    Create,
    Edit(GroupResponse),
    Delete(GroupResponse),
}

#[component]
pub fn GroupsPage() -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");

    let page = RwSignal::new(1u32);
    let search = RwSignal::new(String::new());
    let search_input = RwSignal::new(String::new());
    let dialog = RwSignal::<Option<Dialog>>::new(None);
    let refresh = RwSignal::new(0u32);

    let groups_data = RwSignal::new(None::<Result<Paginated<GroupResponse>, String>>);
    let token_signal = auth.token; // Copy — keep auth for delete handler

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let pg = page.get();
        let s = search.get();
        let _ = refresh.get();
        groups_data.set(None);
        leptos::task::spawn_local(async move {
            let r = api::list_groups(&token, pg, if s.is_empty() { None } else { Some(s) }).await;
            groups_data.set(Some(r));
        });
    });

    let on_search = move |_: leptos::ev::MouseEvent| {
        page.set(1);
        search.set(search_input.get());
    };

    view! {
        <div>
            <div class="page-header">
                <h1>"Groups"</h1>
                <button class="btn btn-primary"
                    on:click=move |_| dialog.set(Some(Dialog::Create))
                >"+ New Group"</button>
            </div>

            <div class="toolbar">
                <input
                    type="text"
                    placeholder="Search groups…"
                    style="max-width:280px"
                    prop:value=move || search_input.get()
                    on:input=move |ev| search_input.set(event_target_value(&ev))
                    on:keydown=move |ev| { if ev.key() == "Enter" { page.set(1); search.set(search_input.get()); } }
                />
                <button class="btn btn-secondary" on:click=on_search>"Search"</button>
                {move || (!search.get().is_empty()).then(|| view! {
                    <button class="btn btn-ghost" on:click=move |_| {
                        search.set(String::new());
                        search_input.set(String::new());
                        page.set(1);
                    }>"Clear"</button>
                })}
            </div>

            {move || match groups_data.get() {
                None => view! { <Spinner /> }.into_any(),
                Some(Err(e)) => view! { <ErrorBox msg=e /> }.into_any(),
                Some(Ok(data)) => {
                    if data.data.is_empty() {
                        view! {
                            <EmptyState icon="👥" title="No groups yet"
                                subtitle="Create a group to organise users." />
                        }.into_any()
                    } else {
                        let meta = data.meta.clone();
                        view! {
                            <div class="table-wrap">
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Name"</th>
                                            <th>"Description"</th>
                                            <th>"Status"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || data.data.clone()
                                            key=|g| g.id
                                            let:g
                                        >
                                            <GroupRow
                                                group=g.clone()
                                                on_edit=move |g| dialog.set(Some(Dialog::Edit(g)))
                                                on_delete=move |g| dialog.set(Some(Dialog::Delete(g)))
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
                        view! { <GroupForm on_close=close on_done=done /> }.into_any(),
                    Dialog::Edit(g) =>
                        view! { <GroupForm group=g on_close=close on_done=done /> }.into_any(),
                    Dialog::Delete(g) => {
                        let name = g.name.clone();
                        let id   = g.id.to_string();
                        let token = auth.token_str();
                        view! {
                            <ConfirmDelete
                                what=name
                                on_confirm=move || {
                                    let id = id.clone();
                                    let token = token.clone();
                                    leptos::task::spawn_local(async move {
                                        let _ = api::delete_group(&token, &id).await;
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
fn GroupRow(
    group: GroupResponse,
    on_edit: impl Fn(GroupResponse) + 'static,
    on_delete: impl Fn(GroupResponse) + 'static,
) -> impl IntoView {
    let g_edit = group.clone();
    let g_del = group.clone();
    view! {
        <tr>
            <td>{group.name.clone()}</td>
            <td class="text-muted">{group.description.clone().unwrap_or_default()}</td>
            <td><StatusBadge status=format!("{}", group.status) /></td>
            <td>
                <div class="td-actions">
                    <button class="btn btn-secondary btn-sm"
                        on:click=move |_| on_edit(g_edit.clone())
                    >"Edit"</button>
                    <button class="btn btn-danger btn-sm"
                        on:click=move |_| on_delete(g_del.clone())
                    >"Delete"</button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn GroupForm(
    #[prop(optional)] group: Option<GroupResponse>,
    on_close: impl Fn() + Clone + Send + 'static,
    on_done: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let is_edit = group.is_some();

    let name = RwSignal::new(group.as_ref().map(|g| g.name.clone()).unwrap_or_default());
    let description = RwSignal::new(
        group
            .as_ref()
            .and_then(|g| g.description.clone())
            .unwrap_or_default(),
    );
    let status = RwSignal::new(
        group
            .as_ref()
            .map(|g| format!("{}", g.status))
            .unwrap_or_else(|| "active".into()),
    );
    let error = RwSignal::<Option<String>>::new(None);
    let pending = RwSignal::new(false);
    let group_id = group.as_ref().map(|g| g.id.to_string());

    let title = if is_edit { "Edit Group" } else { "New Group" };
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
        let desc = description.get();
        let desc_val = if desc.is_empty() { None } else { Some(desc) };
        let gid = group_id.clone();
        let stat = status.get();
        let done = on_done.clone();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = gid {
                let status_parsed = match stat.as_str() {
                    "inactive" => Some(crate::domain::group::GroupStatus::Inactive),
                    _ => Some(crate::domain::group::GroupStatus::Active),
                };
                api::update_group(
                    &token,
                    &id,
                    UpdateGroupRequest {
                        name: Some(n),
                        description: Some(desc_val),
                        status: status_parsed,
                    },
                )
                .await
                .map(|_| ())
            } else {
                api::create_group(
                    &token,
                    CreateGroupRequest {
                        name: n,
                        description: desc_val,
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
                            on:input=move |ev| name.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Description (optional)"</label>
                        <input type="text"
                            prop:value=move || description.get()
                            on:input=move |ev| description.set(event_target_value(&ev))
                        />
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
