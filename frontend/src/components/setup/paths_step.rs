use dioxus::prelude::*;
use crate::state::SetupConfig;

#[component]
pub fn PathsStep(config: Signal<SetupConfig>) -> Element {
    let mut custom_path = use_signal(|| false);

    let default_path = if cfg!(windows) {
        "%USERPROFILE%\\.virtualenvs"
    } else {
        "~/.virtualenvs"
    };

    rsx! {
        div { class: "wizard-step paths-step",
            h2 { "Environment Paths" }
            p { "Configure where virtual environments will be stored." }

            div { class: "path-config",
                div { class: "path-option",
                    input {
                        r#type: "radio",
                        id: "default-path",
                        name: "path-type",
                        checked: !custom_path(),
                        onchange: move |_| {
                            custom_path.set(false);
                            let mut cfg = config();
                            cfg.virtualenv_base_path = default_path.to_string();
                            config.set(cfg);
                        }
                    }
                    label { r#for: "default-path",
                        strong { "Default Location (Recommended)" }
                        div { class: "path-display", "{default_path}" }
                        p { class: "help-text",
                            "Standard location for virtual environments. Compatible with most tools."
                        }
                    }
                }

                div { class: "path-option",
                    input {
                        r#type: "radio",
                        id: "custom-path",
                        name: "path-type",
                        checked: custom_path(),
                        onchange: move |_| custom_path.set(true)
                    }
                    label { r#for: "custom-path",
                        strong { "Custom Location" }
                        p { class: "help-text",
                            "Choose a different directory for your environments."
                        }
                    }
                }

                if custom_path() {
                    div { class: "custom-path-input",
                        label { "Custom Path:" }
                        div { class: "input-group",
                            input {
                                r#type: "text",
                                class: "form-input",
                                value: config().virtualenv_base_path,
                                placeholder: "/path/to/your/environments",
                                oninput: move |evt| {
                                    let mut cfg = config();
                                    cfg.virtualenv_base_path = evt.value().clone();
                                    config.set(cfg);
                                }
                            }
                            button {
                                class: "btn btn-outline",
                                onclick: move |_| {
                                    spawn(async move {
                                        if let Some(dir) = rfd::AsyncFileDialog::new()
                                            .set_title("Select Environment Directory")
                                            .pick_folder()
                                            .await
                                        {
                                            let mut cfg = config();
                                            cfg.virtualenv_base_path = dir.path().display().to_string();
                                            config.set(cfg);
                                        }
                                    });
                                },
                                "Browse..."
                            }
                        }
                    }
                }
            }

            div { class: "info-box",
                h4 { "💡 Path Tips" }
                ul {
                    li { "Avoid spaces in paths for better compatibility" }
                    li { "Use a location with sufficient disk space" }
                    li { "Keep it separate from project directories" }
                    li { "Backup this location regularly if you have important environments" }
                }
            }

            div { class: "path-preview",
                h4 { "Your environments will be created at:" }
                div { class: "preview-path",
                    code { "{config().virtualenv_base_path}/[environment-name]" }
                }
                p { class: "example",
                    "Example: "
                    code {
                        if cfg!(windows) {
                            "%USERPROFILE%\\.virtualenvs\\my-project"
                        } else {
                            "~/.virtualenvs/my-project"
                        }
                    }
                }
            }
        }
    }
}
