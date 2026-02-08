use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualEnvSummary {
    pub total: usize,
    pub active: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VirtualEnvironment {
    pub id: String,
    pub name: String,
    pub path: PathBuf,
    pub language: Language,
    pub version: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub packages: Vec<Package>,
    #[serde(default = "chrono::Utc::now")]
    pub created_at: chrono::DateTime<chrono::Utc>,
    #[serde(default)]
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub project_path: Option<PathBuf>,
    #[serde(default)]
    pub health: EnvironmentHealth,
    #[serde(default)]
    pub size_mb: Option<u64>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Node,
    Rust,
    Java,
    Ruby,
    Php,
    Other(String),
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::Python => "python",
            Language::Node => "node",
            Language::Rust => "rust",
            Language::Java => "java",
            Language::Ruby => "ruby",
            Language::Php => "php",
            Language::Other(name) => name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Package {
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub size: Option<u64>,
    pub dependencies: Vec<String>,
    pub is_dev_dependency: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentHealth {
    pub status: HealthStatus,
    pub issues: Vec<String>,
    pub last_check: chrono::DateTime<chrono::Utc>,
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<u64>,
}

impl Default for EnvironmentHealth {
    fn default() -> Self {
        Self {
            status: HealthStatus::Unknown,
            issues: Vec::new(),
            last_check: chrono::Utc::now(),
            cpu_usage: None,
            memory_usage: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Error,
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub language: Language,
    pub packages: Vec<String>,
    pub scripts: HashMap<String, String>,
    pub files: HashMap<String, String>,
    pub settings: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub language: Language,
    pub version: Option<String>,
    pub template: Option<String>,
    pub project_path: Option<PathBuf>,
    pub packages: Vec<String>,
    pub location: Option<PathBuf>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageOperation {
    pub env_id: String,
    pub operation: PackageOperationType,
    pub packages: Vec<String>,
    pub options: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PackageOperationType {
    Install,
    Uninstall,
    Update,
    Upgrade,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EnvironmentAction {
    pub env_id: String,
    pub action: ActionType,
    pub parameters: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ActionType {
    Activate,
    Deactivate,
    Clone,
    Delete,
    Export,
    Shell,
    RunScript,
}
