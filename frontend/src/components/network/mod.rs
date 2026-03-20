pub mod interface_list;
pub mod vlan_manager;
pub mod network_graph;

use dioxus::prelude::*;
use interface_list::InterfaceList;
use vlan_manager::VlanManager;
use network_graph::NetworkGraph;

#[derive(Clone, Copy, PartialEq)]
enum NetworkTab {
    Interfaces,
    Vlans,
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
                }
            }

            div { class: "view-content",
                match *active_tab.read() {
                    NetworkTab::Topology => rsx! { NetworkGraph {} },
                    NetworkTab::Interfaces => rsx! { InterfaceList {} },
                    NetworkTab::Vlans => rsx! { VlanManager {} },
                }
            }
        }
    }
}
