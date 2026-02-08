use crate::app::BackendBridge;
use crate::state::{AppState, Command, DockerScaffoldConfig};
use crate::components::docker::ManualBuildModal;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn ImageManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut search = use_signal(|| String::new());
    let mut pull_image = use_signal(|| "".to_string());
    let mut show_build = use_signal(|| false);
    let mut show_manual_build = use_signal(|| false);
    let mut build_context = use_signal(|| "".to_string());
    let mut build_tag = use_signal(|| "".to_string());
    let mut show_scaffold = use_signal(|| false);
    let mut scaffold_context = use_signal(|| "".to_string());
    let mut scaffold_base_image = use_signal(|| "".to_string());
    let mut scaffold_ports = use_signal(|| "".to_string());
    let mut scaffold_workdir = use_signal(|| "/app".to_string());
    let mut scaffold_cmd = use_signal(|| "".to_string());
    let mut scaffold_additional = use_signal(|| "".to_string());

    let snapshot = app_state.read();
    let images = snapshot.docker.images.clone();
    let action = snapshot.docker.action.clone();
    drop(snapshot);

    let search_val = search.read().to_lowercase();
    let filtered: Vec<_> = images
        .iter()
        .filter(|img| {
            search_val.is_empty()
                || img.repo_tags.iter().any(|tag| tag.to_lowercase().contains(&search_val))
        })
        .collect();

    let on_pull = {
        let bridge = bridge.clone();
        let mut app_state = app_state.clone();
        let pull_image = pull_image.clone();
        move |_| {
            let img = pull_image.read().trim().to_string();
            if !img.is_empty() {
                let mut st = app_state.write();
                st.docker.action.in_progress = true;
                st.docker.action.last_action = Some("docker pull".to_string());
                st.docker.action.last_ok = None;
                drop(st);
                bridge.send(Command::DockerPullImage { image: img });
            }
        }
    };

    let on_prune = {
        let bridge = bridge.clone();
        move |_| bridge.send(Command::DockerPruneImages)
    };

    rsx! {
        div { class: "docker-view",
            div { class: "view-header",
                div { class: "view-title",
                    h1 { "Images" }
                    span { class: "count-badge", "{images.len()}" }
                }
                div { class: "view-actions",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Search images...",
                        value: "{search}",
                        oninput: move |e| search.set(e.value().clone())
                    }
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Pull image (e.g., nginx:latest)",
                        value: "{pull_image}",
                        style: "width: 250px;",
                        oninput: move |e| pull_image.set(e.value().clone())
                    }
                    button { 
                        class: "btn primary", 
                        onclick: on_pull, 
                        disabled: action.in_progress,
                        style: "display: flex; align-items: center; gap: 8px;",
                        if action.in_progress && action.last_action.as_ref().map(|a| a.contains("pull")).unwrap_or(false) {
                            div { 
                                class: "loading-spinner", 
                                style: "width: 16px; height: 16px; border-width: 2px; margin: 0;" 
                            }
                            "Pulling..."
                        } else {
                            "Pull"
                        }
                    }
                    button { 
                        class: "btn", 
                        onclick: move |_| show_manual_build.set(true),
                        "Manual Build"
                    }
                    button { class: "btn", onclick: move |_| show_scaffold.set(true), "Scaffold" }
                    button { class: "btn", onclick: on_prune, "Clean Up" }
                }
            }

            // Show error banner if last action failed
            if let Some(false) = action.last_ok {
                if let Some(msg) = &action.message {
                    div { class: "alert alert-error",
                        style: "margin: 16px 24px;",
                        strong { "Error: " }
                        "{msg}"
                    }
                }
            }

            if images.is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "📦" }
                    h3 { "No images" }
                    p { "Pull an image to get started." }
                }
            } else {
                table { class: "docker-table",
                    thead {
                        tr {
                            th { class: "col-checkbox", input { r#type: "checkbox" } }
                            th { "Repository" }
                            th { "Tag" }
                            th { "Image ID" }
                            th { "Created" }
                            th { "Size" }
                            th { class: "col-actions", "Actions" }
                        }
                    }
                    tbody {
                        {filtered.iter().map(|img| {
                            let id = img.id.clone();
                            let repo_tag = img.repo_tags.first().unwrap_or(&"<none>".to_string()).clone();
                            let parts: Vec<&str> = repo_tag.split(':').collect();
                            let repo = parts.first().unwrap_or(&"<none>");
                            let tag = parts.get(1).unwrap_or(&"latest");
                            
                            let on_delete = {
                                let id = id.clone();
                                let bridge = bridge.clone();
                                move |_| {
                                    bridge.send(Command::DockerRemoveImage { id: id.clone(), force: false })
                                }
                            };
                            
                            let on_run = {
                                let repo_tag = repo_tag.clone();
                                let bridge = bridge.clone();
                                move |_| {
                                    bridge.send(Command::DockerRunImage { image: repo_tag.clone() })
                                }
                            };
                            
                            rsx! {
                                tr { class: "table-row", key: "{id}",
                                    td { class: "col-checkbox", input { r#type: "checkbox" } }
                                    td { class: "col-name",
                                        div { class: "cell-main", "{repo}" }
                                    }
                                    td { "{tag}" }
                                    td {
                                        div { class: "cell-sub", "{id[..12].to_string()}" }
                                    }
                                    td { class: "col-image", "—" }
                                    td {
                                        div { class: "cell-sub", 
                                            {format!("{:.1} MB", img.size as f64 / 1_000_000.0)}
                                        }
                                    }
                                    td { class: "col-actions",
                                        div { class: "action-buttons",
                                            button { class: "action-btn action-primary", onclick: on_run, title: "Run", "▶" }
                                            button { class: "action-btn action-danger", onclick: on_delete, title: "Delete", "🗑" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            if *show_build.read() {
                div { class: "modal-overlay", onclick: move |_| show_build.set(false),
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
                        h2 { "Build Image" }
                        div { class: "form-group",
                            label { "Context Path" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "/path/to/context",
                                value: "{build_context}",
                                oninput: move |e| build_context.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Image Tag (optional)" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "myapp:latest",
                                value: "{build_tag}",
                                oninput: move |e| build_tag.set(e.value().clone())
                            }
                        }
                        div { class: "modal-actions",
                            button { class: "btn", onclick: move |_| show_build.set(false), "Cancel" }
                            button {
                                class: "btn primary",
                                onclick: {
                                    let bridge = bridge.clone();
                                    let build_context = build_context.clone();
                                    let build_tag = build_tag.clone();
                                    move |_| {
                                        let tag_val = build_tag.read().trim().to_string();
                                        bridge.send(Command::DockerBuildImage {
                                            context_path: build_context.read().clone(),
                                            tag: if tag_val.is_empty() { None } else { Some(tag_val) },
                                        });
                                        show_build.set(false);
                                    }
                                },
                                "Build"
                            }
                        }
                    }
                }
            }

            if *show_scaffold.read() {
                div { class: "modal-overlay", onclick: move |_| show_scaffold.set(false),
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
                        h2 { "Scaffold Dockerfile" }
                        div { class: "form-group",
                            label { "Project Path" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "/path/to/project",
                                value: "{scaffold_context}",
                                oninput: move |e| scaffold_context.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Base Image" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "node:18, python:3.11, etc.",
                                value: "{scaffold_base_image}",
                                oninput: move |e| scaffold_base_image.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Ports (comma-separated)" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "3000, 8080",
                                value: "{scaffold_ports}",
                                oninput: move |e| scaffold_ports.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Working Directory" }
                            input {
                                class: "input",
                                r#type: "text",
                                value: "{scaffold_workdir}",
                                oninput: move |e| scaffold_workdir.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "CMD" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "npm start or python app.py",
                                value: "{scaffold_cmd}",
                                oninput: move |e| scaffold_cmd.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Additional Images (comma-separated)" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "redis:7, postgres:15",
                                value: "{scaffold_additional}",
                                oninput: move |e| scaffold_additional.set(e.value().clone())
                            }
                        }
                        div { class: "modal-actions",
                            button { class: "btn", onclick: move |_| show_scaffold.set(false), "Cancel" }
                            button {
                                class: "btn primary",
                                onclick: {
                                    let bridge = bridge.clone();
                                    let scaffold_context = scaffold_context.clone();
                                    let scaffold_base_image = scaffold_base_image.clone();
                                    let scaffold_ports = scaffold_ports.clone();
                                    let scaffold_workdir = scaffold_workdir.clone();
                                    let scaffold_cmd = scaffold_cmd.clone();
                                    let scaffold_additional = scaffold_additional.clone();
                                    move |_| {
                                        let ports: Vec<u16> = scaffold_ports
                                            .read()
                                            .split(',')
                                            .filter_map(|s| s.trim().parse().ok())
                                            .collect();
                                        let additional_images: Vec<String> = scaffold_additional
                                            .read()
                                            .split(',')
                                            .map(|s| s.trim().to_string())
                                            .filter(|s| !s.is_empty())
                                            .collect();
                                        
                                        bridge.send(Command::DockerScaffold {
                                            config: DockerScaffoldConfig {
                                                context_path: scaffold_context.read().clone(),
                                                base_image: scaffold_base_image.read().clone(),
                                                ports,
                                                workdir: Some(scaffold_workdir.read().clone()),
                                                cmd: if scaffold_cmd.read().is_empty() { None } else { Some(scaffold_cmd.read().clone()) },
                                                additional_images,
                                            }
                                        });
                                        show_scaffold.set(false);
                                    }
                                },
                                "Generate"
                            }
                        }
                    }
                }
            }
        }

        // Manual Build Modal
        ManualBuildModal { show: show_manual_build }
    }
}
