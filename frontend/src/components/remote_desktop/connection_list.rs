use dioxus::prelude::*;
use crate::state::{ConnectionProtocol, ConnectionStatus, RemoteConnection};

#[component]
pub fn ConnectionList(
    connections: Vec<RemoteConnection>,
    on_connect: EventHandler<String>,
    on_edit: EventHandler<String>
) -> Element {
    let loading = use_signal(|| false);
    let mut filter_protocol = use_signal(|| Option::<ConnectionProtocol>::None);

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Remote Connections" }
                div { class: "panel-actions",
                    button {
                        class: if filter_protocol.read().is_none() { "btn primary" } else { "btn" },
                        onclick: move |_| filter_protocol.set(None),
                        "All"
                    }
                    button {
                        class: if *filter_protocol.read() == Some(ConnectionProtocol::Ssh) {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| filter_protocol.set(Some(ConnectionProtocol::Ssh)),
                        "SSH"
                    }
                    button {
                        class: if *filter_protocol.read() == Some(ConnectionProtocol::Rdp) {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| filter_protocol.set(Some(ConnectionProtocol::Rdp)),
                        "RDP"
                    }
                    button {
                        class: if *filter_protocol.read() == Some(ConnectionProtocol::Vnc) {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| filter_protocol.set(Some(ConnectionProtocol::Vnc)),
                        "VNC"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if connections.is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "💻" }
                        div { class: "empty-title", "No remote connections configured" }
                        div { class: "empty-description", "Create a new connection to access remote servers via SSH, RDP, or VNC." }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "Name" }
                                th { "Protocol" }
                                th { "Address" }
                                th { "User" }
                                th { "Status" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for conn in connections.iter() {
                                ConnectionRow {
                                    connection: conn.clone(),
                                    on_connect: move |id| on_connect.call(id),
                                    on_edit: move |id| on_edit.call(id)
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn ConnectionRow(
    connection: RemoteConnection,
    on_connect: EventHandler<String>,
    on_edit: EventHandler<String>
) -> Element {
    let protocol_label = match connection.protocol {
        ConnectionProtocol::Ssh => "SSH",
        ConnectionProtocol::Rdp => "RDP",
        ConnectionProtocol::Vnc => "VNC",
        ConnectionProtocol::Spice => "SPICE",
    };

    let status_class = match connection.status {
        ConnectionStatus::Connected => "status-badge status-running",
        ConnectionStatus::Connecting => "status-badge status-warning",
        ConnectionStatus::Disconnected => "status-badge status-stopped",
        ConnectionStatus::Failed => "status-badge status-stopped",
    };

    let status_text = match connection.status {
        ConnectionStatus::Connected => "● Connected",
        ConnectionStatus::Connecting => "● Connecting",
        ConnectionStatus::Disconnected => "Disconnected",
        ConnectionStatus::Failed => "Failed",
    };

    // Clone connection.id for closure
    let conn_id = connection.id.clone();
    let conn_id_edit = connection.id.clone();

    rsx! {
        tr { class: "table-row",
            td {
                div { class: "cell-main", "{connection.name}" }
                if !connection.tags.is_empty() {
                    div { class: "cell-sub",
                        for tag in connection.tags.iter() {
                            span { class: "tag", "{tag}" }
                        }
                    }
                }
            }
            td {
                span { class: "status-badge", "{protocol_label}" }
            }
            td {
                div { class: "cell-main", "{connection.host}:{connection.port}" }
            }
            td { "{connection.credentials.username}" }
            td {
                span { class: status_class, "{status_text}" }
            }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button {
                        class: "action-btn",
                        title: "Connect",
                        onclick: move |_| on_connect.call(conn_id.clone()),
                        "▶"
                    }
                    button {
                        class: "action-btn",
                        title: "Edit",
                        onclick: move |_| on_edit.call(conn_id_edit.clone()),
                        "✏"
                    }
                    button { class: "action-btn action-danger", title: "Delete", "🗑" }
                }
            }
        }
    }
}

