use crate::state::{ConnectionProtocol, ConnectionStatus, Credentials, RemoteConnection};
use dioxus::prelude::*;

#[component]
pub fn ConnectionForm(
    connection_id: Option<String>,
    on_save: EventHandler<RemoteConnection>,
    on_cancel: EventHandler<()>,
) -> Element {
    let app_state = use_context::<Signal<crate::state::AppState>>();

    // Find the connection if editing
    let existing = connection_id.as_ref().and_then(|id| {
        app_state
            .read()
            .remote_desktop
            .connections
            .iter()
            .find(|c| &c.id == id)
            .cloned()
    });

    let is_edit = existing.is_some();

    let mut name = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.name.clone())
            .unwrap_or_default()
    });
    let mut protocol = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.protocol.clone())
            .unwrap_or(ConnectionProtocol::Ssh)
    });
    let mut host = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.host.clone())
            .unwrap_or_default()
    });
    let mut port = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.port.to_string())
            .unwrap_or_else(|| "22".to_string())
    });
    let mut username = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.credentials.username.clone())
            .unwrap_or_default()
    });
    let mut tags = use_signal(|| {
        existing
            .as_ref()
            .map(|c| c.tags.join(", "))
            .unwrap_or_default()
    });

    let mut protocol_changed_manually = use_signal(|| false);

    // Update port when protocol changes, but only if user changed it manually
    use_effect(move || {
        let p = *protocol.read();
        if *protocol_changed_manually.read() {
            let default_port = match p {
                ConnectionProtocol::Ssh => "22",
                ConnectionProtocol::Vnc => "5900",
            };
            port.set(default_port.to_string());
        }
    });

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_cancel.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 {
                    if is_edit { "Edit Connection" } else { "New Connection" }
                }

                div { class: "form-group",
                    label { "Connection Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Production Server",
                        value: "{name}",
                        oninput: move |e| name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Protocol" }
                    select {
                        class: "input",
                        style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                        onchange: move |e| {
                            protocol_changed_manually.set(true);
                            protocol.set(match e.value().as_str() {
                                "vnc" => ConnectionProtocol::Vnc,
                                _ => ConnectionProtocol::Ssh,
                            });
                        },
                        option { value: "ssh", selected: *protocol.read() == ConnectionProtocol::Ssh, style: "background: #23262d; color: #e4e6eb;", "SSH" }
                        option { value: "vnc", selected: *protocol.read() == ConnectionProtocol::Vnc, style: "background: #23262d; color: #e4e6eb;", "VNC" }
                    }
                }

                div { class: "form-group",
                    label { "Hostname or IP Address" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "192.168.1.100",
                        value: "{host}",
                        oninput: move |e| host.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Port" }
                    input {
                        class: "input",
                        r#type: "number",
                        min: 1,
                        max: 65535,
                        value: "{port}",
                        oninput: move |e| port.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Username" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "admin",
                        value: "{username}",
                        oninput: move |e| username.set(e.value().clone())
                    }
                }

                if *protocol.read() == ConnectionProtocol::Ssh {
                    div { class: "form-group",
                        label {
                            input {
                                r#type: "checkbox",
                                style: "margin-right: 8px;"
                            }
                            "Enable X11 Forwarding"
                        }
                    }
                    div { class: "form-group",
                        label {
                            input {
                                r#type: "checkbox",
                                style: "margin-right: 8px;"
                            }
                            "Enable Compression"
                        }
                    }
                    div { class: "form-group",
                        label { "Port Forwarding (Optional)" }
                        textarea {
                            class: "input",
                            rows: 3,
                            placeholder: "Format: LOCAL_PORT:REMOTE_HOST:REMOTE_PORT\ne.g., 8080:localhost:80"
                        }
                    }
                }

                if *protocol.read() == ConnectionProtocol::Vnc {
                    div { class: "form-group",
                        label { "Quality" }
                        select { class: "input",
                            style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                            option { style: "background: #23262d; color: #e4e6eb;", "Low (fastest)" }
                            option { selected: true, style: "background: #23262d; color: #e4e6eb;", "Medium" }
                            option { style: "background: #23262d; color: #e4e6eb;", "High" }
                            option { style: "background: #23262d; color: #e4e6eb;", "Lossless (slowest)" }
                        }
                    }
                }

                div { class: "form-group",
                    label { "Tags (comma-separated)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., production, web-server",
                        value: "{tags}",
                        oninput: move |e| tags.set(e.value().clone())
                    }
                }

                div { class: "modal-actions",
                    button { class: "btn", onclick: move |_| on_cancel.call(()), "Cancel" }
                    button {
                        class: "btn primary",
                        onclick: move |_| {
                            let new_conn = RemoteConnection {
                                id: connection_id.clone().unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
                                name: name.read().clone(),
                                protocol: *protocol.read(),
                                host: host.read().clone(),
                                port: port.read().parse().unwrap_or(22),
                                credentials: Credentials { username: username.read().clone() },
                                status: ConnectionStatus::Disconnected,
                                last_connected: None,
                                favorite: false,
                                tags: tags.read().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect(),
                            };
                            on_save.call(new_conn);
                        },
                        if is_edit { "Save Changes" } else { "Create Connection" }
                    }
                }
            }
        }
    }

}
