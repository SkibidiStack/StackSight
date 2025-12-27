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
