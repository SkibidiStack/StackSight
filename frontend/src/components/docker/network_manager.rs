use crate::app::BackendBridge;
use crate::state::{AppState, Command};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn NetworkManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut search = use_signal(|| String::new());
    let mut show_create = use_signal(|| false);
    let mut create_name = use_signal(|| String::new());
    let mut create_driver = use_signal(|| "bridge".to_string());

    let snapshot = app_state.read();
    let networks = snapshot.docker.networks.clone();
    drop(snapshot);

    let search_val = search.read().to_lowercase();
    let filtered: Vec<_> = networks
        .iter()
        .filter(|n| search_val.is_empty() || n.name.to_lowercase().contains(&search_val))
        .collect();

    rsx! {
        div { class: "docker-view",
            div { class: "view-header",
                div { class: "view-title",
                    h1 { "Networks" }
                    span { class: "count-badge", "{networks.len()}" }
                }
                div { class: "view-actions",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Search networks...",
                        value: "{search}",
                        oninput: move |e| search.set(e.value().clone())
                    }
                    button { class: "btn primary", onclick: move |_| show_create.set(true), "Create" }
                    button { class: "btn", "Clean Up" }
                }
            }

            if networks.is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "🔗" }
                    h3 { "No networks" }
                    p { "Create a network to connect containers." }
                }
            } else {
                table { class: "docker-table",
                    thead {
                        tr {
                            th { class: "col-checkbox", input { r#type: "checkbox" } }
                            th { "Name" }
                            th { "Driver" }
                            th { "Network ID" }
                            th { class: "col-actions", "Actions" }
                        }
                    }
                    tbody {
                        {filtered.iter().map(|network| {
                            let id = network.id.clone();
                            let name = network.name.clone();

                            let on_delete = {
                                let id = id.clone();
                                let bridge = bridge.clone();
                                move |_| {
                                    bridge.send(Command::DockerRemoveNetwork { id: id.clone() })
                                }
                            };

                            rsx! {
                                tr { class: "table-row", key: "{id}",
                                    td { class: "col-checkbox", input { r#type: "checkbox" } }
                                    td { class: "col-name",
                                        div { class: "cell-main", "{name}" }
                                        div { class: "cell-sub", "{id[..12].to_string()}" }
                                    }
                                    td { "{network.driver}" }
                                    td {
                                        div { class: "cell-sub", "{id[..12].to_string()}" }
                                    }
                                    td { class: "col-actions",
                                        div { class: "action-buttons",
                                            button { class: "action-btn action-danger", onclick: on_delete, title: "Delete", "🗑" }
                                        }
                                    }
                                }
                            }
                        })}
                    }
                }
            }

            if *show_create.read() {
                div { class: "modal-overlay", onclick: move |_| show_create.set(false),
                    div { class: "modal", onclick: move |e| e.stop_propagation(),
                        h2 { "Create Network" }
                        div { class: "form-group",
                            label { "Network Name" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "my-network",
                                value: "{create_name}",
                                oninput: move |e| create_name.set(e.value().clone())
                            }
                        }
                        div { class: "form-group",
                            label { "Driver" }
                            select {
                                class: "input",
                                value: "{create_driver}",
                                onchange: move |e| create_driver.set(e.value().clone()),
                                option { value: "bridge", "bridge" }
                                option { value: "host", "host" }
                                option { value: "overlay", "overlay" }
                                option { value: "macvlan", "macvlan" }
                                option { value: "none", "none" }
                            }
                        }
                        div { class: "modal-actions",
                            button { class: "btn", onclick: move |_| show_create.set(false), "Cancel" }
                            button {
                                class: "btn primary",
                                onclick: {
                                    let bridge = bridge.clone();
                                    let create_name = create_name.clone();
                                    let create_driver = create_driver.clone();
                                    move |_| {
                                        bridge.send(Command::DockerCreateNetwork {
                                            name: create_name.read().clone(),
                                            driver: create_driver.read().clone(),
                                        });
                                        show_create.set(false);
                                    }
                                },
                                "Create"
                            }
                        }
                    }
                }
            }
        }
    }
}
