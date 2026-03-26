use crate::app::BackendBridge;
use crate::state::Command;
use dioxus::prelude::*;

#[component]
pub fn ManualComposeModal(show: Signal<bool>) -> Element {
    let compose_file_path = use_signal(|| String::new());
    let project_path = use_signal(|| String::new());
    let is_deploying = use_signal(|| false);

    let bridge = use_context::<BackendBridge>();

    let select_compose_file = move |_| {
        to_owned![compose_file_path, project_path];
        spawn(async move {
            if let Some(path) = rfd::AsyncFileDialog::new()
                .add_filter("Compose files", &["yml", "yaml"])
                .set_title("Select docker-compose file")
                .pick_file()
                .await
            {
                let selected = path.path().to_string_lossy().to_string();
                compose_file_path.set(selected.clone());

                if let Some(parent) = std::path::Path::new(&selected).parent() {
                    project_path.set(parent.to_string_lossy().to_string());
                }
            }
        });
    };

    let select_project_folder = move |_| {
        to_owned![project_path];
        spawn(async move {
            if let Some(folder) = rfd::AsyncFileDialog::new()
                .set_title("Select project folder")
                .pick_folder()
                .await
            {
                project_path.set(folder.path().to_string_lossy().to_string());
            }
        });
    };

    let deploy_compose = move |_| {
        let compose_file_path_val = compose_file_path.read().clone();
        let project_path_val = project_path.read().clone();

        if compose_file_path_val.is_empty() || project_path_val.is_empty() {
            return;
        }

        to_owned![is_deploying, bridge, show];
        spawn(async move {
            is_deploying.set(true);

            bridge.send(Command::DockerComposeManual {
                compose_file_path: compose_file_path_val,
                project_path: project_path_val,
            });

            show.set(false);
            is_deploying.set(false);
        });
    };

    if !*show.read() {
        return rsx! { div {} };
    }

    rsx! {
        div {
            class: "modal-overlay",
            div {
                class: "modal",
                div {
                    style: "display: flex; justify-content: space-between; align-items: center; margin-bottom: 20px;",
                    h2 { "Manual Compose Deploy" }
                    button {
                        style: "background: none; border: none; font-size: 24px; color: var(--muted); cursor: pointer; padding: 0;",
                        onclick: move |_| show.set(false),
                        "×"
                    }
                }

                div {
                    div {
                        class: "form-group",
                        label { "Compose File" }
                        div {
                            style: "display: flex; gap: 8px;",
                            input {
                                class: "input",
                                style: "flex: 1;",
                                r#type: "text",
                                placeholder: "Select docker-compose.yml...",
                                value: "{compose_file_path}",
                                readonly: true
                            }
                            button {
                                class: "btn primary",
                                onclick: select_compose_file,
                                "Browse"
                            }
                        }
                    }

                    div {
                        class: "form-group",
                        label { "Project Folder" }
                        div {
                            style: "display: flex; gap: 8px;",
                            input {
                                class: "input",
                                style: "flex: 1;",
                                r#type: "text",
                                placeholder: "Select project folder...",
                                value: "{project_path}",
                                readonly: true
                            }
                            button {
                                class: "btn primary",
                                onclick: select_project_folder,
                                "Browse"
                            }
                        }
                    }

                    div {
                        class: "panel",
                        style: "background: var(--panel); margin-top: 16px;",
                        p {
                            style: "color: var(--accent); margin: 0 0 8px 0;",
                            "This will run:"
                        }
                        code {
                            style: "display: block; font-family: monospace; background: var(--bg); padding: 8px; border-radius: 4px;",
                            "docker compose -f [compose-file] up -d --build"
                        }
                    }
                }

                div {
                    style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px;",
                    button {
                        class: "btn",
                        onclick: move |_| show.set(false),
                        "Cancel"
                    }
                    button {
                        class: "btn primary",
                        disabled: *is_deploying.read() || compose_file_path.read().is_empty() || project_path.read().is_empty(),
                        onclick: deploy_compose,
                        if *is_deploying.read() {
                            "Deploying..."
                        } else {
                            "Deploy Stack"
                        }
                    }
                }
            }
        }
    }
}
