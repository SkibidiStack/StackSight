use crate::models::docker::{ContainerSummary, DockerStatsSummary, ImageSummary, NetworkSummary, VolumeSummary};
use crate::models::system::SystemSnapshot;
use crate::models::virtenv::VirtualEnvSummary;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    SystemSnapshot(SystemSnapshot),
    DockerContainers(Vec<ContainerSummary>),
    DockerStats(DockerStatsSummary),
    DockerImages(Vec<ImageSummary>),
    DockerNetworks(Vec<NetworkSummary>),
    DockerVolumes(Vec<VolumeSummary>),
    VirtualEnvSummary(VirtualEnvSummary),
    DockerStatus { connected: bool, error: Option<String> },
    DockerAction { action: String, ok: bool, message: Option<String> },
}
