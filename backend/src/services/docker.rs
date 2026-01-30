use crate::core::event_bus::EventBus;
use crate::models::{
    commands::{Command, DockerScaffoldConfig},
    docker::{ContainerSummary, DockerStatsSummary, ImageSummary, NetworkSummary, VolumeSummary},
    events::Event,
};
use anyhow::Result;
use bollard::container::{ListContainersOptions, RestartContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions};
use bollard::image::{CreateImageOptions, ListImagesOptions, PruneImagesOptions};
use bollard::network::ListNetworksOptions;
use bollard::volume::ListVolumesOptions;
use bollard::{Docker, API_DEFAULT_VERSION};
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};
use futures_util::StreamExt;
use tokio::process::Command as TokioCommand;
use tokio::fs;

pub struct DockerService {
    bus: EventBus,
    client: Option<Docker>,
    poll_interval: Duration,
    cmd_rx: mpsc::Receiver<Command>,
    last_error: Option<String>,
}

impl DockerService {
    pub async fn new(bus: EventBus, cmd_rx: mpsc::Receiver<Command>) -> Result<Self> {
        Ok(Self {
            bus,
            client: None,
            poll_interval: Duration::from_secs(5),
            cmd_rx,
            last_error: None,
        })
    }

    fn publish_status(bus: &EventBus, connected: bool, error: Option<String>) {
        bus.publish(Event::DockerStatus { connected, error });
    }

    fn ensure_client(&mut self) -> bool {
        if self.client.is_some() {
            return true;
        }

        let mut attempts = Vec::new();

        if let Some(path) = docker_socket_from_env() {
            attempts.push(path);
        }

        for candidate in docker_socket_candidates() {
            if !attempts.contains(&candidate) {
                attempts.push(candidate);
            }
        }

        let mut last_err = None;
        for path in attempts {
            if !Path::new(&path).exists() {
                continue;
            }
            match Docker::connect_with_unix(&path, 120, API_DEFAULT_VERSION) {
                Ok(client) => {
                    self.client = Some(client);
                    self.last_error = None;
                    Self::publish_status(&self.bus, true, None);
                    return true;
                }
                Err(err) => {
                    last_err = Some(format!("{}: {}", path, err));
                }
            }
        }

        let msg = last_err.unwrap_or_else(|| "docker socket not found".to_string());
        self.last_error = Some(msg.clone());
        Self::publish_status(&self.bus, false, Some(msg));
        false
    }

    async fn snapshot_containers(client: &Docker) -> Result<Vec<ContainerSummary>> {
        let opts = ListContainersOptions::<String> { all: true, ..Default::default() };
        let containers = client.list_containers(Some(opts)).await?;

        let mapped = containers
            .into_iter()
            .map(|c| ContainerSummary {
                id: c.id.unwrap_or_default(),
                name: c
                    .names
                    .and_then(|mut n| n.pop())
                    .map(|n| n.trim_start_matches('/').to_string())
                    .unwrap_or_default(),
                state: c.state.unwrap_or_default(),
                image: c.image.unwrap_or_default(),
                status: c.status,
            })
            .collect();

        Ok(mapped)
    }

    async fn snapshot_stats(client: &Docker, containers: &[ContainerSummary]) -> DockerStatsSummary {
        let running: Vec<&ContainerSummary> = containers.iter().filter(|c| c.state == "running").collect();
        let running_count = running.len();
        if running_count == 0 {
            return DockerStatsSummary {
                containers: 0,
                cpu_percent_avg: 0.0,
                memory_used: 0,
                memory_limit: 0,
                net_rx: 0,
                net_tx: 0,
            };
        }

        let mut cpu_total = 0.0f64;
        let mut mem_used = 0u64;
        let mut mem_limit = 0u64;
        let mut net_rx = 0u64;
        let mut net_tx = 0u64;
        let mut samples = 0usize;

        for container in running.iter() {
            let mut stream = client.stats(
                &container.id,
                Some(StatsOptions {
                    stream: false,
                    one_shot: true,
                }),
            );

            if let Some(Ok(stats)) = stream.next().await {
                let cpu_delta = stats
                    .cpu_stats
                    .cpu_usage
                    .total_usage
                    .saturating_sub(stats.precpu_stats.cpu_usage.total_usage);

                let system_delta = stats
                    .cpu_stats
                    .system_cpu_usage
                    .unwrap_or_default()
                    .saturating_sub(stats.precpu_stats.system_cpu_usage.unwrap_or_default());

                let online_cpus = stats.cpu_stats.online_cpus.unwrap_or(1) as f64;

                if system_delta > 0 && cpu_delta > 0 {
                    let cpu_percent = (cpu_delta as f64 / system_delta as f64) * online_cpus * 100.0;
                    cpu_total += cpu_percent;
                }

                mem_used = mem_used.saturating_add(stats.memory_stats.usage.unwrap_or_default());
                mem_limit = mem_limit.saturating_add(stats.memory_stats.limit.unwrap_or_default());

                if let Some(networks) = stats.networks {
                    for net in networks.values() {
                        net_rx = net_rx.saturating_add(net.rx_bytes);
                        net_tx = net_tx.saturating_add(net.tx_bytes);
                    }
                }

                samples += 1;
            }
        }

        let avg_cpu = if samples == 0 { 0.0 } else { cpu_total / samples as f64 };

        DockerStatsSummary {
            containers: running_count,
            cpu_percent_avg: avg_cpu,
            memory_used: mem_used,
            memory_limit: mem_limit,
            net_rx,
            net_tx,
        }
    }

