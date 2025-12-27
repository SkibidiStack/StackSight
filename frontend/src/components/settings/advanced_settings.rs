use dioxus::prelude::*;

#[component]
pub fn AdvancedSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Advanced" }
            p { class: "muted", "Tune update channels, feature flags, and tracing verbosity." }
        }
    }
}
