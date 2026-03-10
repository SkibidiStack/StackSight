use dioxus::prelude::*;
use crate::app::BackendBridge;
use crate::state::Command;

#[component]
pub fn ManualBuildModal(show: Signal<bool>) -> Element {
    let dockerfile_path = use_signal(|| String::new());
    let project_path = use_signal(|| String::new());
    let mut image_tag = use_signal(|| String::new());
    let is_building = use_signal(|| false);

    let bridge = use_context::<BackendBridge>();

    let select_dockerfile = move |_| {
        to_owned![dockerfile_path];
        spawn(async move {
            if let Some(path) = rfd::AsyncFileDialog::new()
                .add_filter("Dockerfile", &["*"])
                .set_title("Select Dockerfile")
                .pick_file()
                .await
            {
                dockerfile_path.set(path.path().to_string_lossy().to_string());
            }
        });
    };

    let select_project_folder = move |_| {
        to_owned![project_path];
        spawn(async move {
            if let Some(folder) = rfd::AsyncFileDialog::new()
                .set_title("Select Project Folder")
                .pick_folder()
                .await
            {
                project_path.set(folder.path().to_string_lossy().to_string());
            }
        });
    };

    let build_image = move |_| {
        let dockerfile_path_val = dockerfile_path.read().clone();
        let project_path_val = project_path.read().clone();
        let image_tag_val = image_tag.read().clone();

        if dockerfile_path_val.is_empty() || project_path_val.is_empty() {
            return;
        }

        let tag = if image_tag_val.is_empty() {
            "manual-build:latest".to_string()
        } else {
            image_tag_val
        };

        to_owned![is_building, bridge, show];
        spawn(async move {
            is_building.set(true);
            
            // Use the new CLI docker build command
            bridge.send(Command::DockerBuildManual {
                dockerfile_path: dockerfile_path_val,
                project_path: project_path_val,
                tag: tag.clone(),
            });
            
            // Close modal after starting build
            show.set(false);
            is_building.set(false);
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
                    h2 { 
                        "Manual Image Build" 
                    }
                    button {
                        style: "background: none; border: none; font-size: 24px; color: var(--muted); cursor: pointer; padding: 0;",
                        onclick: move |_| show.set(false),
                        "×"
                    }
                }

                div {
                    
                    // Dockerfile selection
                    div {
                        class: "form-group",
                        label { "Dockerfile" }
                        div {
                            style: "display: flex; gap: 8px;",
                            input {
                                class: "input",
                                style: "flex: 1;",
                                r#type: "text",
                                placeholder: "Select Dockerfile...",
                                value: "{dockerfile_path}",
                                readonly: true
                            }
                            button {
                                class: "btn primary",
                                onclick: select_dockerfile,
                                "Browse"
                            }
                        }
                    }

                    // Project folder selection
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

                    // Image tag input
                    div {
                        class: "form-group",
                        label { "Image Tag (optional)" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "my-app:latest",
                            value: "{image_tag}",
                            oninput: move |evt| image_tag.set(evt.value())
                        }
                    }

                    // Info message
                    div {
                        class: "panel",
                        style: "background: var(--panel); margin-top: 16px;",
                        p {
                            style: "color: var(--accent); margin: 0 0 8px 0;",
                            "This will run: "
                        }
                        code {
                            style: "display: block; font-family: monospace; background: var(--bg); padding: 8px; border-radius: 4px;",
                            "docker build -f [dockerfile] -t [tag] [folder]"
                        }
                    }
                }

                // Actions
                div {
                    style: "display: flex; justify-content: flex-end; gap: 8px; margin-top: 20px;",
                    button {
                        class: "btn",
                        onclick: move |_| show.set(false),
                        "Cancel"
                    }
                    button {
                        class: "btn primary",
                        disabled: *is_building.read() || dockerfile_path.read().is_empty() || project_path.read().is_empty(),
                        onclick: build_image,
                        if *is_building.read() {
                            "Building..."
                        } else {
                            "Build Image"
                        }
                    }
                }
            }
        }
    }
}