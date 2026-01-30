use serde::{Deserialize, Serialize};

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
