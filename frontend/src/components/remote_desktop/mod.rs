pub mod connection_list;
pub mod connection_form;
pub mod session_viewer;
pub mod connection_groups;

use dioxus::prelude::*;
use connection_list::ConnectionList;
use connection_form::ConnectionForm;
use session_viewer::ActiveSessionsPanel;
use crate::services::backend_client::{BackendClient, Command};

#[derive(Clone, Copy, PartialEq)]
enum RemoteDesktopTab {
    Connections,
    ActiveSessions,
}

#[component]
pub fn RemoteDesktopView() -> Element {
    let app_state = use_context::<Signal<crate::state::AppState>>();
    let mut current_view = use_signal(|| RemoteDesktopTab::Connections);
    let mut selected_connection = use_signal(|| Option::<String>::None);
    let mut show_connection_dialog = use_signal(|| false);

    use_effect(move || { let client = BackendClient::new(); spawn(async move { let _ = client.send_command(Command::RemoteDesktopGetConnections).await; }); });
    rsx! {
        div { class: "view",
            div { class: "view-header",
                h1 { "Remote Desktop" }
                div { class: "view-actions",
                    button {
                        class: if *current_view.read() == RemoteDesktopTab::Connections {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| current_view.set(RemoteDesktopTab::Connections),
                        "Connections"
                    }
                    button {
                        class: if *current_view.read() == RemoteDesktopTab::ActiveSessions {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| current_view.set(RemoteDesktopTab::ActiveSessions),
                        "Active Sessions"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            selected_connection.set(None);
                            show_connection_dialog.set(true);
                        },
                        "+ New Connection"
                    }
                }
            }

            div { class: "view-content",
                match *current_view.read() {
                    RemoteDesktopTab::Connections => rsx! {
                        ConnectionList {
                            connections: app_state.read().remote_desktop.connections.clone(),
                            on_connect: move |id: String| {
                                selected_connection.set(Some(id.clone()));
                                let client = BackendClient::new();
                                let conn_id = id.clone();
                                spawn(async move {
                                    if let Err(e) = client.send_command(Command::RemoteDesktopConnect { connection_id: conn_id }).await {
                                        tracing::error!("Failed to connect to remote desktop: {}", e);
                                    }
                                });
                            },
                            on_edit: move |id: String| {
                                selected_connection.set(Some(id));
                                show_connection_dialog.set(true);
                            }
                        }
                    },
                    RemoteDesktopTab::ActiveSessions => rsx! {
                        ActiveSessionsPanel {}
                    },
                }
            }

            if *show_connection_dialog.read() {
                ConnectionForm {
                    connection_id: selected_connection.read().clone(),
                    on_save: move |conn: crate::state::RemoteConnection| {
                        let conn_id = selected_connection.read().clone();
                        let client = BackendClient::new();
                        
                        let protocol_str = match conn.protocol {
                            crate::state::ConnectionProtocol::Ssh => "Ssh",
                            crate::state::ConnectionProtocol::Vnc => "Vnc",
                        };

                        let payload = if let Some(id) = conn_id.clone() {
                            serde_json::json!({
                                "type": "remote_desktop_update_connection",
                                "id": id,
                                "request": {
                                    "name": conn.name.clone(),
                                    "host": conn.host.clone(),
                                    "port": conn.port,
                                    "username": conn.credentials.username.clone()
                                }
                            })
                        } else {
                            serde_json::json!({
                                "type": "remote_desktop_create_connection",
                                "request": {
                                    "name": conn.name.clone(),
                                    "protocol": protocol_str,
                                    "host": conn.host.clone(),
                                    "port": conn.port,
                                    "username": conn.credentials.username.clone()
                                }
                            })
                        };

                        spawn(async move {
                            if let Err(e) = client.send_ws_command(&payload).await {
                                tracing::error!("Failed to save connection: {}", e);
                            }
                        });

                        show_connection_dialog.set(false);
                        selected_connection.set(None);
                    },
                    on_cancel: move |_| {
                        show_connection_dialog.set(false);
                        selected_connection.set(None);
                    }
                }
            }
        }
    }
}
