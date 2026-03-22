use crate::app::BackendBridge;
use crate::state::{AppState, Command};
use dioxus::prelude::*;

#[component]
pub fn EngineManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();

    // Reactive reads - these will update when state changes
    let docker_state = app_state.read().docker.clone();
    let is_connected = docker_state.connected;

    let bridge_start = bridge.clone();
    let on_start_engine = move |_| {
        tracing::info!("Start Engine button clicked");
        bridge_start.send(Command::DockerStartEngine);
    };

    let bridge_stop = bridge.clone();
    let on_stop_engine = move |_| {
        tracing::info!("Stop Engine button clicked");
        bridge_stop.send(Command::DockerStopEngine);
    };

    rsx! {
        div { class: "docker-view",
            div { class: "view-header",
                div { class: "header-left",
                    h2 { "Docker Engine" }
                    div { class: "status-badge",
                        class: if is_connected { "status-running" } else { "status-stopped" },
                        {if is_connected { "● Running" } else { "● Stopped" }}
                    }
                }
                div { class: "header-actions",
                    if !is_connected {
                        button {
                            class: "btn primary",
                            onclick: on_start_engine,
                            "▶ Start Engine"
                        }
                    } else {
                        button {
                            class: "btn danger",
                            onclick: on_stop_engine,
                            "⏹ Stop Engine"
                        }
                    }
                }
            }

            div { class: "section-body", style: "padding: 24px;",
                div { class: "engine-info-panel",
                    div { class: "info-card",
                        h3 { "Engine Status" }
                        div { class: "info-row",
                            span { class: "info-label", "Connection:" }
                            span { class: "info-value", {if is_connected { "Connected" } else { "Disconnected" }} }
                        }
                        if let Some(error) = &docker_state.last_error {
                            div { class: "info-row",
                                span { class: "info-label", "Error:" }
                                span { class: "info-value error-text", "{error}" }
                            }
                        }
                    }

                    div { class: "info-card",
                        h3 { "Quick Stats" }
                        div { class: "info-row",
                            span { class: "info-label", "Containers:" }
                            span { class: "info-value", "{docker_state.containers.len()}" }
                        }
                        div { class: "info-row",
                            span { class: "info-label", "Images:" }
                            span { class: "info-value", "{docker_state.images.len()}" }
                        }
                        div { class: "info-row",
                            span { class: "info-label", "Volumes:" }
                            span { class: "info-value", "{docker_state.volumes.len()}" }
                        }
                        div { class: "info-row",
                            span { class: "info-label", "Networks:" }
                            span { class: "info-value", "{docker_state.networks.len()}" }
                        }
                    }
                }
            }
        }
    }
}