    async fn snapshot_images(client: &Docker) -> Result<Vec<ImageSummary>> {
        let images = client
            .list_images(Some(ListImagesOptions::<String> {
                all: true,
                ..Default::default()
            }))
            .await?;

        Ok(images
            .into_iter()
            .map(|img| ImageSummary {
                id: img.id,
                repo_tags: img.repo_tags,
                size: img.size as u64,
            })
            .collect())
    }

    async fn snapshot_networks(client: &Docker) -> Result<Vec<NetworkSummary>> {
        let networks = client
            .list_networks(Some(ListNetworksOptions::<String> { ..Default::default() }))
            .await?;

        Ok(networks
            .into_iter()
            .map(|net| NetworkSummary {
                id: net.id.unwrap_or_default(),
                name: net.name.unwrap_or_default(),
                driver: net.driver.unwrap_or_default(),
            })
            .collect())
    }

    async fn snapshot_volumes(client: &Docker) -> Result<Vec<VolumeSummary>> {
        let volumes = client
            .list_volumes(Some(ListVolumesOptions::<String> { ..Default::default() }))
            .await?
            .volumes
            .unwrap_or_default();

        Ok(volumes
            .into_iter()
            .map(|vol| VolumeSummary {
                name: vol.name,
                driver: vol.driver,
                mountpoint: vol.mountpoint,
            })
            .collect())
    }

    async fn docker_pull(client: &Docker, image: &str) -> Result<()> {
        let opts = Some(CreateImageOptions {
            from_image: image,
            ..Default::default()
        });
        let mut stream = client.create_image(opts, None, None);
        while let Some(_progress) = stream.next().await.transpose()? {}
        Ok(())
    }

    async fn docker_prune_images(client: &Docker) -> Result<()> {
        let _ = client
            .prune_images(Some(PruneImagesOptions::<String> { ..Default::default() }))
            .await?;
        Ok(())
    }

