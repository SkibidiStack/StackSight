use dioxus::prelude::*;

#[component]
pub fn IconLibrary() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Icons" }
            p { class: "muted", "Centralized icon set for consistency." }
        }
    }
}
