use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn SystemStats() -> Element {
    let app_state = use_context::<Signal<AppState>>();

    let (cpu_usage, memory_used, memory_total, uptime, networks) = {
        let state = app_state.read();
        (
            state.system.cpu_usage,
            state.system.memory_used,
            state.system.memory_total,
            state.system.uptime,
            state.system.networks.clone(),
        )
    };

    let cpu_label = format!("{:.1}%", cpu_usage);

    let mem_percent = if memory_total > 0 {
        (memory_used as f64 / memory_total as f64) * 100.0
    } else {
        0.0
    };
    let mem_label = format!(
        "{:.1}% ({:.1} GB / {:.1} GB)",
        mem_percent,
        memory_used as f64 / 1_073_741_824.0,
        memory_total as f64 / 1_073_741_824.0
    );

    let (net_rx, net_tx) = networks
        .iter()
        .fold((0, 0), |acc, n| (acc.0 + n.received, acc.1 + n.transmitted));
    let net_label = format!(
        "RX: {:.1} MB | TX: {:.1} MB",
        net_rx as f64 / 1_048_576.0,
        net_tx as f64 / 1_048_576.0
    );

    rsx! {
        div { class: "panel",
            h2 { "System Stats" }
            div { class: "grid-two",
                div { class: "stat", span { class: "label", "CPU" } span { class: "value", "{cpu_label}" } }
                div { class: "stat", span { class: "label", "Memory" } span { class: "value", "{mem_label}" } }
                div { class: "stat", span { class: "label", "Network (Total)" } span { class: "value", "{net_label}" } }
                div { class: "stat", span { class: "label", "Uptime" } span { class: "value", "{format_uptime(uptime)}" } }
            }
        }
    }
}

fn format_uptime(secs: u64) -> String {
    let days = secs / 86400;
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    if days > 0 {
        format!("{}d {}h {}m", days, hours, minutes)
    } else {
        format!("{}h {}m", hours, minutes)
    }
}
