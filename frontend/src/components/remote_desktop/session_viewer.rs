use dioxus::prelude::*;

#[component]
pub fn ActiveSessionsPanel() -> Element {
    let sessions = use_signal(|| Vec::<ActiveSession>::new());
    let mut loading = use_signal(|| false);

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Active Remote Sessions" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            spawn(async move {
                                loading.set(true);
                                // Refresh sessions
                                loading.set(false);
                            });
                        },
                        "🔄 Refresh"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if sessions.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🔌" }
                        div { class: "empty-title", "No active sessions" }
                        div { class: "empty-description",
                            "Connect to a remote system to see active sessions here."
                        }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "Connection" }
                                th { "Protocol" }
                                th { "Host" }
                                th { "Duration" }
                                th { "Data Sent" }
                                th { "Data Received" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for session in sessions.read().iter() {
                                SessionRow { session: session.clone() }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SessionRow(session: ActiveSession) -> Element {
    rsx! {
        tr { class: "table-row",
            td {
                div { class: "cell-main", "{session.connection_name}" }
                if let Some(latency) = session.latency {
                    div { class: "cell-sub", "Latency: {latency}ms" }
                }
            }
            td {
                span { class: "status-badge", "{session.protocol:?}" }
            }
            td { "{session.host}" }
            td { "{session.duration}" }
            td { "{format_bytes(session.bytes_sent)}" }
            td { "{format_bytes(session.bytes_received)}" }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button { class: "action-btn", title: "View Details", "📊" }
                    button { class: "action-btn action-danger", title: "Disconnect", "🔌" }
                }
            }
        }
    }
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    format!("{:.2} {}", size, UNITS[unit_index])
}

#[derive(Clone, Debug, PartialEq)]
struct ActiveSession {
    session_id: String,
    connection_name: String,
    protocol: SessionProtocol,
    host: String,
    duration: String,
    bytes_sent: u64,
    bytes_received: u64,
    latency: Option<u32>,
}

#[derive(Clone, Debug, PartialEq)]
enum SessionProtocol {
    Ssh,
    Rdp,
    Vnc,
    Spice,
}
