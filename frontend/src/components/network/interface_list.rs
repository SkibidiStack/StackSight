use dioxus::prelude::*;
use serde::{Serialize, Deserialize};

#[component]
pub fn InterfaceList() -> Element {
    let mut interfaces = use_signal(|| Vec::<NetworkInterface>::new());
    let loading = use_signal(|| false);
    let mut selected_interface = use_signal(|| Option::<String>::None);
    let mut show_create_bridge = use_signal(|| false);
    let mut show_edit_dialog = use_signal(|| false);
    let mut editing_interface = use_signal(|| Option::<NetworkInterface>::None);

    // Load interfaces on mount - data persists in memory only for now
    use_effect(move || {
        spawn(async move {
            // Only load mock data if list is empty
            if interfaces.read().is_empty() {
                let mock_interfaces = vec![
                NetworkInterface {
                    name: "eth0".to_string(),
                    display_name: "Ethernet 0".to_string(),
                    mac_address: Some("52:54:00:12:34:56".to_string()),
                    ip_addresses: vec![IpConfiguration {
                        address: "192.168.1.100".to_string(),
                        netmask: "/24".to_string(),
                    }],
                    status: InterfaceStatus::Up,
                    mtu: 1500,
                    interface_type: InterfaceType::Ethernet,
                },
                NetworkInterface {
                    name: "wlan0".to_string(),
                    display_name: "Wireless 0".to_string(),
                    mac_address: Some("a4:5e:60:c2:89:1f".to_string()),
                    ip_addresses: vec![IpConfiguration {
                        address: "192.168.1.105".to_string(),
                        netmask: "/24".to_string(),
                    }],
                    status: InterfaceStatus::Up,
                    mtu: 1500,
                    interface_type: InterfaceType::Wireless,
                },
                NetworkInterface {
                    name: "lo".to_string(),
                    display_name: "Loopback".to_string(),
                    mac_address: None,
                    ip_addresses: vec![IpConfiguration {
                        address: "127.0.0.1".to_string(),
                        netmask: "/8".to_string(),
                    }],
                    status: InterfaceStatus::Up,
                    mtu: 65536,
                    interface_type: InterfaceType::Loopback,
                },
            ];
            interfaces.set(mock_interfaces);
            }
        });
    });

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Network Interfaces" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            spawn(async move {
                                // Refresh interfaces
                            });
                        },
                        "🔄 Refresh"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            show_create_bridge.set(true);
                        },
                        "+ Create Bridge"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if interfaces.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🌐" }
                        div { class: "empty-title", "No network interfaces" }
                        div { class: "empty-description", "No network interfaces detected on this system." }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "Interface" }
                                th { "Status" }
                                th { "Type" }
                                th { "IP Address" }
                                th { "MAC Address" }
                                th { "MTU" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for iface in interfaces.read().iter() {
                                InterfaceRow {
                                    interface: iface.clone(),
                                    selected: selected_interface.read().as_ref() == Some(&iface.name),
                                    on_select: move |name: String| {
                                        selected_interface.set(Some(name));
                                    },
                                    on_edit: move |i| {
                                        editing_interface.set(Some(i));
                                        show_edit_dialog.set(true);
                                    },
                                    on_delete: move |name| {
                                        tracing::info!("[FRONTEND] Deleting interface: {}", name);
                                        interfaces.write().retain(|i| i.name != name);
                                        tracing::info!("[BACKEND REQUEST] Delete interface: {}", name);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Edit Interface Dialog
            if *show_edit_dialog.read() {
                if let Some(iface) = editing_interface.read().clone() {
                    EditInterfaceDialog {
                        interface: iface,
                        on_close: move |_| show_edit_dialog.set(false),
                        on_save: move |updated: NetworkInterface| {
                            tracing::info!("[FRONTEND] Updating interface: {}", updated.name);
                            let mut ifaces = interfaces.write();
                            if let Some(pos) = ifaces.iter().position(|i| i.name == updated.name) {
                                ifaces[pos] = updated.clone();
                            }
                            tracing::info!("[BACKEND REQUEST] Update interface: {:?}", updated);
                            show_edit_dialog.set(false);
                        }
                    }
                }
            }

            // Create Bridge Dialog
            if *show_create_bridge.read() {
                CreateBridgeDialog {
                    on_close: move |_| show_create_bridge.set(false),
                    on_create: move |bridge: BridgeConfig| {
                        tracing::info!("[FRONTEND] Bridge creation requested: {:?}", bridge);
                        tracing::info!("[BACKEND REQUEST] Creating bridge: name={}, interfaces={:?}, ip={:?}", 
                            bridge.name, bridge.interfaces, bridge.ip_config);
                        
                        // Optimistically update UI
                        let new_interface = NetworkInterface {
                            name: bridge.name.clone(),
                            display_name: format!("Bridge: {}", bridge.name),
                            mac_address: None,
                            ip_addresses: if let Some(ip) = bridge.ip_config.clone() {
                                vec![IpConfiguration {
                                    address: ip.split('/').next().unwrap_or("").to_string(),
                                    netmask: format!("/{}", ip.split('/').nth(1).unwrap_or("24")),
                                }]
                            } else {
                                vec![]
                            },
                            status: InterfaceStatus::Up,
                            mtu: 1500,
                            interface_type: InterfaceType::Bridge,
                        };
                        interfaces.write().push(new_interface);
                        tracing::info!("[FRONTEND] Bridge added to UI: {}", bridge.name);
                        show_create_bridge.set(false);
                    }
                }
            }
        }
    }
}

#[component]
fn EditInterfaceDialog(
    interface: NetworkInterface,
    on_close: EventHandler<()>,
    on_save: EventHandler<NetworkInterface>
) -> Element {
    let mut display_name = use_signal(|| interface.display_name.clone());
    let mut mtu = use_signal(|| interface.mtu.to_string());
    let mut ip_address = use_signal(|| {
        interface.ip_addresses.first()
            .map(|ip| ip.address.clone())
            .unwrap_or_default()
    });
    let mut netmask = use_signal(|| {
        interface.ip_addresses.first()
            .map(|ip| ip.netmask.clone())
            .unwrap_or_default()
    });

    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Edit Interface: {interface.name}" }
                
                div { class: "form-group",
                    label { "Display Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{display_name}",
                        oninput: move |e| display_name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "IP Address" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{ip_address}",
                        oninput: move |e| ip_address.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Netmask" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{netmask}",
                        placeholder: "/24",
                        oninput: move |e| netmask.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "MTU" }
                    input {
                        class: "input",
                        r#type: "number",
                        value: "{mtu}",
                        oninput: move |e| mtu.set(e.value().clone())
                    }
                }

                div { class: "form-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let updated = NetworkInterface {
                                name: interface.name.clone(),
                                display_name: display_name().clone(),
                                mac_address: interface.mac_address.clone(),
                                ip_addresses: vec![IpConfiguration {
                                    address: ip_address().clone(),
                                    netmask: netmask().clone(),
                                }],
                                status: interface.status.clone(),
                                mtu: mtu().parse().unwrap_or(1500),
                                interface_type: interface.interface_type.clone(),
                            };
                            on_save.call(updated);
                        },
                        "Save"
                    }
                }
            }
        }
    }
}

