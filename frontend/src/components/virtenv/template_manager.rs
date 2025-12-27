use dioxus::prelude::*;

#[component]
pub fn TemplateManager() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Templates" }
            p { class: "muted", "Curated stacks for data science, web, and services." }
        }
    }
}
