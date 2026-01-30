use crate::components::{
    dashboard::{ActivityFeed, OverviewPanel, QuickActions, SystemStatusWidget, WelcomeScreen},
    docker::{ComposeBuilder, ContainerList, ImageManager, NetworkManager, StatsMonitor, VolumeManager},
    layout::{MainLayout, Section},
    monitoring::{AlertPanel, ProcessMonitor, ResourceCharts, SystemStats},
    settings::{GeneralSettings, IntegrationSettings, NotificationSettings, ThemeSettings},
    virtenv::{EnvironmentList, LanguageSelector, PackageManager, ProjectWizard},
};
use dioxus::prelude::*;
use dioxus_router::{Routable, Router};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Dashboard {},
    #[route("/docker")]
    Docker {},
    #[route("/environments")]
    Environments {},
    #[route("/monitoring")]
    Monitoring {},
    #[route("/settings")]
    Settings {},
}

#[component]
pub fn Dashboard() -> Element {
    rsx! {
        MainLayout { section: Section::Dashboard, title: "Dashboard".to_string(),
            div { class: "section-grid",
                OverviewPanel {}
                QuickActions {}
                SystemStatusWidget {}
                ActivityFeed {}
                WelcomeScreen {}
            }
        }
    }
}

#[component]
pub fn Docker() -> Element {
    rsx! {
        MainLayout { section: Section::Docker, title: "Docker".to_string(),
            div { class: "grid-two",
                ContainerList {}
                StatsMonitor {}
            }
            div { class: "section-grid",
                ImageManager {}
                ComposeBuilder {}
                NetworkManager {}
                VolumeManager {}
            }
        }
    }
}

#[component]
pub fn Environments() -> Element {
    rsx! {
        MainLayout { section: Section::Environments, title: "Virtual Environments".to_string(),
            div { class: "grid-two",
                EnvironmentList {}
                LanguageSelector {}
            }
            div { class: "section-grid",
                PackageManager {}
                ProjectWizard {}
            }
        }
    }
}

#[component]
pub fn Monitoring() -> Element {
    rsx! {
        MainLayout { section: Section::Monitoring, title: "Monitoring".to_string(),
            div { class: "section-grid",
                SystemStats {}
                ResourceCharts {}
                ProcessMonitor {}
                AlertPanel {}
            }
        }
    }
}

#[component]
pub fn Settings() -> Element {
    rsx! {
        MainLayout { section: Section::Settings, title: "Settings".to_string(),
            div { class: "section-grid",
                GeneralSettings {}
                ThemeSettings {}
                NotificationSettings {}
                IntegrationSettings {}
            }
        }
    }
}

#[component]
pub fn AppRouter() -> Element {
    rsx!(Router::<Route> {})
}
