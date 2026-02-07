use crate::app::BackendBridge;
use crate::state::{AppState, Command, DockerCreateContainerConfig};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn ContainerList() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut search = use_signal(|| String::new());
    let mut show_create = use_signal(|| false);
    let mut create_name = use_signal(|| String::new());
    let mut create_image = use_signal(|| String::new());
    let mut create_ports = use_signal(|| String::new());
    let mut create_env = use_signal(|| String::new());
    let mut create_volumes = use_signal(|| String::new());
    let mut create_cmd = use_signal(|| String::new());
    let mut project_path = use_signal(|| String::new());

    let snapshot = app_state.read();
    let containers = snapshot.docker.containers.clone();
    let connected = snapshot.docker.connected;
    drop(snapshot);

    let search_val = search.read().to_lowercase();
    let filtered: Vec<_> = containers
        .iter()
        .filter(|c| {
            search_val.is_empty()
                || c.name.to_lowercase().contains(&search_val)
                || c.image.to_lowercase().contains(&search_val)
        })
        .collect();

    rsx! {
        div { class: "docker-view",
            div { class: "view-header",
                div { class: "view-title",
                    h1 { "Containers" }
                    span { class: "count-badge", "{containers.len()}" }
                }
                div { class: "view-actions",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Search containers...",
                        value: "{search}",
                        oninput: move |e| search.set(e.value().clone())
                    }
                    button { class: "btn primary", onclick: move |_| show_create.set(true), "Create" }
                    if !connected {
                        span { class: "status-badge status-error", "● Disconnected" }
                    }
                }
            }

            if containers.is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "⬢" }
                    h3 { "No containers" }
                    p { "There are no containers to display." }
                }
            } else {
                table { class: "docker-table",
                    thead {
                        tr {
                            th { class: "col-checkbox", input { r#type: "checkbox" } }
                            th { "Name" }
                            th { "Image" }
                            th { "Status" }
                            th { "Ports" }
                            th { class: "col-actions", "Actions" }
                        }
                    }
                    tbody {
                        {filtered.iter().map(|container| {
                            let name = container.name.clone();
                            let id = container.id.clone();
                            let state = container.state.clone();
                            let status_class = match state.as_str() {
                                "running" => "status-running",
                                "exited" | "stopped" => "status-stopped",
                                _ => "status-warning",
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

                            let on_delete = {
                                let id = id.clone();
                                let bridge = bridge.clone();
                                move |_| {
                                    bridge.send(Command::DockerRemoveContainer { id: id.clone(), force: true })
                                }
                            };

                            let on_logs = {
                                let id = id.clone();
                                let bridge = bridge.clone();
                                move |_| bridge.send(Command::DockerContainerLogs { id: id.clone() })
                            };

                            let ports_str = "—";

                            rsx! {
                                tr { class: "table-row", key: "{id}",
                                    td { class: "col-checkbox", input { r#type: "checkbox" } }
                                    td { class: "col-name",
                                        div { class: "cell-main", "{name}" }
                                        div { class: "cell-sub", "{id[..12].to_string()}" }
                                    }
                                    td { class: "col-image", "{container.image}" }
                                    td { class: "col-status",
                                        span { class: "status-badge {status_class}", "● {state}" }
                                    }
                                    td { class: "col-ports", "{ports_str}" }
                                    td { class: "col-actions",
                                        div { class: "action-buttons",
                                            if state == "running" {
                                                button { class: "action-btn", onclick: on_stop, title: "Stop", "⏸" }
                                                button { class: "action-btn", onclick: on_restart, title: "Restart", "↻" }
                                            } else {
                                                button { class: "action-btn action-primary", onclick: on_start, title: "Start", "▶" }
                                            }
                                            button { class: "action-btn", onclick: on_logs, title: "View Logs", "📄" }
                                            button { class: "action-btn action-danger", onclick: on_delete, title: "Delete", "🗑" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            if *show_create.read() {
                div { class: "modal-overlay", onclick: move |_| show_create.set(false),
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
                        h2 { "Create Container" }
                        div { class: "form-group",
                            label { "Project Folder (optional - auto-generate Dockerfile)" }
                            div { style: "display: flex; gap: 8px;",
                                input {
                                    class: "input",
                                    r#type: "text",
                                    placeholder: "/path/to/project",
                                    value: "{project_path}",
                                    style: "flex: 1;",
                                    oninput: move |e| project_path.set(e.value().clone())
                                }
                                button {
                                    class: "btn",
                                    onclick: {
                                        let mut project_path = project_path.clone();
                                        move |_| {
                                            if let Some(folder) = rfd::FileDialog::new().pick_folder() {
                                                project_path.set(folder.to_string_lossy().to_string());
                                            }
                                        }
                                    },
                                    "📁"
                                }
                                button {
                                    class: "btn primary",
                                    onclick: {
                                        let bridge = bridge.clone();
                                        let project_path = project_path.clone();
                                        move |_| {
                                            let path = project_path.read().trim().to_string();
                                            if !path.is_empty() {
                                                bridge.send(Command::DockerAnalyzeFolder { path });
                                            }
                                        }
                                    },
                                    "Analyze"
                                }
                            }
                        }
                        div { class: "form-group",
                            label { "Container Name" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "my-container",
                                value: "{create_name}",
                                oninput: move |e| create_name.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Image" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "nginx:latest",
                                value: "{create_image}",
                                oninput: move |e| create_image.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Port Mappings (one per line, e.g., 8080:80)" }
                            textarea {
                                class: "input",
                                rows: "3",
                                placeholder: "8080:80\n3000:3000",
                                value: "{create_ports}",
                                oninput: move |e| create_ports.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Environment Variables (one per line, e.g., KEY=value)" }
                            textarea {
                                class: "input",
                                rows: "3",
                                placeholder: "NODE_ENV=production\nPORT=3000",
                                value: "{create_env}",
                                oninput: move |e| create_env.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Volume Mounts (one per line, e.g., /host/path:/container/path)" }
                            textarea {
                                class: "input",
                                rows: "2",
                                placeholder: "/data:/app/data",
                                value: "{create_volumes}",
                                oninput: move |e| create_volumes.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Command (optional, space-separated)" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "npm start",
                                value: "{create_cmd}",
                                oninput: move |e| create_cmd.set(e.value().clone())
                            }
                        }
                        div { class: "modal-actions",
                            button { class: "btn", onclick: move |_| show_create.set(false), "Cancel" }
                            button {
                                class: "btn primary",
                                onclick: {
                                    let bridge = bridge.clone();
                                    let create_name = create_name.clone();
                                    let create_image = create_image.clone();
                                    let create_ports = create_ports.clone();
                                    let create_env = create_env.clone();
                                    let create_volumes = create_volumes.clone();
                                    let create_cmd = create_cmd.clone();
                                    move |_| {
                                        let ports: Vec<String> = create_ports
                                            .read()
                                            .lines()
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        let env: Vec<String> = create_env
                                            .read()
                                            .lines()
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        let volumes: Vec<String> = create_volumes
                                            .read()
                                            .lines()
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        let cmd_str = create_cmd.read().trim().to_string();
                                        let cmd = if cmd_str.is_empty() {
                                            None
                                        } else {
                                            Some(cmd_str.split_whitespace().map(|s| s.to_string()).collect())
                                        };
                                        
                                        bridge.send(Command::DockerCreateContainer {
                                            config: DockerCreateContainerConfig {
                                                name: create_name.read().clone(),
                                                image: create_image.read().clone(),
                                                ports,
                                                env,
                                                volumes,
                                                cmd,
                                            }
                                        });
                                        show_create.set(false);
                                    }
                                },
                                "Create"
                            }
                        }
                    }
                }
            }
        }
    }
}
