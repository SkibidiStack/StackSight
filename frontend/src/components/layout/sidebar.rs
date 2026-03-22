use super::Section;
use crate::router::Route;
use crate::app::ThemeToggle;
use dioxus::prelude::*;
use dioxus_router::Link;

struct NavItem {
    section: Section,
    label: &'static str,
    icon: &'static str,
    route: Route,
}

fn items() -> [NavItem; 8] {
    [
        NavItem {
            section: Section::Containers,
            label: "Containers",
            icon: "⬢",
            route: Route::Containers {},
        },
        NavItem {
            section: Section::Images,
            label: "Images",
            icon: "⬡",
            route: Route::Images {},
        },
        NavItem {
            section: Section::Volumes,
            label: "Volumes",
            icon: "⬢",
            route: Route::Volumes {},
        },
        NavItem {
            section: Section::Networks,
            label: "Networks",
            icon: "⬡",
            route: Route::Networks {},
        },
        NavItem {
            section: Section::Engine,
            label: "Engine",
            icon: "⚙",
            route: Route::Engine {},
        },
        NavItem {
            section: Section::VirtualEnvironment,
            label: "Virtual Envs",
            icon: "◈",
            route: Route::VirtualEnvironments {},
        },
        NavItem {
            section: Section::Monitoring,
            label: "Monitoring",
            icon: "◉",
            route: Route::Monitoring {},
        },
        NavItem {
            section: Section::RemoteDesktop,
            label: "Remote Desktop",
            icon: "⧉",
            route: Route::RemoteDesktop {},
        },
    ]
}

#[component]
pub fn Sidebar(section: Section) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-brand", 
                div { class: "brand-icon", "◆" }
                span { "StackSight" }
                ThemeToggle {}
            }
            nav { class: "sidebar-nav",
                for item in items() {
                    Link { 
                        to: item.route.clone(), 
                        class: if item.section == section { "sidebar-item sidebar-item-active" } else { "sidebar-item" },
                        div { class: "sidebar-icon", "{item.icon}" }
                        span { "{item.label}" }
                    }
                }
            }
        }
    }
}