    async fn docker_build(context_path: &str, tag: Option<String>) -> Result<()> {
        if !Path::new(context_path).exists() {
            return Err(anyhow::anyhow!("build context not found: {context_path}"));
        }

        let mut cmd = TokioCommand::new("docker");
        cmd.arg("build");
        if let Some(tag) = tag {
            if !tag.is_empty() {
                cmd.arg("-t").arg(tag);
            }
        }
        cmd.arg(context_path);
        let output = cmd.output().await?;
        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
            let message = if stderr.is_empty() {
                format!("docker build failed with status: {}", output.status)
            } else {
                stderr
            };
            Err(anyhow::anyhow!(message))
        }
    }

    async fn docker_scaffold(config: &DockerScaffoldConfig) -> Result<()> {
        let dir = PathBuf::from(config.context_path.trim());
        if !dir.exists() {
            return Err(anyhow::anyhow!("context path not found: {}", config.context_path));
        }
        if !dir.is_dir() {
            return Err(anyhow::anyhow!("context path is not a directory: {}", config.context_path));
        }

        let dockerfile_path = dir.join("Dockerfile");
        if dockerfile_path.exists() {
            return Err(anyhow::anyhow!("Dockerfile already exists in {}", config.context_path));
        }

        if config.base_image.trim().is_empty() {
            return Err(anyhow::anyhow!("base image is required"));
        }

        let workdir = config
            .workdir
            .clone()
            .unwrap_or_else(|| "/app".to_string());
        let mut lines = Vec::new();
        lines.push(format!("FROM {}", config.base_image.trim()));
        lines.push(format!("WORKDIR {}", workdir.trim()));
        lines.push("COPY . .".to_string());

        if !config.ports.is_empty() {
            let ports = config
                .ports
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(" ");
            lines.push(format!("EXPOSE {}", ports));
        }

        if let Some(cmd) = config.cmd.clone() {
            if !cmd.trim().is_empty() {
                lines.push(format!("CMD {}", cmd.trim()));
            }
        }

        let contents = lines.join("\n") + "\n";

        fs::write(&dockerfile_path, contents).await?;

        Ok(())
    }

    async fn handle_command(&mut self, cmd: Command) {
        if !self.ensure_client() {
            return;
        }

        let Some(client) = self.client.as_ref() else {
            return;
        };

        match cmd {
            Command::DockerStart { id } => {
                if let Err(err) = client.start_container(&id, None::<StartContainerOptions<String>>).await {
                    warn!(container = %id, error = ?err, "docker start failed");
                }
            }
            Command::DockerStop { id } => {
                if let Err(err) = client.stop_container(&id, Some(StopContainerOptions { t: 5 })).await {
                    warn!(container = %id, error = ?err, "docker stop failed");
                }
            }
            Command::DockerRestart { id } => {
                if let Err(err) = client.restart_container(&id, Some(RestartContainerOptions { t: 5 })).await {
                    warn!(container = %id, error = ?err, "docker restart failed");
                }
            }
            Command::DockerList => {}
            Command::DockerListImages => {}
            Command::DockerPullImage { image } => {
                let result = Self::docker_pull(client, &image).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "pull image".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "pull image".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerPruneImages => {
                let result = Self::docker_prune_images(client).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "clean images".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "clean images".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerBuildImage { context_path, tag } => {
                let result = Self::docker_build(&context_path, tag).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerListNetworks => {}
            Command::DockerListVolumes => {}
            Command::DockerScaffold { config } => {
                let result = Self::docker_scaffold(&config).await;
                match result {
                    Ok(()) => {
                        self.bus.publish(Event::DockerAction { action: "scaffold dockerfile".to_string(), ok: true, message: None });
                        for image in config.additional_images.iter() {
                            let image = image.trim();
                            if image.is_empty() {
                                continue;
                            }
                            let pull = Self::docker_pull(client, image).await;
                            if let Err(err) = pull {
                                self.bus.publish(Event::DockerAction { action: "pull image".to_string(), ok: false, message: Some(err.to_string()) });
                            }
                        }
                    }
                    Err(err) => self.bus.publish(Event::DockerAction { action: "scaffold dockerfile".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
        }

        match Self::snapshot_containers(client).await {
            Ok(containers) => {
                let stats = Self::snapshot_stats(client, &containers).await;
                self.bus.publish(Event::DockerContainers(containers));
                self.bus.publish(Event::DockerStats(stats));
            }
            Err(err) => warn!(error = ?err, "docker refresh failed"),
        }

        if let Ok(images) = Self::snapshot_images(client).await {
            self.bus.publish(Event::DockerImages(images));
        }

        if let Ok(networks) = Self::snapshot_networks(client).await {
            self.bus.publish(Event::DockerNetworks(networks));
        }

        if let Ok(volumes) = Self::snapshot_volumes(client).await {
            self.bus.publish(Event::DockerVolumes(volumes));
        }
    }
}

fn docker_socket_from_env() -> Option<String> {
    let host = env::var("DOCKER_HOST").ok()?;
    if let Some(path) = host.strip_prefix("unix://") {
        return Some(path.to_string());
    }
    None
}

fn docker_socket_candidates() -> Vec<String> {
    let mut candidates = vec!["/var/run/docker.sock".to_string(), "/run/docker.sock".to_string()];

    if let Ok(runtime_dir) = env::var("XDG_RUNTIME_DIR") {
        candidates.push(format!("{}/docker.sock", runtime_dir));
    }

    if let Ok(uid) = env::var("UID") {
        candidates.push(format!("/run/user/{}/docker.sock", uid));
    }

    if let Ok(home) = env::var("HOME") {
        candidates.push(format!("{}/.docker/desktop/docker.sock", home));
        candidates.push(format!("{}/.docker/run/docker.sock", home));
    }

    candidates
}

#[async_trait::async_trait]
impl crate::services::Service for DockerService {
    async fn start(&mut self) -> Result<()> {
        info!("docker service start");
        let _ = self.ensure_client();
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        let poll_interval = self.poll_interval;

        loop {
            tokio::select! {
                biased;
                maybe_cmd = self.cmd_rx.recv() => {
                    if let Some(cmd) = maybe_cmd {
                        self.handle_command(cmd).await;
                    } else {
                        break;
                    }
                }
                _ = tokio::time::sleep(poll_interval) => {
                    if !self.ensure_client() {
                        continue;
                    }

                    if let Some(client) = self.client.as_ref() {
                        match Self::snapshot_containers(client).await {
                            Ok(containers) => {
                                let stats = Self::snapshot_stats(client, &containers).await;
                                self.bus.publish(Event::DockerContainers(containers));
                                self.bus.publish(Event::DockerStats(stats));
                            }
                            Err(err) => warn!(error = ?err, "docker snapshot failed"),
                        }

                        if let Ok(images) = Self::snapshot_images(client).await {
                            self.bus.publish(Event::DockerImages(images));
                        }

                        if let Ok(networks) = Self::snapshot_networks(client).await {
                            self.bus.publish(Event::DockerNetworks(networks));
                        }

                        if let Ok(volumes) = Self::snapshot_volumes(client).await {
                            self.bus.publish(Event::DockerVolumes(volumes));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
