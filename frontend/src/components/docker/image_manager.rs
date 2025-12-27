use dioxus::prelude::*;

#[component]
pub fn ImageManager() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Images" }
            div { class: "muted", "Pull, tag, and clean images." }
            div { class: "action-bar",
                button { class: "btn primary", "Pull image" }
                button { class: "btn", "Build" }
                button { class: "btn", "Clean" }
            }
        }
    }
}
