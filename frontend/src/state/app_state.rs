use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use crate::state::messages::{DiskInfo, NetworkInfo, ProcessInfo, Alert, LoadAvg, NetworkTopologyData};

#[derive(Clone, Serialize, Deserialize)]
pub struct AppState {
    pub user: UserPreferences,
    pub system: SystemStatus,
    pub docker: DockerState,
    pub virtenv: VirtualEnvState,
    pub network: NetworkState,
    pub remote_desktop: RemoteDesktopState,
    pub ui: UiState,
    pub setup: SetupConfig,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            user: UserPreferences::default(),
            system: SystemStatus::default(),
            docker: DockerState::default(),
            virtenv: VirtualEnvState::default(),
            network: NetworkState::default(),
            remote_desktop: RemoteDesktopState::default(),
            ui: UiState::default(),
            setup: SetupConfig::load_or_default(),
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
    pub memory_used: u64,
    pub memory_total: u64,
    pub swap_used: u64,
    pub swap_total: u64,
    pub uptime: u64,
    pub load_avg: Option<LoadAvg>,
    pub disks: Vec<DiskInfo>,
    pub networks: Vec<NetworkInfo>,
    pub processes: Vec<ProcessInfo>,
    pub alerts: Vec<Alert>,
    #[serde(skip)]
    pub cpu_history: Vec<f32>,
    #[serde(skip)]
    pub memory_history: Vec<u64>,
    #[serde(skip)]
    pub network_rx_history: Vec<u64>,
    #[serde(skip)]
    pub network_tx_history: Vec<u64>,
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

#[derive(Clone, Serialize, Deserialize)]
pub struct VirtualEnvState {
    pub environments: usize,
    pub active: usize,
    pub environment_list: Vec<VirtualEnvironment>,
    pub selected_env: Option<String>,
    pub templates: Vec<EnvironmentTemplate>,
    pub creating: bool,
    pub last_error: Option<String>,
    pub package_operation: Option<PackageOperationStatus>,
}

impl Default for VirtualEnvState {
    fn default() -> Self {
        // Don't load from file - backend will send environments via WebSocket
        tracing::info!("Initialized VirtualEnvState - waiting for backend to send environments");
        
        Self {
            environment_list: Vec::new(),
            environments: 0,
            active: 0,
            selected_env: None,
            templates: Vec::new(),
            creating: false,
            last_error: None,
            package_operation: None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct VirtualEnvironment {
    pub id: String,
    pub name: String,
    pub language: String,
    pub version: String,
    pub is_active: bool,
    
    #[serde(default)]
    pub packages: Vec<BackendPackage>,
    
    // We'll ignore the count from JSON and calculate it from packages array if present
    #[serde(default, skip_deserializing)] 
    pub package_count: usize,

    pub size_mb: Option<u64>,
    
    #[serde(deserialize_with = "deserialize_friendly_date")]
    pub created_at: String,
    
    #[serde(default)]
    #[serde(deserialize_with = "deserialize_friendly_date_opt")]
    pub last_used: Option<String>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub project_path: Option<String>,
    #[serde(default)]
    pub health: Option<BackendHealth>,
    #[serde(default)]
    pub health_status: String,
    #[serde(default)]
    pub path: Option<String>,
}

// Custom deserializer to make dates readable
fn deserialize_friendly_date<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    // Try to parse as RFC3339/ISO8601
    match chrono::DateTime::parse_from_rfc3339(&s) {
        Ok(dt) => Ok(dt.format("%Y-%m-%d %H:%M").to_string()),
        Err(_) => Ok(s), // Return original string if parse fails
    }
}

// Custom deserializer for optional dates
fn deserialize_friendly_date_opt<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    match opt {
        Some(s) => match chrono::DateTime::parse_from_rfc3339(&s) {
            Ok(dt) => Ok(Some(dt.format("%Y-%m-%d %H:%M").to_string())),
            Err(_) => Ok(Some(s)),
        },
        None => Ok(None),
    }
}

// Custom deserializer to populate package_count from packages array if missing
fn deserialize_package_count<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: serde::Deserializer<'de>,
{
    // We can't easily access the sibling "packages" field here during streaming deserialization.
    // However, since we define package_count AFTER packages in the struct, and we can use a custom logic:
    // Actually, usually fields are independent.
    // The issue is likely that "package_count" doesn't exist in the JSON, so it defaults to 0.
    // But we want it to reflect packages.len().
    // The best way is to implement a custom deserializer for the whole struct or fix it after load.
    
    // Instead of complex deserialization, let's just accept the value (creates 0 if missing)
    let count = usize::deserialize(deserializer)?;
    Ok(count)
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct BackendPackage {
    pub name: String,
    pub version: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub is_dev_dependency: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct BackendHealth {
    pub status: String,
    #[serde(default)]
    pub issues: Vec<String>,
    pub last_check: String,
    #[serde(default)]
    pub cpu_usage: Option<f32>,
    #[serde(default)]
    pub memory_usage: Option<u64>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct EnvironmentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: String,
    pub package_count: usize,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct PackageOperationStatus {
    pub env_id: String,
    pub operation: String,
    pub packages: Vec<String>,
    pub in_progress: bool,
    pub success: Option<bool>,
    pub message: Option<String>,
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
    pub terminal_visible: bool,
    pub terminal_pending_command: Option<String>,
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

// Persistence functions for virtual environments
fn get_config_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "devenv", "manager") {
        let config_dir = proj_dirs.config_dir();
        std::fs::create_dir_all(config_dir)?;
        Ok(config_dir.to_path_buf())
    } else {
        Err("Could not determine config directory".into())
    }
}

fn get_environments_file_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let mut path = get_config_dir()?;
    path.push("environments.json");
    Ok(path)
}

pub fn save_virtual_environments(environments: &[VirtualEnvironment]) -> Result<(), Box<dyn std::error::Error>> {
    let path = get_environments_file_path()?;
    tracing::info!("Attempting to save {} virtual environments to: {:?}", environments.len(), path);
    
    let json = serde_json::to_string_pretty(environments)?;
    tracing::debug!("JSON to save: {}", json);
    
    std::fs::write(&path, json)?;
    tracing::info!("Successfully saved {} virtual environments to file", environments.len());
    Ok(())
}

// Test function to verify persistence is working
pub fn test_persistence() {
    tracing::info!("Testing persistence system...");
    
    match get_config_dir() {
        Ok(dir) => tracing::info!("Config directory: {:?}", dir),
        Err(e) => tracing::error!("Failed to get config directory: {}", e),
    }
    
    match get_environments_file_path() {
        Ok(path) => {
            tracing::info!("Environments file path: {:?}", path);
            tracing::info!("File exists: {}", path.exists());
        },
        Err(e) => tracing::error!("Failed to get environments file path: {}", e),
    }
}

// Setup Configuration
#[derive(Clone, Serialize, Deserialize)]
pub struct SetupConfig {
    pub completed: bool,
    pub docker_path: Option<String>,
    pub virtualenv_base_path: String,
    pub python_paths: Vec<PathConfig>,
    pub node_paths: Vec<PathConfig>,
    pub rust_path: Option<String>,
    pub go_path: Option<String>,
    pub java_path: Option<String>,
    pub dotnet_path: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PathConfig {
    pub version: String,
    pub path: String,
    pub is_default: bool,
}

impl SetupConfig {
    pub fn load_or_default() -> Self {
        match Self::load() {
            Ok(config) => config,
            Err(_) => Self::default(),
        }
    }

    fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let mut path = get_config_dir()?;
        path.push("setup.json");
        
        if !path.exists() {
            return Err("Setup config not found".into());
        }
        
        let content = std::fs::read_to_string(path)?;
        let config: SetupConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut path = get_config_dir()?;
        std::fs::create_dir_all(&path)?;
        path.push("setup.json");
        
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }
}

impl Default for SetupConfig {
    fn default() -> Self {
        // Detect default paths based on OS
        let virtualenv_base = if cfg!(windows) {
            "%USERPROFILE%\\.virtualenvs".to_string()
        } else {
            "~/.virtualenvs".to_string()
        };

        Self {
            completed: false,
            docker_path: None,
            virtualenv_base_path: virtualenv_base,
            python_paths: Vec::new(),
            node_paths: Vec::new(),
            rust_path: None,
            go_path: None,
            java_path: None,
            dotnet_path: None,
        }
    }
}

// Network state management
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NetworkState {
    pub interfaces: Vec<NetworkInterface>,
    pub vlans: Vec<VlanConfig>,
    pub routes: Vec<Route>,
    pub firewall_rules: Vec<FirewallRule>,
    pub topology: Option<NetworkTopologyData>,
    pub topology_scanning: bool,
    pub loading: bool,
    pub last_error: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct NetworkInterface {
    pub name: String,
    pub display_name: String,
    pub status: String,
    pub ip_addresses: Vec<String>,
    pub mac_address: Option<String>,
    pub mtu: u32,
    pub interface_type: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct VlanConfig {
    pub id: u16,
    pub name: String,
    pub parent_interface: String,
    pub enabled: bool,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Route {
    pub destination: String,
    pub gateway: String,
    pub interface: String,
    pub metric: u32,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct FirewallRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action: String,
    pub direction: String,
}

// Remote Desktop state management
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct RemoteDesktopState {
    pub connections: Vec<RemoteConnection>,
    pub active_sessions: Vec<ActiveSession>,
    pub groups: Vec<ConnectionGroup>,
    pub loading: bool,
    pub last_error: Option<String>,
}

#[derive(Clone, Copy, Serialize, Deserialize, Debug, PartialEq)]
pub enum ConnectionProtocol {
    Ssh,
    Rdp,
    Vnc,
    Spice,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct Credentials {
    pub username: String,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,
    pub protocol: ConnectionProtocol,
    pub host: String,
    pub port: u16,
    pub credentials: Credentials,
    pub status: ConnectionStatus,
    pub last_connected: Option<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ActiveSession {
    pub session_id: String,
    pub connection_id: String,
    pub connection_name: String,
    pub started_at: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct ConnectionGroup {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub connection_count: usize,
}
