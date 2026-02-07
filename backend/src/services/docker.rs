use crate::core::event_bus::EventBus;
use crate::models::{
    commands::{Command, DockerScaffoldConfig, DockerCreateContainerConfig},
    docker::{ContainerSummary, DockerStatsSummary, ImageSummary, NetworkSummary, VolumeSummary},
    events::Event,
};
use anyhow::Result;
use bollard::container::{ListContainersOptions, RestartContainerOptions, StartContainerOptions, StatsOptions, StopContainerOptions, CreateContainerOptions, Config, RemoveContainerOptions, LogsOptions};
use bollard::image::{CreateImageOptions, ListImagesOptions, PruneImagesOptions, RemoveImageOptions};
use bollard::network::{ListNetworksOptions, CreateNetworkOptions};
use bollard::volume::{ListVolumesOptions, CreateVolumeOptions, RemoveVolumeOptions};
use bollard::service::{HostConfig, PortBinding};
use bollard::{Docker, API_DEFAULT_VERSION};
use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};
use futures_util::StreamExt;
use tar::{Builder as TarBuilder, Header};
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

    // Disabled automatic build to prevent crashes - use manual build UI instead
    // async fn docker_build_with_bollard(docker: &Docker, dockerfile_content: &str, tag: Option<String>, project_path: &Path) -> Result<()> {
    //     // Implementation disabled - causes connection crashes
    //     Err(anyhow::anyhow!("Automatic build disabled - use manual build UI"))
    // }

    async fn docker_build_from_dockerfile(&self, _path: &str, _dockerfile: &str, _tag: Option<String>) -> Result<()> {
        // Automatic build disabled to prevent crashes
        // Users should use the manual build UI instead
        Err(anyhow::anyhow!("Automatic build has been disabled to prevent crashes. Please use the manual build feature in the UI."))
    }

    async fn docker_build_manual(dockerfile_path: &str, project_path: &str, tag: &str) -> Result<()> {
        use tokio::process::Command;
        
        info!("Starting manual Docker build using CLI");
        info!("Dockerfile: {}", dockerfile_path);
        info!("Project path: {}", project_path);
        info!("Tag: {}", tag);
        
        // Verify paths exist
        if !std::path::Path::new(dockerfile_path).exists() {
            return Err(anyhow::anyhow!("Dockerfile not found: {}", dockerfile_path));
        }
        
        if !std::path::Path::new(project_path).exists() {
            return Err(anyhow::anyhow!("Project path not found: {}", project_path));
        }
        
        // Build docker command
        let mut cmd = Command::new("docker");
        cmd.arg("build")
           .arg("-f")
           .arg(dockerfile_path)
           .arg("-t")
           .arg(tag)
           .arg(project_path)
           .current_dir(project_path);
        
        info!("Running: docker build -f {} -t {} {}", dockerfile_path, tag, project_path);
        
        // Execute the command
        let output = cmd.output().await?;
        
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            info!("Build successful: {}", stdout);
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            Err(anyhow::anyhow!("Build failed:\nSTDOUT: {}\nSTDERR: {}", stdout, stderr))
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

    async fn docker_create_container(client: &Docker, config: &DockerCreateContainerConfig) -> Result<()> {
        let mut exposed_ports = HashMap::new();
        let mut port_bindings = HashMap::new();

        for port_spec in config.ports.iter() {
            let parts: Vec<&str> = port_spec.split(':').collect();
            if parts.len() == 2 {
                let container_port = format!("{}/tcp", parts[1]);
                exposed_ports.insert(container_port.clone(), HashMap::new());
                port_bindings.insert(
                    container_port,
                    Some(vec![PortBinding {
                        host_ip: Some("0.0.0.0".to_string()),
                        host_port: Some(parts[0].to_string()),
                    }]),
                );
            }
        }

        let mut binds = Vec::new();
        for vol in config.volumes.iter() {
            binds.push(vol.clone());
        }

        let host_config = if !port_bindings.is_empty() || !binds.is_empty() {
            Some(HostConfig {
                port_bindings: if port_bindings.is_empty() { None } else { Some(port_bindings) },
                binds: if binds.is_empty() { None } else { Some(binds) },
                ..Default::default()
            })
        } else {
            None
        };

        let container_config = Config {
            image: Some(config.image.clone()),
            env: if config.env.is_empty() { None } else { Some(config.env.clone()) },
            exposed_ports: if exposed_ports.is_empty() { None } else { Some(exposed_ports) },
            cmd: config.cmd.clone(),
            host_config,
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: config.name.clone(),
            ..Default::default()
        };

        client.create_container(Some(options), container_config).await?;
        Ok(())
    }

    async fn docker_create_network(client: &Docker, name: &str, driver: &str) -> Result<()> {
        let opts = CreateNetworkOptions {
            name: name.to_string(),
            driver: driver.to_string(),
            ..Default::default()
        };

        client.create_network(opts).await?;
        Ok(())
    }

    async fn docker_create_volume(client: &Docker, name: &str, driver: Option<&str>) -> Result<()> {
        let opts = CreateVolumeOptions {
            name: name.to_string(),
            driver: driver.unwrap_or("local").to_string(),
            ..Default::default()
        };

        client.create_volume(opts).await?;
        Ok(())
    }

    async fn docker_remove_container(client: &Docker, id: &str, force: bool) -> Result<()> {
        let opts = Some(RemoveContainerOptions {
            force,
            ..Default::default()
        });
        client.remove_container(id, opts).await?;
        Ok(())
    }

    async fn docker_remove_image(client: &Docker, id: &str, force: bool) -> Result<()> {
        let opts = Some(RemoveImageOptions {
            force,
            ..Default::default()
        });
        client.remove_image(id, opts, None).await?;
        Ok(())
    }

    async fn docker_remove_network(client: &Docker, id: &str) -> Result<()> {
        client.remove_network(id).await?;
        Ok(())
    }

    async fn docker_remove_volume(client: &Docker, name: &str, force: bool) -> Result<()> {
        let opts = Some(RemoveVolumeOptions { force });
        client.remove_volume(name, opts).await?;
        Ok(())
    }

    async fn docker_container_logs(client: &Docker, id: &str) -> Result<String> {
        use bollard::container::LogOutput;
        
        let opts = Some(LogsOptions::<String> {
            stdout: true,
            stderr: true,
            tail: "100".to_string(),
            ..Default::default()
        });

        let mut stream = client.logs(id, opts);
        let mut logs = Vec::new();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(output) => match output {
                    LogOutput::StdOut { message } | LogOutput::StdErr { message } => {
                        logs.push(String::from_utf8_lossy(&message).to_string());
                    }
                    _ => {}
                },
                Err(_) => break,
            }
        }

        Ok(logs.join(""))
    }

    async fn docker_run_image(client: &Docker, image: &str) -> Result<()> {
        // Generate a simple container name from image
        let image_name = image.split(':').next().unwrap_or(image)
            .split('/').last().unwrap_or("container");
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let container_name = format!("{}-{}", image_name, timestamp);

        // Create container with minimal config
        let container_config = Config {
            image: Some(image.to_string()),
            ..Default::default()
        };

        let options = CreateContainerOptions {
            name: container_name.clone(),
            ..Default::default()
        };

        let response = client.create_container(Some(options), container_config).await?;
        
        // Start the container immediately
        client.start_container(&response.id, None::<StartContainerOptions<String>>).await?;
        
        Ok(())
    }

    async fn docker_analyze_folder(path: &str) -> Result<String> {
        let project_path = std::path::Path::new(path);
        if !project_path.exists() || !project_path.is_dir() {
            return Err(anyhow::anyhow!("Path does not exist or is not a directory"));
        }

        let mut base_image = "ubuntu:22.04";
        let workdir = "/app";
        let mut install_cmds = Vec::new();
        let mut copy_cmd = "COPY . .";
        let mut expose_ports = Vec::new();
        let mut cmd = "";

        // Detect project type
        if project_path.join("package.json").exists() {
            base_image = "node:18";
            install_cmds.push("RUN npm install".to_string());
            expose_ports.push(3000);
            cmd = "CMD [\"npm\", \"start\"]";
        } else if project_path.join("requirements.txt").exists() {
            base_image = "python:3.11";
            install_cmds.push("COPY requirements.txt .".to_string());
            install_cmds.push("RUN pip install --no-cache-dir -r requirements.txt".to_string());
            copy_cmd = "COPY . .";
            expose_ports.push(8000);
            cmd = "CMD [\"python\", \"app.py\"]";
        } else if project_path.join("Cargo.toml").exists() {
            base_image = "rust:1.75";
            install_cmds.push("COPY Cargo.toml Cargo.lock ./".to_string());
            install_cmds.push("RUN mkdir src && echo 'fn main() {}' > src/main.rs".to_string());
            install_cmds.push("RUN cargo build --release".to_string());
            copy_cmd = "COPY . .";
            install_cmds.push("RUN cargo build --release".to_string());
            expose_ports.push(8080);
            cmd = "CMD [\"./target/release/app\"]";
        } else if project_path.join("go.mod").exists() {
            base_image = "golang:1.21";
            install_cmds.push("COPY go.mod go.sum ./".to_string());
            install_cmds.push("RUN go mod download".to_string());
            copy_cmd = "COPY . .";
            install_cmds.push("RUN go build -o app .".to_string());
            expose_ports.push(8080);
            cmd = "CMD [\"./app\"]";
        } else if project_path.join(".csproj").exists() || project_path.join("Program.cs").exists() {
            base_image = "mcr.microsoft.com/dotnet/sdk:8.0";
            install_cmds.push("RUN dotnet restore".to_string());
            install_cmds.push("RUN dotnet build -c Release".to_string());
            expose_ports.push(5000);
            cmd = "CMD [\"dotnet\", \"run\"]";
        }

        // Build Dockerfile
        let mut dockerfile = format!("FROM {}\n\n", base_image);
        dockerfile.push_str(&format!("WORKDIR {}\n\n", workdir));
        
        let has_install_cmds = !install_cmds.is_empty();
        for install_cmd in install_cmds {
            dockerfile.push_str(&format!("{}\n", install_cmd));
        }
        
        if has_install_cmds {
            dockerfile.push('\n');
        }
        
        dockerfile.push_str(&format!("{}\n\n", copy_cmd));
        
        if !expose_ports.is_empty() {
            dockerfile.push_str(&format!("EXPOSE {}\n\n", expose_ports.iter().map(|p| p.to_string()).collect::<Vec<_>>().join(" ")));
        }
        
        if !cmd.is_empty() {
            dockerfile.push_str(&format!("{}\n", cmd));
        }

        Ok(dockerfile)
    }

    async fn docker_save_dockerfile(path: &str, dockerfile: &str) -> Result<()> {
        let project_path = Path::new(path);
        if !project_path.exists() || !project_path.is_dir() {
            return Err(anyhow::anyhow!("Project path does not exist or is not a directory"));
        }

        let dockerfile_path = project_path.join("Dockerfile");
        fs::write(&dockerfile_path, dockerfile).await?;
        Ok(())
    }

    async fn handle_command(&mut self, cmd: Command) {
        // Handle engine control commands first (these work without Docker client)
        match &cmd {
            Command::DockerStartEngine => {
                info!("Received DockerStartEngine command");
                let result = Self::start_docker_engine().await;
                match result {
                    Ok(()) => {
                        info!("Docker engine started successfully");
                        // Force reconnection attempt
                        self.client = None;
                        let _ = self.ensure_client();
                        self.bus.publish(Event::DockerAction { action: "start engine".to_string(), ok: true, message: Some("Docker engine started".to_string()) });
                    }
                    Err(err) => {
                        warn!("Failed to start Docker engine: {:?}", err);
                        self.bus.publish(Event::DockerAction { action: "start engine".to_string(), ok: false, message: Some(err.to_string()) });
                    }
                }
                return;
            }
            Command::DockerStopEngine => {
                info!("Received DockerStopEngine command");
                let result = Self::stop_docker_engine().await;
                match result {
                    Ok(()) => {
                        info!("Docker engine stopped successfully");
                        // Clear client connection and update status
                        self.client = None;
                        self.last_error = Some("Docker engine stopped".to_string());
                        Self::publish_status(&self.bus, false, Some("Docker engine stopped".to_string()));
                        self.bus.publish(Event::DockerAction { action: "stop engine".to_string(), ok: true, message: Some("Docker engine stopped".to_string()) });
                    }
                    Err(err) => {
                        warn!("Failed to stop Docker engine: {:?}", err);
                        self.bus.publish(Event::DockerAction { action: "stop engine".to_string(), ok: false, message: Some(err.to_string()) });
                    }
                }
                return;
            }
            Command::DockerGetEngineLogs => {
                info!("Received DockerGetEngineLogs command");
                let result = Self::get_docker_logs().await;
                match result {
                    Ok(logs) => {
                        info!("Retrieved {} bytes of Docker logs", logs.len());
                        self.bus.publish(Event::DockerEngineLogs { logs });
                    }
                    Err(err) => {
                        warn!("Failed to get Docker logs: {:?}", err);
                        self.bus.publish(Event::DockerAction { action: "get engine logs".to_string(), ok: false, message: Some(err.to_string()) });
                    }
                }
                return;
            }
            _ => {} // Continue to regular command handling
        }

        if !self.ensure_client() {
            return;
        }

        let Some(client) = self.client.as_ref() else {
            return;
        };

        match cmd {
            // Engine commands are handled above
            Command::DockerStartEngine | Command::DockerStopEngine | Command::DockerGetEngineLogs => {
                // These are handled above before client check
            }
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
            Command::DockerRemoveContainer { id, force } => {
                let result = Self::docker_remove_container(client, &id, force).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "remove container".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "remove container".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerContainerLogs { id } => {
                let result = Self::docker_container_logs(client, &id).await;
                match result {
                    Ok(logs) => {
                        self.bus.publish(Event::DockerLogs { container_id: id, logs });
                    }
                    Err(err) => self.bus.publish(Event::DockerAction { action: "get logs".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerCreateContainer { config } => {
                let result = Self::docker_create_container(client, &config).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "create container".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "create container".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerAnalyzeFolder { path } => {
                let result = Self::docker_analyze_folder(&path).await;
                match result {
                    Ok(dockerfile) => self.bus.publish(Event::DockerfileGenerated { path, dockerfile }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "analyze folder".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerSaveDockerfile { path, dockerfile } => {
                let result = Self::docker_save_dockerfile(&path, &dockerfile).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerfileSaved { path }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "save dockerfile".to_string(), ok: false, message: Some(err.to_string()) }),
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
            Command::DockerRemoveImage { id, force } => {
                let result = Self::docker_remove_image(client, &id, force).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "remove image".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "remove image".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerRunImage { image } => {
                let result = Self::docker_run_image(client, &image).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "run image".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "run image".to_string(), ok: false, message: Some(err.to_string()) }),
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
                // For simple builds without custom Dockerfile, read the existing Dockerfile
                let dockerfile_path = Path::new(&context_path).join("Dockerfile");
                if !dockerfile_path.exists() {
                    self.bus.publish(Event::DockerAction { 
                        action: "build image".to_string(), 
                        ok: false, 
                        message: Some("Dockerfile not found in context path".to_string()) 
                    });
                    return;
                }
                
                let dockerfile_content = match fs::read_to_string(&dockerfile_path).await {
                    Ok(content) => content,
                    Err(e) => {
                        self.bus.publish(Event::DockerAction { 
                            action: "build image".to_string(), 
                            ok: false, 
                            message: Some(format!("Failed to read Dockerfile: {}", e)) 
                        });
                        return;
                    }
                };
                
                let result = self.docker_build_from_dockerfile(&context_path, &dockerfile_content, tag).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerBuildFromDockerfile { path, dockerfile, tag } => {
                // Automatic build disabled to prevent crashes
                warn!("Automatic build disabled - directing user to manual build UI");
                self.bus.publish(Event::DockerAction { 
                    action: "build image".to_string(), 
                    ok: false, 
                    message: Some("Automatic build disabled to prevent crashes. Use the 'Manual Build' button in the Images section.".to_string()) 
                });
            }
            Command::DockerBuildManual { dockerfile_path, project_path, tag } => {
                let result = Self::docker_build_manual(&dockerfile_path, &project_path, &tag).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: true, message: Some(format!("Successfully built {}", tag)) }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "build image".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerListNetworks => {}
            Command::DockerCreateNetwork { name, driver } => {
                let result = Self::docker_create_network(client, &name, &driver).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "create network".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "create network".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerRemoveNetwork { id } => {
                let result = Self::docker_remove_network(client, &id).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "remove network".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "remove network".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerListVolumes => {}
            Command::DockerCreateVolume { name, driver } => {
                let result = Self::docker_create_volume(client, &name, driver.as_deref()).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "create volume".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "create volume".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
            Command::DockerRemoveVolume { name, force } => {
                let result = Self::docker_remove_volume(client, &name, force).await;
                match result {
                    Ok(()) => self.bus.publish(Event::DockerAction { action: "remove volume".to_string(), ok: true, message: None }),
                    Err(err) => self.bus.publish(Event::DockerAction { action: "remove volume".to_string(), ok: false, message: Some(err.to_string()) }),
                }
            }
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

// Engine control functions
impl DockerService {
    async fn start_docker_engine() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use tokio::process::Command;
            
            // Try with pkexec (GUI sudo) for systemctl
            info!("Attempting to start Docker with pkexec systemctl");
            let pkexec_result = Command::new("pkexec")
                .args(&["systemctl", "start", "docker"])
                .output()
                .await;
            
            if let Ok(output) = pkexec_result {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!("pkexec systemctl start docker - exit: {:?}, stdout: '{}', stderr: '{}'", output.status.code(), stdout, stderr);
                
                if output.status.success() {
                    return Ok(());
                }
                // If pkexec failed, check the error
                if stderr.contains("dismissed") || stderr.contains("cancelled") {
                    return Err(anyhow::anyhow!("Authentication cancelled by user"));
                }
                if stderr.contains("not authorized") {
                    return Err(anyhow::anyhow!("Not authorized to start Docker service"));
                }
            } else {
                warn!("Failed to execute pkexec: {:?}", pkexec_result.err());
            }
            
            // Try systemctl without sudo (might work if user is in docker group)
            info!("Attempting to start Docker with regular systemctl");
            let systemctl_result = Command::new("systemctl")
                .args(&["start", "docker"])
                .output()
                .await;
            
            if let Ok(output) = systemctl_result {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);
                info!("systemctl start docker - exit: {:?}, stdout: '{}', stderr: '{}'", output.status.code(), stdout, stderr);
                
                if output.status.success() {
                    return Ok(());
                }
                if stderr.contains("Access denied") {
                    return Err(anyhow::anyhow!("Failed to start Docker: Permission denied. Run 'sudo systemctl start docker' in terminal, or add your user to the docker group."));
                }
                if stderr.contains("not found") || stderr.contains("could not be found") {
                    // Check if Docker Desktop is installed instead
                    info!("Docker service not found, checking for Docker Desktop");
                    
                    // Check if Docker Desktop is available
                    let desktop_check = Command::new("docker")
                        .args(&["context", "ls"])
                        .output()
                        .await;
                    
                    if let Ok(output) = desktop_check {
                        let contexts = String::from_utf8_lossy(&output.stdout);
                        info!("Docker contexts: {}", contexts);
                        
                        if contexts.contains("desktop-linux") {
                            // Try to start Docker Desktop service (systemd user service)
                            info!("Trying to start Docker Desktop user service");
                            let desktop_start = Command::new("systemctl")
                                .args(&["--user", "start", "docker-desktop"])
                                .output()
                                .await;
                            
                            if let Ok(desktop_output) = desktop_start {
                                let desktop_stderr = String::from_utf8_lossy(&desktop_output.stderr);
                                info!("Docker Desktop start result: exit={:?}, stderr='{}'", desktop_output.status.code(), desktop_stderr);
                                
                                if desktop_output.status.success() {
                                    // Wait a moment for the service to initialize
                                    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
                                    return Ok(());
                                }
                            }
                            
                            return Err(anyhow::anyhow!("Docker Desktop is installed but couldn't start the daemon. Try starting Docker Desktop manually from your applications menu."));
                        }
                    }
                    
                    return Err(anyhow::anyhow!("Docker service not found. Install Docker with 'sudo apt install docker.io' or install Docker Desktop"));
                }
                if stderr.contains("Failed to start") {
                    return Err(anyhow::anyhow!("Docker service failed to start: {}", stderr));
                }
            } else {
                warn!("Failed to execute systemctl: {:?}", systemctl_result.err());
            }
            
            Err(anyhow::anyhow!("Failed to start Docker daemon. Try running 'sudo systemctl start docker' in a terminal or ensure Docker daemon is installed."))
        }
        
        #[cfg(target_os = "macos")]
        {
            use tokio::process::Command;
            // On macOS, try to start Docker Desktop or Colima
            let docker_desktop = Command::new("open")
                .args(&["-a", "Docker"])
                .output()
                .await;
            
            if let Ok(output) = docker_desktop {
                if output.status.success() {
                    return Ok(());
                }
            }
            
            // Try Colima
            let colima_result = Command::new("colima")
                .args(&["start"])
                .output()
                .await;
            
            if let Ok(output) = colima_result {
                if output.status.success() {
                    return Ok(());
                }
            }
            
            Err(anyhow::anyhow!("Failed to start Docker. Install Docker Desktop or Colima."))
        }
        
        #[cfg(target_os = "windows")]
        {
            use tokio::process::Command;
            // On Windows, try to start Docker Desktop service
            let result = Command::new("powershell")
                .args(&["-Command", "Start-Service", "com.docker.service"])
                .output()
                .await;
            
            if let Ok(output) = result {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to start Docker: {}", stderr));
            }
            
            Err(anyhow::anyhow!("Failed to start Docker engine. Make sure Docker Desktop is installed."))
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow::anyhow!("Starting Docker engine is not supported on this platform"))
        }
    }

    async fn stop_docker_engine() -> Result<()> {
        #[cfg(target_os = "linux")]
        {
            use tokio::process::Command;
            
            // Try with pkexec (GUI sudo) for systemctl
            let pkexec_result = Command::new("pkexec")
                .args(&["systemctl", "stop", "docker"])
                .output()
                .await;
            
            if let Ok(output) = pkexec_result {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("dismissed") || stderr.contains("cancelled") {
                    return Err(anyhow::anyhow!("Authentication cancelled by user"));
                }
            }
            
            // Try systemctl without sudo
            let systemctl_result = Command::new("systemctl")
                .args(&["stop", "docker"])
                .output()
                .await;
            
            if let Ok(output) = systemctl_result {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("Access denied") {
                    return Err(anyhow::anyhow!("Failed to stop Docker: Permission denied. Run 'sudo systemctl stop docker' in terminal."));
                }
            }
            
            Err(anyhow::anyhow!("Failed to stop Docker daemon. Try running 'sudo systemctl stop docker' in a terminal."))
        }
        
        #[cfg(target_os = "macos")]
        {
            use tokio::process::Command;
            // Try to stop Colima
            let colima_result = Command::new("colima")
                .args(&["stop"])
                .output()
                .await;
            
            if let Ok(output) = colima_result {
                if output.status.success() {
                    return Ok(());
                }
            }
            
            Err(anyhow::anyhow!("Cannot stop Docker Desktop programmatically on macOS. Please quit Docker Desktop manually."))
        }
        
        #[cfg(target_os = "windows")]
        {
            use tokio::process::Command;
            let result = Command::new("powershell")
                .args(&["-Command", "Stop-Service", "com.docker.service"])
                .output()
                .await;
            
            if let Ok(output) = result {
                if output.status.success() {
                    return Ok(());
                }
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(anyhow::anyhow!("Failed to stop Docker: {}", stderr));
            }
            
            Err(anyhow::anyhow!("Failed to stop Docker engine"))
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow::anyhow!("Stopping Docker engine is not supported on this platform"))
        }
    }

    async fn get_docker_logs() -> Result<String> {
        #[cfg(target_os = "linux")]
        {
            use tokio::process::Command;
            // Try journalctl first for systemd systems
            let journalctl_result = Command::new("journalctl")
                .args(&["-u", "docker", "-n", "100", "--no-pager"])
                .output()
                .await;
            
            if let Ok(output) = journalctl_result {
                if output.status.success() {
                    let logs = String::from_utf8_lossy(&output.stdout).to_string();
                    if !logs.trim().is_empty() && !logs.contains("No entries") {
                        return Ok(logs);
                    }
                }
            }
            
            // Try dockerd logs if running
            let dockerd_logs = Command::new("journalctl")
                .args(&["-u", "dockerd", "-n", "100", "--no-pager"])
                .output()
                .await;
            
            if let Ok(output) = dockerd_logs {
                if output.status.success() {
                    let logs = String::from_utf8_lossy(&output.stdout).to_string();
                    if !logs.trim().is_empty() && !logs.contains("No entries") {
                        return Ok(logs);
                    }
                }
            }
            
            // Fall back to log files
            let log_paths = vec![
                "/var/log/docker.log",
                "/var/log/docker/docker.log",
                "/var/log/syslog",
            ];
            
            for path in log_paths {
                if let Ok(content) = fs::read_to_string(path).await {
                    let docker_logs: Vec<&str> = content
                        .lines()
                        .filter(|line| line.contains("docker") || line.contains("Docker"))
                        .rev()
                        .take(100)
                        .collect();
                    if !docker_logs.is_empty() {
                        return Ok(docker_logs.into_iter().rev().collect::<Vec<_>>().join("\n"));
                    }
                }
            }
            
            Ok("No Docker engine logs found.\n\nPossible reasons:\n- Docker is not running via systemd (may be Docker Desktop)\n- Docker has not logged any events yet\n- You may need elevated permissions to read Docker logs\n\nTry: sudo journalctl -u docker -n 100".to_string())
        }
        
        #[cfg(target_os = "macos")]
        {
            use tokio::process::Command;
            // Check Docker Desktop logs
            let home = std::env::var("HOME").unwrap_or_else(|_| "/Users".to_string());
            let log_path = format!("{}/Library/Containers/com.docker.docker/Data/log/vm/docker.log", home);
            
            if let Ok(content) = fs::read_to_string(&log_path).await {
                let lines: Vec<&str> = content.lines().rev().take(100).collect();
                return Ok(lines.into_iter().rev().collect::<Vec<_>>().join("\n"));
            }
            
            // Try Colima logs
            let colima_result = Command::new("colima")
                .args(&["logs", "--tail", "100"])
                .output()
                .await;
            
            if let Ok(output) = colima_result {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
            
            Ok("No Docker logs found. Install Docker Desktop or Colima.".to_string())
        }
        
        #[cfg(target_os = "windows")]
        {
            use tokio::process::Command;
            // Get Docker Desktop logs from Event Viewer
            let result = Command::new("powershell")
                .args(&[
                    "-Command",
                    "Get-EventLog -LogName Application -Source Docker -Newest 100 | Format-Table -AutoSize | Out-String -Width 200"
                ])
                .output()
                .await;
            
            if let Ok(output) = result {
                if output.status.success() {
                    return Ok(String::from_utf8_lossy(&output.stdout).to_string());
                }
            }
            
            Ok("Failed to fetch Docker logs from Event Viewer".to_string())
        }
        
        #[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
        {
            Err(anyhow::anyhow!("Getting Docker logs is not supported on this platform"))
        }
    }
}

// Helper function to add directory contents to tar
fn add_directory_to_tar(tar: &mut TarBuilder<&mut Vec<u8>>, dir: &Path, prefix: &str) -> Result<()> {
    use std::fs::File;
    use std::io::Read;
    
    // Skip common directories that shouldn't be in build context
    let skip_dirs = [".git", "target", "node_modules", "__pycache__", ".vscode", ".idea", "build", "dist"];
    let skip_files = [".dockerignore", ".gitignore", "Dockerfile"]; // Dockerfile is added separately
    
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        
        // Skip hidden files and common build/cache directories
        if name.starts_with('.') && !name.starts_with(".env") {
            continue;
        }
        
        if skip_dirs.contains(&name.as_str()) || skip_files.contains(&name.as_str()) {
            continue;
        }
        
        let tar_path = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{}/{}", prefix, name)
        };
        
        if path.is_dir() {
            // Recursively add directory contents (with depth limit)
            if prefix.split('/').count() < 5 { // Max depth of 5
                add_directory_to_tar(tar, &path, &tar_path)?;
            }
        } else if path.is_file() {
            // Check file size - skip files larger than 10MB
            if let Ok(metadata) = path.metadata() {
                if metadata.len() > 10 * 1024 * 1024 {
                    continue;
                }
            }
            
            // Read file and add to tar
            match File::open(&path) {
                Ok(mut file) => {
                    let mut header = Header::new_gnu();
                    let metadata = file.metadata()?;
                    header.set_path(&tar_path)?;
                    header.set_size(metadata.len());
                    header.set_cksum();
                    
                    let mut data = Vec::new();
                    file.read_to_end(&mut data)?;
                    
                    tar.append_data(&mut header, &tar_path, &data[..])?;
                },
                Err(e) => {
                    // Skip files that can't be read (permissions, etc.)
                    warn!("Skipping file {} due to read error: {}", path.display(), e);
                }
            }
        }
    }
    
    Ok(())
}
