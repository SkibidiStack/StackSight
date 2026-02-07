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
