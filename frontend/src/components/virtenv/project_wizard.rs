use dioxus::prelude::*;

#[component]
pub fn ProjectWizard() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Project Wizard" }
            p { class: "muted", "Create a workspace with env, dependencies, and compose snippet." }
            button { class: "btn primary", "Launch wizard" }
        }
    }
}
