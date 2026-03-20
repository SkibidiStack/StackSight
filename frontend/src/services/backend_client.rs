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
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Command {
    VirtEnvCreate { request: CreateEnvironmentRequest },
    VirtEnvDelete { env_id: String },
    VirtEnvActivate { env_id: String },
    VirtEnvDeactivate { env_id: String },
    VirtEnvInstallPackages { operation: PackageOperation },
    VirtEnvList,
    VirtEnvGetTemplates,
    NetworkGetRoutes,
    NetworkAddRoute { destination: String, gateway: String, interface: Option<String>, metric: Option<u32> },
    NetworkDeleteRoute { destination: String },
    NetworkGetFirewallRules,
    NetworkCreateFirewallRule { name: String, action: String, direction: String, protocol: Option<String>, source_ip: Option<String>, source_port: Option<u16>, destination_ip: Option<String>, destination_port: Option<u16> },
    NetworkDeleteFirewallRule { rule_id: String },
    NetworkGetInterfaces,
    NetworkCreateVlan { vlan_id: u16, name: String, parent_interface: String, ip_address: Option<String>, netmask: Option<String> },
    NetworkDeleteVlan { parent_interface: String, vlan_id: u16 },
    RemoteDesktopGetConnections,
    RemoteDesktopGetGroups,
}

pub struct BackendClient {
    websocket_url: String,
}

impl BackendClient {
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
    pub async fn get_routes(&self) -> Result<Vec<Route>> {
        let cmd = Command::NetworkGetRoutes;
        let response = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "network_routes_updated",
            5
        ).await?;
        
        // Extract routes from the event
        if let Some(routes_array) = response.get("routes") {
            let routes: Vec<Route> = serde_json::from_value(routes_array.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse routes: {}", e))?;
            Ok(routes)
        } else {
            Ok(vec![])
        }
    }
    
    pub async fn get_firewall_rules(&self) -> Result<Vec<FirewallRule>> {
        let cmd = Command::NetworkGetFirewallRules;
        let response = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "network_firewall_rules_updated",
            5
        ).await?;
        
        if let Some(rules_array) = response.get("rules") {
            let rules: Vec<FirewallRule> = serde_json::from_value(rules_array.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse firewall rules: {}", e))?;
            Ok(rules)
        } else {
            Ok(vec![])
        }
    }
    
    pub async fn get_network_interfaces(&self) -> Result<Vec<VlanConfig>> {
        let cmd = Command::NetworkGetInterfaces;
        let response = self.send_and_wait_for_event(
            cmd,
            |event_type, _| event_type == "NetworkInterfacesUpdated",
            5
        ).await?;
        
        if let Some(interfaces_array) = response.get("interfaces") {
            // Backend sends NetworkInterface objects, each with a vlans array
            // Extract all VLANs from all interfaces
            let interfaces: Vec<NetworkInterfaceWrapper> = serde_json::from_value(interfaces_array.clone())
                .map_err(|e| anyhow::anyhow!("Failed to parse interfaces: {}", e))?;
            
            let mut all_vlans = Vec::new();
            for iface in interfaces {
                all_vlans.extend(iface.vlans);
            }
            Ok(all_vlans)
        } else {
            Ok(vec![])
        }
    }
}

// Helper struct to parse NetworkInterface from backend
#[derive(Clone, Debug, Serialize, Deserialize)]
struct NetworkInterfaceWrapper {
    vlans: Vec<VlanConfig>,
}

// Data types for network operations
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Route {
    pub destination: String,
    pub gateway: String,
    pub interface: String,
    pub metric: u32,
    pub route_type: RouteType,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum RouteType {
    Static,
    Dynamic,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FirewallRule {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub action: FirewallAction,
    pub direction: TrafficDirection,
    pub protocol: Option<String>,
    pub source_ip: Option<String>,
    pub source_port: Option<u16>,
    pub destination_ip: Option<String>,
    pub destination_port: Option<u16>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum FirewallAction {
    Allow,
    Deny,
    Log,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub enum TrafficDirection {
    Inbound,
    Outbound,
    Both,
}

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