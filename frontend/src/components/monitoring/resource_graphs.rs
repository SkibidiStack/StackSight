use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;
use std::collections::VecDeque;

const MAX_DATA_POINTS: usize = 60; // Keep last 60 data points

#[derive(Clone, Debug)]
pub struct ResourceHistory {
    pub cpu_history: VecDeque<f32>,
    pub memory_history: VecDeque<f32>,
    pub network_rx_history: VecDeque<f32>,
    pub network_tx_history: VecDeque<f32>,
    pub last_rx_total: u64,
    pub last_tx_total: u64,
    pub smoothed_rx: f32,  // Smoothed display value for RX
    pub smoothed_tx: f32,  // Smoothed display value for TX
    pub smoothed_cpu: f32, // Smoothed display value for CPU
    pub smoothed_mem: f32, // Smoothed display value for Memory
}

impl Default for ResourceHistory {
    fn default() -> Self {
        Self {
            cpu_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            memory_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            network_rx_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            network_tx_history: VecDeque::with_capacity(MAX_DATA_POINTS),
            last_rx_total: 0,
            last_tx_total: 0,
            smoothed_rx: 0.0,
            smoothed_tx: 0.0,
            smoothed_cpu: 0.0,
            smoothed_mem: 0.0,
        }
    }
}

#[component]
pub fn ResourceGraphs() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let mut resource_history = use_signal(|| ResourceHistory::default());

    // Update history with current values
    use_effect(move || {
        let state = app_state.read();
        let mut history = resource_history.write();

        // Smoothing factors (0.0 = all old value, 1.0 = all new value)
        // Lower = smoother but slower response, Higher = faster but jittery
        let cpu_mem_smoothing = 0.3; // CPU and memory update relatively smoothly
        let network_smoothing = 0.05; // Network can spike dramatically, use heavy smoothing for persistence

        // Add current CPU usage with smoothing
        let cpu_value = state.system.cpu_usage;
        history.smoothed_cpu =
            history.smoothed_cpu * (1.0 - cpu_mem_smoothing) + cpu_value * cpu_mem_smoothing;

        if history.cpu_history.len() >= MAX_DATA_POINTS {
            history.cpu_history.pop_front();
        }
        history.cpu_history.push_back(cpu_value);

        // Add current memory usage percentage with smoothing
        let mem_percent = if state.system.memory_total > 0 {
            (state.system.memory_used as f64 / state.system.memory_total as f64) * 100.0
        } else {
            0.0
        } as f32;
        history.smoothed_mem =
            history.smoothed_mem * (1.0 - cpu_mem_smoothing) + mem_percent * cpu_mem_smoothing;

        if history.memory_history.len() >= MAX_DATA_POINTS {
            history.memory_history.pop_front();
        }
        history.memory_history.push_back(mem_percent);

        // Calculate network rate (difference from last sample) in KB/s
        let (net_rx, net_tx) = state
            .system
            .networks
            .iter()
            .fold((0, 0), |acc, n| (acc.0 + n.received, acc.1 + n.transmitted));

        // Calculate rate based on difference from last total
        let rx_rate = if history.last_rx_total > 0 {
            let diff = net_rx.saturating_sub(history.last_rx_total);
            diff as f32 / 1024.0 // KB/s
        } else {
            0.0
        };

        let tx_rate = if history.last_tx_total > 0 {
            let diff = net_tx.saturating_sub(history.last_tx_total);
            diff as f32 / 1024.0 // KB/s
        } else {
            0.0
        };

        // Apply more aggressive smoothing to network rates for display (prevents flickering on spikes)
        history.smoothed_rx =
            history.smoothed_rx * (1.0 - network_smoothing) + rx_rate * network_smoothing;
        history.smoothed_tx =
            history.smoothed_tx * (1.0 - network_smoothing) + tx_rate * network_smoothing;

        // Update last totals
        history.last_rx_total = net_rx;
        history.last_tx_total = net_tx;

        if history.network_rx_history.len() >= MAX_DATA_POINTS {
            history.network_rx_history.pop_front();
        }
        history.network_rx_history.push_back(rx_rate);

        if history.network_tx_history.len() >= MAX_DATA_POINTS {
            history.network_tx_history.pop_front();
        }
        history.network_tx_history.push_back(tx_rate);
    });

    let history = resource_history.read();

    rsx! {
        div { class: "resource-graphs-container",
            ResourceGraphCard {
                title: "CPU Usage",
                data: history.cpu_history.iter().copied().collect::<Vec<_>>(),
                display_value: history.smoothed_cpu,
                unit: "%",
                max_value: 100.0,
                color: "#4dabf7"
            }
            ResourceGraphCard {
                title: "Memory Usage",
                data: history.memory_history.iter().copied().collect::<Vec<_>>(),
                display_value: history.smoothed_mem,
                unit: "%",
                max_value: 100.0,
                color: "#51cf66"
            }
            ResourceGraphCard {
                title: "Network RX",
                data: history.network_rx_history.iter().copied().collect::<Vec<_>>(),
                display_value: history.smoothed_rx,
                unit: "KB/s",
                max_value: 0.0, // Auto scale
                color: "#ff6b6b"
            }
            ResourceGraphCard {
                title: "Network TX",
                data: history.network_tx_history.iter().copied().collect::<Vec<_>>(),
                display_value: history.smoothed_tx,
                unit: "KB/s",
                max_value: 0.0, // Auto scale
                color: "#ffa94d"
            }
        }
    }
}

