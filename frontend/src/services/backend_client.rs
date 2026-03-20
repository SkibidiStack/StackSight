use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;

// Re-export common types needed for communication
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    Python,
    Node,
    Rust,
    Go,
    #[serde(rename = "dotnet")]
    DotNet,
    Java,
    Ruby,
    Php,
    Other(String),
}

impl Language {
    pub fn as_str(&self) -> &str {
        match self {
            Language::Python => "python",
            Language::Node => "node", 
            Language::Rust => "rust",
            Language::Go => "go",
            Language::DotNet => "dotnet",
            Language::Java => "java",
            Language::Ruby => "ruby",
            Language::Php => "php",
            Language::Other(name) => name,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateEnvironmentRequest {
    pub name: String,
    pub language: Language,
    pub version: Option<String>,
    pub template: Option<String>,
    pub project_path: Option<String>,
    pub packages: Vec<String>,
    pub location: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageOperation {
    pub env_id: String,
    pub operation: PackageOperationType,
    pub packages: Vec<String>,
    pub options: HashMap<String, String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PackageOperationType {
    Install,
    Uninstall,
    Update,
    Upgrade,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateVlanRequest {
    pub vlan_id: u16,
    pub name: String,
    pub parent_interface: String,
    pub ip_address: Option<String>,
    pub netmask: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CreateBridgeRequest {
    pub name: String,
    pub interfaces: Vec<String>,
    pub ip_config: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    VirtEnvCreate { request: CreateEnvironmentRequest },
    VirtEnvDelete { env_id: String },
    VirtEnvActivate { env_id: String },
    VirtEnvDeactivate { env_id: String },
    VirtEnvInstallPackages { operation: PackageOperation },
    VirtEnvList,
    VirtEnvGetTemplates,
    SystemGetProcessList,
    SystemKillProcess { pid: String },
    NetworkScanDevices,
    NetworkCreateVlan { request: CreateVlanRequest },
    NetworkDeleteVlan { parent_interface: String, vlan_id: u16 },
    NetworkGetVlans,
    NetworkGetInterfaces,
    NetworkCreateBridge { request: CreateBridgeRequest },
    NetworkDeleteBridge { name: String },
    RemoteDesktopGetConnections,
    RemoteDesktopGetGroups,
    RemoteDesktopConnect { connection_id: String },
    RemoteDesktopDisconnect { connection_id: String },
}

pub struct BackendClient {
    websocket_url: String,
}

impl BackendClient {
    pub async fn create_bridge(&self, bridge: &crate::components::network::interface_list::BridgeConfig) -> Result<()> {
        let cmd = Command::NetworkCreateBridge {
            request: CreateBridgeRequest {
                name: bridge.name.clone(),
                interfaces: bridge.interfaces.clone(),
                ip_config: bridge.ip_config.clone(),
            }
        };
        let _ = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "NetworkInterfacesUpdated" || event_type == "network_interfaces_updated",
            5
        ).await;
        Ok(())
    }

    pub async fn get_network_interfaces(&self) -> Result<Vec<crate::components::network::interface_list::NetworkInterface>> {
        let raw = self.get_all_interfaces_raw().await?;
        let parsed = raw.into_iter()
            .filter_map(|val| serde_json::from_value(val).ok())
            .collect();
        Ok(parsed)
    }

    pub fn new() -> Self {
        Self {
            websocket_url: "ws://127.0.0.1:8765".to_string(),
        }
    }

    /// Send a raw JSON value over WebSocket (used for commands not in the typed enum).
    pub async fn send_ws_command(&self, payload: &serde_json::Value) -> Result<()> {
        let (ws_stream, _) = connect_async(&self.websocket_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to backend: {}", e))?;
        let (mut write, _) = ws_stream.split();
        let json = serde_json::to_string(payload)
            .map_err(|e| anyhow::anyhow!("Failed to serialize: {}", e))?;
        write.send(Message::Text(json)).await
            .map_err(|e| anyhow::anyhow!("Failed to send: {}", e))?;
        let _ = write.close().await;
        Ok(())
    }

    pub async fn send_command(&self, command: Command) -> Result<()> {
        tracing::info!("Sending command to backend: {:?}", command);
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.websocket_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to backend: {}", e))?;
        
        let (mut write, _read) = ws_stream.split();
        
        // Serialize command to JSON
        let command_json = serde_json::to_string(&command)
            .map_err(|e| anyhow::anyhow!("Failed to serialize command: {}", e))?;
        
        // Send command
        write.send(Message::Text(command_json)).await
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))?;
        
        // Close connection
        write.close().await
            .map_err(|e| anyhow::anyhow!("Failed to close connection: {}", e))?;
        
        tracing::info!("Command sent successfully");
        Ok(())
    }
    
    /// Send a command and wait for a specific event response
    pub async fn send_and_wait_for_event<F>(&self, command: Command, predicate: F, timeout_secs: u64) -> Result<serde_json::Value>
    where
        F: Fn(&str, &serde_json::Value) -> bool,
    {
        use futures_util::{SinkExt, StreamExt};
        
        tracing::info!("Sending command and waiting for event response");
        
        // Connect to WebSocket
        let (ws_stream, _) = connect_async(&self.websocket_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to backend: {}", e))?;
        
        let (mut write, mut read) = ws_stream.split();
        
        // Serialize and send command
        let command_json = serde_json::to_string(&command)
            .map_err(|e| anyhow::anyhow!("Failed to serialize command: {}", e))?;
        
        write.send(Message::Text(command_json)).await
            .map_err(|e| anyhow::anyhow!("Failed to send command: {}", e))?;
        
        // Wait for the response event with timeout
        let timeout = tokio::time::Duration::from_secs(timeout_secs);
        match tokio::time::timeout(timeout, async {
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        tracing::debug!("Received WebSocket message: {}", text);
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&text) {
                            // Check if this is a VirtualEnvCreated, VirtualEnvList, or other event
                            if let Some(event_type_val) = value.as_object().and_then(|obj| {
                                // Try both direct event type and nested in "type" field
                                obj.keys().next()
                            }) {
                                tracing::debug!("Event type: {}", event_type_val);
                                if predicate(event_type_val, &value) {
                                    return Ok(value);
                                }
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        return Err(anyhow::anyhow!("WebSocket connection closed"));
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("WebSocket error: {}", e));
                    }
                    _ => {}
                }
            }
            Err(anyhow::anyhow!("WebSocket closed without receiving expected event"))
        }).await {
            Ok(result) => result,
            Err(_) => Err(anyhow::anyhow!("Timeout waiting for event after {} seconds", timeout_secs)),
        }
    }
    
    pub async fn create_environment(&self, request: CreateEnvironmentRequest) -> Result<()> {
        tracing::info!("Creating environment: {} ({:?})", request.name, request.language);
        
        // For now, create actual virtual environment locally as a proof of concept
        // In a real implementation, this would send over WebSocket to backend
        match request.language {
            Language::Python => {
                self.create_python_environment(&request).await?;
            }
            _ => {
                tracing::warn!("Language {:?} not implemented yet", request.language);
            }
        }
        
        self.send_command(Command::VirtEnvCreate { request }).await
    }
    
    async fn create_python_environment(&self, request: &CreateEnvironmentRequest) -> Result<()> {
        use tokio::process::Command;
        
        // Determine the environment path
        let env_path = if let Some(location) = &request.location {
            std::path::PathBuf::from(location).join(&request.name)
        } else {
            // Default to user's home directory
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(home).join(".virtualenvs").join(&request.name)
        };
        
        tracing::info!("Creating Python virtual environment at: {:?}", env_path);
        
        // Create the directory
        tokio::fs::create_dir_all(&env_path).await?;
        
        // Determine Python command
        let python_cmd = if let Some(version) = &request.version {
            if version == "default" || version.is_empty() {
                "python3".to_string()
            } else {
                format!("python{}", version)
            }
        } else {
            "python3".to_string()
        };
        
        // Create virtual environment
        let output = Command::new(&python_cmd)
            .args(&["-m", "venv", env_path.to_str().unwrap()])
            .output()
            .await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Failed to create virtual environment: {}", error));
        }
        
        tracing::info!("Successfully created virtual environment at {:?}", env_path);
        
        // Install packages if specified
        if !request.packages.is_empty() {
            self.install_packages_in_env(&env_path, &request.packages).await?;
        }
        
        Ok(())
    }
    
    async fn install_packages_in_env(&self, env_path: &std::path::Path, packages: &[String]) -> Result<()> {
        use tokio::process::Command;
        
        if packages.is_empty() {
            return Ok(());
        }
        
        tracing::info!("Installing packages: {:?} in {:?}", packages, env_path);
        
        // Determine pip path
        let pip_path = if cfg!(windows) {
            env_path.join("Scripts").join("pip.exe")
        } else {
            env_path.join("bin").join("pip")
        };
        
        // Install packages
        let mut cmd = Command::new(&pip_path);
        cmd.arg("install");
        for package in packages {
            cmd.arg(package);
        }
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to install some packages: {}", error);
        } else {
            tracing::info!("Successfully installed packages");
        }
        
        Ok(())
    }
    
    pub async fn install_package(&self, env_id: String, package_name: String) -> Result<()> {
        tracing::info!("Installing single package '{}' in environment '{}'", package_name, env_id);
        
        // Send the command to the backend with the env_id
        // The backend will look up the environment from its loaded data (including the correct path from JSON)
        let operation = PackageOperation {
            env_id,
            operation: PackageOperationType::Install,
            packages: vec![package_name],
            options: HashMap::new(),
        };
        
        self.send_command(Command::VirtEnvInstallPackages { operation }).await
    }
    
    pub async fn delete_environment(&self, env_id: String) -> Result<()> {
        self.send_command(Command::VirtEnvDelete { env_id }).await
    }
    
    pub async fn activate_environment(&self, env_id: String) -> Result<()> {
        self.send_command(Command::VirtEnvActivate { env_id }).await
    }
    
    pub async fn deactivate_environment(&self, env_id: String) -> Result<()> {
        self.send_command(Command::VirtEnvDeactivate { env_id }).await
    }
    
    pub async fn list_environments(&self) -> Result<()> {
        self.send_command(Command::VirtEnvList).await
    }
    
    pub async fn get_templates(&self) -> Result<()> {
        self.send_command(Command::VirtEnvGetTemplates).await
    }
    
    // Network operations
    pub async fn get_vlans(&self) -> Result<Vec<VlanConfig>> {
        let cmd = Command::NetworkGetVlans;
        let response = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "NetworkVlansUpdated" || event_type == "network_vlans_updated",
            5
        ).await?;
        
        let vlans_array = response.get("NetworkVlansUpdated").and_then(|o| o.get("vlans"))
            .or_else(|| response.get("vlans"));
        
        if let Some(vlans_array) = vlans_array {
            tracing::info!("Received vlans array");
            let vlans: Vec<VlanConfig> = match serde_json::from_value(vlans_array.clone()) {
                Ok(v) => v,
                Err(e) => {
                    tracing::error!("Failed to parse vlans: {}", e);
                    return Err(anyhow::anyhow!("Failed to parse vlans: {}", e));
                }
            };
            
            tracing::info!("Extracted VLANs: {:?}", vlans);
            Ok(vlans)
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_all_interfaces_raw(&self) -> Result<Vec<serde_json::Value>> {
        let cmd = Command::NetworkGetInterfaces;
        let response = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "NetworkInterfacesUpdated" || event_type == "network_interfaces_updated",
            5
        ).await?;
        
        let interfaces_array = response.get("NetworkInterfacesUpdated").and_then(|o| o.get("interfaces"))
            .or_else(|| response.get("interfaces"));
            
        if let Some(interfaces_array) = interfaces_array {
            if let Some(arr) = interfaces_array.as_array() {
                return Ok(arr.clone());
            }
        }
        Ok(vec![])
    }
}

// Helper struct to parse NetworkInterface from backend
#[derive(Clone, Debug, Serialize, Deserialize)]
struct NetworkInterfaceWrapper {
    vlans: Vec<VlanConfig>,
}

// Data types for network operations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct VlanConfig {
    pub id: u16,
    pub name: String,
    pub parent_interface: String,
    pub ip_config: Option<String>,
    pub enabled: bool,
}

impl Default for BackendClient {
    fn default() -> Self {
        Self::new()
    }
}