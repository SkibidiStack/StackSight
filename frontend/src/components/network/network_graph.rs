use dioxus::prelude::*;
use crate::state::AppState;
use crate::state::messages::{NetworkDevice, NetworkDeviceType, NetworkTopologyData, Command};
use crate::app::BackendBridge;

// ── Layout helpers ─────────────────────────────────────────────────────────────

/// Lay devices out in a circular arc around the gateway node at the centre.
fn layout_positions(count: usize, cx: f64, cy: f64, radius: f64) -> Vec<(f64, f64)> {
    if count == 0 {
        return Vec::new();
    }
    (0..count)
        .map(|i| {
            let angle = (i as f64) * std::f64::consts::TAU / (count as f64)
                - std::f64::consts::FRAC_PI_2; // start at top
            let x = cx + radius * angle.cos();
            let y = cy + radius * angle.sin();
            (x, y)
        })
        .collect()
}

fn device_icon(dt: &NetworkDeviceType) -> &'static str {
    match dt {
        NetworkDeviceType::Gateway => "🌐",
        NetworkDeviceType::LocalMachine => "💻",
        NetworkDeviceType::Host => "🖥",
        NetworkDeviceType::Unknown => "❓",
    }
}

fn device_label(d: &NetworkDevice) -> String {
    if let Some(ref hn) = d.hostname {
        format!("{}\n{}", hn, d.ip)
    } else {
        d.ip.clone()
    }
}

fn device_color(dt: &NetworkDeviceType) -> &'static str {
    match dt {
        NetworkDeviceType::Gateway => "#f59e0b",      // amber
        NetworkDeviceType::LocalMachine => "#4dabf7",  // blue
        NetworkDeviceType::Host => "#51cf66",          // green
        NetworkDeviceType::Unknown => "#8b92a0",       // muted
    }
}

// ── Main component ─────────────────────────────────────────────────────────────

