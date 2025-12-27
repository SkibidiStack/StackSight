use crate::core::error::CoreError;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub telemetry: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self { telemetry: true }
    }
}

impl AppConfig {
    pub async fn load(path: &str) -> Result<Self> {
        let expanded = shellexpand::tilde(path).to_string();
        let p = Path::new(&expanded);
        if !p.exists() {
            return Ok(AppConfig::default());
        }

        let data = fs::read_to_string(p).await.map_err(|e| CoreError::Config(e.to_string()))?;
        let cfg: AppConfig = toml::from_str(&data).map_err(|e| CoreError::Config(e.to_string()))?;
        Ok(cfg)
    }
}
