use dioxus::prelude::*;

#[component]
pub fn ProcessMonitor() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Processes" }
            div { class: "muted", "Process metrics are not available yet." }
        }
    }
}
