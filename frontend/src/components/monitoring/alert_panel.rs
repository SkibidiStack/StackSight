use dioxus::prelude::*;

#[component]
pub fn AlertPanel() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Alerts" }
            div { class: "muted", "Define thresholds and review active alerts." }
            div { class: "chip", "1 critical" }
            div { class: "chip", "2 warnings" }
        }
    }
}
