pub mod interface_list;
pub mod vlan_manager;
pub mod route_table;
pub mod firewall_rules;
pub mod network_graph;

use dioxus::prelude::*;
use interface_list::InterfaceList;
use vlan_manager::VlanManager;
use route_table::RouteTable;
use firewall_rules::FirewallRules;
use network_graph::NetworkGraph;

#[derive(Clone, Copy, PartialEq)]
enum NetworkTab {
    Interfaces,
    Vlans,
    Routes,
    Firewall,
    Topology,
}

#[component]
pub fn NetworkManagerView() -> Element {
    let mut active_tab = use_signal(|| NetworkTab::Topology);

    rsx! {
        div { class: "network-manager-view",
            div { class: "view-header",
                div { class: "view-title",
                    h1 { "Network Manager" }
                }
                div { class: "view-actions",
                    button {
                        class: if *active_tab.read() == NetworkTab::Topology { "btn primary" } else { "btn" },
                        onclick: move |_| active_tab.set(NetworkTab::Topology),
                        "🗺 Topology"
                    }
                    button {
                        class: if *active_tab.read() == NetworkTab::Interfaces { "btn primary" } else { "btn" },
                        onclick: move |_| active_tab.set(NetworkTab::Interfaces),
                        "Interfaces"
                    }
                    button {
                        class: if *active_tab.read() == NetworkTab::Vlans { "btn primary" } else { "btn" },
                        onclick: move |_| active_tab.set(NetworkTab::Vlans),
                        "VLANs"
                    }
                    button {
                        class: if *active_tab.read() == NetworkTab::Routes { "btn primary" } else { "btn" },
                        onclick: move |_| active_tab.set(NetworkTab::Routes),
                        "Routes"
                    }
                    button {
                        class: if *active_tab.read() == NetworkTab::Firewall { "btn primary" } else { "btn" },
                        onclick: move |_| active_tab.set(NetworkTab::Firewall),
                        "Firewall"
                    }
                }
            }

            div { class: "view-content",
                match *active_tab.read() {
                    NetworkTab::Topology => rsx! { NetworkGraph {} },
                    NetworkTab::Interfaces => rsx! { InterfaceList {} },
                    NetworkTab::Vlans => rsx! { VlanManager {} },
                    NetworkTab::Routes => rsx! { RouteTable {} },
                    NetworkTab::Firewall => rsx! { FirewallRules {} },
                }
            }
        }
    }
}
