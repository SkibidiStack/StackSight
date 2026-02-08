use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn SystemStatusWidget() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let cpu_usage = snapshot.system.cpu_usage;
    let alerts_count = snapshot.system.alerts.len();
    let docker_connected = snapshot.docker.connected;
    drop(snapshot);

    let cpu_label = if cpu_usage.is_finite() {
        format!("{:.1}%", cpu_usage)
    } else {
        "—".to_string()
    };

    let alerts_label = if alerts_count == 0 {
        "None".to_string()
    } else {
        format!("{alerts_count} open")
    };

    let docker_label = if docker_connected { "Connected" } else { "Offline" };

    rsx! {
        div { class: "panel",
            h2 { "Health" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "{cpu_label}" } }
                div { class: "stat", span { class: "label", "Alerts" } span { class: "value", "{alerts_label}" } }
                div { class: "stat", span { class: "label", "Docker" } span { class: "value", "{docker_label}" } }
                div { class: "stat", span { class: "label", "Disk" } span { class: "value", "—" } }
            }
        }
    }
}