#[component]
fn ResourceGraphCard(
    title: String,
    data: Vec<f32>,
    display_value: f32,
    unit: String,
    max_value: f32,
    color: String,
) -> Element {
    // Use the smoothed display value instead of the raw last value
    let current_value = display_value;

    // Calculate max for auto-scaling with dynamic range and padding
    let display_max = if max_value > 0.0 {
        max_value
    } else {
        // For auto-scaling, use the max value with 20% padding for better visualization
        let data_max = data.iter().copied().fold(0.0f32, f32::max);
        if data_max > 0.0 {
            (data_max * 1.2).max(10.0) // At least 10 units for scale
        } else {
            10.0 // Minimum scale
        }
    };

    // Calculate average
    let avg_value = if !data.is_empty() {
        data.iter().sum::<f32>() / data.len() as f32
    } else {
        0.0
    };

    rsx! {
        div { class: "panel resource-graph-card",
            div { class: "graph-card-header",
                h3 { class: "graph-title", "{title}" }
                div { class: "graph-stats",
                    span { class: "current-value", "{current_value:.1} {unit}" }
                    span { class: "avg-value", "Avg: {avg_value:.1} {unit}" }
                }
            }

            div { class: "graph-container",
                svg {
                    class: "resource-graph",
                    view_box: "0 0 300 100",
                    preserve_aspect_ratio: "none",

                    // Grid lines
                    for i in 0..5 {
                        line {
                            x1: "0",
                            y1: format!("{}", i * 25),
                            x2: "300",
                            y2: format!("{}", i * 25),
                            stroke: "var(--border)",
                            stroke_width: "0.5",
                            opacity: "0.3"
                        }
                    }

                    // Data line and area
                    if !data.is_empty() {
                        {
                            let line_points = data.iter()
                                .enumerate()
                                .map(|(i, &value)| {
                                    let x = (i as f32 / (MAX_DATA_POINTS - 1) as f32) * 300.0;
                                    let y = 100.0 - ((value / display_max).min(1.0) * 100.0);
                                    (x, y)
                                })
                                .collect::<Vec<_>>();

                            let line_path = line_points.iter()
                                .map(|(x, y)| format!("{},{}", x, y))
                                .collect::<Vec<_>>()
                                .join(" ");

                            // Create properly closed area path
                            let mut area_points = line_points.clone();
                            // Add bottom-right corner
                            area_points.push((300.0, 100.0));
                            // Add bottom-left corner
                            area_points.push((0.0, 100.0));

                            let area_path = area_points.iter()
                                .map(|(x, y)| format!("{},{}", x, y))
                                .collect::<Vec<_>>()
                                .join(" ");

                            rsx! {
                                // Fill area under the line (render first so line is on top)
                                polygon {
                                    points: "{area_path}",
                                    fill: "{color}",
                                    opacity: "0.2",
                                    stroke: "none"
                                }

                                // Line on top
                                polyline {
                                    points: "{line_path}",
                                    fill: "none",
                                    stroke: "{color}",
                                    stroke_width: "2",
                                    stroke_linejoin: "round",
                                    stroke_linecap: "round"
                                }
                            }
                        }
                    }
                }
            }

            div { class: "graph-footer",
                span { class: "graph-label", "Last {MAX_DATA_POINTS} samples" }
                span { class: "graph-range", "Max: {display_max:.1} {unit}" }
            }
        }
    }
}
