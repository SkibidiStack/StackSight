use dioxus::prelude::*;
use crate::state::{AppState, PackageOperationStatus};
use crate::components::virtenv::WebPackageModal;
use dioxus_signals::Signal;

#[component]
pub fn PackageManager(env_id: String) -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let environment = app_state.read().virtenv.environment_list
        .iter()
        .find(|env| env.id == env_id)
        .cloned();
    let package_operation = app_state.read().virtenv.package_operation.clone();
    
    let mut search_query = use_signal(|| String::new());
    let mut selected_packages = use_signal(|| Vec::<String>::new());
    let mut show_install_modal = use_signal(|| false);
    let mut show_web_modal = use_signal(|| false);
    
    match environment {
        Some(env) => rsx! {
            div { class: "package-manager",
                div { class: "manager-header",
                    h2 { "Package Manager - {env.name}" }
                    div { class: "manager-info",
                        span { class: "language-badge", "{env.language} {env.version}" }
                        span { class: "package-count", "{env.package_count} packages" }
                    }
                }
                
                div { class: "manager-toolbar",
                    div { class: "search-section",
                        input {
                            r#type: "text",
                            class: "search-input",
                            placeholder: "Search installed packages...",
                            value: "{search_query()}",
                            oninput: move |evt| search_query.set(evt.value())
                        }
                        button { class: "btn btn-outline", "🔍" }
                    }
                    
                    div { class: "action-section",
                        button { 
                            class: "btn btn-primary",
                            onclick: move |_| show_install_modal.set(true),
                            "📦 Install Package"
                        }
                        button { 
                            class: "btn btn-success",
                            onclick: move |_| {
                                tracing::info!("Web package modal button clicked");
                                show_web_modal.set(true);
                            },
                            "🌐 Install from Web"
                        }
                        button { 
                            class: "btn btn-outline",
                            disabled: selected_packages().is_empty(),
                            "🗑️ Uninstall Selected"
                        }
                        button { 
                            class: "btn btn-outline",
                            "🔄 Update All"
                        }
                        button { 
                            class: "btn btn-outline",
                            "📄 Export Requirements"
                        }
                    }
                }
                
                if let Some(operation) = &package_operation {
                    if operation.env_id == env_id {
                        OperationStatus { operation: operation.clone() }
                    }
                }
                
                div { class: "packages-content",
                    PackageList { 
                        env_id: env_id.clone(),
                        search_query: search_query(),
                        selected_packages: selected_packages.clone()
                    }
                }
                
                if show_install_modal() {
                    InstallPackageModal { 
                        env_id: env_id.clone(),
                        language: env.language.clone(),
                        on_close: move |_| show_install_modal.set(false)
                    }
                }
                
                if show_web_modal() {
                    {
                        tracing::info!("Rendering WebPackageModal");
                        rsx! {
                            WebPackageModal { 
                                env_id: env_id.clone(),
                                language: env.language.clone(),
                                on_close: move |_| {
                                    tracing::info!("WebPackageModal closed");
                                    show_web_modal.set(false);
                                }
                            }
                        }
                    }
                }
            }
        },
        None => rsx! {
            div { class: "error-state",
                "Environment not found"
            }
        }
    }
}

#[component]
fn OperationStatus(operation: PackageOperationStatus) -> Element {
    let status_class = if operation.in_progress {
        "operation-status progress"
    } else if operation.success == Some(true) {
        "operation-status success"
    } else if operation.success == Some(false) {
        "operation-status error"
    } else {
        "operation-status"
    };
    
    rsx! {
        div { class: "{status_class}",
            div { class: "operation-info",
                if operation.in_progress {
                    span { class: "operation-icon spinning", "⚙️" }
                } else if operation.success == Some(true) {
                    span { class: "operation-icon", "✅" }
                } else if operation.success == Some(false) {
                    span { class: "operation-icon", "❌" }
                } else {
                    span { class: "operation-icon", "ℹ️" }
                }
                
                div { class: "operation-details",
                    div { class: "operation-title",
                        "{operation.operation} packages"
                    }
                    div { class: "operation-packages",
                        for (i, pkg) in operation.packages.iter().enumerate() {
                            if i > 0 { ", " }
                            "{pkg}"
                        }
                    }
                    if let Some(message) = &operation.message {
                        div { class: "operation-message", "{message}" }
                    }
                }
            }
        }
    }
}

