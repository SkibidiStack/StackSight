pub mod connection_list;
pub mod connection_form;
pub mod session_viewer;
pub mod connection_groups;

use dioxus::prelude::*;
use connection_list::ConnectionList;
use connection_form::ConnectionForm;
use session_viewer::ActiveSessionsPanel;

#[derive(Clone, Copy, PartialEq)]
enum RemoteDesktopTab {
    Connections,
    ActiveSessions,
}

#[component]
pub fn RemoteDesktopView() -> Element {
    let mut current_view = use_signal(|| RemoteDesktopTab::Connections);
    let mut selected_connection = use_signal(|| Option::<String>::None);
    let mut show_connection_dialog = use_signal(|| false);

    rsx! {
        div { class: "view",
            div { class: "view-header",
                h1 { "Remote Desktop" }
                div { class: "view-actions",
                    button {
                        class: if *current_view.read() == RemoteDesktopTab::Connections {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| current_view.set(RemoteDesktopTab::Connections),
                        "Connections"
                    }
                    button {
                        class: if *current_view.read() == RemoteDesktopTab::ActiveSessions {
                            "btn primary"
                        } else {
                            "btn"
                        },
                        onclick: move |_| current_view.set(RemoteDesktopTab::ActiveSessions),
                        "Active Sessions"
                    }
                    button {
                        class: "btn btn-primary",
                        onclick: move |_| show_connection_dialog.set(true),
                        "+ New Connection"
                    }
                }
            }

            div { class: "view-content",
                match *current_view.read() {
                    RemoteDesktopTab::Connections => rsx! {
                        ConnectionList {
                            on_connect: move |id: String| {
                                // Handle connection
                                selected_connection.set(Some(id));
                            },
                            on_edit: move |id: String| {
                                selected_connection.set(Some(id));
                                show_connection_dialog.set(true);
                            }
                        }
                    },
                    RemoteDesktopTab::ActiveSessions => rsx! {
                        ActiveSessionsPanel {}
                    },
                }
            }

            if *show_connection_dialog.read() {
                ConnectionForm {
                    connection_id: selected_connection.read().clone(),
                    on_save: move |_| {
                        show_connection_dialog.set(false);
                        selected_connection.set(None);
                    },
                    on_cancel: move |_| {
                        show_connection_dialog.set(false);
                        selected_connection.set(None);
                    }
                }
            }
        }
    }
}
