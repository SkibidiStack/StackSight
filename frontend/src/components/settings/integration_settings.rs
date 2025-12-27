use dioxus::prelude::*;

#[component]
pub fn IntegrationSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Integrations" }
            p { class: "muted", "Registry auth, API keys, and plugin options." }
        }
    }
}
