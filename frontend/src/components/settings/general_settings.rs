use dioxus::prelude::*;

#[component]
pub fn GeneralSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "General" }
            p { class: "muted", "App language, telemetry, and startup behavior." }
        }
    }
}
