use dioxus::prelude::*;

#[component]
pub fn ComposeBuilder() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Compose Builder" }
            div { class: "muted", "Visual composer for docker-compose.yml." }
            div { class: "chip", "Drag services here" }
        }
    }
}
