use super::Section;
use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::Link;

struct NavItem {
    section: Section,
    label: &'static str,
    description: &'static str,
    route: Route,
}

fn items() -> [NavItem; 5] {
    [
        NavItem {
            section: Section::Dashboard,
            label: "Dashboard",
            description: "System pulse",
            route: Route::Dashboard {},
        },
        NavItem {
            section: Section::Docker,
            label: "Docker",
            description: "Containers & images",
            route: Route::Docker {},
        },
        NavItem {
            section: Section::Environments,
            label: "Environments",
            description: "Language stacks",
            route: Route::Environments {},
        },
        NavItem {
            section: Section::Monitoring,
            label: "Monitoring",
            description: "Metrics & alerts",
            route: Route::Monitoring {},
        },
        NavItem {
            section: Section::Settings,
            label: "Settings",
            description: "Preferences",
            route: Route::Settings {},
        },
    ]
}

#[component]
pub fn Sidebar(section: Section) -> Element {
    rsx! {
        aside { class: "sidebar",
            div { class: "sidebar-brand", "DevEnv Manager" }
            for item in items() {
                Link { to: item.route.clone(), class: if item.section == section { "nav-item nav-active" } else { "nav-item" },
                    div { class: "nav-title", "{item.label}" }
                    span { class: "nav-subtitle", "{item.description}" }
                }
            }
        }
    }
}
