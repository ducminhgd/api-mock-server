use leptos::prelude::*;

use super::api;
use super::auth::AuthCtx;
use super::components::{
    ConfirmDelete, EmptyState, ErrorBox, Modal, Pagination, Spinner, StatusBadge,
};
use crate::application::dto::group::GroupResponse;
use crate::application::dto::pagination::Paginated;
use crate::application::dto::user::{CreateUserRequest, UpdateUserRequest, UserResponse};
use crate::domain::user::UserStatus;

#[derive(Clone)]
enum Dialog {
    Create,
    Edit(UserResponse),
    Delete(UserResponse),
    ResetPassword(UserResponse, String),
}

#[component]
pub fn UsersPage() -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");

    let page = RwSignal::new(1u32);
    let search = RwSignal::new(String::new());
    let search_input = RwSignal::new(String::new());
    let dialog = RwSignal::<Option<Dialog>>::new(None);
    let refresh = RwSignal::new(0u32);

    let users_data = RwSignal::new(None::<Result<Paginated<UserResponse>, String>>);
    let token_signal = auth.token; // Copy — keep for Effect, on_reset, and delete handlers
    let on_reset_token = auth.token; // Copy — for on_reset inline closure

    let all_groups = RwSignal::new(Vec::<GroupResponse>::new());
    let groups_token = auth.token;

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let pg = page.get();
        let s = search.get();
        let _ = refresh.get();
        users_data.set(None);
        leptos::task::spawn_local(async move {
            let r = api::list_users(&token, pg, if s.is_empty() { None } else { Some(s) }).await;
            users_data.set(Some(r));
        });
    });

    Effect::new(move |_| {
        let token = groups_token.get().unwrap_or_default();
        leptos::task::spawn_local(async move {
            if let Ok(gs) = api::list_groups_all(&token).await {
                all_groups.set(gs);
            }
        });
    });

    view! {
        <div>
            <div class="page-header">
                <h1>"Users"</h1>
                <button class="btn btn-primary"
                    on:click=move |_| dialog.set(Some(Dialog::Create))
                >"+ New User"</button>
            </div>

            <div class="toolbar">
                <input
                    type="text"
                    placeholder="Search users…"
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

            {move || match users_data.get() {
                None => view! { <Spinner /> }.into_any(),
                Some(Err(e)) => view! { <ErrorBox msg=e /> }.into_any(),
                Some(Ok(data)) => {
                    if data.data.is_empty() {
                        view! {
                            <EmptyState icon="👤" title="No users yet"
                                subtitle="Create the first user account." />
                        }.into_any()
                    } else {
                        let meta = data.meta.clone();
                        view! {
                            <div class="table-wrap">
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Username"</th>
                                            <th>"Group"</th>
                                            <th>"Role"</th>
                                            <th>"Status"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || data.data.clone()
                                            key=|u| u.id
                                            let:u
                                        >
                                            <UserRow
                                                user=u.clone()
                                                all_groups=all_groups
                                                on_edit=move |u| dialog.set(Some(Dialog::Edit(u)))
                                                on_delete=move |u| dialog.set(Some(Dialog::Delete(u)))
                                                on_reset={
                                                    let tok = on_reset_token;
                                                    let d = dialog;
                                                    move |user: UserResponse| {
                                                        let token_str = tok.get().unwrap_or_default();
                                                        let uid = user.id.to_string();
                                                        leptos::task::spawn_local(async move {
                                                            match api::reset_password(&token_str, &uid).await {
                                                                Ok(r) => d.set(Some(Dialog::ResetPassword(user, r.password))),
                                                                Err(e) => leptos::logging::warn!("reset_password error: {e}"),
                                                            }
                                                        });
                                                    }
                                                }
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
                        view! { <UserForm on_close=close on_done=done /> }.into_any(),
                    Dialog::Edit(u) =>
                        view! { <UserForm user=u on_close=close on_done=done /> }.into_any(),
                    Dialog::Delete(u) => {
                        let name = u.username.clone();
                        let id   = u.id.to_string();
                        let token = auth.token_str();
                        view! {
                            <ConfirmDelete
                                what=name
                                on_confirm=move || {
                                    let id = id.clone();
                                    let tok = token.clone();
                                    leptos::task::spawn_local(async move {
                                        let _ = api::delete_user(&tok, &id).await;
                                    });
                                    done();
                                }
                                on_cancel=close
                            />
                        }.into_any()
                    }
                    Dialog::ResetPassword(u, pw) => {
                        let close2 = close;
                        view! {
                            <Modal title="New Password".to_string() on_close=close>
                                <div class="modal-body">
                                    <p style="margin-bottom:.75rem">
                                        {format!("New password for {}:", u.username)}
                                    </p>
                                    <div class="code-block">{pw}</div>
                                    <p class="text-muted text-sm" style="margin-top:.75rem">
                                        "Copy this password now — it will not be shown again."
                                    </p>
                                </div>
                                <div class="modal-footer">
                                    <button class="btn btn-primary" on:click=move |_| close2()>"Done"</button>
                                </div>
                            </Modal>
                        }.into_any()
                    }
                }
            })}
        </div>
    }
}

