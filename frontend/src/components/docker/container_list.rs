use crate::app::BackendBridge;
use crate::state::{AppState, Command};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn ContainerList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();

    let snapshot = app_state.read();
    let containers = snapshot.docker.containers.clone();
    let connected = snapshot.docker.connected;
    let last_error = snapshot.docker.last_error.clone();
    drop(snapshot);

    let status_label = if connected { "Connected" } else { "Offline" };
    let status_class = if connected { "status-running" } else { "status-stopped" };

    let on_refresh = {
        let bridge = bridge.clone();
        move |_| bridge.send(Command::DockerList)
    };

    rsx! {
        div { class: "panel",
            div { class: "header", style: "padding: 0 0 12px 0; border: none; background: transparent;",
                h2 { "Containers" }
                div { class: "action-bar",
                    span { class: "pill", span { class: "status-dot {status_class}" } "{status_label}" }
                    button { class: "btn ghost small", onclick: on_refresh, "Refresh" }
                }
            }
            if let Some(err) = last_error {
                div { class: "pill", style: "border-color: #e66b6b; color: #e66b6b;", "{err}" }
            }
            if containers.is_empty() {
                div { class: "muted", "No containers detected yet. Start Docker or refresh to sync." }
            } else {
                table { style: "width: 100%; border-spacing: 0 10px;",
                    tbody {
                        {containers.iter().map(|container| {
                            let name = container.name.clone();
                            let id = container.id.clone();
                            let state = container.state.clone();
                            let status_text = container.status.clone().unwrap_or_else(|| state.clone());
                            let status_class = match state.as_str() {
                                "running" => "status-running",
                                "exited" | "stopped" => "status-stopped",
                                _ => "status-unknown",
                            };

                            let on_start = {
                                let bridge = bridge.clone();
                                let id = id.clone();
                                move |_| bridge.send(Command::DockerStart { id: id.clone() })
                            };

                            let on_stop = {
                                let bridge = bridge.clone();
                                let id = id.clone();
                                move |_| bridge.send(Command::DockerStop { id: id.clone() })
                            };

                            let on_restart = {
                                let bridge = bridge.clone();
                                let id = id.clone();
                                move |_| bridge.send(Command::DockerRestart { id: id.clone() })
                            };

                            rsx! {
                                tr { class: "nav-link", key: "{id}",
                                    td {
                                        div { style: "display: flex; flex-direction: column; gap: 4px;",
                                            span { style: "font-weight: 600;", "{name}" }
                                            span { class: "muted", "{container.image}" }
                                        }
                                    }
                                    td {
                                        span { class: "pill",
                                            span { class: "status-dot {status_class}" }
                                            "{status_text}"
                                        }
                                    }
                                    td { class: "row-actions",
                                        if state == "running" {
                                            button { class: "btn ghost small", onclick: on_restart, "Restart" }
                                            button { class: "btn danger small", onclick: on_stop, "Stop" }
                                        } else {
                                            button { class: "btn primary small", onclick: on_start, "Start" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }
        }
    }
}
