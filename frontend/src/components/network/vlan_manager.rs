use dioxus::prelude::*;
use dioxus::document;
use serde::{Serialize, Deserialize};

#[component]
pub fn VlanManager() -> Element {
    let mut vlans = use_signal(|| Vec::<VlanConfig>::new());
    let loading = use_signal(|| false);
    let mut show_create_dialog = use_signal(|| false);
    let mut show_edit_dialog = use_signal(|| false);
    let mut editing_vlan = use_signal(|| Option::<VlanConfig>::None);
    
    // Load VLANs on mount
    use_effect(move || {
        spawn(async move {
            // Try to load from localStorage first
            let eval_result = document::eval(
                r#"localStorage.getItem('network_vlans')"#
            ).await;
            
            if let Ok(result_value) = eval_result {
                if let Ok(result_str) = serde_json::from_value::<String>(result_value.clone()) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<VlanConfig>>(&result_str) {
                        vlans.set(loaded);
                        return;
                    }
                }
            }
            
            // Generate mock data only if nothing in localStorage
            let mock_vlans = vec![
                VlanConfig {
                    id: 10,
                    name: "production".to_string(),
                    parent_interface: "eth0".to_string(),
                    ip_config: Some("192.168.10.1/24".to_string()),
                    enabled: true,
                },
                VlanConfig {
                    id: 20,
                    name: "development".to_string(),
                    parent_interface: "eth0".to_string(),
                    ip_config: Some("192.168.20.1/24".to_string()),
                    enabled: true,
                },
                VlanConfig {
                    id: 30,
                    name: "guest".to_string(),
                    parent_interface: "eth1".to_string(),
                    ip_config: Some("192.168.30.1/24".to_string()),
                    enabled: false,
                },
            ];
            vlans.set(mock_vlans);
        });
    });
    
    // Save to localStorage whenever VLANs change
    use_effect(move || {
        let vlan_list = vlans.read().clone();
        spawn(async move {
            if let Ok(json) = serde_json::to_string(&vlan_list) {
                let escaped = json.replace('\'', "\\\\'");
                let script = format!("localStorage.setItem('network_vlans', '{}')", escaped); 
                let _ = document::eval(&script).await;
            }
        });
    });

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "VLAN Configuration" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            spawn(async move {
                                // Refresh VLANs
                            });
                        },
                        "🔄 Refresh"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_dialog.set(true),
                        "+ Create VLAN"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if vlans.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🔀" }
                        div { class: "empty-title", "No VLANs configured" }
                        div { class: "empty-description",
                            "VLANs allow you to segment network traffic for better security and organization."
                        }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "VLAN ID" }
                                th { "Name" }
                                th { "Parent Interface" }
                                th { "IP Address" }
                                th { "Status" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for vlan in vlans.read().iter() {
                                VlanRow { 
                                    vlan: vlan.clone(),
                                    on_edit: move |v| {
                                        editing_vlan.set(Some(v));
                                        show_edit_dialog.set(true);
                                    },
                                    on_delete: move |id| {
                                        tracing::info!("[FRONTEND] Deleting VLAN ID: {}", id);
                                        vlans.write().retain(|v| v.id != id);
                                        tracing::info!("[BACKEND REQUEST] Delete VLAN: {}", id);
                                        // Persist to localStorage
                                        spawn(async move {
                                            if let Ok(json) = serde_json::to_string(&*vlans.read()) {
                                                let escaped = json.replace('\'', "\\\\'");
                                                let script = format!("localStorage.setItem('network_vlans', '{}')", escaped); 
                                                let _ = document::eval(&script).await;
                                            }
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if *show_edit_dialog.read() {
            if let Some(v) = editing_vlan.read().clone() {
                EditVlanDialog {
                    vlan: v,
                    on_close: move |_| show_edit_dialog.set(false),
                    on_save: move |updated: VlanConfig| {
                        tracing::info!("[FRONTEND] Updating VLAN: {} (ID: {})", updated.name, updated.id);
                        let mut vlan_list = vlans.write();
                        if let Some(pos) = vlan_list.iter().position(|v| v.id == updated.id) {
                            vlan_list[pos] = updated.clone();
                        }
                        tracing::info!("[BACKEND REQUEST] Update VLAN: {:?}", updated);
                        show_edit_dialog.set(false);
                    }
                }
            }
        }

        if *show_create_dialog.read() {
            CreateVlanDialog {
                on_close: move |_| show_create_dialog.set(false),
                on_create: move |vlan: VlanConfig| {
                    tracing::info!("[FRONTEND] VLAN creation requested: {:?}", vlan);
                    tracing::info!("[BACKEND REQUEST] Creating VLAN: id={}, name={}, parent={}, ip={:?}", 
                        vlan.id, vlan.name, vlan.parent_interface, vlan.ip_config);
                    vlans.write().push(vlan.clone());
                    tracing::info!("[FRONTEND] VLAN added to UI: {} (ID: {})", vlan.name, vlan.id);
                    show_create_dialog.set(false);
                }
            }
        }
    }
}

#[component]
fn VlanRow(vlan: VlanConfig, on_edit: EventHandler<VlanConfig>, on_delete: EventHandler<u16>) -> Element {
    let vlan_id = vlan.id;
    
    let status_class = if vlan.enabled { "status-running" } else { "status-stopped" };
    
    rsx! {
        tr { class: "table-row",
            td { "{vlan.id}" }
            td {
                div { class: "cell-main", "{vlan.name}" }
            }
            td { "{vlan.parent_interface}" }
            td {
                if let Some(ref ip) = vlan.ip_config {
                    "{ip}"
                } else {
                    span { class: "muted", "Not configured" }
                }
            }
            td {
                span { class: "status-badge {status_class}",
                    "● ",
                    if vlan.enabled { "Enabled" } else { "Disabled" }
                }
            }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button { 
                        class: "action-btn", 
                        title: "Edit",
                        onclick: move |_| {
                            on_edit.call(vlan.clone());
                        },
                        "✏" 
                    }
                    button { 
                        class: "action-btn action-danger", 
                        title: "Delete",
                        onclick: move |_| {
                            on_delete.call(vlan_id);
                        },
                        "🗑" 
                    }
                }
            }
        }
    }
}

#[component]
fn EditVlanDialog(
    vlan: VlanConfig,
    on_close: EventHandler<()>,
    on_save: EventHandler<VlanConfig>
) -> Element {
    let mut name = use_signal(|| vlan.name.clone());
    let mut parent_interface = use_signal(|| vlan.parent_interface.clone());
    let mut ip_config = use_signal(|| vlan.ip_config.clone().unwrap_or_default());
    let mut enabled = use_signal(|| vlan.enabled);

    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Edit VLAN {vlan.id}" }
                
                div { class: "form-group",
                    label { "Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{name}",
                        oninput: move |e| name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Parent Interface" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{parent_interface}",
                        oninput: move |e| parent_interface.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "IP Configuration" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{ip_config}",
                        placeholder: "192.168.1.1/24",
                        oninput: move |e| ip_config.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label {
                        input {
                            r#type: "checkbox",
                            checked: enabled(),
                            oninput: move |e| enabled.set(e.checked())
                        }
                        " Enabled"
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
                            let updated = VlanConfig {
                                id: vlan.id,
                                name: name().clone(),
                                parent_interface: parent_interface().clone(),
                                ip_config: if ip_config().is_empty() { None } else { Some(ip_config().clone()) },
                                enabled: enabled(),
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
fn CreateVlanDialog(
    on_close: EventHandler<()>,
    on_create: EventHandler<VlanConfig>
) -> Element {
    let mut vlan_id = use_signal(|| String::new());
    let mut vlan_name = use_signal(|| String::new());
    let mut parent_interface = use_signal(|| String::new());
    let mut ip_address = use_signal(|| String::new());
    let mut netmask = use_signal(|| String::from("24"));

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 { "Create VLAN" }

                div { class: "form-group",
                    label { "VLAN ID" }
                    input {
                        class: "input",
                        r#type: "number",
                        placeholder: "1-4094",
                        min: 1,
                        max: 4094,
                        value: "{vlan_id}",
                        oninput: move |e| vlan_id.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "VLAN Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., production",
                        value: "{vlan_name}",
                        oninput: move |e| vlan_name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Parent Interface" }
                    select {
                        class: "input",
                        style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                        value: "{parent_interface}",
                        onchange: move |e| parent_interface.set(e.value().clone()),
                        option { value: "", style: "background: #23262d; color: #e4e6eb;", "Select interface..." }
                        option { value: "eth0", style: "background: #23262d; color: #e4e6eb;", "eth0" }
                        option { value: "eth1", style: "background: #23262d; color: #e4e6eb;", "eth1" }
                        option { value: "enp0s3", style: "background: #23262d; color: #e4e6eb;", "enp0s3" }
                    }
                }

                div { class: "form-group",
                    label { "IP Address (Optional)" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., 192.168.1.1",
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
                            let vlan = VlanConfig {
                                id: vlan_id.read().parse().unwrap_or(0),
                                name: vlan_name.read().clone(),
                                parent_interface: parent_interface.read().clone(),
                                ip_config: if !ip_address.read().is_empty() {
                                    Some(format!("{}/{}", ip_address.read(), netmask.read()))
                                } else {
                                    None
                                },
                                enabled: true,
                            };
                            tracing::info!("VLAN creation requested: id={}, name={}, parent={}, ip={:?}", 
                                vlan.id, vlan.name, vlan.parent_interface, vlan.ip_config);
                            on_create.call(vlan);
                        },
                        disabled: vlan_id.read().is_empty() || parent_interface.read().is_empty(),
                        "Create"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct VlanConfig {
    id: u16,
    name: String,
    parent_interface: String,
    ip_config: Option<String>,
    enabled: bool,
}
