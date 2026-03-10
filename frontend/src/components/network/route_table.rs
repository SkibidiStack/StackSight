use dioxus::prelude::*;
use dioxus::document;
use serde::{Serialize, Deserialize};

#[component]
pub fn RouteTable() -> Element {
    let mut routes = use_signal(|| Vec::<Route>::new());
    let mut loading = use_signal(|| false);
    let mut show_add_dialog = use_signal(|| false);
    let mut show_edit_dialog = use_signal(|| false);
    let mut editing_route = use_signal(|| Option::<Route>::None);
    
    // Load routes on mount
    use_effect(move || {
        spawn(async move {
            // Try to load from localStorage first
            let eval_result = document::eval(
                r#"localStorage.getItem('network_routes')"#
            ).await;
            
            if let Ok(result_value) = eval_result {
                if let Ok(result_str) = serde_json::from_value::<String>(result_value.clone()) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<Route>>(&result_str) {
                        routes.set(loaded);
                        return;
                    }
                }
            }
            
            // Generate mock data only if nothing in localStorage
            let mock_routes = vec![
                Route {
                    destination: "0.0.0.0/0".to_string(),
                    gateway: "192.168.1.1".to_string(),
                    interface: "eth0".to_string(),
                    metric: 100,
                    route_type: RouteType::Static,
                },
                Route {
                    destination: "192.168.1.0/24".to_string(),
                    gateway: "0.0.0.0".to_string(),
                    interface: "eth0".to_string(),
                    metric: 0,
                    route_type: RouteType::Static,
                },
                Route {
                    destination: "10.0.0.0/8".to_string(),
                    gateway: "192.168.1.254".to_string(),
                    interface: "eth0".to_string(),
                    metric: 200,
                    route_type: RouteType::Static,
                },
            ];
            routes.set(mock_routes);
        });
    });
    
    // Save to localStorage whenever routes change
    use_effect(move || {
        let route_list = routes.read().clone();
        spawn(async move {
            if let Ok(json) = serde_json::to_string(&route_list) {
                let escaped = json.replace('\'', "\\\\'");
                let script = format!("localStorage.setItem('network_routes', '{}')", escaped); 
                let _ = document::eval(&script).await;
            }
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
                                    on_delete: move |dest| {
                                        tracing::info!("[FRONTEND] Deleting route: {}", dest);
                                        routes.write().retain(|r| r.destination != dest);
                                        tracing::info!("[BACKEND REQUEST] Delete route: {}", dest);
                                        // Persist to localStorage
                                        spawn(async move {
                                            if let Ok(json) = serde_json::to_string(&*routes.read()) {
                                                let escaped = json.replace('\'', "\\\\'");
                                                let script = format!("localStorage.setItem('network_routes', '{}')", escaped); 
                                                let _ = document::eval(&script).await;
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
                        tracing::info!("[BACKEND REQUEST] Adding route: dest={}, gateway={}, interface={}, metric={}",
                            route.destination, route.gateway, route.interface, route.metric);
                        routes.write().push(route.clone());
                        tracing::info!("[FRONTEND] Route added to UI: {} via {}", route.destination, route.gateway);
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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Route {
    destination: String,
    gateway: String,
    interface: String,
    metric: u32,
    route_type: RouteType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum RouteType {
    Static,
    Dynamic,
    Default,
}
