use super::sidebar::Sidebar;
use crate::components::common::Terminal;
use crate::state::AppState;
use dioxus::prelude::*;
use dioxus_signals::Signal;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Containers,
    Images,
    Volumes,
    Networks,
    Engine,
    VirtualEnvironment,
    Monitoring,
    NetworkManager,
    RemoteDesktop,
}

#[component]
pub fn MainLayout(section: Section, title: String, children: Element) -> Element {
    let mut app_state = use_context::<Signal<AppState>>();
    let terminal_visible = app_state.read().ui.terminal_visible;
    
    rsx! {
        div { class: "app-shell",
            Sidebar { section }
            div { class: "content-area",
                div { 
                    class: format!("main-content {}", if terminal_visible { "with-terminal" } else { "" }),
                    div { class: "section-header",
                        h1 { "{title}" }
                        button {
                            class: format!("btn btn-outline terminal-toggle {}", if terminal_visible { "active" } else { "" }),
                            onclick: move |_| {
                                let mut state = app_state.write();
                                state.ui.terminal_visible = !state.ui.terminal_visible;
                            },
                            "🖥️ Terminal"
                        }
                    }
                    div { class: "section-body",
                        {children}
                    }
                }
                if terminal_visible {
                    div { class: "terminal-panel",
                        Terminal {}
                    }
                }
            }
        }
    }
}
