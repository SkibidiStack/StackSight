use dioxus::prelude::*;

#[component]
pub fn WelcomeScreen() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Welcome" }
            p { class: "muted", "Start with templates for containers or virtual environments, or jump into monitoring." }
            div { class: "action-bar",
                button { class: "btn primary", "Launch wizard" }
                button { class: "btn", "View docs" }
            }
        }
    }
}
