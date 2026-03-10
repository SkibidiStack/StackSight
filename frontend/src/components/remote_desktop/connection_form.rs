use dioxus::prelude::*;

#[component]
pub fn ConnectionForm(
    connection_id: Option<String>,
    on_save: EventHandler<()>,
    on_cancel: EventHandler<()>
) -> Element {
    let is_edit = connection_id.is_some();
    
    let mut name = use_signal(|| String::new());
    let mut protocol = use_signal(|| ConnectionProtocol::Ssh);
    let mut host = use_signal(|| String::new());
    let mut port = use_signal(|| String::from("22"));
    let mut username = use_signal(|| String::new());
    let mut auth_method = use_signal(|| AuthMethod::Password);
    let mut password = use_signal(|| String::new());
    let mut private_key_path = use_signal(|| String::new());
    let mut tags = use_signal(|| String::new());

    // Update port when protocol changes
    use_effect(move || {
        let default_port = match *protocol.read() {
            ConnectionProtocol::Ssh => "22",
            ConnectionProtocol::Rdp => "3389",
            ConnectionProtocol::Vnc => "5900",
            ConnectionProtocol::Spice => "5900",
        };
        port.set(default_port.to_string());
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
                            protocol.set(match e.value().as_str() {
                                "rdp" => ConnectionProtocol::Rdp,
                                "vnc" => ConnectionProtocol::Vnc,
                                "spice" => ConnectionProtocol::Spice,
                                _ => ConnectionProtocol::Ssh,
                            });
                        },
                        option { value: "ssh", selected: *protocol.read() == ConnectionProtocol::Ssh, style: "background: #23262d; color: #e4e6eb;", "SSH" }
                        option { value: "rdp", selected: *protocol.read() == ConnectionProtocol::Rdp, style: "background: #23262d; color: #e4e6eb;", "RDP" }
                        option { value: "vnc", selected: *protocol.read() == ConnectionProtocol::Vnc, style: "background: #23262d; color: #e4e6eb;", "VNC" }
                        option { value: "spice", selected: *protocol.read() == ConnectionProtocol::Spice, style: "background: #23262d; color: #e4e6eb;", "SPICE" }
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
                            label { "Authentication Method" }
                            div { style: "display: flex; gap: 8px;",
                                button {
                                    r#type: "button",
                                    class: if *auth_method.read() == AuthMethod::Password {
                                        "btn primary"
                                    } else {
                                        "btn"
                                    },
                                    onclick: move |_| auth_method.set(AuthMethod::Password),
                                    "Password"
                                }
                                button {
                                    r#type: "button",
                                    class: if *auth_method.read() == AuthMethod::PrivateKey {
                                        "btn primary"
                                    } else {
                                        "btn"
                                    },
                                    onclick: move |_| auth_method.set(AuthMethod::PrivateKey),
                                    "Private Key"
                                }
                            }
                        }
                    }

                    if *auth_method.read() == AuthMethod::Password || *protocol.read() != ConnectionProtocol::Ssh {
                        div { class: "form-group",
                            label { "Password" }
                            input {
                                class: "input",
                                r#type: "password",
                                placeholder: "Enter password",
                                value: "{password}",
                                oninput: move |e| password.set(e.value().clone())
                            }
                            small { style: "display: block; margin-top: 4px; color: #888;",
                                "⚠ Passwords are encrypted and stored securely"
                            }
                        }
                    }

                    if *auth_method.read() == AuthMethod::PrivateKey && *protocol.read() == ConnectionProtocol::Ssh {
                        div { class: "form-group",
                            label { "Private Key Path" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "e.g., ~/.ssh/id_rsa",
                                value: "{private_key_path}",
                                oninput: move |e| private_key_path.set(e.value().clone())
                            }
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

                    if *protocol.read() == ConnectionProtocol::Rdp {
                        div { class: "form-group",
                            label {
                                input {
                                    r#type: "checkbox",
                                    checked: true,
                                    style: "margin-right: 8px;"
                                }
                                "Enable Clipboard Sharing"
                            }
                        }
                        div { class: "form-group",
                            label {
                                input {
                                    r#type: "checkbox",
                                    checked: true,
                                    style: "margin-right: 8px;"
                                }
                                "Enable Audio"
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
                        onclick: move |_| on_save.call(()),
                        if is_edit { "Save Changes" } else { "Create Connection" }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum ConnectionProtocol {
    Ssh,
    Rdp,
    Vnc,
    Spice,
}

#[derive(Clone, Copy, PartialEq)]
enum AuthMethod {
    Password,
    PrivateKey,
}
