use crate::models::docker::ContainerSummary;
use crate::models::system::SystemSnapshot;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    SystemSnapshot(SystemSnapshot),
    DockerContainers(Vec<ContainerSummary>),
}
