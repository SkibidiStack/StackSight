use crate::app::BackendBridge;
use crate::state::{AppState, Command};
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[component]
pub fn VolumeManager() -> Element {
    let app_state = use_context::<Signal<AppState>>();
    let bridge = use_context::<BackendBridge>();
    let mut search = use_signal(|| String::new());
    let mut show_create = use_signal(|| false);
    let mut create_name = use_signal(|| String::new());
    let mut create_driver = use_signal(|| "local".to_string());
    
    let snapshot = app_state.read();
    let volumes = snapshot.docker.volumes.clone();
    drop(snapshot);

    let search_val = search.read().to_lowercase();
    let filtered: Vec<_> = volumes
        .iter()
        .filter(|v| {
            search_val.is_empty()
                || v.name.to_lowercase().contains(&search_val)
        })
        .collect();

    rsx! {
        div { class: "docker-view",
            div { class: "view-header",
                div { class: "view-title",
                    h1 { "Volumes" }
                    span { class: "count-badge", "{volumes.len()}" }
                }
                div { class: "view-actions",
                    input {
                        class: "search-input",
                        r#type: "text",
                        placeholder: "Search volumes...",
                        value: "{search}",
                        oninput: move |e| search.set(e.value().clone())
                    }
                    button { class: "btn primary", onclick: move |_| show_create.set(true), "Create" }
                    button { class: "btn", "Clean Up" }
                }
            }

            if volumes.is_empty() {
                div { class: "empty-state",
                    div { class: "empty-icon", "💾" }
                    h3 { "No volumes" }
                    p { "Create a volume to persist data." }
                }
            } else {
                table { class: "docker-table",
                    thead {
                        tr {
                            th { class: "col-checkbox", input { r#type: "checkbox" } }
                            th { "Volume Name" }
                            th { "Driver" }
                            th { "Mount Point" }
                            th { class: "col-actions", "Actions" }
                        }
                    }
                    tbody {
                        {filtered.iter().map(|volume| {
                            let name = volume.name.clone();
                            
                            let on_delete = {
                                let name = name.clone();
                                let bridge = bridge.clone();
                                move |_| {
                                    bridge.send(Command::DockerRemoveVolume { name: name.clone(), force: false })
                                }
                            };
                            
                            rsx! {
                                tr { class: "table-row", key: "{name}",
                                    td { class: "col-checkbox", input { r#type: "checkbox" } }
                                    td { class: "col-name",
                                        div { class: "cell-main", "{name}" }
                                    }
                                    td { "{volume.driver}" }
                                    td { class: "col-image", "{volume.mountpoint}" }
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
                        h2 { "Create Volume" }
                        div { class: "form-group",
                            label { "Volume Name" }
                            input {
                                class: "input",
                                r#type: "text",
                                placeholder: "my-volume",
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
                                option { value: "local", "local" }
                                option { value: "nfs", "nfs" }
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
                                        bridge.send(Command::DockerCreateVolume {
                                            name: create_name.read().clone(),
                                            driver: Some(create_driver.read().clone()),
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
