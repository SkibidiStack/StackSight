use crate::components::{
    docker::{ContainerList, ImageManager, NetworkManager, VolumeManager, EngineManager},
    layout::{MainLayout, Section},
    monitoring::MonitoringDashboard,
    virtenv::EnvironmentList,
};
use dioxus::prelude::*;
use dioxus_router::{Routable, Router};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[route("/")]
    Containers {},
    #[route("/images")]
    Images {},
    #[route("/volumes")]
    Volumes {},
    #[route("/networks")]
    Networks {},
    #[route("/engine")]
    Engine {},
    #[route("/virtual-environments")]
    VirtualEnvironments {},
    #[route("/monitoring")]
    Monitoring {},
}

#[component]
pub fn Containers() -> Element {
    rsx! {
        MainLayout { section: Section::Containers, title: "Containers".to_string(),
            ContainerList {}
        }
    }
}

#[component]
pub fn Images() -> Element {
    rsx! {
        MainLayout { section: Section::Images, title: "Images".to_string(),
            ImageManager {}
        }
    }
}

#[component]
pub fn Volumes() -> Element {
    rsx! {
        MainLayout { section: Section::Volumes, title: "Volumes".to_string(),
            VolumeManager {}
        }
    }
}

#[component]
pub fn Networks() -> Element {
    rsx! {
        MainLayout { section: Section::Networks, title: "Networks".to_string(),
            NetworkManager {}
        }
    }
}

#[component]
pub fn Engine() -> Element {
    rsx! {
        MainLayout { section: Section::Engine, title: "Docker Engine".to_string(),
            EngineManager {}
        }
    }
}

#[component]
pub fn VirtualEnvironments() -> Element {
    rsx! {
        MainLayout { section: Section::VirtualEnvironment, title: "Virtual Environments".to_string(),
            EnvironmentList {}
        }
    }
}

#[component]
pub fn AppRouter() -> Element {
    rsx!(Router::<Route> {})
}

#[component]
pub fn Monitoring() -> Element {
    rsx! {
        MainLayout { section: Section::Monitoring, title: "System Monitoring".to_string(),
            MonitoringDashboard {}
        }
    }
}
