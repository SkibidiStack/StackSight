use anyhow::Result;
use tokio::process::Command;

pub async fn run(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd).args(args).output().await?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
