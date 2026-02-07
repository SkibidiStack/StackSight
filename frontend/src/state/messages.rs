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
    DockerLogs { container_id: String, logs: String },
    DockerfileGenerated { path: String, dockerfile: String },
    DockerfileSaved { path: String },
    DockerEngineLogs { logs: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    DockerList,
    DockerStart { id: String },
    DockerStop { id: String },
    DockerRestart { id: String },
    DockerRemoveContainer { id: String, force: bool },
    DockerContainerLogs { id: String },
    DockerCreateContainer { config: DockerCreateContainerConfig },
    DockerAnalyzeFolder { path: String },
    DockerSaveDockerfile { path: String, dockerfile: String },
    DockerListImages,
    DockerPullImage { image: String },
    DockerRemoveImage { id: String, force: bool },
    DockerRunImage { image: String },
    DockerPruneImages,
    DockerBuildImage { context_path: String, tag: Option<String> },
    DockerBuildFromDockerfile { path: String, dockerfile: String, tag: Option<String> },
    DockerBuildManual { dockerfile_path: String, project_path: String, tag: String },
    DockerListNetworks,
    DockerCreateNetwork { name: String, driver: String },
    DockerRemoveNetwork { id: String },
    DockerListVolumes,
    DockerCreateVolume { name: String, driver: Option<String> },
    DockerRemoveVolume { name: String, force: bool },
    DockerScaffold { config: DockerScaffoldConfig },
    DockerStartEngine,
    DockerStopEngine,
    DockerGetEngineLogs,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerCreateContainerConfig {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub env: Vec<String>,
    pub volumes: Vec<String>,
    pub cmd: Option<Vec<String>>,
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
