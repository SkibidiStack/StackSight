use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn ProcessMonitor() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    
    let processes = {
        let state = app_state.read();
        let mut procs = state.system.processes.clone();
        // Sort by CPU usage mainly
        procs.sort_by(|a, b| b.cpu_usage.partial_cmp(&a.cpu_usage).unwrap_or(std::cmp::Ordering::Equal));
        procs.truncate(20); // Top 20
        procs
    };

    rsx! {
        div { class: "panel",
            h2 { "Top Processes" }
            
            if processes.is_empty() {
                div { class: "muted", "Waiting for process data..." }
            } else {
                table { class: "process-table",
                    thead {
                        tr {
                            th { "PID" }
                            th { "Name" }
                            th { "CPU" }
                            th { "Mem" }
                        }
                    }
                    tbody {
                        for proc in processes {
                            tr { key: "{proc.pid}",
                                td { "{proc.pid}" }
                                td { "{proc.name}" }
                                td { "{proc.cpu_usage:.1}%" }
                                td { "{proc.memory / 1024 / 1024} MB" }
                            }
                        }
                    }
                }
            }
        }
    }
}

