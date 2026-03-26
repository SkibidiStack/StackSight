pub mod connection_form;
pub mod connection_groups;
pub mod connection_list;
pub mod session_viewer;

use crate::services::backend_client::{BackendClient, Command};
use connection_form::ConnectionForm;
use connection_list::ConnectionList;
use dioxus::prelude::*;

#[component]
pub fn RemoteDesktopView() -> Element {
    let mut app_state = use_context::<Signal<crate::state::AppState>>();
    let mut selected_connection = use_signal(|| Option::<String>::None);
    let mut show_connection_dialog = use_signal(|| false);
    let mut requested_connections = use_signal(|| false);

    use_effect(move || {
        if requested_connections() {
            return;
        }

        if !app_state.read().remote_desktop.connections.is_empty() {
            requested_connections.set(true);
            return;
        }

        requested_connections.set(true);
        let client = BackendClient::new();
        let mut app_state_effect = app_state;
        spawn(async move {
            match client.send_command(Command::RemoteDesktopGetConnections).await {
                Ok(_) => {}
                Err(e) => {
                    let mut state = app_state_effect.write();
                    crate::state::push_toast(
                        &mut state.ui,
                        format!("Failed to load remote connections: {}", e),
                        crate::state::ToastType::Error,
                    );
                }
            }
        });
    });
    rsx! {
        div { class: "view",
            div { class: "view-header",
                h1 { "Remote Desktop" }
                div { class: "view-actions",
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
                ConnectionList {
                    connections: app_state.read().remote_desktop.connections.clone(),
                    on_connect: move |id: String| {
                        selected_connection.set(Some(id.clone()));
                        let client = BackendClient::new();
                        let conn_id = id.clone();
                        let mut app_state_connect = app_state;
                        {
                            let mut state = app_state_connect.write();
                            crate::state::push_toast(
                                &mut state.ui,
                                "Connecting to remote desktop...",
                                crate::state::ToastType::Info,
                            );
                        }
                        spawn(async move {
                            if let Err(e) = client.send_command(Command::RemoteDesktopConnect { connection_id: conn_id }).await {
                                tracing::error!("Failed to connect to remote desktop: {}", e);
                                let mut state = app_state_connect.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    format!("Remote desktop connection failed: {}", e),
                                    crate::state::ToastType::Error,
                                );
                            } else {
                                let mut state = app_state_connect.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    "Remote desktop connection request sent",
                                    crate::state::ToastType::Success,
                                );
                            }
                        });
                    },
                    on_edit: move |id: String| {
                        selected_connection.set(Some(id));
                        show_connection_dialog.set(true);
                    },
                    on_delete: move |id: String| {
                        let client = BackendClient::new();
                        let mut app_state_delete = app_state;
                        spawn(async move {
                            {
                                let mut state = app_state_delete.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    "Deleting remote connection...",
                                    crate::state::ToastType::Info,
                                );
                            }

                            if let Err(e) = client
                                .send_command(Command::RemoteDesktopDeleteConnection { id })
                                .await
                            {
                                tracing::error!("Failed to delete remote connection: {}", e);
                                let mut state = app_state_delete.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    format!("Failed to delete remote connection: {}", e),
                                    crate::state::ToastType::Error,
                                );
                            } else {
                                let mut state = app_state_delete.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    "Remote connection deleted",
                                    crate::state::ToastType::Success,
                                );
                            }
                        });
                    }
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
                                let mut state = app_state.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    format!("Failed to save remote connection: {}", e),
                                    crate::state::ToastType::Error,
                                );
                            } else {
                                let mut state = app_state.write();
                                crate::state::push_toast(
                                    &mut state.ui,
                                    "Remote connection saved",
                                    crate::state::ToastType::Success,
                                );
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
