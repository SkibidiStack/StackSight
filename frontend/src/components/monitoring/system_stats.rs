use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn SystemStats() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let cpu_usage = snapshot.system.cpu_usage;
    let mem_usage = snapshot.system.memory_usage;
    drop(snapshot);

    let cpu_label = if cpu_usage.is_finite() {
        format!("{:.1}%", cpu_usage)
    } else {
        "—".to_string()
    };

    let mem_label = if mem_usage.is_finite() {
        format!("{:.1}%", mem_usage)
    } else {
        "—".to_string()
    };

    rsx! {
        div { class: "panel",
            h2 { "System Stats" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "{cpu_label}" } }
                div { class: "stat", span { class: "label", "Memory" } span { class: "value", "{mem_label}" } }
                div { class: "stat", span { class: "label", "Disk IO" } span { class: "value", "—" } }
                div { class: "stat", span { class: "label", "Network" } span { class: "value", "—" } }
            }
        }
    }
}
