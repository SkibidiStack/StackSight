use crate::app::BackendBridge;
use crate::state::{AppState, Command, DockerScaffoldConfig};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn ImageManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut pull_image = use_signal(|| "".to_string());
    let mut build_context = use_signal(|| "".to_string());
    let mut build_tag = use_signal(|| "".to_string());
    let mut scaffold_context = use_signal(|| "".to_string());
    let mut scaffold_base_image = use_signal(|| "".to_string());
    let mut scaffold_ports = use_signal(|| "".to_string());
    let mut scaffold_workdir = use_signal(|| "/app".to_string());
    let mut scaffold_cmd = use_signal(|| "".to_string());
    let mut scaffold_additional = use_signal(|| "".to_string());

    let snapshot = app_state.read();
    let images = snapshot.docker.images.clone();
    let last_error = snapshot.docker.last_error.clone();
    let action = snapshot.docker.action.clone();
    drop(snapshot);

    let mut action_text: Option<String> = None;
    let mut action_style: Option<String> = None;
    if let Some(action_name) = action.last_action.clone() {
        if action.in_progress {
            action_text = Some(format!("{action_name} in progress..."));
        } else if let Some(ok) = action.last_ok {
            if ok {
                action_text = Some(format!("{action_name} succeeded"));
                action_style = Some("border-color: #2dc7a2; color: #2dc7a2;".to_string());
            } else {
                let msg = action
                    .message
                    .clone()
                    .unwrap_or_else(|| format!("{action_name} failed"));
                action_text = Some(msg);
                action_style = Some("border-color: #e66b6b; color: #e66b6b;".to_string());
            }
        }
    }
    let action_style = action_style.unwrap_or_default();

    let on_pull = {
        let bridge = bridge.clone();
        let mut app_state = app_state.clone();
        let pull_image = pull_image.clone();
        move |_| {
            let image = pull_image.read().trim().to_string();
            if !image.is_empty() {
                let mut state = app_state.write();
                state.docker.action.in_progress = true;
                state.docker.action.last_action = Some("pull image".to_string());
                state.docker.action.last_ok = None;
                state.docker.action.message = None;
                bridge.send(Command::DockerPullImage { image })
            }
        }
    };

    let on_build = {
        let bridge = bridge.clone();
        let mut app_state = app_state.clone();
        let build_context = build_context.clone();
        let build_tag = build_tag.clone();
        move |_| {
            let context_path = build_context.read().trim().to_string();
            if !context_path.is_empty() {
                let tag_value = build_tag.read().trim().to_string();
                let tag = if tag_value.is_empty() { None } else { Some(tag_value) };
                let mut state = app_state.write();
                state.docker.action.in_progress = true;
                state.docker.action.last_action = Some("build image".to_string());
                state.docker.action.last_ok = None;
                state.docker.action.message = None;
                bridge.send(Command::DockerBuildImage { context_path, tag })
            }
        }
    };

    let on_clean = {
        let bridge = bridge.clone();
        let mut app_state = app_state.clone();
        move |_| {
            let mut state = app_state.write();
            state.docker.action.in_progress = true;
            state.docker.action.last_action = Some("clean images".to_string());
            state.docker.action.last_ok = None;
            state.docker.action.message = None;
            bridge.send(Command::DockerPruneImages)
        }
    };

    let on_scaffold = {
        let bridge = bridge.clone();
        let mut app_state = app_state.clone();
        let scaffold_context = scaffold_context.clone();
        let scaffold_base_image = scaffold_base_image.clone();
        let scaffold_ports = scaffold_ports.clone();
        let scaffold_workdir = scaffold_workdir.clone();
        let scaffold_cmd = scaffold_cmd.clone();
        let scaffold_additional = scaffold_additional.clone();
        move |_| {
            let context_path = scaffold_context.read().trim().to_string();
            let base_image = scaffold_base_image.read().trim().to_string();
            if !context_path.is_empty() && !base_image.is_empty() {
                let ports = scaffold_ports
                    .read()
                    .split(|c| c == ',' || c == ' ')
                    .filter_map(|p| p.trim().parse::<u16>().ok())
                    .collect::<Vec<_>>();

                let workdir_value = scaffold_workdir.read().trim().to_string();
                let workdir = if workdir_value.is_empty() { None } else { Some(workdir_value) };

                let cmd_value = scaffold_cmd.read().trim().to_string();
                let cmd = if cmd_value.is_empty() { None } else { Some(cmd_value) };

                let additional_images = scaffold_additional
                    .read()
                    .split(|c| c == ',' || c == ' ')
                    .map(|i| i.trim().to_string())
                    .filter(|i| !i.is_empty())
                    .collect::<Vec<_>>();

                let mut state = app_state.write();
                state.docker.action.in_progress = true;
                state.docker.action.last_action = Some("scaffold dockerfile".to_string());
                state.docker.action.last_ok = None;
                state.docker.action.message = None;
                bridge.send(Command::DockerScaffold {
                    config: DockerScaffoldConfig {
                        context_path,
                        base_image,
                        ports,
                        workdir,
                        cmd,
                        additional_images,
                    },
                })
            }
        }
    };

    let on_refresh = {
        let bridge = bridge.clone();
        move |_| bridge.send(Command::DockerListImages)
    };

    rsx! {
        div { class: "panel",
            h2 { "Images" }
            div { class: "muted", "Pull, tag, and clean images." }
            if let Some(err) = last_error {
                div { class: "pill", style: "border-color: #e66b6b; color: #e66b6b;", "{err}" }
            }
            if let Some(text) = action_text {
                div { class: "pill", style: "{action_style}", "{text}" }
            }
            div { style: "display: grid; gap: 10px; margin-top: 10px;",
                div { style: "display: grid; gap: 6px;",
                    span { class: "muted", "Pull image" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g. nginx:latest",
                        value: "{pull_image.read()}",
                        oninput: move |evt| pull_image.set(evt.value())
                    }
                }
                div { style: "display: grid; gap: 6px;",
                    span { class: "muted", "Build context" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "/path/to/context",
                        value: "{build_context.read()}",
                        oninput: move |evt| build_context.set(evt.value())
                    }
                }
                div { style: "display: grid; gap: 6px;",
                    span { class: "muted", "Build tag (optional)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "my-image:latest",
                        value: "{build_tag.read()}",
                        oninput: move |evt| build_tag.set(evt.value())
                    }
                }
                div { style: "display: grid; gap: 6px;",
                    span { class: "muted", "Scaffold Dockerfile" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "/path/to/project",
                        value: "{scaffold_context.read()}",
                        oninput: move |evt| scaffold_context.set(evt.value())
                    }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Base image (e.g. node:20-alpine)",
                        value: "{scaffold_base_image.read()}",
                        oninput: move |evt| scaffold_base_image.set(evt.value())
                    }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Ports (e.g. 3000, 8080)",
                        value: "{scaffold_ports.read()}",
                        oninput: move |evt| scaffold_ports.set(evt.value())
                    }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Workdir (default /app)",
                        value: "{scaffold_workdir.read()}",
                        oninput: move |evt| scaffold_workdir.set(evt.value())
                    }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "CMD (e.g. [\"npm\", \"start\"])",
                        value: "{scaffold_cmd.read()}",
                        oninput: move |evt| scaffold_cmd.set(evt.value())
                    }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "Additional images to pull (e.g. redis:7, postgres:16)",
                        value: "{scaffold_additional.read()}",
                        oninput: move |evt| scaffold_additional.set(evt.value())
                    }
                    div { class: "action-bar", style: "gap: 8px;",
                        button { class: "btn", onclick: on_scaffold, disabled: action.in_progress, "Create Dockerfile" }
                    }
                }
            }
            div { class: "action-bar",
                button { class: "btn ghost", onclick: on_refresh, "Refresh" }
                button { class: "btn primary", onclick: on_pull, disabled: action.in_progress, "Pull image" }
                button { class: "btn", onclick: on_build, disabled: action.in_progress, "Build" }
                button { class: "btn", onclick: on_clean, disabled: action.in_progress, "Clean" }
            }
            if images.is_empty() {
                div { class: "muted", "No images found." }
            } else {
                ul { style: "list-style: none; padding: 0; margin: 12px 0 0; display: flex; flex-direction: column; gap: 8px;",
                    {images.iter().map(|image| {
                        let label = image.repo_tags.first().cloned().unwrap_or_else(|| "<none>".to_string());
                        let size_mb = image.size / 1_048_576;
                        rsx! {
                            li { class: "nav-link", key: "{image.id}",
                                span { "{label}" }
                                span { class: "muted", "{size_mb} MB" }
                            }
                        }
                    })}
                }
            }
        }
    }
}
