use dioxus::prelude::*;

#[component]
pub fn PackageManager() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Packages" }
            div { class: "muted", "Unified package operations." }
            div { class: "chip", "numpy 1.26" }
            div { class: "chip", "express 4.18" }
        }
    }
}
