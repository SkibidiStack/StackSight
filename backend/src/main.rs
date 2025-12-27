mod core;
mod models;
mod platform;
mod services;
mod utils;

use anyhow::Result;
use clap::Parser;
use core::config::AppConfig;
use core::logging::init_tracing;
use core::service_manager::ServiceManager;

#[derive(Parser, Debug)]
#[command(name = "devenv-backend", version, about = "DevEnv Manager backend daemon")]
struct Cli {
    /// Path to config file
    #[arg(long, env = "DEVENV_CONFIG", default_value = "~/.config/devenv/config.toml")]
    config: String,

    /// Enable verbose logging
    #[arg(long, short, action = clap::ArgAction::Count)]
    verbose: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    init_tracing(cli.verbose);

    let config = AppConfig::load(cli.config.as_str()).await?;
    let mut manager = ServiceManager::new(config).await?;

    manager.start().await?;
    manager.run().await?;

    Ok(())
}
