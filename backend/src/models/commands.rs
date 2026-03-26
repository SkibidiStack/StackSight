use serde::{Deserialize, Serialize};

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
    DockerComposeManual { compose_file_path: String, project_path: String },
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
    VirtEnvCreate { request: crate::models::virtenv::CreateEnvironmentRequest },
    VirtEnvDelete { env_id: String },
    VirtEnvActivate { env_id: String },
    VirtEnvDeactivate { env_id: String },
    VirtEnvInstallPackages { operation: crate::models::virtenv::PackageOperation },
    VirtEnvList,
    VirtEnvGetTemplates,

    // System Commands
    SystemGetProcessList,
    SystemKillProcess { pid: String },

    // Network Commands
    NetworkScanDevices,
    NetworkCreateVlan { request: crate::models::network::CreateVlanRequest },
    NetworkDeleteVlan { parent_interface: String, vlan_id: u16 },
    NetworkGetVlans,
    NetworkGetInterfaces,
    NetworkCreateBridge { request: crate::models::network::CreateBridgeRequest },
    NetworkDeleteBridge { name: String },
    NetworkUpdateInterface { request: crate::models::network::UpdateInterfaceRequest },
    NetworkUpdateVlan { request: crate::models::network::VlanConfig },
    
    // Remote Desktop Commands
    RemoteDesktopCreateConnection { request: crate::models::remote_desktop::CreateConnectionRequest },
    RemoteDesktopUpdateConnection { id: String, request: crate::models::remote_desktop::UpdateConnectionRequest },
    RemoteDesktopDeleteConnection { id: String },
    RemoteDesktopGetConnections,
    RemoteDesktopConnect { connection_id: String },
    RemoteDesktopDisconnect { connection_id: String },
    RemoteDesktopCreateGroup { name: String, color: Option<String> },
    RemoteDesktopAddToGroup { group_id: String, connection_id: String },
    RemoteDesktopGetGroups,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerCreateContainerConfig {
    pub name: String,
    pub image: String,
    pub ports: Vec<String>,
    pub env: Vec<String>,
    pub volumes: Vec<String>,
    pub cmd: Option<Vec<String>>,
}
