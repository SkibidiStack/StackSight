use dioxus::prelude::*;

#[component]
pub fn ThemeSettings() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Theme" }
            p { class: "muted", "Switch between light and dark palettes." }
            div { class: "action-bar",
                button { class: "btn primary", "Dark" }
                button { class: "btn", "Light" }
            }
        }
    }
}
