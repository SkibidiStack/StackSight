use crate::state::app_state::{ContainerSummary, ImageSummary, NetworkSummary, VolumeSummary};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    SystemSnapshot(SystemSnapshot),
    DockerContainers(Vec<ContainerSummary>),
    DockerStats {
        containers: usize,
        cpu_percent_avg: f64,
        memory_used: u64,
        memory_limit: u64,
        net_rx: u64,
        net_tx: u64,
    },
    DockerImages(Vec<ImageSummary>),
    DockerNetworks(Vec<NetworkSummary>),
    DockerVolumes(Vec<VolumeSummary>),
    VirtualEnvSummary { total: usize, active: usize },
    DockerStatus { connected: bool, error: Option<String> },
    DockerAction { action: String, ok: bool, message: Option<String> },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    DockerList,
    DockerStart { id: String },
    DockerStop { id: String },
    DockerRestart { id: String },
    DockerListImages,
    DockerPullImage { image: String },
    DockerPruneImages,
    DockerBuildImage { context_path: String, tag: Option<String> },
    DockerListNetworks,
    DockerListVolumes,
    DockerScaffold { config: DockerScaffoldConfig },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerScaffoldConfig {
    pub context_path: String,
    pub base_image: String,
    pub ports: Vec<u16>,
    pub workdir: Option<String>,
    pub cmd: Option<String>,
    pub additional_images: Vec<String>,
}
