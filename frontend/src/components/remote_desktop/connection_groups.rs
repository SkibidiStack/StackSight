use dioxus::prelude::*;

#[component]
pub fn ConnectionGroups() -> Element {
    let mut groups = use_signal(|| Vec::<ConnectionGroup>::new());
    let mut show_create_dialog = use_signal(|| false);

    rsx! {
        div { class: "connection-groups",
            div { class: "groups-header",
                h3 { "Connection Groups" }
                button {
                    class: "btn btn-primary",
                    onclick: move |_| show_create_dialog.set(true),
                    "➕ New Group"
                }
            }

            div { class: "groups-list",
                for group in groups.read().iter() {
                    GroupCard { group: group.clone() }
                }
            }

            if *show_create_dialog.read() {
                CreateGroupDialog {
                    on_close: move |_| show_create_dialog.set(false),
                    on_create: move |group: ConnectionGroup| {
                        groups.write().push(group);
                        show_create_dialog.set(false);
                    }
                }
            }
        }
    }
}

#[component]
fn GroupCard(group: ConnectionGroup) -> Element {
    rsx! {
        div {
            class: "group-card",
            style: if let Some(ref color) = group.color {
                format!("border-left: 4px solid {}", color)
            } else {
                String::new()
            },

            div { class: "group-header",
                h4 { "{group.name}" }
                span { class: "connection-count", "{group.connection_count} connections" }
            }

            div { class: "group-actions",
                button { class: "btn btn-sm btn-secondary", "✏️ Edit" }
                button { class: "btn btn-sm btn-danger", "🗑️ Delete" }
            }
        }
    }
}

#[component]
fn CreateGroupDialog(
    on_close: EventHandler<()>,
    on_create: EventHandler<ConnectionGroup>,
) -> Element {
    let mut name = use_signal(|| String::new());
    let mut color = use_signal(|| String::from("#3B82F6"));

    rsx! {
        div { class: "modal-overlay",
            onclick: move |_| on_close.call(()),

            div {
                class: "modal-dialog",
                onclick: move |e| e.stop_propagation(),

                div { class: "modal-header",
                    h2 { "Create Group" }
                    button {
                        class: "btn-close",
                        onclick: move |_| on_close.call(()),
                        "×"
                    }
                }

                div { class: "modal-body",
                    div { class: "form-group",
                        label { "Group Name" }
                        input {
                            r#type: "text",
                            class: "form-control",
                            placeholder: "e.g., Production Servers",
                            value: "{name}",
                            oninput: move |e| name.set(e.value().clone())
                        }
                    }

                    div { class: "form-group",
                        label { "Color" }
                        input {
                            r#type: "color",
                            class: "form-control",
                            value: "{color}",
                            oninput: move |e| color.set(e.value().clone())
                        }
                    }
                }

                div { class: "modal-footer",
                    button {
                        class: "btn btn-secondary",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| {
                            let group = ConnectionGroup {
                                id: uuid::Uuid::new_v4().to_string(),
                                name: name.read().clone(),
                                color: Some(color.read().clone()),
                                connection_count: 0,
                            };
                            on_create.call(group);
                        },
                        disabled: name.read().is_empty(),
                        "Create"
                    }
                }
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct ConnectionGroup {
    id: String,
    name: String,
    color: Option<String>,
    connection_count: usize,
}
