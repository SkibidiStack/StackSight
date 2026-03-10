use crate::state::app_state::{ContainerSummary, ImageSummary, NetworkSummary, VolumeSummary, VirtualEnvironment};
use crate::services::backend_client::CreateEnvironmentRequest;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub uptime: u64,
    pub load_avg: LoadAvg,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LoadAvg {
    pub one: f64,
    pub five: f64,
    pub fifteen: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DiskInfo {
    pub name: String,
    pub mount_point: String,
    pub total_space: u64,
    pub available_space: u64,
    pub file_system: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub name: String,
    pub received: u64,
    pub transmitted: u64,
    pub packets_recv: u64,
    pub packets_sent: u64,
    pub errors_on_recv: u64,
    pub errors_on_sent: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProcessInfo {
    pub pid: String,
    pub name: String,
    pub cpu_usage: f32,
    pub memory: u64,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<Vec<String>>,
    pub parent_pid: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Alert {
    pub level: AlertLevel,
    pub title: String,
    pub message: String,
    pub timestamp: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertLevel {
    Info,
    Warning,
    Critical,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    SystemSnapshot(SystemSnapshot),
    SystemProcessList(Vec<ProcessInfo>),
    SystemAlert(Alert),
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
    VirtualEnvList(Vec<VirtualEnvironment>),
    VirtualEnvCreated { environment: VirtualEnvironment },
    VirtualEnvDeleted { env_id: String },
    PackageOperationCompleted { env_id: String, success: bool, message: Option<String> },
    DockerStatus { connected: bool, error: Option<String> },
    DockerAction { action: String, ok: bool, message: Option<String> },
    DockerLogs { container_id: String, logs: String },
    DockerfileGenerated { path: String, dockerfile: String },
    DockerfileSaved { path: String },
    DockerEngineLogs { logs: String },
    NetworkTopology(NetworkTopologyData),
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub enum NetworkDeviceType {
    Gateway,
    LocalMachine,
    Host,
    Unknown,
}

impl Default for NetworkDeviceType {
    fn default() -> Self { NetworkDeviceType::Unknown }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkDevice {
    pub ip: String,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub interface: String,
    pub device_type: NetworkDeviceType,
    pub is_reachable: bool,
    pub vendor: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct NetworkTopologyData {
    pub devices: Vec<NetworkDevice>,
    pub gateway: Option<String>,
    pub local_ip: Option<String>,
    pub scan_time: String,
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

    // Virtual Environment Commands
    VirtEnvCreate { request: CreateEnvironmentRequest },
    VirtEnvDelete { env_id: String },
    VirtEnvActivate { env_id: String },
    VirtEnvDeactivate { env_id: String },
    VirtEnvInstallPackages { operation: PackageOperation },
    VirtEnvList,
    VirtEnvGetTemplates,

    // System Commands
    SystemGetProcessList,
    SystemKillProcess { pid: String },

    // Network Commands
    NetworkScanDevices,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageOperation {
    pub env_id: String,
    pub operation: PackageOperationType,
    pub packages: Vec<String>,
    pub options: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PackageOperationType {
    Install,
    Uninstall,
    Update,
    Upgrade,
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
