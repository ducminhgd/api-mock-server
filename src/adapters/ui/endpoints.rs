use leptos::prelude::*;

use super::api;
use super::auth::AuthCtx;
use super::components::{
    ConfirmDelete, EmptyState, ErrorBox, MethodBadge, Modal, Pagination, Spinner, StatusBadge,
};
use crate::application::dto::endpoint::{
    CreateEndpointRequest, EndpointResponse, UpdateEndpointRequest,
};
use crate::application::dto::pagination::Paginated;
use crate::domain::endpoint::{EndpointStatus, HttpMethod};

#[derive(Clone)]
enum Dialog {
    Create,
    Edit(EndpointResponse),
    Delete(EndpointResponse),
}

#[component]
pub fn EndpointList(collection_id: String) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let cid = StoredValue::new(collection_id);

    let page = RwSignal::new(1u32);
    let search = RwSignal::new(String::new());
    let search_input = RwSignal::new(String::new());
    let dialog = RwSignal::<Option<Dialog>>::new(None);
    let refresh = RwSignal::new(0u32);

    let endpoints_data = RwSignal::new(None::<Result<Paginated<EndpointResponse>, String>>);
    let token_signal = auth.token; // Copy — keep auth for delete handler

    Effect::new(move |_| {
        let token = token_signal.get().unwrap_or_default();
        let pg = page.get();
        let s = search.get();
        let _ = refresh.get();
        let collection_id = cid.get_value();
        endpoints_data.set(None);
        leptos::task::spawn_local(async move {
            let r = api::list_endpoints(
                &token,
                &collection_id,
                pg,
                if s.is_empty() { None } else { Some(s) },
            )
            .await;
            endpoints_data.set(Some(r));
        });
    });

    view! {
        <div>
            <div class="page-header" style="margin-bottom:1rem">
                <h2 style="font-size:1.1rem;font-weight:600">"Endpoints"</h2>
                <button class="btn btn-primary btn-sm"
                    on:click=move |_| dialog.set(Some(Dialog::Create))
                >"+ Add Endpoint"</button>
            </div>

            <div class="toolbar">
                <input
                    type="text"
                    placeholder="Search endpoints…"
                    style="max-width:260px"
                    prop:value=move || search_input.get()
                    on:input=move |ev| search_input.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            page.set(1);
                            search.set(search_input.get());
                        }
                    }
                />
                <button class="btn btn-secondary btn-sm" on:click=move |_| {
                    page.set(1);
                    search.set(search_input.get());
                }>"Search"</button>
            </div>

            {move || match endpoints_data.get() {
                None => view! { <Spinner /> }.into_any(),
                Some(Err(e)) => view! { <ErrorBox msg=e /> }.into_any(),
                Some(Ok(data)) => {
                    if data.data.is_empty() {
                        view! {
                            <EmptyState icon="🔗" title="No endpoints"
                                subtitle="Add endpoints to start mocking responses." />
                        }.into_any()
                    } else {
                        let meta = data.meta.clone();
                        view! {
                            <div class="table-wrap">
                                <table>
                                    <thead>
                                        <tr>
                                            <th>"Method"</th>
                                            <th>"Path"</th>
                                            <th>"Name"</th>
                                            <th>"Status"</th>
                                            <th>"Delay"</th>
                                            <th>"Actions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <For
                                            each=move || data.data.clone()
                                            key=|e| e.id
                                            let:ep
                                        >
                                            <EndpointRow
                                                ep=ep.clone()
                                                on_edit=move |e| dialog.set(Some(Dialog::Edit(e)))
                                                on_delete=move |e| dialog.set(Some(Dialog::Delete(e)))
                                            />
                                        </For>
                                    </tbody>
                                </table>
                            </div>
                            <Pagination
                                page=meta.page
                                total=meta.total
                                limit=meta.limit
                                on_page=move |p| page.set(p)
                            />
                        }.into_any()
                    }
                }
            }}

            {move || dialog.get().map(|d| {
                let cid_v = cid.get_value();
                let close = move || dialog.set(None);
                let done  = move || { dialog.set(None); refresh.update(|n| *n += 1); };
                match d {
                    Dialog::Create => view! {
                        <EndpointForm
                            collection_id=cid_v
                            on_close=close
                            on_done=done
                        />
                    }.into_any(),
                    Dialog::Edit(ep) => view! {
                        <EndpointForm
                            collection_id=cid_v
                            endpoint=ep
                            on_close=close
                            on_done=done
                        />
                    }.into_any(),
                    Dialog::Delete(ep) => {
                        let name = ep.name.clone();
                        let eid  = ep.id.to_string();
                        let token = auth.token_str();
                        view! {
                            <ConfirmDelete
                                what=name
                                on_confirm=move || {
                                    let eid = eid.clone();
                                    let tok = token.clone();
                                    let cid2 = cid_v.clone();
                                    leptos::task::spawn_local(async move {
                                        let _ = api::delete_endpoint(&tok, &cid2, &eid).await;
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
fn EndpointRow(
    ep: EndpointResponse,
    on_edit: impl Fn(EndpointResponse) + 'static,
    on_delete: impl Fn(EndpointResponse) + 'static,
) -> impl IntoView {
    let e_edit = ep.clone();
    let e_del = ep.clone();
    let delay = if ep.delay_ms > 0 {
        format!("{}ms", ep.delay_ms)
    } else {
        "-".into()
    };
    view! {
        <tr>
            <td><MethodBadge method=format!("{}", ep.method) /></td>
            <td><span class="mono text-sm">{ep.path.clone()}</span></td>
            <td>{ep.name.clone()}</td>
            <td><StatusBadge status=format!("{}", ep.status) /></td>
            <td class="text-muted text-sm">{delay}</td>
            <td>
                <div class="td-actions">
                    <button class="btn btn-secondary btn-sm"
                        on:click=move |_| on_edit(e_edit.clone())
                    >"Edit"</button>
                    <button class="btn btn-danger btn-sm"
                        on:click=move |_| on_delete(e_del.clone())
                    >"Delete"</button>
                </div>
            </td>
        </tr>
    }
}

#[component]
fn EndpointForm(
    collection_id: String,
    #[prop(optional)] endpoint: Option<EndpointResponse>,
    on_close: impl Fn() + Clone + Send + 'static,
    on_done: impl Fn() + Clone + Send + 'static,
) -> impl IntoView {
    let auth = use_context::<AuthCtx>().expect("AuthCtx");
    let is_edit = endpoint.is_some();

    let name = RwSignal::new(
        endpoint
            .as_ref()
            .map(|e| e.name.clone())
            .unwrap_or_default(),
    );
    let method = RwSignal::new(
        endpoint
            .as_ref()
            .map(|e| format!("{}", e.method))
            .unwrap_or_else(|| "get".into()),
    );
    let path = RwSignal::new(
        endpoint
            .as_ref()
            .map(|e| e.path.clone())
            .unwrap_or_else(|| "/".into()),
    );
    let status_code = RwSignal::new(endpoint.as_ref().map(|e| e.status_code).unwrap_or(200u16));
    let delay_ms = RwSignal::new(endpoint.as_ref().map(|e| e.delay_ms).unwrap_or(0u32));
    let body = RwSignal::new(
        endpoint
            .as_ref()
            .and_then(|e| e.response_body.clone())
            .unwrap_or_default(),
    );
    let content_type = RwSignal::new(
        endpoint
            .as_ref()
            .and_then(|e| e.response_content_type.clone())
            .unwrap_or_else(|| "application/json".into()),
    );
    let headers = RwSignal::new(
        endpoint
            .as_ref()
            .and_then(|e| e.response_headers.clone())
            .unwrap_or_default(),
    );
    let ep_status = RwSignal::new(
        endpoint
            .as_ref()
            .map(|e| format!("{}", e.status))
            .unwrap_or_else(|| "active".into()),
    );
    let error = RwSignal::<Option<String>>::new(None);
    let pending = RwSignal::new(false);
    let eid = endpoint.as_ref().map(|e| e.id.to_string());
    let cid = collection_id;

    let title = if is_edit {
        "Edit Endpoint"
    } else {
        "New Endpoint"
    };
    let close = on_close.clone();

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        let n = name.get();
        let p = path.get();
        if n.is_empty() || p.is_empty() {
            error.set(Some("Name and path are required.".into()));
            return;
        }
        error.set(None);
        pending.set(true);

        let token = auth.token_str();
        let m = method.get();
        let sc = status_code.get();
        let dm = delay_ms.get();
        let b = body.get();
        let ct = content_type.get();
        let hdr = headers.get();
        let ep_stat = ep_status.get();
        let cid_c = cid.clone();
        let eid_c = eid.clone();
        let done = on_done.clone();

        leptos::task::spawn_local(async move {
            let method_parsed = match m.to_lowercase().as_str() {
                "post" => HttpMethod::Post,
                "put" => HttpMethod::Put,
                "patch" => HttpMethod::Patch,
                "delete" => HttpMethod::Delete,
                "head" => HttpMethod::Head,
                "options" => HttpMethod::Options,
                _ => HttpMethod::Get,
            };
            let result = if let Some(id) = eid_c {
                let status_parsed = match ep_stat.as_str() {
                    "inactive" => Some(EndpointStatus::Inactive),
                    _ => Some(EndpointStatus::Active),
                };
                api::update_endpoint(
                    &token,
                    &cid_c,
                    &id,
                    UpdateEndpointRequest {
                        name: Some(n),
                        method: Some(method_parsed),
                        path: Some(p),
                        status_code: Some(sc),
                        delay_ms: Some(dm),
                        response_headers: Some(if hdr.is_empty() { None } else { Some(hdr) }),
                        response_body: Some(if b.is_empty() { None } else { Some(b) }),
                        response_content_type: Some(if ct.is_empty() { None } else { Some(ct) }),
                        status: status_parsed,
                    },
                )
                .await
                .map(|_| ())
            } else {
                api::create_endpoint(
                    &token,
                    &cid_c,
                    CreateEndpointRequest {
                        name: n,
                        method: method_parsed,
                        path: p,
                        status_code: Some(sc),
                        delay_ms: Some(dm),
                        response_headers: if hdr.is_empty() { None } else { Some(hdr) },
                        response_body: if b.is_empty() { None } else { Some(b) },
                        response_content_type: if ct.is_empty() { None } else { Some(ct) },
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
        <Modal title=title.to_string() large=true on_close=close>
            <form on:submit=on_submit>
                <div class="modal-body">
                    {move || error.get().map(|e| view! { <ErrorBox msg=e /> })}
                    <div style="display:grid;grid-template-columns:1fr 1fr;gap:.75rem">
                        <div class="form-group">
                            <label class="form-label">"Name"</label>
                            <input type="text" required
                                prop:value=move || name.get()
                                on:input=move |ev| name.set(event_target_value(&ev))
                            />
                        </div>
                        <div class="form-group">
                            <label class="form-label">"Method"</label>
                            <select
                                prop:value=move || method.get()
                                on:change=move |ev| method.set(event_target_value(&ev))
                            >
                                <option value="get">"GET"</option>
                                <option value="post">"POST"</option>
                                <option value="put">"PUT"</option>
                                <option value="patch">"PATCH"</option>
                                <option value="delete">"DELETE"</option>
                                <option value="head">"HEAD"</option>
                                <option value="options">"OPTIONS"</option>
                            </select>
                        </div>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Path"</label>
                        <input type="text" required placeholder="/path/{param}"
                            prop:value=move || path.get()
                            on:input=move |ev| path.set(event_target_value(&ev))
                        />
                        <p class="form-hint">{r#"Use {<param>} for path parameters."#}</p>
                    </div>
                    <div style="display:grid;grid-template-columns:1fr 1fr 1fr;gap:.75rem">
                        <div class="form-group">
                            <label class="form-label">"HTTP Status"</label>
                            <input type="number" min="100" max="599"
                                prop:value=move || status_code.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse() {
                                        status_code.set(v);
                                    }
                                }
                            />
                        </div>
                        <div class="form-group">
                            <label class="form-label">"Delay (ms)"</label>
                            <input type="number" min="0"
                                prop:value=move || delay_ms.get()
                                on:input=move |ev| {
                                    if let Ok(v) = event_target_value(&ev).parse() {
                                        delay_ms.set(v);
                                    }
                                }
                            />
                        </div>
                        {is_edit.then(|| view! {
                            <div class="form-group">
                                <label class="form-label">"Status"</label>
                                <select
                                    prop:value=move || ep_status.get()
                                    on:change=move |ev| ep_status.set(event_target_value(&ev))
                                >
                                    <option value="active">"Active"</option>
                                    <option value="inactive">"Inactive"</option>
                                </select>
                            </div>
                        })}
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Content-Type"</label>
                        <input type="text" placeholder="application/json"
                            prop:value=move || content_type.get()
                            on:input=move |ev| content_type.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Response Headers (JSON object)"</label>
                        <textarea
                            placeholder="{\"X-Custom\": \"value\"}"
                            prop:value=move || headers.get()
                            on:input=move |ev| headers.set(event_target_value(&ev))
                            style="min-height:80px"
                        ></textarea>
                    </div>
                    <div class="form-group">
                        <label class="form-label">"Response Body"</label>
                        <textarea
                            placeholder="{\"key\": \"value\"}"
                            prop:value=move || body.get()
                            on:input=move |ev| body.set(event_target_value(&ev))
                            style="min-height:160px"
                        ></textarea>
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
