use super::Section;
use crate::router::Route;
use dioxus::prelude::*;
use dioxus_router::prelude::*;

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
        aside { class: "panel", style: "width: 280px; height: 100%; display: flex; flex-direction: column; gap: 12px; padding: 18px;",
            div { class: "badge", "DevEnv Manager" }
            for item in items() {
                Link { to: item.route.clone(), class: if item.section == section { "nav-link nav-active" } else { "nav-link" },
                    div { style: "display: flex; flex-direction: column; gap: 4px;",
                        span { style: "font-weight: 600;", "{item.label}" }
                        span { class: "muted", "{item.description}" }
                    }
                    if item.section == section {
                        span { class: "chip", "Live" }
                    }
                }
            }
        }
    }
}
