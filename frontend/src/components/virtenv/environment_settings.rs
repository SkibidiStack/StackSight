use dioxus::prelude::*;

#[component]
pub fn EnvironmentSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Environment Settings" }
            p { class: "muted", "Set pythonpath, scripts, and activation hooks." }
        }
    }
}
