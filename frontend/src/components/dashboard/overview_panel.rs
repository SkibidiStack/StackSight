use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn OverviewPanel() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let container_total = snapshot.docker.containers.len();
    let running = snapshot
        .docker
        .containers
        .iter()
        .filter(|c| c.state == "running")
        .count();
    let cpu_usage = snapshot.system.cpu_usage;
    let mem_usage = snapshot.system.memory_usage;
    let env_total = snapshot.virtenv.environments;
    drop(snapshot);

    let containers_label = if container_total == 0 {
        "No containers".to_string()
    } else {
        format!("{running} running / {container_total} total")
    };

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

    let env_label = if env_total == 0 {
        "None detected".to_string()
    } else {
        format!("{env_total} available")
    };

    rsx! {
        div { class: "panel",
            h2 { "System Overview" }
            div { class: "muted", "Live snapshot of containers, environments, and host health." }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "Containers" } span { class: "value", "{containers_label}" } }
                div { class: "stat", span { class: "label", "Virtual envs" } span { class: "value", "{env_label}" } }
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "{cpu_label}" } }
                div { class: "stat", span { class: "label", "Memory" } span { class: "value", "{mem_label}" } }
            }
        }
    }
}
