use dioxus::prelude::*;

#[derive(Clone, PartialEq)]
pub struct LanguageOption {
    pub id: String,
    pub name: String,
    pub versions: Vec<String>,
    pub logo_class: String,
    pub description: String,
}

#[component]
pub fn LanguageSelector(
    selected_language: Option<String>,
    on_language_select: EventHandler<String>,
) -> Element {
    let languages = get_supported_languages();

    rsx! {
        div { class: "language-selector",
            h3 { "Select Language & Version" }
            div { class: "language-grid",
                for lang in languages {
                    LanguageCard {
                        language: lang.clone(),
                        selected: selected_language.as_ref().map_or(false, |s| s == &lang.id),
                        on_select: move |_| on_language_select.call(lang.id.clone())
                    }
                }
            }
        }
    }
}

#[component]
fn LanguageCard(language: LanguageOption, selected: bool, on_select: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: format!("language-card {}", if selected { "selected" } else { "" }),
            onclick: move |_| on_select.call(()),
            div { class: format!("language-icon {}", language.logo_class) }
            div { class: "language-info",
                div { class: "language-name", "{language.name}" }
                div { class: "language-description", "{language.description}" }
                div { class: "language-versions",
                    "Versions: "
                    for (i, version) in language.versions.iter().enumerate() {
                        if i > 0 { ", " }
                        span { class: "version", "{version}" }
                    }
                }
            }
            if selected {
                div { class: "selected-indicator", "✓" }
            }
        }
    }
}

fn get_supported_languages() -> Vec<LanguageOption> {
    vec![
        LanguageOption {
            id: "python".to_string(),
            name: "Python".to_string(),
            versions: vec![
                "3.13".to_string(),
                "3.12".to_string(),
                "3.11".to_string(),
                "3.10".to_string(),
            ],
            logo_class: "logo-python".to_string(),
            description: "High-level programming language with extensive libraries".to_string(),
        },
        LanguageOption {
            id: "node".to_string(),
            name: "Node.js".to_string(),
            versions: vec!["22".to_string(), "20".to_string(), "18".to_string()],
            logo_class: "logo-node".to_string(),
            description: "JavaScript runtime built on Chrome's V8 JavaScript engine".to_string(),
        },
        LanguageOption {
            id: "rust".to_string(),
            name: "Rust".to_string(),
            versions: vec![
                "stable".to_string(),
                "beta".to_string(),
                "nightly".to_string(),
            ],
            logo_class: "logo-rust".to_string(),
            description: "Systems programming language focused on safety and performance"
                .to_string(),
        },
        LanguageOption {
            id: "java".to_string(),
            name: "Java".to_string(),
            versions: vec!["24".to_string(), "21".to_string(), "17".to_string()],
            logo_class: "logo-java".to_string(),
            description: "Object-oriented programming language and computing platform".to_string(),
        },
    ]
}
