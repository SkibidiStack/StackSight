use crate::core::event_bus::EventBus;
use crate::models::{commands::Command, events::Event};
use crate::services::{communication::CommunicationService, docker::DockerService, network::NetworkService, system::SystemService, virtenv::VirtualEnvService};
use crate::services::{filesystem::FileSystemService, ServiceHandle};
use crate::{core::config::AppConfig, core::error::CoreError};
use anyhow::Result;
use tokio::task::JoinSet;
use tokio::sync::mpsc;
use tracing::info;

#[allow(dead_code)]
pub struct ServiceManager {
    config: AppConfig,
    bus: EventBus,
    services: Vec<ServiceHandle>,
}

impl ServiceManager {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let bus = EventBus::new(128);
        let mut services = Vec::new();
        let (docker_cmd_tx, docker_cmd_rx) = mpsc::channel::<Command>(128);
        let (virtenv_cmd_tx, virtenv_cmd_rx) = mpsc::channel::<Command>(128);
        let (system_cmd_tx, system_cmd_rx) = mpsc::channel::<Command>(128);
        let (network_cmd_tx, network_cmd_rx) = mpsc::channel::<Command>(128);

        // Main command channel for incoming commands from communication service
        let (main_cmd_tx, mut main_cmd_rx) = mpsc::channel::<Command>(128);

        // Create a command dispatcher task to route commands to appropriate services
        let docker_tx = docker_cmd_tx.clone();
        let virtenv_tx = virtenv_cmd_tx.clone();
        let system_tx = system_cmd_tx.clone();
        let net_tx = network_cmd_tx.clone();
        tokio::spawn(async move {
            while let Some(cmd) = main_cmd_rx.recv().await {
                match &cmd {
                    Command::VirtEnvCreate { .. } |
                    Command::VirtEnvDelete { .. } |
                    Command::VirtEnvActivate { .. } |
                    Command::VirtEnvDeactivate { .. } |
                    Command::VirtEnvInstallPackages { .. } |
                    Command::VirtEnvList |
                    Command::VirtEnvGetTemplates => {
                        let _ = virtenv_tx.send(cmd).await;
                    }
                    Command::SystemGetProcessList |
                    Command::SystemKillProcess { .. } => {
                        let _ = system_tx.send(cmd).await;
                    }
                    Command::NetworkScanDevices => {
                        let _ = net_tx.send(cmd).await;
                    }
                    _ => {
                        // Docker and other commands
                        let _ = docker_tx.send(cmd).await;
                    }
                }
            }
        });

        services.push(ServiceHandle::Docker(DockerService::new(bus.clone(), docker_cmd_rx).await?));
        services.push(ServiceHandle::VirtualEnv(VirtualEnvService::new(bus.clone(), virtenv_cmd_rx).await?));
        services.push(ServiceHandle::System(SystemService::new(bus.clone(), system_cmd_rx).await?));
        services.push(ServiceHandle::FileSystem(FileSystemService::new(bus.clone()).await?));
        services.push(ServiceHandle::Network(NetworkService::new(bus.clone(), network_cmd_rx).await?));
        services.push(ServiceHandle::Communication(CommunicationService::new(bus.clone(), main_cmd_tx).await?));

        Ok(Self { config, bus, services })
    }

    pub async fn start(&mut self) -> Result<()> {
        info!("Starting services");
        for svc in &mut self.services {
            svc.start().await?;
        }
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let mut tasks = JoinSet::new();
        for svc in self.services.drain(..) {
            tasks.spawn(async move { svc.run().await });
        }

        while let Some(res) = tasks.join_next().await {
            match res {
                Ok(Ok(())) => {}
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(CoreError::Service(e.to_string()).into()),
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn publish(&self, event: Event) {
        self.bus.publish(event);
    }
}
