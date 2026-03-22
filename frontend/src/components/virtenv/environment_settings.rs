use crate::state::{AppState, VirtualEnvironment};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn EnvironmentSettings(env_id: String) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let environment = app_state
        .read()
        .virtenv
        .environment_list
        .iter()
        .find(|env| env.id == env_id)
        .cloned();

    let mut active_section = use_signal(|| "general".to_string());

    match environment {
        Some(env) => rsx! {
            div { class: "environment-settings",
                div { class: "settings-header",
                    h2 { "Environment Settings - {env.name}" }
                    div { class: "env-info",
                        span { class: "language-badge", "{env.language} {env.version}" }
                        if env.is_active {
                            span { class: "status-badge active", "ACTIVE" }
                        }
                    }
                }

                div { class: "settings-layout",
                    div { class: "settings-sidebar",
                        SettingsNav {
                            active_section: active_section(),
                            on_section_change: move |section: String| active_section.set(section)
                        }
                    }

                    div { class: "settings-content",
                        match active_section().as_str() {
                            "general" => rsx! { GeneralSettings { environment: env.clone() } },
                            "paths" => rsx! { PathSettings { environment: env.clone() } },
                            "variables" => rsx! { EnvironmentVariables { environment: env.clone() } },
                            "integrations" => rsx! { IntegrationSettings { environment: env.clone() } },
                            "advanced" => rsx! { AdvancedSettings { environment: env.clone() } },
                            "danger" => rsx! { DangerZone { environment: env.clone() } },
                            _ => rsx! { div { "Unknown section" } }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div { class: "error-state",
                "Environment not found"
            }
        },
    }
}

#[component]
fn SettingsNav(active_section: String, on_section_change: EventHandler<String>) -> Element {
    let sections = vec![
        ("general", "General", "⚙️"),
        ("paths", "Paths", "📁"),
        ("variables", "Variables", "📝"),
        ("integrations", "Integrations", "🔗"),
        ("advanced", "Advanced", "🔧"),
        ("danger", "Danger Zone", "⚠️"),
    ];

    rsx! {
        nav { class: "settings-nav",
            for (section_id, title, icon) in sections {
                button {
                    class: format!("nav-item {}",
                        if active_section == section_id { "active" } else { "" }
                    ),
                    onclick: move |_| on_section_change.call(section_id.to_string()),
                    span { class: "nav-icon", "{icon}" }
                    span { class: "nav-title", "{title}" }
                }
            }
        }
    }
}

#[component]
fn GeneralSettings(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-section",
            h3 { "General Settings" }

            div { class: "setting-group",
                h4 { "Environment Information" }
                div { class: "setting-item",
                    label { "Name" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: "{environment.name}",
                        readonly: true
                    }
                    div { class: "setting-help", "Environment name cannot be changed" }
                }

                div { class: "setting-item",
                    label { "Description" }
                    textarea {
                        class: "form-textarea",
                        placeholder: "Add a description for this environment..."
                    }
                }

                div { class: "setting-item",
                    label { "Language Version" }
                    select { class: "form-select",
                        option { value: "{environment.version}", "{environment.version} (current)" }
                        option { "3.11" }
                        option { "3.10" }
                    }
                    div { class: "setting-help", "Changing version will recreate the environment" }
                }
            }

            div { class: "setting-group",
                h4 { "Behavior" }
                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Auto-activate when opening project"
                    }
                    div { class: "setting-help", "Automatically activate this environment when opening the associated project" }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Auto-install missing packages"
                    }
                    div { class: "setting-help", "Automatically install packages when they're imported but not found" }
                }
            }
        }
    }
}

#[component]
fn PathSettings(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-section",
            h3 { "Path Configuration" }

            div { class: "setting-group",
                h4 { "Environment Paths" }
                div { class: "setting-item",
                    label { "Environment Location" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: "~/.virtualenvs/{environment.name}",
                        readonly: true
                    }
                    button { class: "btn btn-outline btn-sm", "Browse" }
                }

                div { class: "setting-item",
                    label { "Python Executable" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: if cfg!(windows) {
                            "%USERPROFILE%\\.virtualenvs\\{environment.name}\\Scripts\\python.exe"
                        } else {
                            "~/.virtualenvs/{environment.name}/bin/python"
                        },
                        readonly: true
                    }
                }

                div { class: "setting-item",
                    label { "Site Packages" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: if cfg!(windows) {
                            "%USERPROFILE%\\.virtualenvs\\{environment.name}\\Lib\\site-packages"
                        } else {
                            "~/.virtualenvs/{environment.name}/lib/python{environment.version}/site-packages"
                        },
                        readonly: true
                    }
                }
            }

            div { class: "setting-group",
                h4 { "Project Association" }
                div { class: "setting-item",
                    label { "Project Directory" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        placeholder: "Associate with a project directory..."
                    }
                    button { class: "btn btn-outline btn-sm", "Browse" }
                }
            }
        }
    }
}

