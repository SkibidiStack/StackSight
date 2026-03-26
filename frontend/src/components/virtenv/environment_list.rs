use crate::components::virtenv::{ProjectWizard, WebPackageModal};
use crate::services::backend_client::{CreateEnvironmentRequest, Language};
use crate::state::{AppState, Command, VirtualEnvironment};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn EnvironmentList() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let total = snapshot.virtenv.environments;
    let active = snapshot.virtenv.active;
    let environments = snapshot.virtenv.environment_list.clone();
    let creating = snapshot.virtenv.creating;
    let last_error = snapshot.virtenv.last_error.clone();
    drop(snapshot);

    let mut show_wizard = use_signal(|| false);

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Virtual Environments" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            // Request fresh environment list from backend
                            let backend_bridge = use_context::<crate::app::BackendBridge>();
                            backend_bridge.send(Command::VirtEnvList);
                        },
                        "🔄 Refresh"
                    }
                    button {
                        class: "btn btn-primary",
                        disabled: creating,
                        onclick: move |_| {
                            show_wizard.set(true);
                        },
                        if creating { "Creating..." } else { "+ New Environment" }
                    }
                }
            }
            div { class: "panel-content",
                if let Some(error) = &last_error {
                    div { class: "error-message",
                        "⚠️ {error}"
                    }
                }
                if environments.is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🐍" }
                        div { class: "empty-title", "No Virtual Environments" }
                        div { class: "empty-description",
                            "Create your first virtual environment to get started with isolated development environments."
                        }
                    }
                } else {
                    div { class: "environment-list",
                        for env in environments {
                            EnvironmentCard { environment: env.clone() }
                        }
                    }
                }
                div { class: "summary-info",
                    if total == 0 {
                        "No environments detected"
                    } else if active == 0 {
                        "{total} environments"
                    } else {
                        "{active} active / {total} total"
                    }
                }
            }
        }

        if show_wizard() {
            ProjectWizard {
                on_close: move |_| show_wizard.set(false),
                on_create: move |form: crate::components::virtenv::CreateEnvironmentForm| {
                    // Set creating state
                    {
                        let mut state = app_state.write();
                        state.virtenv.creating = true;
                    }

                    // Convert form to backend request
                    let language = match form.language.as_deref() {
                        Some("python") => Language::Python,
                        Some("node") => Language::Node,
                        Some("rust") => Language::Rust,
                        Some("java") => Language::Java,
                        Some("ruby") => Language::Ruby,
                        Some("php") => Language::Php,
                        Some(other) => Language::Other(other.to_string()),
                        None => Language::Python, // Default
                    };

                    let request = CreateEnvironmentRequest {
                        name: form.name.clone(),
                        language,
                        version: form.version.clone(),
                        template: form.template.clone(),
                        project_path: form.location.clone(),
                        packages: if matches!(form.language.as_deref(), Some("java")) {
                            Vec::new()
                        } else {
                            form.packages.clone()
                        },
                        location: form.location.clone(),
                    };

                    // Send to backend via the global bridge - app.rs will handle the VirtualEnvCreated event
                    let backend_bridge = use_context::<crate::app::BackendBridge>();
                    backend_bridge.send(Command::VirtEnvCreate { request });

                    // Clear creating state
                    {
                        let mut state = app_state.write();
                        state.virtenv.creating = false;
                        state.virtenv.last_error = None;
                    }

                    show_wizard.set(false);
                }
            }
        }
    }
}

