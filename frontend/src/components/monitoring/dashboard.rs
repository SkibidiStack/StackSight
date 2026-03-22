use crate::components::monitoring::{AlertPanel, ProcessMonitor, ResourceGraphs, SystemStats};
use dioxus::prelude::*;

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
