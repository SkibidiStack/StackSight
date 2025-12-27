use super::{header::Header, sidebar::Sidebar};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Dashboard,
    Docker,
    Environments,
    Monitoring,
    Settings,
}

#[component]
pub fn MainLayout(section: Section, title: String, children: Element) -> Element {
    rsx! {
        div { class: "app-shell",
            Sidebar { section }
            div { class: "content-area",
                Header { title }
                div { class: "section-body",
                    {children}
                }
            }
        }
    }
}
