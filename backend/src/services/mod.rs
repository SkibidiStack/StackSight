pub mod communication;
pub mod config;
pub mod docker;
pub mod filesystem;
pub mod system;
pub mod virtenv;
pub mod network;
pub mod remote_desktop;

use anyhow::Result;

#[async_trait::async_trait]
pub trait Service {
    async fn start(&mut self) -> Result<()>;
    async fn run(self) -> Result<()>;
}

#[allow(dead_code)]
pub enum ServiceHandle {
    Docker(docker::DockerService),
    VirtualEnv(virtenv::VirtualEnvService),
    System(system::SystemService),
    FileSystem(filesystem::FileSystemService),
    Communication(communication::CommunicationService),
    Network(network::NetworkService),
    RemoteDesktop(remote_desktop::RemoteDesktopService),
}

impl ServiceHandle {
    pub async fn start(&mut self) -> Result<()> {
        match self {
            ServiceHandle::Docker(s) => s.start().await,
            ServiceHandle::VirtualEnv(s) => s.start().await,
            ServiceHandle::System(s) => s.start().await,
            ServiceHandle::FileSystem(s) => s.start().await,
            ServiceHandle::Communication(s) => s.start().await,
            ServiceHandle::Network(s) => s.start().await,
            ServiceHandle::RemoteDesktop(s) => s.start().await,
        }
    }

    pub async fn run(self) -> Result<()> {
        match self {
            ServiceHandle::Docker(s) => s.run().await,
            ServiceHandle::VirtualEnv(s) => s.run().await,
            ServiceHandle::System(s) => s.run().await,
            ServiceHandle::FileSystem(s) => s.run().await,
            ServiceHandle::Communication(s) => s.run().await,
            ServiceHandle::Network(s) => s.run().await,
            ServiceHandle::RemoteDesktop(s) => s.run().await,
        }
    }
}
