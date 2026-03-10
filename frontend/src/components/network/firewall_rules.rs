use dioxus::prelude::*;
use dioxus::document;
use serde::{Serialize, Deserialize};

#[component]
pub fn FirewallRules() -> Element {
    let mut rules = use_signal(|| Vec::<FirewallRule>::new());
    let mut loading = use_signal(|| false);
    let mut show_create_dialog = use_signal(|| false);
    let mut show_edit_dialog = use_signal(|| false);
    let mut editing_rule = use_signal(|| Option::<FirewallRule>::None);
    
    // Load rules on mount
    use_effect(move || {
        spawn(async move {
            // Try to load from localStorage first
            let eval_result = document::eval(
                r#"localStorage.getItem('firewall_rules')"#
            ).await;
            
            if let Ok(result_value) = eval_result {
                if let Ok(result_str) = serde_json::from_value::<String>(result_value.clone()) {
                    if let Ok(loaded) = serde_json::from_str::<Vec<FirewallRule>>(&result_str) {
                        rules.set(loaded);
                        return;
                    }
                }
            }
            
            // Generate mock data only if nothing in localStorage
            let mock_rules = vec![
                FirewallRule {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Allow SSH".to_string(),
                    enabled: true,
                    action: FirewallAction::Allow,
                    direction: TrafficDirection::Inbound,
                    protocol: Some("tcp".to_string()),
                    source_ip: Some("0.0.0.0/0".to_string()),
                    source_port: None,
                    destination_ip: None,
                    destination_port: Some(22),
                },
                FirewallRule {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Allow HTTP/HTTPS".to_string(),
                    enabled: true,
                    action: FirewallAction::Allow,
                    direction: TrafficDirection::Inbound,
                    protocol: Some("tcp".to_string()),
                    source_ip: Some("0.0.0.0/0".to_string()),
                    source_port: None,
                    destination_ip: None,
                    destination_port: Some(80),
                },
                FirewallRule {
                    id: uuid::Uuid::new_v4().to_string(),
                    name: "Block Outbound SMTP".to_string(),
                    enabled: false,
                    action: FirewallAction::Deny,
                    direction: TrafficDirection::Outbound,
                    protocol: Some("tcp".to_string()),
                    source_ip: None,
                    source_port: None,
                    destination_ip: Some("0.0.0.0/0".to_string()),
                    destination_port: Some(25),
                },
            ];
            rules.set(mock_rules);
        });
    });
    
    // Save to localStorage whenever rules change
    use_effect(move || {
        let rule_list = rules.read().clone();
        spawn(async move {
            if let Ok(json) = serde_json::to_string(&rule_list) {
                let escaped = json.replace('\'', "\\\\'");
                let script = format!("localStorage.setItem('firewall_rules', '{}')", escaped); 
                let _ = document::eval(&script).await;
            }
        });
    });

    rsx! {
        div { class: "panel",
            div { class: "panel-header",
                h2 { "Firewall Rules" }
                div { class: "panel-actions",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| {
                            spawn(async move {
                                loading.set(true);
                                // Refresh rules
                                loading.set(false);
                            });
                        },
                        "🔄 Refresh"
                    }
                    button {
                        class: "btn btn-warning",
                        "⚠ Flush All"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_create_dialog.set(true),
                        "+ Add Rule"
                    }
                }
            }

            div { class: "panel-content",
                if *loading.read() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "⏳" }
                        div { class: "empty-title", "Loading..." }
                    }
                } else if rules.read().is_empty() {
                    div { class: "empty-state",
                        div { class: "empty-icon", "🛡" }
                        div { class: "empty-title", "No firewall rules configured" }
                        div { class: "empty-description",
                            "Create rules to control network traffic and secure your system."
                        }
                    }
                } else {
                    table { class: "docker-table",
                        thead {
                            tr {
                                th { "Name" }
                                th { "Action" }
                                th { "Direction" }
                                th { "Protocol" }
                                th { "Source" }
                                th { "Destination" }
                                th { "Status" }
                                th { class: "col-actions", "Actions" }
                            }
                        }
                        tbody {
                            for rule in rules.read().iter() {
                                FirewallRuleRow { 
                                    rule: rule.clone(),
                                    on_edit: move |r| {
                                        editing_rule.set(Some(r));
                                        show_edit_dialog.set(true);
                                    },
                                    on_delete: move |id| {
                                        tracing::info!("[FRONTEND] Deleting firewall rule: {}", id);
                                        rules.write().retain(|r| r.id != id);
                                        tracing::info!("[BACKEND REQUEST] Delete firewall rule: {}", id);
                                        // Persist to localStorage
                                        spawn(async move {
                                            if let Ok(json) = serde_json::to_string(&*rules.read()) {
                                                let escaped = json.replace('\'', "\\\\'");
                                                let script = format!("localStorage.setItem('firewall_rules', '{}')", escaped); 
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

            if *show_edit_dialog.read() {
                if let Some(r) = editing_rule.read().clone() {
                    EditRuleDialog {
                        rule: r,
                        on_close: move |_| show_edit_dialog.set(false),
                        on_save: move |updated: FirewallRule| {
                            tracing::info!("[FRONTEND] Updating firewall rule: {}", updated.name);
                            let mut rule_list = rules.write();
                            if let Some(pos) = rule_list.iter().position(|r| r.id == updated.id) {
                                rule_list[pos] = updated.clone();
                            }
                            tracing::info!("[BACKEND REQUEST] Update firewall rule: {:?}", updated);
                            show_edit_dialog.set(false);
                        }
                    }
                }
            }

            if *show_create_dialog.read() {
                CreateRuleDialog {
                    on_close: move |_| show_create_dialog.set(false),
                    on_create: move |rule: FirewallRule| {
                        tracing::info!("[FRONTEND] Firewall rule creation requested: {:?}", rule);
                        tracing::info!("[BACKEND REQUEST] Creating firewall rule: name={}, action={:?}, direction={:?}, protocol={:?}, src={}:{:?}, dst={}:{:?}",
                            rule.name, rule.action, rule.direction, rule.protocol,
                            rule.source_ip.as_deref().unwrap_or("any"),
                            rule.source_port,
                            rule.destination_ip.as_deref().unwrap_or("any"),
                            rule.destination_port
                        );
                        rules.write().push(rule.clone());
                        tracing::info!("[FRONTEND] Firewall rule added to UI: {} (ID: {})", rule.name, rule.id);
                        show_create_dialog.set(false);
                    }
                }
            }
        }
    }
}

#[component]
fn FirewallRuleRow(rule: FirewallRule, on_edit: EventHandler<FirewallRule>, on_delete: EventHandler<String>) -> Element {
    let rule_id = rule.id.clone();
    
    let action_class = match rule.action {
        FirewallAction::Allow => "status-badge status-running",
        FirewallAction::Deny => "status-badge status-stopped",
        FirewallAction::Log => "status-badge status-warning",
    };

    rsx! {
        tr { class: "table-row",
            td {
                div { class: "cell-main", "{rule.name}" }
            }
            td {
                span { class: action_class, "{rule.action:?}" }
            }
            td { "{rule.direction:?}" }
            td { "{rule.protocol.as_deref().unwrap_or(\"Any\")}" }
            td { "{rule.source_ip.as_deref().unwrap_or(\"Any\")}" }
            td { "{rule.destination_ip.as_deref().unwrap_or(\"Any\")}" }
            td {
                span {
                    class: if rule.enabled { "status-badge status-running" } else { "status-badge status-stopped" },
                    "● ",
                    if rule.enabled { "Enabled" } else { "Disabled" }
                }
            }
            td { class: "col-actions",
                div { class: "action-buttons",
                    button { 
                        class: "action-btn",
                        title: "Edit",
                        onclick: move |_| {
                            on_edit.call(rule.clone());
                        },
                        "✏" 
                    }
                    button { 
                        class: "action-btn action-danger", 
                        title: "Delete",
                        onclick: move |_| {
                            on_delete.call(rule_id.clone());
                        },
                        "🗑" 
                    }
                }
            }
        }
    }
}

#[component]
fn EditRuleDialog(
    rule: FirewallRule,
    on_close: EventHandler<()>,
    on_save: EventHandler<FirewallRule>
) -> Element {
    let mut name = use_signal(|| rule.name.clone());
    let action = use_signal(|| rule.action.clone());
    let direction = use_signal(|| rule.direction.clone());
    let mut protocol = use_signal(|| rule.protocol.clone().unwrap_or_default());
    let mut source_ip = use_signal(|| rule.source_ip.clone().unwrap_or_default());
    let mut source_port = use_signal(|| rule.source_port.map(|p| p.to_string()).unwrap_or_default());
    let mut destination_ip = use_signal(|| rule.destination_ip.clone().unwrap_or_default());
    let mut destination_port = use_signal(|| rule.destination_port.map(|p| p.to_string()).unwrap_or_default());
    let mut enabled = use_signal(|| rule.enabled);

    rsx! {
        div { class: "modal",
            div { class: "modal-content",
                h2 { "Edit Firewall Rule" }
                
                div { class: "form-group",
                    label { "Rule Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{name}",
                        oninput: move |e| name.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Protocol" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{protocol}",
                        placeholder: "tcp, udp, icmp, or any",
                        oninput: move |e| protocol.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Source IP" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{source_ip}",
                        placeholder: "0.0.0.0/0",
                        oninput: move |e| source_ip.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Source Port" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{source_port}",
                        placeholder: "Leave empty for any",
                        oninput: move |e| source_port.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Destination IP" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{destination_ip}",
                        placeholder: "0.0.0.0/0",
                        oninput: move |e| destination_ip.set(e.value().clone())
                    }
                }

                div { class: "form-group",
                    label { "Destination Port" }
                    input {
                        class: "input",
                        r#type: "text",
                        value: "{destination_port}",
                        placeholder: "Leave empty for any",
                        oninput: move |e| destination_port.set(e.value().clone())
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
                            let updated = FirewallRule {
                                id: rule.id.clone(),
                                name: name().clone(),
                                enabled: enabled(),
                                action: action(),
                                direction: direction(),
                                protocol: if protocol().is_empty() { None } else { Some(protocol().clone()) },
                                source_ip: if source_ip().is_empty() { None } else { Some(source_ip().clone()) },
                                source_port: source_port().parse().ok(),
                                destination_ip: if destination_ip().is_empty() { None } else { Some(destination_ip().clone()) },
                                destination_port: destination_port().parse().ok(),
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
fn CreateRuleDialog(
    on_close: EventHandler<()>,
    on_create: EventHandler<FirewallRule>
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut action = use_signal(|| FirewallAction::Allow);
    let mut direction = use_signal(|| TrafficDirection::Inbound);
    let mut protocol = use_signal(|| String::from("tcp"));
    let mut source_ip = use_signal(|| String::new());
    let mut source_port = use_signal(|| String::new());
    let mut dest_ip = use_signal(|| String::new());
    let mut dest_port = use_signal(|| String::new());

    rsx! {
        div { class: "modal-overlay", onclick: move |_| on_close.call(()),
            div {
                class: "modal",
                onclick: move |e| e.stop_propagation(),
                h2 { "Create Firewall Rule" }

                div { class: "form-group",
                    label { "Rule Name" }
                    input {
                        class: "input",
                        r#type: "text",
                        placeholder: "e.g., Allow SSH",
                        value: "{name}",
                        oninput: move |e| name.set(e.value().clone())
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                    div { class: "form-group",
                        label { "Action" }
                        select {
                            class: "input",
                            style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                            onchange: move |e| {
                                action.set(match e.value().as_str() {
                                    "deny" => FirewallAction::Deny,
                                    "log" => FirewallAction::Log,
                                    _ => FirewallAction::Allow,
                                });
                            },
                            option { value: "allow", style: "background: #23262d; color: #e4e6eb;", "Allow" }
                            option { value: "deny", style: "background: #23262d; color: #e4e6eb;", "Deny" }
                            option { value: "log", style: "background: #23262d; color: #e4e6eb;", "Log" }
                        }
                    }
                    div { class: "form-group",
                        label { "Direction" }
                        select {
                            class: "input",
                            style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                            onchange: move |e| {
                                direction.set(match e.value().as_str() {
                                    "outbound" => TrafficDirection::Outbound,
                                    "both" => TrafficDirection::Both,
                                    _ => TrafficDirection::Inbound,
                                });
                            },
                            option { value: "inbound", style: "background: #23262d; color: #e4e6eb;", "Inbound" }
                            option { value: "outbound", style: "background: #23262d; color: #e4e6eb;", "Outbound" }
                            option { value: "both", style: "background: #23262d; color: #e4e6eb;", "Both" }
                        }
                    }
                }

                div { class: "form-group",
                    label { "Protocol" }
                    select {
                        class: "input",
                        style: "background: #23262d; color: #e4e6eb; border: 1px solid #3a3d47;",
                        value: "{protocol}",
                        onchange: move |e| protocol.set(e.value().clone()),
                        option { value: "tcp", style: "background: #23262d; color: #e4e6eb;", "TCP" }
                        option { value: "udp", style: "background: #23262d; color: #e4e6eb;", "UDP" }
                        option { value: "icmp", style: "background: #23262d; color: #e4e6eb;", "ICMP" }
                        option { value: "any", style: "background: #23262d; color: #e4e6eb;", "Any" }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                    div { class: "form-group",
                        label { "Source IP" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Any",
                            value: "{source_ip}",
                            oninput: move |e| source_ip.set(e.value().clone())
                        }
                    }
                    div { class: "form-group",
                        label { "Source Port" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Any",
                            value: "{source_port}",
                            oninput: move |e| source_port.set(e.value().clone())
                        }
                    }
                }

                div { style: "display: grid; grid-template-columns: 1fr 1fr; gap: 16px;",
                    div { class: "form-group",
                        label { "Destination IP" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Any",
                            value: "{dest_ip}",
                            oninput: move |e| dest_ip.set(e.value().clone())
                        }
                    }
                    div { class: "form-group",
                        label { "Destination Port" }
                        input {
                            class: "input",
                            r#type: "text",
                            placeholder: "Any",
                            value: "{dest_port}",
                            oninput: move |e| dest_port.set(e.value().clone())
                        }
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
                            let rule = FirewallRule {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: name.read().clone(),
                                enabled: true,
                                action: *action.read(),
                                direction: *direction.read(),
                                protocol: if protocol.read().is_empty() {
                                    None
                                } else {
                                    Some(protocol.read().clone())
                                },
                                source_ip: if source_ip.read().is_empty() {
                                    None
                                } else {
                                    Some(source_ip.read().clone())
                                },
                                source_port: source_port.read().parse().ok(),
                                destination_ip: if dest_ip.read().is_empty() {
                                    None
                                } else {
                                    Some(dest_ip.read().clone())
                                },
                                destination_port: dest_port.read().parse().ok(),
                            };
                            tracing::info!("Firewall rule creation requested: name={}, action={:?}, direction={:?}, protocol={:?}, src={}:{:?}, dst={}:{:?}",
                                rule.name, rule.action, rule.direction, rule.protocol,
                                rule.source_ip.as_deref().unwrap_or("any"),
                                rule.source_port,
                                rule.destination_ip.as_deref().unwrap_or("any"),
                                rule.destination_port
                            );
                            on_create.call(rule);
                        },
                        disabled: name.read().is_empty(),
                        "Create Rule"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct FirewallRule {
    id: String,
    name: String,
    enabled: bool,
    action: FirewallAction,
    direction: TrafficDirection,
    protocol: Option<String>,
    source_ip: Option<String>,
    source_port: Option<u16>,
    destination_ip: Option<String>,
    destination_port: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum FirewallAction {
    Allow,
    Deny,
    Log,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
enum TrafficDirection {
    Inbound,
    Outbound,
    Both,
}