#[component]
fn EnvironmentVariables(environment: VirtualEnvironment) -> Element {
    let mut variables = use_signal(|| {
        vec![
            ("PYTHONPATH".to_string(), "/custom/path".to_string()),
            ("DEBUG".to_string(), "true".to_string()),
        ]
    });
    let mut new_key = use_signal(|| String::new());
    let mut new_value = use_signal(|| String::new());

    rsx! {
        div { class: "settings-section",
            h3 { "Environment Variables" }

            div { class: "setting-group",
                h4 { "Custom Variables" }
                div { class: "variables-list",
                    for (i, (key, value)) in variables().iter().enumerate() {
                        div { class: "variable-item",
                            input {
                                r#type: "text",
                                class: "form-input variable-key",
                                value: "{key}",
                                placeholder: "Variable name"
                            }
                            input {
                                r#type: "text",
                                class: "form-input variable-value",
                                value: "{value}",
                                placeholder: "Variable value"
                            }
                            button {
                                class: "btn btn-outline btn-sm remove-btn",
                                onclick: move |_| {
                                    variables.write().remove(i);
                                },
                                "×"
                            }
                        }
                    }

                    div { class: "variable-item new-variable",
                        input {
                            r#type: "text",
                            class: "form-input variable-key",
                            value: "{new_key()}",
                            placeholder: "Variable name",
                            oninput: move |evt| new_key.set(evt.value())
                        }
                        input {
                            r#type: "text",
                            class: "form-input variable-value",
                            value: "{new_value()}",
                            placeholder: "Variable value",
                            oninput: move |evt| new_value.set(evt.value())
                        }
                        button {
                            class: "btn btn-primary btn-sm",
                            disabled: new_key().trim().is_empty(),
                            onclick: move |_| {
                                if !new_key().trim().is_empty() {
                                    variables.write().push((new_key().trim().to_string(), new_value().trim().to_string()));
                                    new_key.set(String::new());
                                    new_value.set(String::new());
                                }
                            },
                            "Add"
                        }
                    }
                }
            }

            div { class: "setting-group",
                h4 { "Path Variables" }
                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox", checked: true }
                        "Include system PATH"
                    }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Isolate environment PATH"
                    }
                    div { class: "setting-help", "Prevent access to system-wide packages" }
                }
            }
        }
    }
}

#[component]
fn IntegrationSettings(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-section",
            h3 { "IDE & Tool Integrations" }

            div { class: "setting-group",
                h4 { "Code Editors" }
                div { class: "integration-item",
                    div { class: "integration-info",
                        strong { "VS Code" }
                        div { class: "integration-status active", "Connected" }
                    }
                    button { class: "btn btn-outline btn-sm", "Configure" }
                }

                div { class: "integration-item",
                    div { class: "integration-info",
                        strong { "PyCharm" }
                        div { class: "integration-status inactive", "Not configured" }
                    }
                    button { class: "btn btn-primary btn-sm", "Setup" }
                }

                div { class: "integration-item",
                    div { class: "integration-info",
                        strong { "Jupyter" }
                        div { class: "integration-status active", "Available" }
                    }
                    button { class: "btn btn-outline btn-sm", "Launch" }
                }
            }

            div { class: "setting-group",
                h4 { "Version Control" }
                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Include requirements.txt in git"
                    }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Auto-generate .gitignore for Python"
                    }
                }
            }
        }
    }
}

#[component]
fn AdvancedSettings(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-section",
            h3 { "Advanced Settings" }

            div { class: "setting-group",
                h4 { "Python Configuration" }
                div { class: "setting-item",
                    label { "Python Optimization Level" }
                    select { class: "form-select",
                        option { value: "0", "0 - No optimization (default)" }
                        option { value: "1", "1 - Remove assertions" }
                        option { value: "2", "2 - Remove docstrings" }
                    }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Enable hash randomization"
                    }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Enable development mode warnings"
                    }
                }
            }

            div { class: "setting-group",
                h4 { "Package Management" }
                div { class: "setting-item",
                    label { "Package Index URL" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: "https://pypi.org/simple",
                        placeholder: "PyPI index URL"
                    }
                }

                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Use system site packages"
                    }
                    div { class: "setting-help", "Allow access to system-wide Python packages" }
                }
            }
        }
    }
}

#[component]
fn DangerZone(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-section danger-zone",
            h3 { "Danger Zone" }
            div { class: "warning-text",
                "These actions are permanent and cannot be undone. Proceed with caution."
            }

            div { class: "setting-group",
                div { class: "danger-action",
                    div { class: "action-info",
                        strong { "Reset Environment" }
                        div { "Remove all packages and reset to base state" }
                    }
                    button { class: "btn btn-danger", "Reset Environment" }
                }

                div { class: "danger-action",
                    div { class: "action-info",
                        strong { "Clone Environment" }
                        div { "Create an exact copy of this environment" }
                    }
                    button { class: "btn btn-outline", "Clone Environment" }
                }

                div { class: "danger-action",
                    div { class: "action-info",
                        strong { "Export Environment" }
                        div { "Export environment configuration and packages" }
                    }
                    button { class: "btn btn-outline", "Export Environment" }
                }

                div { class: "danger-action",
                    div { class: "action-info",
                        strong { "Delete Environment" }
                        div { "Permanently delete this environment and all its packages" }
                    }
                    button { class: "btn btn-danger", "Delete Environment" }
                }
            }
        }
    }
}
