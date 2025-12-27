use crate::core::event_bus::EventBus;
use anyhow::Result;
use sysinfo::System;
use tracing::info;

pub struct SystemService {
    bus: EventBus,
    sys: System,
}

impl SystemService {
    pub async fn new(bus: EventBus) -> Result<Self> {
        Ok(Self { bus, sys: System::new_all() })
    }
}

#[async_trait::async_trait]
impl crate::services::Service for SystemService {
    async fn start(&mut self) -> Result<()> {
        info!("system service start");
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        loop {
            self.sys.refresh_all();
            let cpu = self.sys.global_cpu_info().cpu_usage();
            let mem = self.sys.used_memory();
            let total = self.sys.total_memory();
            let payload = crate::models::system::SystemSnapshot { cpu_usage: cpu, memory_used: mem, memory_total: total };
            self.bus.publish(crate::models::events::Event::SystemSnapshot(payload));
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
}
