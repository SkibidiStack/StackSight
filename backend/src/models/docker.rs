use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub state: String,
    pub image: String,
    pub status: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DockerStatsSummary {
    pub containers: usize,
    pub cpu_percent_avg: f64,
    pub memory_used: u64,
    pub memory_limit: u64,
    pub net_rx: u64,
    pub net_tx: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ImageSummary {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NetworkSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}
