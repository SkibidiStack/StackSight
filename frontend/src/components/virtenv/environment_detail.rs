use dioxus::prelude::*;
use crate::state::{AppState, VirtualEnvironment};
use crate::components::virtenv::WebPackageModal;
use dioxus_signals::Signal;

#[component]
pub fn EnvironmentDetail(env_id: String) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let environment = app_state.read().virtenv.environment_list
        .iter()
        .find(|env| env.id == env_id)
        .cloned();
    
    match environment {
        Some(env) => rsx! {
            div { class: "environment-detail",
                div { class: "detail-header",
                    div { class: "env-info",
                        h2 { "{env.name}" }
                        div { class: "env-meta",
                            span { class: "language-badge", "{env.language} {env.version}" }
                            if env.is_active {
                                span { class: "status-badge active", "ACTIVE" }
                            }
                            span { 
                                class: format!("health-badge {}", env.health_status.to_lowercase()),
                                "{env.health_status}"
                            }
                        }
                    }
                    div { class: "detail-actions",
                        if env.is_active {
                            button { 
                                class: "btn btn-secondary",
                                onclick: move |_| {
                                    // TODO: Deactivate environment
                                },
                                "Deactivate"
                            }
                        } else {
                            button { 
                                class: "btn btn-primary",
                                onclick: move |_| {
                                    // TODO: Activate environment
                                },
                                "Activate"
                            }
                        }
                        button { 
                            class: "btn btn-outline",
                            onclick: move |_| {
                                // TODO: Open in terminal
                            },
                            "🖥️ Terminal"
                        }
                        button { 
                            class: "btn btn-outline",
                            onclick: move |_| {
                                // TODO: Clone environment
                            },
                            "📋 Clone"
                        }
                        button { 
                            class: "btn btn-danger",
                            onclick: move |_| {
                                // TODO: Delete environment with confirmation
                            },
                            "🗑️ Delete"
                        }
                    }
                }
                
                div { class: "detail-tabs",
                    EnvironmentTabs { environment: env.clone() }
                }
            }
        },
        None => rsx! {
            div { class: "error-state",
                div { class: "error-icon", "⚠️" }
                div { class: "error-title", "Environment Not Found" }
                div { class: "error-description", 
                    "The environment with ID {env_id} could not be found."
                }
            }
        }
    }
}

#[component]
fn EnvironmentTabs(environment: VirtualEnvironment) -> Element {
    let mut active_tab = use_signal(|| "overview".to_string());
    
    let tabs = vec![
        ("overview", "Overview", "📊"),
        ("packages", "Packages", "📦"), 
        ("settings", "Settings", "⚙️"),
        ("activity", "Activity", "📈"),
    ];
    
    rsx! {
        div { class: "tab-container",
            div { class: "tab-nav",
                for (tab_id, title, icon) in tabs {
                    button {
                        class: format!("tab-button {}", 
                            if active_tab() == tab_id { "active" } else { "" }
                        ),
                        onclick: move |_| active_tab.set(tab_id.to_string()),
                        span { class: "tab-icon", "{icon}" }
                        span { class: "tab-title", "{title}" }
                    }
                }
            }
            
            div { class: "tab-content",
                match active_tab().as_str() {
                    "overview" => rsx! { OverviewTab { environment: environment.clone() } },
                    "packages" => rsx! { PackagesTab { environment: environment.clone() } },
                    "settings" => rsx! { SettingsTab { environment: environment.clone() } },
                    "activity" => rsx! { ActivityTab { environment: environment.clone() } },
                    _ => rsx! { div { "Unknown tab" } }
                }
            }
        }
    }
}

