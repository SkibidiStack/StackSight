use super::sidebar::Sidebar;
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Section {
    Containers,
    Images,
    Volumes,
    Networks,
    Engine,
}

#[component]
pub fn MainLayout(section: Section, title: String, children: Element) -> Element {
    rsx! {
        div { class: "app-shell",
            Sidebar { section }
            div { class: "content-area",
                div { class: "section-body",
                    {children}
                }
            }
        }
    }
}
