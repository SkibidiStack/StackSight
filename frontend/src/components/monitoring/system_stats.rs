use dioxus::prelude::*;

#[component]
pub fn SystemStats() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "System Stats" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "22%" } }
                div { class: "stat", span { class: "label", "Memory" } span { class: "value", "9.2 GB" } }
                div { class: "stat", span { class: "label", "Disk IO" } span { class: "value", "120 MB/s" } }
                div { class: "stat", span { class: "label", "Network" } span { class: "value", "180 Mbps" } }
            }
        }
    }
}