#[component]
fn InterfaceRow(
    interface: NetworkInterface,
    selected: bool,
    on_select: EventHandler<String>,
    on_edit: EventHandler<NetworkInterface>,
    on_delete: EventHandler<String>
) -> Element {
    let interface_name = interface.name.clone();
    let interface_for_edit = interface.clone();
    
    let status_class = match interface.status {
        InterfaceStatus::Up => "status-running",
        InterfaceStatus::Down => "status-stopped",
        InterfaceStatus::Unknown => "status-warning",
    };

    let ip_addrs = if !interface.ip_addresses.is_empty() {
        interface.ip_addresses.iter()
            .map(|ip| format!("{}{}", ip.address, ip.netmask))
            .collect::<Vec<_>>()
            .join(", ")
    } else {
        "—".to_string()
    };

    rsx! {
        tr {
            class: if selected { "table-row selected" } else { "table-row" },
            onclick: move |_| on_select.call(interface.name.clone()),
            
            td { class: "col-name",
                div { class: "cell-main", "{interface.display_name}" }
                div { class: "cell-sub", "{interface.name}" }
            }
            td { class: "col-status",
                span { class: "status-badge {status_class}",
                    "● ",
                    match interface.status {
                        InterfaceStatus::Up => "Up",
                        InterfaceStatus::Down => "Down",
                        InterfaceStatus::Unknown => "Unknown",
                    }
                }
            }
            td { "{interface.interface_type:?}" }
            td { "{ip_addrs}" }
            td { "{interface.mac_address.as_deref().unwrap_or(\"—\")}" }
            td { "{interface.mtu}" }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button { 
                        class: "action-btn", 
                        title: "Edit",
                        onclick: move |_| {
                            on_edit.call(interface_for_edit.clone());
                        },
                        "✏" 
                    }
                    button { 
                        class: "action-btn action-danger", 
                        title: "Delete",
                        onclick: move |_| {
                            let name = interface_name.clone();
                            on_delete.call(name);
                        },
                        "🗑" 
                    }
                }
            }
        }
    }
}