#[component]
fn EnvironmentCard(environment: VirtualEnvironment) -> Element {
    let env_name = environment.name.clone();
    let env_name_for_packages = env_name.clone();
    let env_path = environment.path.clone();
    let is_active = environment.is_active;
    let health_color = match environment.health_status.as_str() {
        "Healthy" => "green",
        "Warning" => "yellow",
        "Error" => "red",
        _ => "gray",
    };

    let mut app_state = use_context::<Signal<AppState>>();
    let mut show_package_modal = use_signal(|| false);
    let mut show_details_modal = use_signal(|| false);

    rsx! {
        div {
            class: format!("environment-row {}", if is_active { "active" } else { "" }),
            onclick: move |_evt| {
                // Open details modal when clicking on the card
                show_details_modal.set(true);
            },
            oncontextmenu: move |evt| {
                evt.prevent_default();
                if let Some(path) = env_path.clone() {
                    // Open folder in file manager (this will work on most desktop environments)
                    spawn(async move {
                        #[cfg(target_os = "linux")]
                        {
                            use std::process::Command;
                            let _ = Command::new("xdg-open").arg(&path).spawn();
                        }
                        #[cfg(target_os = "windows")]
                        {
                            use std::process::Command;
                            let _ = Command::new("explorer").arg(&path).spawn();
                        }
                        #[cfg(target_os = "macos")]
                        {
                            use std::process::Command;
                            let _ = Command::new("open").arg(&path).spawn();
                        }
                        tracing::info!("Opened folder: {}", path);
                    });
                }
            },

            div { class: "env-main-info",
                div { class: "env-header",
                    div { class: "env-name-section",
                        h3 { class: "env-name", "{environment.name}" }
                        div { class: "env-badges",
                            span { class: format!("language-badge {}", environment.language.to_lowercase()),
                                "{environment.language} {environment.version}"
                            }
                            if is_active {
                                span { class: "status-badge active", "ACTIVE" }
                            }
                            span {
                                class: format!("health-indicator {}", health_color),
                                "●"
                            }
                        }
                    }
                    div { class: "env-actions",
                        if is_active {
                            button {
                                class: "btn btn-sm btn-secondary",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    // Send deactivate command to terminal
                                    let mut state = app_state.write();
                                    state.ui.terminal_visible = true;
                                    state.ui.terminal_pending_command = Some("env deactivate".to_string());
                                    tracing::info!("Sending deactivate command to terminal");
                                },
                                "Deactivate"
                            }
                        } else {
                            button {
                                class: "btn btn-sm btn-primary",
                                onclick: {
                                    let env_name_clone = env_name.clone();
                                    let env_path_clone = environment.path.clone();
                                    move |evt| {
                                        evt.stop_propagation();
                                        let mut state = app_state.write();
                                        state.ui.terminal_visible = true;

                                        // Send both activate and cd commands
                                        if let Some(path) = &env_path_clone {
                                            state.ui.terminal_pending_command = Some(format!(
                                                "env activate \"{}\" && cd \"{}\"",
                                                env_name_clone, path
                                            ));
                                        } else {
                                            state.ui.terminal_pending_command =
                                                Some(format!("env activate \"{}\"", env_name_clone));
                                        }

                                        tracing::info!("Sending activate and cd commands for environment: {}", env_name_clone);
                                    }
                                },
                                "Activate"
                            }
                        }
                        if !environment.language.eq_ignore_ascii_case("java") {
                            button {
                                class: "btn btn-sm btn-outline",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    tracing::info!("Package manager for: {}", env_name_for_packages);
                                    show_package_modal.set(true);
                                },
                                "Add Packages"
                            }
                        }
                        button {
                            class: "btn btn-sm btn-danger",
                            onclick: {
                                let env_id_clone = environment.id.clone();
                                move |evt| {
                                    evt.stop_propagation();
                                    tracing::info!("🗑️ Deleting environment: {}", env_id_clone);

                                    // Send delete command via bridge - app.rs will handle VirtualEnvDeleted event
                                    let backend_bridge = use_context::<crate::app::BackendBridge>();
                                    backend_bridge.send(Command::VirtEnvDelete { env_id: env_id_clone.clone() });
                                }
                            },
                            "Delete"
                        }
                    }
                }
            }
        }

        if show_package_modal() {
            {
                let env_id = environment.id.clone();
                let env_language = environment.language.clone();
                let mut modal_signal = show_package_modal.clone();
                rsx! {
                    WebPackageModal {
                        env_id: env_id.clone(),
                        language: env_language.clone(),
                        on_close: move |_| {
                            tracing::info!("WebPackageModal closed for environment: {}", env_id);
                            modal_signal.set(false);
                        }
                    }
                }
            }
        }

        if show_details_modal() {
            {
                let env_clone = environment.clone();
                let mut modal_signal = show_details_modal.clone();
                rsx! {
                    EnvironmentDetailsModal {
                        environment: env_clone,
                        on_close: move |_| {
                            modal_signal.set(false);
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn EnvironmentDetailsModal(environment: VirtualEnvironment, on_close: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal details-modal",
                onclick: move |evt| evt.stop_propagation(),

                div { class: "modal-header",
                    h3 { "Environment Details: {environment.name}" }
                    button {
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "modal-content",
                    div { class: "details-section",
                        h4 { "General Information" }
                        div { class: "detail-grid",
                            div { class: "detail-item",
                                span { class: "detail-label", "Language:" }
                                span { class: "detail-value",
                                    span { class: format!("language-badge {}", environment.language.to_lowercase()),
                                        "{environment.language}"
                                    }
                                }
                            }
                            div { class: "detail-item",
                                span { class: "detail-label", "Version:" }
                                span { class: "detail-value", "{environment.version}" }
                            }
                            div { class: "detail-item",
                                span { class: "detail-label", "Status:" }
                                span { class: "detail-value",
                                    if environment.is_active {
                                        span { class: "status-badge active", "ACTIVE" }
                                    } else {
                                        "Inactive"
                                    }
                                }
                            }
                            div { class: "detail-item",
                                span { class: "detail-label", "Path:" }
                                span { class: "detail-value path",
                                    if let Some(path) = &environment.path {
                                        "{path}"
                                    } else {
                                        "N/A"
                                    }
                                }
                            }
                            if let Some(size) = environment.size_mb {
                                div { class: "detail-item",
                                    span { class: "detail-label", "Size:" }
                                    span { class: "detail-value", "{size} MB" }
                                }
                            }
                            div { class: "detail-item",
                                span { class: "detail-label", "Created:" }
                                span { class: "detail-value", "{environment.created_at}" }
                            }
                            if let Some(last_used) = &environment.last_used {
                                div { class: "detail-item",
                                    span { class: "detail-label", "Last Used:" }
                                    span { class: "detail-value", "{last_used}" }
                                }
                            }
                            if let Some(template) = &environment.template {
                                div { class: "detail-item",
                                    span { class: "detail-label", "Template:" }
                                    span { class: "detail-value", "{template}" }
                                }
                            }
                        }
                    }

                    div { class: "details-section packages-section",
                        h4 { "Installed Packages ({environment.package_count})" }
                        if environment.packages.is_empty() {
                            div { class: "empty-packages",
                                "No packages installed"
                            }
                        } else {
                            div { class: "packages-list",
                                for package in &environment.packages {
                                    div { class: "package-item",
                                        div { class: "package-name",
                                            "{package.name}"
                                            if package.is_dev_dependency {
                                                span { class: "dev-badge", "dev" }
                                            }
                                        }
                                        div { class: "package-version", "{package.version}" }
                                    }
                                }
                            }
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Close"
                    }
                }
            }
        }
    }
}
