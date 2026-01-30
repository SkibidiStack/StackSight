use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn StatsMonitor() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let snapshot = app_state.read();
    let stats = snapshot.docker.stats.clone();
    drop(snapshot);

    let cpu_label = if stats.cpu_percent_avg.is_finite() {
        format!("{:.1}%", stats.cpu_percent_avg)
    } else {
        "—".to_string()
    };

    let mem_label = if stats.memory_limit > 0 {
        let used_gb = stats.memory_used as f64 / 1024.0 / 1024.0 / 1024.0;
        let limit_gb = stats.memory_limit as f64 / 1024.0 / 1024.0 / 1024.0;
        format!("{:.2} / {:.2} GB", used_gb, limit_gb)
    } else {
        "—".to_string()
    };

    let net_label = format!("{} MB / {} MB", stats.net_rx / 1_048_576, stats.net_tx / 1_048_576);

    rsx! {
        div { class: "panel",
            h2 { "Container Stats" }
            if stats.containers == 0 {
                div { class: "muted", "No running containers." }
            } else {
                div { class: "grid-two",
                    div { class: "stat", span { class: "label", "CPU avg" } span { class: "value", "{cpu_label}" } }
                    div { class: "stat", span { class: "label", "Memory" } span { class: "value", "{mem_label}" } }
                    div { class: "stat", span { class: "label", "Network" } span { class: "value", "{net_label}" } }
                    div { class: "stat", span { class: "label", "Containers" } span { class: "value", "{stats.containers}" } }
                }
            }
        }
    }
}