// Simplified models for frontend (should match backend models)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct NetworkInterface {
    name: String,
    display_name: String,
    mac_address: Option<String>,
    ip_addresses: Vec<IpConfiguration>,
    status: InterfaceStatus,
    mtu: u32,
    interface_type: InterfaceType,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct IpConfiguration {
    address: String,
    netmask: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum InterfaceStatus {
    Up,
    Down,
    Unknown,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
enum InterfaceType {
    Ethernet,
    Wireless,
    Virtual,
    Loopback,
    Bridge,
    Vlan,
    Other,
}

#[component]
fn CreateBridgeDialog(
    on_close: EventHandler<()>,
    on_create: EventHandler<BridgeConfig>
) -> Element {
    let mut bridge_name = use_signal(|| String::new());
    let mut interface1 = use_signal(|| String::new());
    let mut interface2 = use_signal(|| String::new());
    let mut ip_address = use_signal(|| String::new());
    let mut netmask = use_signal(|| String::from("24"));

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 { "Create Network Bridge" }

                div { class: "form-group",
                    label { "Bridge Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., br0",
                        value: "{bridge_name}",
                        oninput: move |e| bridge_name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "First Interface" }
                    select {
                        class: "input",
                        style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                        value: "{interface1}",
                        onchange: move |e| interface1.set(e.value().clone()),
                        option { value: "", style: "background: #23262d; color: #e4e6eb;", "Select interface..." }
                        option { value: "eth0", style: "background: #23262d; color: #e4e6eb;", "eth0" }
                        option { value: "eth1", style: "background: #23262d; color: #e4e6eb;", "eth1" }
                        option { value: "enp0s3", style: "background: #23262d; color: #e4e6eb;", "enp0s3" }
                        option { value: "enp0s8", style: "background: #23262d; color: #e4e6eb;", "enp0s8" }
                    }
                }

                div { class: "form-group",
                    label { "Second Interface (Optional)" }
                    select {
                        class: "input",
                        style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                        value: "{interface2}",
                        onchange: move |e| interface2.set(e.value().clone()),
                        option { value: "", style: "background: #23262d; color: #e4e6eb;", "None" }
                        option { value: "eth0", style: "background: #23262d; color: #e4e6eb;", "eth0" }
                        option { value: "eth1", style: "background: #23262d; color: #e4e6eb;", "eth1" }
                        option { value: "enp0s3", style: "background: #23262d; color: #e4e6eb;", "enp0s3" }
                        option { value: "enp0s8", style: "background: #23262d; color: #e4e6eb;", "enp0s8" }
                    }
                }

                div { class: "form-group",
                    label { "IP Address (Optional)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., 192.168.100.1",
                        value: "{ip_address}",
                        oninput: move |e| ip_address.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Netmask Prefix" }
                    input {
                        class: "input",
                        r#type: "number",
                        placeholder: "24",
                        min: 1,
                        max: 32,
                        value: "{netmask}",
                        oninput: move |e| netmask.set(e.value().clone())
                    }
                }

                div { class: "form-actions",
                    button {
                        class: "btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn-primary",
                        onclick: move |_| {
                            let bridge = BridgeConfig {
                                name: bridge_name.read().clone(),
                                interfaces: vec![
                                    interface1.read().clone(),
                                    interface2.read().clone()
                                ].into_iter().filter(|s| !s.is_empty()).collect(),
                                ip_config: if !ip_address.read().is_empty() {
                                    Some(format!("{}/{}", ip_address.read(), netmask.read()))
                                } else {
                                    None
                                },
                            };
                            tracing::info!("Bridge creation requested: {:?}", bridge);
                            on_create.call(bridge);
                        },
                        disabled: bridge_name.read().is_empty() || interface1.read().is_empty(),
                        "Create Bridge"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct BridgeConfig {
    name: String,
    interfaces: Vec<String>,
    ip_config: Option<String>,
}
