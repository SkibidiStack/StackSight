use crate::core::event_bus::EventBus;
use crate::models::events::Event;
use crate::models::virtenv::VirtualEnvSummary;
use anyhow::Result;
use std::env;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

pub struct VirtualEnvService {
    bus: EventBus,
    poll_interval: std::time::Duration,
}

impl VirtualEnvService {
    pub async fn new(bus: EventBus) -> Result<Self> {
        Ok(Self { bus, poll_interval: std::time::Duration::from_secs(10) })
    }
}

#[async_trait::async_trait]
impl crate::services::Service for VirtualEnvService {
    async fn start(&mut self) -> Result<()> {
        info!("virtenv service start");
        Ok(())
    }

    async fn run(self) -> Result<()> {
        let mut interval = tokio::time::interval(self.poll_interval);
        loop {
            interval.tick().await;
            match count_python_envs().await {
                Ok(total) => {
                    let summary = VirtualEnvSummary { total, active: 0 };
                    self.bus.publish(Event::VirtualEnvSummary(summary));
                }
                Err(err) => {
                    info!(error = ?err, "virtenv scan failed");
                }
            }
        }
    }
}

async fn count_python_envs() -> Result<usize> {
    let mut total = 0usize;
    for root in python_env_roots() {
        if !root.exists() {
            continue;
        }
        total += count_envs_in_dir(&root).await.unwrap_or(0);
    }
    Ok(total)
}

async fn count_envs_in_dir(root: &Path) -> Result<usize> {
    let mut count = 0usize;
    let mut entries = fs::read_dir(root).await?;
    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if entry.file_type().await?.is_dir() {
            if is_python_venv(&path).await {
                count += 1;
            }
        }
    }
    Ok(count)
}

async fn is_python_venv(path: &Path) -> bool {
    let cfg = path.join("pyvenv.cfg");
    fs::metadata(cfg).await.is_ok()
}

fn python_env_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(home) = env::var("HOME") {
        roots.push(PathBuf::from(format!("{home}/.virtualenvs")));
        roots.push(PathBuf::from(format!("{home}/.local/share/virtualenvs")));
        roots.push(PathBuf::from(format!("{home}/.venvs")));
    }
    if let Ok(workon_home) = env::var("WORKON_HOME") {
        roots.push(PathBuf::from(workon_home));
    }
    roots
}
