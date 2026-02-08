use dioxus::prelude::*;
use crate::components::monitoring::{SystemStats, ProcessMonitor, AlertPanel, ResourceGraphs};

#[component]
pub fn MonitoringDashboard() -> Element {
    rsx! {
        div { class: "monitoring-dashboard",
            div { class: "monitoring-grid",
                div { class: "monitoring-col-main",
                    SystemStats {}
                    ResourceGraphs {}
                }
                div { class: "monitoring-col-side",
                    AlertPanel {}
                    ProcessMonitor {}
                }
            }
        }
    }
}