#[component]
fn OverviewTab(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "overview-tab",
            div { class: "info-grid",
                div { class: "info-card",
                    h4 { "Environment Details" }
                    div { class: "info-item",
                        span { class: "label", "Created:" }
                        span { class: "value", "{environment.created_at}" }
                    }
                    if let Some(last_used) = &environment.last_used {
                        div { class: "info-item",
                            span { class: "label", "Last used:" }
                            span { class: "value", "{last_used}" }
                        }
                    }
                    if let Some(template) = &environment.template {
                        div { class: "info-item",
                            span { class: "label", "Template:" }
                            span { class: "value template", "{template}" }
                        }
                    }
                    div { class: "info-item",
                        span { class: "label", "Packages:" }
                        span { class: "value", "{environment.package_count}" }
                    }
                }
                
                div { class: "info-card",
                    h4 { "Health Status" }
                    div { class: "health-status",
                        div { 
                            class: format!("health-indicator large {}", environment.health_status.to_lowercase()),
                            "{environment.health_status}"
                        }
                        if environment.health_status != "Healthy" {
                            div { class: "health-details",
                                "Issues detected - click for details"
                            }
                        }
                    }
                }
                
                div { class: "info-card",
                    h4 { "Quick Actions" }
                    div { class: "quick-actions",
                        button { class: "action-btn", "📦 Manage Packages" }
                        button { class: "action-btn", "🔧 Environment Variables" }
                        button { class: "action-btn", "📋 Export Configuration" }
                        button { class: "action-btn", "🔄 Sync with Project" }
                    }
                }
            }
        }
    }
}

#[component]
fn PackagesTab(environment: VirtualEnvironment) -> Element {
    let mut show_web_modal = use_signal(|| false);
    
    rsx! {
        div { class: "packages-tab",
            div { class: "packages-header",
                h4 { "Installed Packages ({environment.package_count})" }
                div { class: "packages-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| {
                            // TODO: Show local install modal
                        },
                        "📦 Install Package" 
                    }
                    button { 
                        class: "btn btn-success",
                        onclick: move |_| {
                            tracing::info!("Web package modal button clicked for environment: {}", environment.id);
                            show_web_modal.set(true);
                        },
                        "🌐 Install from Web" 
                    }
                    button { class: "btn btn-outline", "🔄 Update All" }
                    button { class: "btn btn-outline", "📄 Requirements.txt" }
                }
            }
            div { class: "package-search",
                input {
                    r#type: "text",
                    class: "search-input",
                    placeholder: "Search packages..."
                }
            }
            div { class: "packages-list",
                div { class: "package-item",
                    span { class: "package-name", "Loading packages..." }
                }
            }
            
            if show_web_modal() {
                {
                    let env_id = environment.id.clone();
                    tracing::info!("Rendering WebPackageModal for environment: {}", env_id);
                    rsx! {
                        WebPackageModal { 
                            env_id: env_id.clone(),
                            language: environment.language.clone(),
                            on_close: move |_| {
                                tracing::info!("WebPackageModal closed for environment: {}", env_id);
                                show_web_modal.set(false);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn SettingsTab(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "settings-tab",
            div { class: "settings-section",
                h4 { "Environment Settings" }
                div { class: "setting-item",
                    label { "Environment Name" }
                    input {
                        r#type: "text",
                        class: "form-input",
                        value: "{environment.name}",
                        readonly: true
                    }
                }
                div { class: "setting-item",
                    label { "Python Version" }
                    select { class: "form-select",
                        option { value: "{environment.version}", "{environment.version} (current)" }
                    }
                }
            }
            
            div { class: "settings-section",
                h4 { "Advanced Settings" }
                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Auto-activate on project open"
                    }
                }
                div { class: "setting-item",
                    label { class: "checkbox-label",
                        input { r#type: "checkbox" }
                        "Enable package auto-updates"
                    }
                }
            }
            
            div { class: "settings-section danger",
                h4 { "Danger Zone" }
                button { class: "btn btn-danger", "Delete Environment" }
            }
        }
    }
}

#[component]
fn ActivityTab(environment: VirtualEnvironment) -> Element {
    rsx! {
        div { class: "activity-tab",
            div { class: "activity-header",
                h4 { "Recent Activity" }
                select { class: "form-select",
                    option { "Last 7 days" }
                    option { "Last 30 days" }
                    option { "All time" }
                }
            }
            div { class: "activity-timeline",
                div { class: "activity-item",
                    div { class: "activity-icon", "🎯" }
                    div { class: "activity-content",
                        div { class: "activity-title", "Environment created" }
                        div { class: "activity-time", "{environment.created_at}" }
                    }
                }
                if let Some(last_used) = &environment.last_used {
                    div { class: "activity-item",
                        div { class: "activity-icon", "🚀" }
                        div { class: "activity-content",
                            div { class: "activity-title", "Environment activated" }
                            div { class: "activity-time", "{last_used}" }
                        }
                    }
                }
            }
        }
    }
}
