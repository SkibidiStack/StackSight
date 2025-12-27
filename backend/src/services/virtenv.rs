use crate::core::event_bus::EventBus;
use anyhow::Result;
use tracing::info;

pub struct VirtualEnvService {
    bus: EventBus,
}

impl VirtualEnvService {
    pub async fn new(bus: EventBus) -> Result<Self> {
        Ok(Self { bus })
    }
}

#[async_trait::async_trait]
impl crate::services::Service for VirtualEnvService {
    async fn start(&mut self) -> Result<()> {
        info!("virtenv service start");
        Ok(())
    }

    async fn run(self) -> Result<()> {
        let _ = self.bus;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }
}
