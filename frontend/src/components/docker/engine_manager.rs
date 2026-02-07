use crate::state::{AppState, Command};
use crate::app::BackendBridge;
use dioxus::prelude::*;

#[component]
pub fn EngineManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut auto_refresh = use_signal(|| false);

    // Reactive reads - these will update when state changes
    let docker_state = app_state.read().docker.clone();
    let is_connected = docker_state.connected;
    let engine_logs = app_state.read().ui.engine_logs.clone().unwrap_or_else(|| "No logs loaded yet. Click 'Refresh Logs' to fetch.".to_string());
    
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
    
    let bridge_refresh = bridge.clone();
    let on_refresh_logs = move |_| {
        tracing::info!("Refresh Logs button clicked");
        bridge_refresh.send(Command::DockerGetEngineLogs);
    };
    
    let bridge_toggle = bridge.clone();
    let on_toggle_auto_refresh = move |_| {
        let new_state = !auto_refresh();
        auto_refresh.set(new_state);
        if new_state {
            // Trigger immediate refresh when enabled
            bridge_toggle.send(Command::DockerGetEngineLogs);
        }
    };

    rsx! {
        div { class: "docker-section",
            div { class: "section-header",
                div { class: "header-left",
                    h2 { "Docker Engine" }
                    div { class: "status-badge", 
                        class: if is_connected { "status-running" } else { "status-stopped" },
                        {if is_connected { "● Running" } else { "● Stopped" }}
                    }
                }
                div { class: "header-actions",
                    button { 
                        class: "btn", 
                        onclick: on_refresh_logs,
                        "🔄 Refresh Logs" 
                    }
                    button { 
                        class: "btn", 
                        onclick: on_toggle_auto_refresh,
                        class: if auto_refresh() { "primary" } else { "" },
                        {if auto_refresh() { "⏸ Stop Auto-Refresh" } else { "▶ Auto-Refresh" }}
                    }
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
            
            div { class: "logs-panel",
                h3 { "Engine Logs" }
                div { class: "logs-container",
                    pre { class: "logs-content",
                        "{engine_logs}"
                    }
                }
            }
        }
    }
}
