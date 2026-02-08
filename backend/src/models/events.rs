use crate::models::docker::{ContainerSummary, DockerStatsSummary, ImageSummary, NetworkSummary, VolumeSummary};
use crate::models::system::{SystemSnapshot, ProcessInfo, Alert};
use crate::models::virtenv::{VirtualEnvSummary, VirtualEnvironment, EnvironmentTemplate, PackageOperation};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    SystemSnapshot(SystemSnapshot),
    SystemProcessList(Vec<ProcessInfo>),
    SystemAlert(Alert),
    DockerContainers(Vec<ContainerSummary>),
    DockerStats(DockerStatsSummary),
    DockerImages(Vec<ImageSummary>),
    DockerNetworks(Vec<NetworkSummary>),
    DockerVolumes(Vec<VolumeSummary>),
    VirtualEnvSummary(VirtualEnvSummary),
    VirtualEnvironments(Vec<VirtualEnvironment>),
    VirtualEnvCreated { environment: VirtualEnvironment },
    VirtualEnvDeleted { env_id: String },
    VirtualEnvActivated { env_id: String },
    VirtualEnvDeactivated { env_id: String },
    VirtualEnvUpdated { environment: VirtualEnvironment },
    VirtualEnvTemplates(Vec<EnvironmentTemplate>),
    VirtualEnvList(Vec<VirtualEnvironment>),
    VirtualEnvError { message: String },
    PackageOperationStarted { operation: PackageOperation },
    PackageOperationCompleted { env_id: String, success: bool, message: Option<String> },
    DockerStatus { connected: bool, error: Option<String> },
    DockerAction { action: String, ok: bool, message: Option<String> },
    DockerLogs { container_id: String, logs: String },
    DockerfileGenerated { path: String, dockerfile: String },
    DockerfileSaved { path: String },
    DockerEngineLogs { logs: String },
}