#[component]
fn PackageList(env_id: String, search_query: String, selected_packages: Signal<Vec<String>>) -> Element {
    // Mock package data - in real implementation, this would come from the backend
    let packages = vec![
        ("numpy", "1.24.3", "Fundamental package for array computing"),
        ("pandas", "2.0.1", "Data manipulation and analysis library"),
        ("matplotlib", "3.7.1", "Comprehensive library for creating visualizations"),
        ("requests", "2.31.0", "Simple HTTP library for Python"),
        ("scikit-learn", "1.2.2", "Machine learning library"),
    ];
    
    let filtered_packages: Vec<_> = packages.into_iter()
        .filter(|(name, _, _)| {
            search_query.is_empty() || name.to_lowercase().contains(&search_query.to_lowercase())
        })
        .collect();
    let filtered_packages_for_select_all = filtered_packages.clone();
    
    rsx! {
        div { class: "package-list",
            div { class: "list-header",
                div { class: "select-all",
                    input { 
                        r#type: "checkbox",
                        onchange: move |evt| {
                            if evt.checked() {
                                let all_packages: Vec<String> = filtered_packages_for_select_all.iter()
                                    .map(|(name, _, _)| name.to_string())
                                    .collect();
                                selected_packages.set(all_packages);
                            } else {
                                selected_packages.set(Vec::new());
                            }
                        }
                    }
                    "Select All"
                }
                div { class: "sort-options",
                    select { class: "form-select",
                        option { "Sort by name" }
                        option { "Sort by version" }
                        option { "Sort by size" }
                    }
                }
            }
            
            div { class: "packages-grid",
                for (name, version, description) in filtered_packages {
                    PackageItem {
                        name: name.to_string(),
                        version: version.to_string(),
                        description: description.to_string(),
                        selected: selected_packages().contains(&name.to_string()),
                        on_select: {
                            let package_name = name.to_string();
                            move |selected: bool| {
                                let mut current = selected_packages();
                                if selected {
                                    if !current.contains(&package_name) {
                                        current.push(package_name.clone());
                                    }
                                } else {
                                    current.retain(|p| p != &package_name);
                                }
                                selected_packages.set(current);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn PackageItem(name: String, version: String, description: String, selected: bool, on_select: EventHandler<bool>) -> Element {
    rsx! {
        div { class: format!("package-item {}", if selected { "selected" } else { ""}),
            div { class: "package-checkbox",
                input { 
                    r#type: "checkbox",
                    checked: selected,
                    onchange: move |evt| on_select.call(evt.checked())
                }
            }
            div { class: "package-info",
                div { class: "package-header",
                    span { class: "package-name", "{name}" }
                    span { class: "package-version", "v{version}" }
                }
                div { class: "package-description", "{description}" }
            }
            div { class: "package-actions",
                button { class: "btn-icon", title: "Update package", "🔄" }
                button { class: "btn-icon", title: "View details", "ℹ️" }
                button { class: "btn-icon danger", title: "Uninstall", "🗑️" }
            }
        }
    }
}

#[component]
fn InstallPackageModal(env_id: String, language: String, on_close: EventHandler<()>) -> Element {
    let mut package_name = use_signal(|| String::new());
    let mut package_version = use_signal(|| String::new());
    
    rsx! {
        div { class: "modal-overlay",
            div { class: "install-modal",
                div { class: "modal-header",
                    h3 { "Install Package" }
                    button { 
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                div { class: "modal-content",
                    div { class: "install-form",
                        div { class: "form-group",
                            label { "Package Name" }
                            input {
                                r#type: "text",
                                class: "form-input",
                                placeholder: match language.as_str() {
                                    "python" => "e.g., numpy, pandas, requests",
                                    "node" => "e.g., express, lodash, axios",
                                    "rust" => "e.g., tokio, serde, reqwest",
                                    _ => "Package name"
                                },
                                value: "{package_name()}",
                                oninput: move |evt| package_name.set(evt.value())
                            }
                        }
                        
                        div { class: "form-group",
                            label { "Version (Optional)" }
                            input {
                                r#type: "text",
                                class: "form-input",
                                placeholder: "latest",
                                value: "{package_version()}",
                                oninput: move |evt| package_version.set(evt.value())
                            }
                        }
                    }
                }
                
                div { class: "modal-actions",
                    button { 
                        class: "btn btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button { 
                        class: "btn btn-primary",
                        disabled: package_name().trim().is_empty(),
                        onclick: move |_| {
                            // TODO: Trigger package installation
                            on_close.call(());
                        },
                        "Install Package"
                    }
                }
            }
        }
    }
}