#[component]
pub fn NetworkGraph() -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let topology: Option<NetworkTopologyData> = app_state.read().network.topology.clone();
    let scanning = app_state.read().network.topology_scanning;
    let mut selected = use_signal(|| Option::<String>::None); // selected IP

    let on_scan = move |_| {
        let mut state = app_state.write();
        state.network.topology_scanning = true;
        tracing::info!("Initiating network scan via bridge...");
        bridge.send(Command::NetworkScanDevices);
    };

    rsx! {
        div { class: "panel network-graph-panel",
            div { class: "panel-header",
                h2 { "Network Topology" }
                div { class: "panel-actions",
                    if scanning {
                        span { class: "scanning-badge", "⏳ Scanning…" }
                    }
                    button {
                        class: if scanning { "btn btn-secondary disabled" } else { "btn btn-primary" },
                        onclick: on_scan,
                        disabled: scanning,
                        "🔍 Scan Network"
                    }
                }
            }

            div { class: "panel-content",
                match &topology {
                    None => rsx! {
                        div { class: "empty-state",
                            div { class: "empty-icon", "🌐" }
                            div { class: "empty-title", "No topology data" }
                            div { class: "empty-description",
                                "Click \"Scan Network\" to discover devices on your local network."
                            }
                        }
                    },
                    Some(topo) => {
                        let topo = topo.clone();
                        let sel = selected.read().clone();
                        let selected_device = sel.as_ref().and_then(|ip| {
                            topo.devices.iter().find(|d| &d.ip == ip).cloned()
                        });

                        rsx! {
                            div { 
                                style: "display: flex; gap: 20px; height: 100%; align-items: center; justify-content: center;",
                                // ── SVG graph ───────────────────────────────
                                div { 
                                    style: "flex: 1; display: flex; align-items: center; justify-content: center; padding: 20px;",
                                    NetworkGraphSvg {
                                        topology: topo.clone(),
                                        selected_ip: sel.clone(),
                                        on_select: move |ip: String| {
                                            let mut s = selected.write();
                                            if s.as_ref() == Some(&ip) {
                                                *s = None;
                                            } else {
                                                *s = Some(ip);
                                            }
                                        }
                                    }
                                }

                                // ── Device detail panel ─────────────────────
                                div { 
                                    style: "width: 350px; max-height: 600px; overflow-y: auto; background: #23262d; border-radius: 12px; padding: 20px; display: flex; flex-direction: column; gap: 16px;",
                                    div { 
                                        style: "display: flex; flex-direction: column; gap: 12px; padding-bottom: 16px; border-bottom: 1px solid #3a3d47;",
                                        if let Some(ref gw) = topo.gateway {
                                            div { 
                                                style: "display: flex; justify-content: space-between; align-items: center;",
                                                span { style: "color: #888; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px;", "Gateway" }
                                                span { style: "color: #e4e6eb; font-weight: 500;", "{gw}" }
                                            }
                                        }
                                        if let Some(ref lip) = topo.local_ip {
                                            div { 
                                                style: "display: flex; justify-content: space-between; align-items: center;",
                                                span { style: "color: #888; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px;", "Local IP" }
                                                span { style: "color: #e4e6eb; font-weight: 500;", "{lip}" }
                                            }
                                        }
                                        div { 
                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                            span { style: "color: #888; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px;", "Devices" }
                                            span { style: "display: inline-block; padding: 4px 10px; background: #4dabf7; color: white; border-radius: 12px; font-size: 12px; font-weight: 600;", "{topo.devices.len()}" }
                                        }
                                        div { 
                                            style: "display: flex; justify-content: space-between; align-items: center;",
                                            span { style: "color: #888; font-size: 12px; text-transform: uppercase; letter-spacing: 0.5px;", "Scanned" }
                                            span { style: "color: #888; font-size: 12px;", "{topo.scan_time}" }
                                        }
                                    }

                                    if let Some(dev) = selected_device {
                                        DeviceDetailCard { device: dev }
                                    } else {
                                        // Device list
                                        div { style: "display: flex; flex-direction: column; gap: 8px; overflow-y: auto;",
                                            for device in topo.devices.iter() {
                                                DeviceListRow {
                                                    device: device.clone(),
                                                    selected: sel.as_ref() == Some(&device.ip),
                                                    on_click: {
                                                        let ip = device.ip.clone();
                                                        move |_| {
                                                            let mut s = selected.write();
                                                            if s.as_ref() == Some(&ip) {
                                                                *s = None;
                                                            } else {
                                                                *s = Some(ip.clone());
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ── SVG topology graph ─────────────────────────────────────────────────────────

#[component]
fn NetworkGraphSvg(
    topology: NetworkTopologyData,
    selected_ip: Option<String>,
    on_select: EventHandler<String>,
) -> Element {
    let width = 520.0f64;
    let height = 460.0f64;
    let cx = width / 2.0;
    let cy = height / 2.0;

    // Separate gateway/local from other hosts
    let (_special, _hosts): (Vec<_>, Vec<_>) = topology
        .devices
        .iter()
        .partition(|d| {
            d.device_type == NetworkDeviceType::Gateway
                || d.device_type == NetworkDeviceType::LocalMachine
        });

    // Gateway node at centre; local machine slightly offset if present
    let gateway_pos: Option<(f64, f64)> = topology
        .devices
        .iter()
        .find(|d| d.device_type == NetworkDeviceType::Gateway)
        .map(|_| (cx, cy));

    // All non-gateway devices placed in a ring
    let ring_devices: Vec<&NetworkDevice> = topology
        .devices
        .iter()
        .filter(|d| d.device_type != NetworkDeviceType::Gateway)
        .collect();
    let ring_pos = layout_positions(ring_devices.len(), cx, cy, 170.0);

    let _center_ip = topology
        .devices
        .iter()
        .find(|d| d.device_type == NetworkDeviceType::Gateway)
        .map(|d| d.ip.clone());

    rsx! {
        svg {
            width: "{width}",
            height: "{height}",
            view_box: "0 0 {width} {height}",
            class: "network-svg",
            xmlns: "http://www.w3.org/2000/svg",

            // ── Background ───────────────────────────────────────────────────
            rect {
                width: "{width}",
                height: "{height}",
                rx: "12",
                fill: "var(--surface)",
            }

            // ── Concentric ring guide ────────────────────────────────────────
            circle {
                cx: "{cx}",
                cy: "{cy}",
                r: "170",
                fill: "none",
                stroke: "var(--border)",
                stroke_width: "1",
                stroke_dasharray: "4 6",
            }
            circle {
                cx: "{cx}",
                cy: "{cy}",
                r: "90",
                fill: "none",
                stroke: "var(--border-light)",
                stroke_width: "1",
                stroke_dasharray: "3 5",
            }

            // ── Edges from gateway to each device ────────────────────────────
            {ring_devices.iter().zip(ring_pos.iter()).map(|(dev, (dx, dy))| {
                let gx = gateway_pos.map(|(x, _)| x).unwrap_or(cx);
                let gy = gateway_pos.map(|(_, y)| y).unwrap_or(cy);
                let is_sel = selected_ip.as_deref() == Some(&dev.ip);
                let stroke = if is_sel { "var(--accent)" } else { "var(--border)" };
                let stroke_w = if is_sel { "2" } else { "1" };
                rsx! {
                    line {
                        key: "edge-{dev.ip}",
                        x1: "{gx}", y1: "{gy}",
                        x2: "{dx}", y2: "{dy}",
                        stroke: "{stroke}",
                        stroke_width: "{stroke_w}",
                        stroke_opacity: "0.7",
                    }
                }
            })}

            // ── Gateway node ─────────────────────────────────────────────────
            {gateway_pos.map(|(gx, gy)| {
                let gw_dev = topology.devices.iter().find(|d| d.device_type == NetworkDeviceType::Gateway);
                let ip = gw_dev.map(|d| d.ip.as_str()).unwrap_or("gateway");
                let is_sel = selected_ip.as_deref() == Some(ip);
                let ring_stroke = if is_sel { "var(--accent)" } else { "#f59e0b" };
                let gw_key = format!("gw-{}", ip);
                rsx! {
                    g {
                        key: "{gw_key}",
                        class: "node-group clickable",
                        onclick: {
                            let ip = ip.to_string();
                            let h = on_select.clone();
                            move |_| h.call(ip.clone())
                        },
                        circle { cx: "{gx}", cy: "{gy}", r: "30", fill: "#f59e0b33", stroke: "{ring_stroke}", stroke_width: if is_sel { "3" } else { "2" } }
                        text { x: "{gx}", y: "{gy + 5.0}", text_anchor: "middle", font_size: "22", "🌐" }
                        text { x: "{gx}", y: "{gy + 48.0}", text_anchor: "middle", font_size: "10", fill: "var(--text)", "{ip}" }
                    }
                }
            })}

            // ── Ring nodes ───────────────────────────────────────────────────
            {ring_devices.iter().zip(ring_pos.iter()).map(|(dev, (dx, dy))| {
                let is_sel = selected_ip.as_deref() == Some(&dev.ip);
                let color = device_color(&dev.device_type);
                let icon = device_icon(&dev.device_type);
                let ring_stroke = if is_sel { "var(--accent)" } else { color };
                let ring_w = if is_sel { "3" } else { "2" };
                let label = dev.hostname.clone().unwrap_or_else(|| dev.ip.clone());
                // Truncate long labels
                let label = if label.len() > 16 { format!("{}…", &label[..14]) } else { label };
                let ip = dev.ip.clone();
                rsx! {
                    g {
                        key: "node-{dev.ip}",
                        class: "node-group clickable",
                        onclick: {
                            let h = on_select.clone();
                            let ip2 = ip.clone();
                            move |_| h.call(ip2.clone())
                        },
                        circle { cx: "{dx}", cy: "{dy}", r: "22", fill: "{color}22", stroke: "{ring_stroke}", stroke_width: "{ring_w}" }
                        text { x: "{dx}", y: "{dy + 7.0}", text_anchor: "middle", font_size: "16", "{icon}" }
                        text { x: "{dx}", y: "{dy + 36.0}", text_anchor: "middle", font_size: "9", fill: "var(--text)", "{label}" }
                        text { x: "{dx}", y: "{dy + 47.0}", text_anchor: "middle", font_size: "8", fill: "var(--muted)", "{ip}" }
                    }
                }
            })}

            // ── Empty state inside SVG if no devices ─────────────────────────
            if topology.devices.is_empty() {
                text { x: "{cx}", y: "{cy}", text_anchor: "middle", fill: "var(--muted)", font_size: "14", "No devices found" }
            }
        }
    }
}

// ── Sub-components ─────────────────────────────────────────────────────────────

#[component]
fn DeviceDetailCard(device: NetworkDevice) -> Element {
    let type_label = match &device.device_type {
        NetworkDeviceType::Gateway => "Gateway",
        NetworkDeviceType::LocalMachine => "This Machine",
        NetworkDeviceType::Host => "Host",
        NetworkDeviceType::Unknown => "Unknown",
    };
    let icon = device_icon(&device.device_type);
    let color = device_color(&device.device_type);

    rsx! {
        div { 
            style: "padding: 20px; background: #2a2d35; border-radius: 12px;",
            
            // Header with icon and identity
            div { 
                style: "display: flex; align-items: center; gap: 16px; margin-bottom: 20px; padding-bottom: 20px; border-bottom: 1px solid #3a3d47;",
                div { 
                    style: "width: 80px; height: 80px; display: flex; align-items: center; justify-content: center; background: linear-gradient(135deg, {color}22, {color}44); border: 2px solid {color}; border-radius: 12px; box-shadow: 0 4px 12px {color}33;",
                    span { style: "color: {color}; font-size: 48px;", "{icon}" }
                }
                div { style: "flex: 1;",
                    h3 { style: "margin: 0 0 8px 0; font-size: 20px; color: #e4e6eb;",
                        if let Some(ref hn) = device.hostname {
                            "{hn}"
                        } else {
                            "{type_label}"
                        }
                    }
                    div { style: "font-size: 16px; color: #b0b3b8; margin-bottom: 8px;", "{device.ip}" }
                    span { 
                        style: "display: inline-block; padding: 4px 12px; border-radius: 6px; background: {color}22; color: {color}; border: 1px solid {color}; font-size: 12px;", 
                        "{type_label}" 
                    }
                }
            }

            // Status indicator
            div { 
                style: if device.is_reachable {
                    "display: flex; align-items: center; gap: 12px; padding: 16px; margin-bottom: 20px; background: linear-gradient(135deg, #10b98122, #10b98144); border-left: 4px solid #10b981; border-radius: 8px;"
                } else {
                    "display: flex; align-items: center; gap: 12px; padding: 16px; margin-bottom: 20px; background: linear-gradient(135deg, #ef444422, #ef444444); border-left: 4px solid #ef4444; border-radius: 8px;"
                },
                span { 
                    style: if device.is_reachable { "font-size: 24px; color: #10b981;" } else { "font-size: 24px; color: #ef4444;" },
                    if device.is_reachable { "✓" } else { "✗" }
                }
                div {
                    div { style: "font-size: 11px; color: #888; text-transform: uppercase; letter-spacing: 0.5px;", "Connection Status" }
                    div { 
                        style: if device.is_reachable { "font-size: 14px; font-weight: 600; color: #10b981;" } else { "font-size: 14px; font-weight: 600; color: #ef4444;" },
                        if device.is_reachable { "Reachable" } else { "Unreachable" }
                    }
                }
            }

            // Information grid
            div { style: "display: grid; grid-template-columns: 1fr; gap: 12px;",
                // MAC Address
                if let Some(ref mac) = device.mac {
                    div { 
                        style: "display: flex; gap: 12px; padding: 14px; background: #23262d; border-radius: 8px; border: 1px solid #3a3d47;",
                        div { style: "font-size: 20px;", "🔖" }
                        div { style: "flex: 1;",
                            div { style: "font-size: 11px; color: #888; text-transform: uppercase; margin-bottom: 4px;", "MAC Address" }
                            div { style: "font-family: monospace; color: #e4e6eb; font-size: 13px;", "{mac}" }
                        }
                    }
                }

                // Hostname
                if let Some(ref hn) = device.hostname {
                    div { 
                        style: "display: flex; gap: 12px; padding: 14px; background: #23262d; border-radius: 8px; border: 1px solid #3a3d47;",
                        div { style: "font-size: 20px;", "🏷️" }
                        div { style: "flex: 1;",
                            div { style: "font-size: 11px; color: #888; text-transform: uppercase; margin-bottom: 4px;", "Hostname" }
                            div { style: "color: #e4e6eb; font-size: 13px;", "{hn}" }
                        }
                    }
                }

                // Interface
                if !device.interface.is_empty() {
                    div { 
                        style: "display: flex; gap: 12px; padding: 14px; background: #23262d; border-radius: 8px; border: 1px solid #3a3d47;",
                        div { style: "font-size: 20px;", "🔌" }
                        div { style: "flex: 1;",
                            div { style: "font-size: 11px; color: #888; text-transform: uppercase; margin-bottom: 4px;", "Interface" }
                            div { style: "color: #e4e6eb; font-size: 13px;", "{device.interface}" }
                        }
                    }
                }

                // Vendor
                if let Some(ref vendor) = device.vendor {
                    div { 
                        style: "display: flex; gap: 12px; padding: 14px; background: #23262d; border-radius: 8px; border: 1px solid #3a3d47;",
                        div { style: "font-size: 20px;", "🏢" }
                        div { style: "flex: 1;",
                            div { style: "font-size: 11px; color: #888; text-transform: uppercase; margin-bottom: 4px;", "Vendor" }
                            div { style: "color: #e4e6eb; font-size: 13px;", "{vendor}" }
                        }
                    }
                }

                // Device Type
                div { 
                    style: "display: flex; gap: 12px; padding: 14px; background: #23262d; border-radius: 8px; border: 1px solid #3a3d47;",
                    div { style: "font-size: 20px; color: {color};", "{icon}" }
                    div { style: "flex: 1;",
                        div { style: "font-size: 11px; color: #888; text-transform: uppercase; margin-bottom: 4px;", "Device Type" }
                        div { style: "color: #e4e6eb; font-size: 13px;", "{type_label}" }
                    }
                }
            }
        }
    }
}

#[component]
fn DeviceListRow(
    device: NetworkDevice,
    selected: bool,
    on_click: EventHandler<()>,
) -> Element {
    let icon = device_icon(&device.device_type);
    let color = device_color(&device.device_type);
    let label = device.hostname.clone().unwrap_or_else(|| device.ip.clone());
    let type_name = match &device.device_type {
        NetworkDeviceType::Gateway => "Gateway",
        NetworkDeviceType::LocalMachine => "This Machine",
        NetworkDeviceType::Host => "Host",
        NetworkDeviceType::Unknown => "Unknown",
    };

    rsx! {
        div {
            style: if selected {
                "display: flex; align-items: center; gap: 12px; padding: 12px; margin-bottom: 8px; background: {color}11; border-radius: 8px; border-left: 4px solid {color}; cursor: pointer; transition: all 0.2s;"
            } else {
                "display: flex; align-items: center; gap: 12px; padding: 12px; margin-bottom: 8px; background: #23262d; border-radius: 8px; border-left: 4px solid transparent; cursor: pointer; transition: all 0.2s; hover: background: #2a2d35;"
            },
            onclick: move |_| on_click.call(()),
            
            // Icon with colored background
            div { 
                style: "width: 40px; height: 40px; display: flex; align-items: center; justify-content: center; background: {color}22; border: 2px solid {color}66; border-radius: 8px; flex-shrink: 0;",
                span { style: "color: {color}; font-size: 20px;", "{icon}" }
            }
            
            // Device info
            div { style: "flex: 1; min-width: 0;",
                div { style: "display: flex; align-items: center; justify-content: space-between; margin-bottom: 4px;",
                    span { style: "color: #e4e6eb; font-weight: 500; font-size: 14px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;", "{label}" }
                    span { 
                        style: if device.is_reachable { "color: #10b981; font-size: 12px; margin-left: 8px;" } else { "color: #ef4444; font-size: 12px; margin-left: 8px;" },
                        if device.is_reachable { "●" } else { "○" }
                    }
                }
                if label != device.ip {
                    div { style: "color: #888; font-size: 12px; margin-bottom: 2px;", "{device.ip}" }
                }
                div { style: "display: flex; align-items: center; gap: 8px; font-size: 11px;",
                    span { style: "color: {color};", "{type_name}" }
                    if let Some(ref mac) = device.mac {
                        span { style: "color: #555;", "•" }
                        span { style: "font-family: monospace; color: #777;", "{mac}" }
                    }
                }
            }

            // Chevron indicator
            if selected {
                div { style: "color: {color}; font-size: 20px; margin-left: 8px;", "›" }
            }
        }
    }
}
