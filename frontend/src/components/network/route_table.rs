use dioxus::prelude::*;
use crate::services::backend_client::{BackendClient, Route, RouteType};

#[component]
pub fn RouteTable() -> Element {
    let mut routes = use_signal(|| Vec::<Route>::new());
    let mut loading = use_signal(|| false);
    let mut show_add_dialog = use_signal(|| false);
    let mut show_edit_dialog = use_signal(|| false);
    let mut editing_route = use_signal(|| Option::<Route>::None);
    
    // Load routes from backend on mount
    use_effect(move || {
        spawn(async move {
            loading.set(true);
            let client = BackendClient::new();
            match client.get_routes().await {
                Ok(loaded_routes) => {
                    routes.set(loaded_routes);
                }
                Err(e) => {
                    tracing::error!("Failed to load routes: {}", e);
                }
            }
            loading.set(false);
        });
    });

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Routing Table" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            spawn(async move {
                                loading.set(true);
                                // Refresh routes
                                loading.set(false);
                            });
                        },
                        "🔄 Refresh"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_add_dialog.set(true),
                        "+ Add Route"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if routes.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🛣" }
                        div { class: "empty-title", "No routes configured" }
                        div { class: "empty-description",
                            "Routes control how network traffic is directed between different networks."
                        }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "Destination" }
                                th { "Gateway" }
                                th { "Interface" }
                                th { "Metric" }
                                th { "Type" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for route in routes.read().iter() {
                                RouteRow { 
                                    route: route.clone(),
                                    on_edit: move |r| {
                                        editing_route.set(Some(r));
                                        show_edit_dialog.set(true);
                                    },
                                    on_delete: move |dest: String| {
                                        tracing::info!("[FRONTEND] Deleting route: {}", dest);
                                        spawn(async move {
                                            let client = BackendClient::new();
                                            let cmd = serde_json::json!({
                                                "type": "network_delete_route",
                                                "destination": dest
                                            });
                                            if let Ok(_) = client.send_ws_command(&cmd).await {
                                                tracing::info!("[BACKEND] Route deleted: {}", dest);
                                                // Reload routes after deletion
                                                if let Ok(loaded_routes) = client.get_routes().await {
                                                    routes.set(loaded_routes);
                                                }
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if *show_edit_dialog.read() {
                if let Some(r) = editing_route.read().clone() {
                    EditRouteDialog {
                        route: r,
                        on_close: move |_| show_edit_dialog.set(false),
                        on_save: move |updated: Route| {
                            tracing::info!("[FRONTEND] Updating route: {}", updated.destination);
                            let mut route_list = routes.write();
                            if let Some(pos) = route_list.iter().position(|r| r.destination == updated.destination && r.gateway == updated.gateway) {
                                route_list[pos] = updated.clone();
                            }
                            tracing::info!("[BACKEND REQUEST] Update route: {:?}", updated);
                            show_edit_dialog.set(false);
                        }
                    }
                }
            }

            if *show_add_dialog.read() {
                AddRouteDialog {
                    on_close: move |_| show_add_dialog.set(false),
                    on_add: move |route: Route| {
                        tracing::info!("[FRONTEND] Route creation requested: {:?}", route);
                        spawn(async move {
                            let client = BackendClient::new();
                            let cmd = serde_json::json!({
                                "type": "network_add_route",
                                "request": {
                                    "destination": route.destination,
                                    "gateway": route.gateway,
                                    "interface": Some(route.interface),
                                    "metric": Some(route.metric)
                                }
                            });
                            if let Ok(_) = client.send_ws_command(&cmd).await {
                                tracing::info!("[BACKEND] Route added: {} via {}", route.destination, route.gateway);
                                // Reload routes after creation
                                if let Ok(loaded_routes) = client.get_routes().await {
                                    routes.set(loaded_routes);
                                }
                            }
                        });
                        show_add_dialog.set(false);
                    }
                }
            }
        }
    }
}

#[component]
fn RouteRow(route: Route, on_edit: EventHandler<Route>, on_delete: EventHandler<String>) -> Element {
    let dest = route.destination.clone();
    
    rsx! {
        tr { class: "table-row",
            td {
                div { class: "cell-main", "{route.destination}" }
            }
            td { "{route.gateway}" }
            td { "{route.interface}" }
            td { "{route.metric}" }
            td {
                span { class: "status-badge", "{route.route_type:?}" }
            }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button { 
                        class: "action-btn", 
                        title: "Edit",
                        onclick: move |_| {
                            on_edit.call(route.clone());
                        },
                        "✏" 
                    }
                    button { 
                        class: "action-btn action-danger", 
                        title: "Delete",
                        onclick: move |_| {
                            on_delete.call(dest.clone());
                        },
                        "🗑" 
                    }
                }
            }
        }
    }
}

#[component]
fn EditRouteDialog(
    route: Route,
    on_close: EventHandler<()>,
    on_save: EventHandler<Route>
) -> Element {
    let mut destination = use_signal(|| route.destination.clone());
    let mut gateway = use_signal(|| route.gateway.clone());
    let mut interface = use_signal(|| route.interface.clone());
    let mut metric = use_signal(|| route.metric.to_string());

    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Edit Route" }
                
                div { class: "form-group",
                    label { "Destination" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{destination}",
                        oninput: move |e| destination.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Gateway" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{gateway}",
                        oninput: move |e| gateway.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Interface" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{interface}",
                        oninput: move |e| interface.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Metric" }
                    input {
                        class: "input",
                        r#type: "number",
                        value: "{metric}",
                        oninput: move |e| metric.set(e.value().clone())
                    }
                }

                div { class: "form-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let updated = Route {
                                destination: destination().clone(),
                                gateway: gateway().clone(),
                                interface: interface().clone(),
                                metric: metric().parse().unwrap_or(0),
                                route_type: route.route_type.clone(),
                            };
                            on_save.call(updated);
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn AddRouteDialog(
    on_close: EventHandler<()>,
    on_add: EventHandler<Route>
) -> Element {
    let mut destination = use_signal(|| String::new());
    let mut gateway = use_signal(|| String::new());
    let mut interface = use_signal(|| String::new());
    let mut metric = use_signal(|| String::from("100"));

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 { "Add Route" }

                div { class: "form-group",
                    label { "Destination (CIDR)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., 192.168.2.0/24 or default",
                        value: "{destination}",
                        oninput: move |e| destination.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Gateway" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., 192.168.1.1",
                        value: "{gateway}",
                        oninput: move |e| gateway.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Interface (Optional)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., eth0",
                        value: "{interface}",
                        oninput: move |e| interface.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Metric" }
                    input {
                        class: "input",
                        r#type: "number",
                        value: "{metric}",
                        oninput: move |e| metric.set(e.value().clone())
                    }
                }

                div { class: "form-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let route = Route {
                                destination: destination.read().clone(),
                                gateway: gateway.read().clone(),
                                interface: interface.read().clone(),
                                metric: metric.read().parse().unwrap_or(100),
                                route_type: RouteType::Static,
                            };
                            tracing::info!("Route creation requested: dest={}, gateway={}, interface={}, metric={}",
                                route.destination, route.gateway, route.interface, route.metric);
                            on_add.call(route);
                        },
                        disabled: destination.read().is_empty() || gateway.read().is_empty(),
                        "Add Route"
                    }
                }
            }
        }
    }
}

