use anyhow::{anyhow, Result};
use tokio::process::Command;
use std::path::Path;

pub async fn run(cmd: &str, args: &[&str]) -> Result<String> {
    let output = Command::new(cmd).args(args).output().await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(anyhow!(
            "Command '{}' failed with exit code {:?}\nStdout: {}\nStderr: {}",
            cmd,
            output.status.code(),
            stdout,
            stderr
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}

pub async fn run_in_dir(cmd: &str, args: &[&str], working_dir: &Path) -> Result<String> {
    let output = Command::new(cmd)
        .args(args)
        .current_dir(working_dir)
        .output()
        .await?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(anyhow!(
            "Command '{}' failed in directory {:?} with exit code {:?}\nStdout: {}\nStderr: {}",
            cmd,
            working_dir,
            output.status.code(),
            stdout,
            stderr
        ));
    }
    
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    Ok(stdout)
}
