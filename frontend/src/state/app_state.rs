use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    pub user: UserPreferences,
    pub system: SystemStatus,
    pub docker: DockerState,
    pub virtenv: VirtualEnvState,
    pub ui: UiState,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user: UserPreferences::default(),
            system: SystemStatus::default(),
            docker: DockerState::default(),
            virtenv: VirtualEnvState::default(),
            ui: UiState::default(),
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: Theme,
    pub notifications_enabled: bool,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: Theme::Dark,
            notifications_enabled: true,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct SystemStatus {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub alerts: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ContainerSummary {
    pub id: String,
    pub name: String,
    pub state: String,
    pub image: String,
    pub status: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DockerState {
    pub containers: Vec<ContainerSummary>,
    pub connected: bool,
    pub last_error: Option<String>,
    pub stats: DockerStats,
    pub images: Vec<ImageSummary>,
    pub networks: Vec<NetworkSummary>,
    pub volumes: Vec<VolumeSummary>,
    pub action: DockerActionStatus,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DockerStats {
    pub containers: usize,
    pub cpu_percent_avg: f64,
    pub memory_used: u64,
    pub memory_limit: u64,
    pub net_rx: u64,
    pub net_tx: u64,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct DockerActionStatus {
    pub in_progress: bool,
    pub last_action: Option<String>,
    pub last_ok: Option<bool>,
    pub message: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ImageSummary {
    pub id: String,
    pub repo_tags: Vec<String>,
    pub size: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NetworkSummary {
    pub id: String,
    pub name: String,
    pub driver: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VolumeSummary {
    pub name: String,
    pub driver: String,
    pub mountpoint: String,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct VirtualEnvState {
    pub environments: usize,
    pub active: usize,
}

#[derive(Clone, Serialize, Deserialize, Default)]
pub struct UiState {
    pub current_route: String,
    pub loading: bool,
    pub toasts: Vec<Toast>,
    pub logs_modal: Option<LogsModal>,
    pub dockerfile_editor: Option<DockerfileEditor>,
    pub build_confirmation: Option<String>,
    pub engine_logs: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct LogsModal {
    pub container_id: String,
    pub logs: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct DockerfileEditor {
    pub path: String,
    pub dockerfile: String,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct Toast {
    pub id: u64,
    pub message: String,
    pub toast_type: ToastType,
    pub timestamp: u64,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ToastType {
    Success,
    Error,
    Info,
}

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Theme {
    Dark,
    Light,
}

impl Default for Theme {
    fn default() -> Self {
        Theme::Dark
    }
}
