use crate::core::event_bus::EventBus;
use crate::models::{commands::Command, docker::ContainerSummary, events::Event};
use anyhow::Result;
use bollard::container::{ListContainersOptions, RestartContainerOptions, StartContainerOptions, StopContainerOptions};
use bollard::Docker;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{info, warn};

pub struct DockerService {
    bus: EventBus,
    client: Docker,
    poll_interval: Duration,
    cmd_rx: mpsc::Receiver<Command>,
}

impl DockerService {
    pub async fn new(bus: EventBus, cmd_rx: mpsc::Receiver<Command>) -> Result<Self> {
        let client = Docker::connect_with_local_defaults()?;
        Ok(Self { bus, client, poll_interval: Duration::from_secs(5), cmd_rx })
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

    async fn handle_command(client: &Docker, bus: &EventBus, cmd: Command) {
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
        }

        match Self::snapshot_containers(client).await {
            Ok(containers) => bus.publish(Event::DockerContainers(containers)),
            Err(err) => warn!(error = ?err, "docker refresh failed"),
        }
    }
}

#[async_trait::async_trait]
impl crate::services::Service for DockerService {
    async fn start(&mut self) -> Result<()> {
        info!("docker service start");
        let _ = self.client.ping().await?;
        Ok(())
    }

    async fn run(self) -> Result<()> {
        let mut cmd_rx = self.cmd_rx;
        let bus = self.bus.clone();
        let client = self.client.clone();
        let poll_interval = self.poll_interval;

        loop {
            tokio::select! {
                biased;
                maybe_cmd = cmd_rx.recv() => {
                    if let Some(cmd) = maybe_cmd {
                        Self::handle_command(&client, &bus, cmd).await;
                    } else {
                        break;
                    }
                }
                _ = tokio::time::sleep(poll_interval) => {
                    match Self::snapshot_containers(&client).await {
                        Ok(containers) => bus.publish(Event::DockerContainers(containers)),
                        Err(err) => warn!(error = ?err, "docker snapshot failed"),
                    }
                }
            }
        }
        Ok(())
    }
}
