use crate::core::event_bus::EventBus;
use anyhow::Result;
use tracing::info;

pub struct FileSystemService {
    bus: EventBus,
}

impl FileSystemService {
    pub async fn new(bus: EventBus) -> Result<Self> {
        Ok(Self { bus })
    }
}

#[async_trait::async_trait]
impl crate::services::Service for FileSystemService {
    async fn start(&mut self) -> Result<()> {
        info!("filesystem service start");
        Ok(())
    }

    async fn run(self) -> Result<()> {
        let _ = self.bus;
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        Ok(())
    }
}
