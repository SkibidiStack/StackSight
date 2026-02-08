use dioxus::prelude::*;
use crate::state::{AppState, EnvironmentTemplate};
use dioxus_signals::Signal;

#[component]
pub fn TemplateManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let templates = &app_state.read().virtenv.templates;
    
    let mut selected_language = use_signal(|| "all".to_string());
    let mut show_create_modal = use_signal(|| false);
    
    let languages = vec![
        ("all", "All Languages"),
        ("python", "Python"),
        ("node", "Node.js"),
        ("rust", "Rust"),
        ("java", "Java"),
    ];
    
    let filtered_templates: Vec<_> = templates.iter()
        .filter(|template| {
            selected_language() == "all" || template.language == selected_language()
        })
        .collect();
    
    rsx! {
        div { class: "template-manager",
            div { class: "manager-header",
                h2 { "Environment Templates" }
                div { class: "manager-actions",
                    button { 
                        class: "btn btn-primary",
                        onclick: move |_| show_create_modal.set(true),
                        "+ Create Template"
                    }
                    button { class: "btn btn-outline", "📥 Import" }
                    button { class: "btn btn-outline", "📤 Export" }
                }
            }
            
            div { class: "manager-filters",
                div { class: "language-filter",
                    label { "Filter by Language:" }
                    select { 
                        class: "form-select",
                        value: "{selected_language()}",
                        onchange: move |evt| selected_language.set(evt.value())
                    }
                    for (value, label) in languages {
                        option { value: "{value}", "{label}" }
                    }
                }
                
                div { class: "search-filter",
                    input {
                        r#type: "text",
                        class: "search-input",
                        placeholder: "Search templates..."
                    }
                }
            }
            
            div { class: "templates-grid",
                if filtered_templates.is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "📄" }
                        div { class: "empty-title", "No Templates Found" }
                        div { class: "empty-description", 
                            "No templates match your current filter criteria."
                        }
                    }
                } else {
                    for template in filtered_templates {
                        TemplateCard { template: template.clone() }
                    }
                }
            }
            
            if show_create_modal() {
                CreateTemplateModal { 
                    on_close: move |_| show_create_modal.set(false)
                }
            }
        }
    }
}

#[component]
fn TemplateCard(template: EnvironmentTemplate) -> Element {
    let language_icon = match template.language.as_str() {
        "python" => "🐍",
        "node" => "🟢",
        "rust" => "🦀", 
        "java" => "☕",
        _ => "📄",
    };
    
    rsx! {
        div { class: "template-card",
            div { class: "card-header",
                div { class: "template-info",
                    span { class: "template-icon", "{language_icon}" }
                    div { class: "template-details",
                        div { class: "template-name", "{template.name}" }
                        div { class: "template-language", "{template.language}" }
                    }
                }
                div { class: "template-actions",
                    button { class: "btn-icon", title: "Edit template", "✏️" }
                    button { class: "btn-icon", title: "Duplicate template", "📋" }
                    button { class: "btn-icon danger", title: "Delete template", "🗑️" }
                }
            }
            
            div { class: "card-body",
                div { class: "template-description", "{template.description}" }
                div { class: "template-stats",
                    span { class: "stat",
                        span { class: "stat-icon", "📦" }
                        span { class: "stat-value", "{template.package_count} packages" }
                    }
                }
            }
            
            div { class: "card-footer",
                button { 
                    class: "btn btn-primary btn-sm",
                    "Use Template"
                }
                button { 
                    class: "btn btn-outline btn-sm",
                    "Preview"
                }
            }
        }
    }
}

#[component]
fn CreateTemplateModal(on_close: EventHandler<()>) -> Element {
    let mut template_name = use_signal(|| String::new());
    let mut template_description = use_signal(|| String::new());
    let mut selected_language = use_signal(|| "python".to_string());
    let mut packages = use_signal(|| Vec::<String>::new());
    let mut package_input = use_signal(|| String::new());
    
    let languages = vec![
        ("python", "Python"),
        ("node", "Node.js"),
        ("rust", "Rust"),
        ("java", "Java"),
    ];
    
    rsx! {
        div { class: "modal-overlay",
            div { class: "create-template-modal",
                div { class: "modal-header",
                    h3 { "Create New Template" }
                    button { 
                        class: "close-btn",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }
                
                div { class: "modal-content",
                    div { class: "template-form",
                        div { class: "form-group",
                            label { "Template Name" }
                            input {
                                r#type: "text",
                                class: "form-input",
                                placeholder: "e.g., React Development Environment",
                                value: "{template_name()}",
                                oninput: move |evt| template_name.set(evt.value())
                            }
                        }
                        
                        div { class: "form-group",
                            label { "Description" }
                            textarea {
                                class: "form-textarea",
                                placeholder: "Describe what this template is for...",
                                value: "{template_description()}",
                                oninput: move |evt| template_description.set(evt.value())
                            }
                        }
                        
                        div { class: "form-group",
                            label { "Language" }
                            select { 
                                class: "form-select",
                                value: "{selected_language()}",
                                onchange: move |evt| selected_language.set(evt.value())
                            }
                            for (value, label) in languages {
                                option { value: "{value}", "{label}" }
                            }
                        }
                        
                        div { class: "form-group",
                            label { "Packages" }
                            div { class: "package-input-group",
                                input {
                                    r#type: "text",
                                    class: "form-input",
                                    placeholder: "Package name",
                                    value: "{package_input()}",
                                    oninput: move |evt| package_input.set(evt.value()),
                                    onkeydown: move |evt| {
                                        if evt.key() == Key::Enter && !package_input().trim().is_empty() {
                                            packages.write().push(package_input().trim().to_string());
                                            package_input.set(String::new());
                                        }
                                    }
                                }
                                button { 
                                    class: "btn btn-secondary",
                                    disabled: package_input().trim().is_empty(),
                                    onclick: move |_| {
                                        if !package_input().trim().is_empty() {
                                            packages.write().push(package_input().trim().to_string());
                                            package_input.set(String::new());
                                        }
                                    },
                                    "Add"
                                }
                            }
                            
                            if !packages().is_empty() {
                                div { class: "package-tags",
                                    for (i, package) in packages().iter().enumerate() {
                                        span { class: "package-tag",
                                            "{package}"
                                            button { 
                                                class: "remove-tag",
                                                onclick: move |_| {
                                                    packages.write().remove(i);
                                                },
                                                "×"
                                            }
                                        }
                                    }
                                }
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
                        disabled: template_name().trim().is_empty(),
                        onclick: move |_| {
                            // TODO: Create template
                            on_close.call(());
                        },
                        "Create Template"
                    }
                }
            }
        }
    }
}
