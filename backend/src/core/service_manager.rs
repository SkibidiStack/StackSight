use crate::core::event_bus::EventBus;
use crate::models::{commands::Command, events::Event};
use crate::services::{communication::CommunicationService, docker::DockerService, system::SystemService, virtenv::VirtualEnvService};
use crate::services::{filesystem::FileSystemService, ServiceHandle};
use crate::{core::config::AppConfig, core::error::CoreError};
use anyhow::Result;
use tokio::task::JoinSet;
use tokio::sync::mpsc;
use tracing::info;

pub struct ServiceManager {
    config: AppConfig,
    bus: EventBus,
    services: Vec<ServiceHandle>,
}

impl ServiceManager {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let bus = EventBus::new(128);
        let mut services = Vec::new();
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>(128);

        services.push(ServiceHandle::Docker(DockerService::new(bus.clone(), cmd_rx).await?));
        services.push(ServiceHandle::VirtualEnv(VirtualEnvService::new(bus.clone()).await?));
        services.push(ServiceHandle::System(SystemService::new(bus.clone()).await?));
        services.push(ServiceHandle::FileSystem(FileSystemService::new(bus.clone()).await?));
        services.push(ServiceHandle::Communication(CommunicationService::new(bus.clone(), cmd_tx).await?));

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

    pub fn publish(&self, event: Event) {
        self.bus.publish(event);
    }
}
