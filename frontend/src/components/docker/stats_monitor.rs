use dioxus::prelude::*;

#[component]
pub fn StatsMonitor() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Container Stats" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU avg" } span { class: "value", "16%" } }
                div { class: "stat", span { class: "label", "Memory avg" } span { class: "value", "1.8 GB" } }
                div { class: "stat", span { class: "label", "Network" } span { class: "value", "120 Mbps" } }
                div { class: "stat", span { class: "label", "IO" } span { class: "value", "Stable" } }
            }
        }
    }
}
