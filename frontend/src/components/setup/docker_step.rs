use dioxus::prelude::*;
use crate::state::SetupConfig;

#[component]
pub fn DockerStep(config: Signal<SetupConfig>) -> Element {
    let mut docker_detected = use_signal(|| false);
    let mut docker_status = use_signal(|| "Checking...".to_string());
    let mut checking = use_signal(|| false);

    let mut check_docker = move |_| {
        checking.set(true);
        spawn(async move {
            // Try common Docker socket locations
            let paths = if cfg!(windows) {
                vec!["//./pipe/docker_engine", "tcp://localhost:2375"]
            } else {
                vec!["/var/run/docker.sock", "~/.docker/run/docker.sock"]
            };

            for path in paths {
                // Simple file check - real app would try to connect
                if cfg!(unix) && std::path::Path::new(path).exists() {
                    docker_detected.set(true);
                    docker_status.set(format!("✓ Docker found at {}", path));
                    
                    let mut cfg = config();
                    cfg.docker_path = Some(path.to_string());
                    config.set(cfg);
                    checking.set(false);
                    return;
                }
            }

            docker_status.set("Docker not detected. You can install it later.".to_string());
            checking.set(false);
        });
    };

    // Auto-check on mount
    use_effect(move || {
        check_docker(());
    });

    rsx! {
        div { class: "wizard-step docker-step",
            h2 { "Docker Setup" }
            p { "Docker is required for container management features." }

            div { class: "detection-panel",
                div { class: "status-display",
                    if checking() {
                        div { class: "spinner", "Detecting Docker..." }
                    } else if docker_detected() {
                        div { class: "status-success",
                            "✓ {docker_status}"
                        }
                    } else {
                        div { class: "status-warning",
                            "⚠ {docker_status}"
                        }
                    }
                }

                button {
                    class: "btn btn-outline",
                    onclick: move |_| check_docker(()),
                    disabled: checking(),
                    "🔄 Re-check Docker"
                }
            }

            if !docker_detected() {
                div { class: "help-box",
                    h4 { "Need to install Docker?" }
                    ul {
                        li {
                            strong { "Windows/macOS: " }
                            a { href: "https://www.docker.com/products/docker-desktop", "Download Docker Desktop" }
                        }
                        li {
                            strong { "Linux: " }
                            "Run: "
                            code { "curl -fsSL https://get.docker.com | sh" }
                        }
                    }
                    p { class: "note",
                        "You can skip this step and install Docker later. "
                        "Other features will still work."
                    }
                }
            }

            div { class: "manual-config",
                h4 { "Manual Configuration (Optional)" }
                label { "Docker Socket/Endpoint:" }
                input {
                    r#type: "text",
                    class: "form-input",
                    value: config().docker_path.unwrap_or_default(),
                    placeholder: if cfg!(windows) { "//./pipe/docker_engine" } else { "/var/run/docker.sock" },
                    oninput: move |evt| {
                        let mut cfg = config();
                        cfg.docker_path = Some(evt.value().clone());
                        config.set(cfg);
                    }
                }
            }
        }
    }
}
