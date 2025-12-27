use dioxus::prelude::*;

#[component]
pub fn OverviewPanel() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "System Overview" }
            div { class: "muted", "Live snapshot of containers, environments, and host health." }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "Containers" } span { class: "value", "6 running" } }
                div { class: "stat", span { class: "label", "Virtual envs" } span { class: "value", "4 active" } }
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "22%" } }
                div { class: "stat", span { class: "label", "Memory" } span { class: "value", "45%" } }
            }
        }
    }
}
