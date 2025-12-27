use dioxus::prelude::*;

#[component]
pub fn SystemStatusWidget() -> Element {
    rsx! {
        div { class: "panel",
            h2 { "Health" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU spikes" } span { class: "value", "Low" } }
                div { class: "stat", span { class: "label", "Alerts" } span { class: "value", "1 open" } }
                div { class: "stat", span { class: "label", "Disk" } span { class: "value", "68%" } }
                div { class: "stat", span { class: "label", "Network" } span { class: "value", "Stable" } }
            }
        }
    }
}
