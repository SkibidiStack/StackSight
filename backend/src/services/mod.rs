pub mod communication;
pub mod config;
pub mod docker;
pub mod filesystem;
pub mod system;
pub mod virtenv;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Service {
    async fn start(&mut self) -> Result<()>;
    async fn run(self) -> Result<()>;
}

pub enum ServiceHandle {
    Docker(docker::DockerService),
    VirtualEnv(virtenv::VirtualEnvService),
    System(system::SystemService),
    FileSystem(filesystem::FileSystemService),
    Communication(communication::CommunicationService),
}

impl ServiceHandle {
    pub async fn start(&mut self) -> Result<()> {
        match self {
            ServiceHandle::Docker(s) => s.start().await,
            ServiceHandle::VirtualEnv(s) => s.start().await,
            ServiceHandle::System(s) => s.start().await,
            ServiceHandle::FileSystem(s) => s.start().await,
            ServiceHandle::Communication(s) => s.start().await,
        }
    }

    pub async fn run(self) -> Result<()> {
        match self {
            ServiceHandle::Docker(s) => s.run().await,
            ServiceHandle::VirtualEnv(s) => s.run().await,
            ServiceHandle::System(s) => s.run().await,
            ServiceHandle::FileSystem(s) => s.run().await,
            ServiceHandle::Communication(s) => s.run().await,
        }
    }
}