#[component]
fn UserRow(
    user: UserResponse,
    all_groups: RwSignal<Vec<GroupResponse>>,
    on_edit: impl Fn(UserResponse) + 'static,
    on_delete: impl Fn(UserResponse) + 'static,
    on_reset: impl Fn(UserResponse) + 'static,
) -> impl IntoView {
    let u_edit = user.clone();
    let u_del = user.clone();
    let u_reset = user.clone();
    let gid = user.group_id;
    let group_name = move || {
        if let Some(gid) = gid {
            all_groups
                .get()
                .into_iter()
                .find(|g| g.id == gid)
                .map(|g| g.name)
                .unwrap_or_else(|| "—".into())
        } else {
            "—".into()
        }
    };
    view! {
        <tr>
            <td><span class="mono">{user.username.clone()}</span></td>
            <td class="text-muted text-sm">{group_name}</td>
            <td><span class=format!("badge badge-{}", format!("{}", user.role).to_lowercase())>
                {format!("{}", user.role)}
            </span></td>
            <td><StatusBadge status=format!("{}", user.status) /></td>
            <td>
                <div class="td-actions">
                    <button class="btn btn-secondary btn-sm"
                        on:click=move |_| on_edit(u_edit.clone())
                    >"Edit"</button>
                    <button class="btn btn-secondary btn-sm"
                        on:click=move |_| on_reset(u_reset.clone())
                    >"Reset PW"</button>
                    <button class="btn btn-danger btn-sm"
                        on:click=move |_| on_delete(u_del.clone())
                    >"Delete"</button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn UserForm(
    #[prop(optional)] user: Option<UserResponse>,
    on_close: impl Fn() + Clone + Send + 'static,
    on_done: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let is_edit = user.is_some();

    let username = RwSignal::new(
        user.as_ref()
            .map(|u| u.username.clone())
            .unwrap_or_default(),
    );
    let password = RwSignal::new(String::new());
    let group_id = RwSignal::new(
        user.as_ref()
            .and_then(|u| u.group_id.map(|id| id.to_string()))
            .unwrap_or_default(),
    );
    let status = RwSignal::new(
        user.as_ref()
            .map(|u| format!("{}", u.status))
            .unwrap_or_else(|| "active".into()),
    );
    let groups_list = RwSignal::new(Vec::<GroupResponse>::new());
    let error = RwSignal::<Option<String>>::new(None);
    let pending = RwSignal::new(false);
    let uid = user.as_ref().map(|u| u.id.to_string());

    let token_signal = auth.token; // Copy — for groups Effect
    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        leptos::task::spawn_local(async move {
            if let Ok(gs) = api::list_groups_all(&token).await {
                groups_list.set(gs);
            }
        });
    });

    let title = if is_edit { "Edit User" } else { "New User" };
    let close = on_close.clone();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let uname = username.get();
        if uname.is_empty() {
            error.set(Some("Username is required.".into()));
            return;
        }
        if !is_edit && password.get().is_empty() {
            error.set(Some("Password is required for new users.".into()));
            return;
        }
        error.set(None);
        pending.set(true);

        let token = auth.token_str();
        let pw = password.get();
        let stat = status.get();
        let gid_str = group_id.get();
        let parsed_gid: Option<uuid::Uuid> = if gid_str.is_empty() {
            None
        } else {
            gid_str.parse().ok()
        };
        let id = uid.clone();
        let done = on_done.clone();

        leptos::task::spawn_local(async move {
            let result = if let Some(id) = id {
                let status_parsed = match stat.as_str() {
                    "inactive" => Some(UserStatus::Inactive),
                    _ => Some(UserStatus::Active),
                };
                api::update_user(
                    &token,
                    &id,
                    UpdateUserRequest {
                        username: Some(uname),
                        group_id: Some(parsed_gid),
                        status: status_parsed,
                    },
                )
                .await
                .map(|_| ())
            } else {
                api::create_user(
                    &token,
                    CreateUserRequest {
                        username: uname,
                        password: pw,
                        group_id: parsed_gid,
                        role: None,
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
                        <label class="form-label">"Username"</label>
                        <input type="text" required
                            prop:value=move || username.get()
                            on:input=move |ev| username.set(event_target_value(&ev))
                        />
                    </div>
                    {(!is_edit).then(|| view! {
                        <div class="form-group">
                            <label class="form-label">"Password"</label>
                            <input type="password"
                                prop:value=move || password.get()
                                on:input=move |ev| password.set(event_target_value(&ev))
                            />
                        </div>
                    })}
                    <div class="form-group">
                        <label class="form-label">"Group (optional)"</label>
                        <select
                            on:change=move |ev| group_id.set(event_target_value(&ev))
                        >
                            <option value="" selected=move || group_id.get().is_empty()>
                                "— No group —"
                            </option>
                            {move || groups_list.get().into_iter().map(|g| {
                                let id = g.id.to_string();
                                let name = g.name.clone();
                                let selected = group_id.get() == id;
                                view! { <option value=id selected=selected>{name}</option> }
                            }).collect_view()}
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
