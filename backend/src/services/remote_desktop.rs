use anyhow::{anyhow, Context, Result};
use crate::core::event_bus::EventBus;
use crate::models::commands::Command;
use crate::models::events::Event;
use crate::models::remote_desktop::*;
use std::collections::HashMap;
use std::process::{Child, Stdio};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::fs;
use tracing::{debug, info, warn};

#[allow(dead_code)]
#[derive(Clone)]
pub struct RemoteDesktopService {
    bus: EventBus,
    connections: Arc<RwLock<HashMap<String, RemoteConnection>>>,
    active_sessions: Arc<RwLock<HashMap<String, ActiveSession>>>,
    session_processes: Arc<RwLock<HashMap<String, SessionProcess>>>,
    groups: Arc<RwLock<HashMap<String, ConnectionGroup>>>,
    command_rx: Arc<tokio::sync::Mutex<mpsc::Receiver<Command>>>,
}

struct SessionProcess {
    #[allow(dead_code)]
    process: Option<Child>,
    #[allow(dead_code)]
    protocol: ConnectionProtocol,
}

#[allow(dead_code)]
impl RemoteDesktopService {
    pub fn new(bus: EventBus, command_rx: mpsc::Receiver<Command>) -> Self {
        Self {
            bus,
            connections: Arc::new(RwLock::new(HashMap::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            session_processes: Arc::new(RwLock::new(HashMap::new())),
            groups: Arc::new(RwLock::new(HashMap::new())),
            command_rx: Arc::new(tokio::sync::Mutex::new(command_rx)),
        }
    }

    /// Initialize the service
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing remote desktop service");
        
        // Check for required clients
        self.check_client_availability().await?;
        
        // Load saved connections and groups from config
        self.load_connections().await?;
        self.load_groups().await?;
        
        Ok(())
    }

    /// Check if required remote desktop clients are installed
    async fn check_client_availability(&self) -> Result<()> {
        let mut available = Vec::new();
        let mut missing = Vec::new();

        // Check SSH
        if std::process::Command::new("ssh").arg("-V").output().is_ok() {
            available.push("SSH");
        } else {
            missing.push("SSH (openssh-client)");
        }

        // Check RDP clients
        #[cfg(target_os = "linux")]
        {
            if std::process::Command::new("xfreerdp").arg("--version").output().is_ok() {
                available.push("RDP (xfreerdp)");
            } else if std::process::Command::new("rdesktop").arg("-h").output().is_ok() {
                available.push("RDP (rdesktop)");
            } else {
                missing.push("RDP (xfreerdp or rdesktop)");
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Windows has built-in RDP client (mstsc.exe)
            available.push("RDP (built-in)");
        }

        #[cfg(target_os = "macos")]
        {
            // Check for Microsoft Remote Desktop
            available.push("RDP (check Microsoft Remote Desktop app)");
        }

        // Check VNC
        if std::process::Command::new("vncviewer").arg("-h").output().is_ok() {
            available.push("VNC");
        } else {
            missing.push("VNC (vncviewer)");
        }

        info!("Available remote desktop clients: {:?}", available);
        if !missing.is_empty() {
            warn!("Missing remote desktop clients: {:?}", missing);
        }

        Ok(())
    }

    /// Load saved connections from configuration
    async fn load_connections(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let file_path = config_dir.join("connections.json");
        
        if !file_path.exists() {
            info!("No saved connections file found");
            return Ok(());
        }
        
        let json = fs::read_to_string(&file_path).await?;
        let saved_connections: Vec<RemoteConnection> = serde_json::from_str(&json)?;
        
        let mut connections = self.connections.write().await;
        for conn in saved_connections {
            connections.insert(conn.id.clone(), conn);
        }
        
        info!("Loaded {} saved connections", connections.len());
        Ok(())
    }

    /// Save connections to configuration
    async fn save_connections(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir).await?;
        
        let file_path = config_dir.join("connections.json");
        let connections = self.connections.read().await;
        let connections_vec: Vec<&RemoteConnection> = connections.values().collect();
        let json = serde_json::to_string_pretty(&connections_vec)?;
        
        fs::write(&file_path, json).await?;
        info!("Saved {} connections to file", connections.len());
        Ok(())
    }

    /// Get config directory for remote desktop data
    fn get_config_dir() -> Result<std::path::PathBuf> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| anyhow!("Could not determine config directory"))?
            .join("manager")
            .join("remote_desktop");
        Ok(config_dir)
    }

    /// Load saved groups from configuration
    async fn load_groups(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        let file_path = config_dir.join("groups.json");
        
        if !file_path.exists() {
            info!("No saved groups file found");
            return Ok(());
        }
        
        let json = fs::read_to_string(&file_path).await?;
        let saved_groups: Vec<ConnectionGroup> = serde_json::from_str(&json)?;
        
        let mut groups = self.groups.write().await;
        for group in saved_groups {
            groups.insert(group.id.clone(), group);
        }
        
        info!("Loaded {} saved groups", groups.len());
        Ok(())
    }

    /// Save groups to configuration
    async fn save_groups(&self) -> Result<()> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir).await?;
        
        let file_path = config_dir.join("groups.json");
        let groups = self.groups.read().await;
        let groups_vec: Vec<&ConnectionGroup> = groups.values().collect();
        let json = serde_json::to_string_pretty(&groups_vec)?;
        
        fs::write(&file_path, json).await?;
        info!("Saved {} groups to file", groups.len());
        Ok(())
    }

    /// Get all saved connections
    pub async fn get_connections(&self) -> Result<Vec<RemoteConnection>> {
        let connections = self.connections.read().await;
        Ok(connections.values().cloned().collect())
    }

    /// Get a specific connection
    pub async fn get_connection(&self, id: &str) -> Result<RemoteConnection> {
        let connections = self.connections.read().await;
        connections
            .get(id)
            .cloned()
            .ok_or_else(|| anyhow!("Connection '{}' not found", id))
    }

    /// Create a new connection
    pub async fn create_connection(&self, request: CreateConnectionRequest) -> Result<RemoteConnection> {
        info!("Creating connection: {}", request.name);

        let connection_id = uuid::Uuid::new_v4().to_string();
        
        // Default port based on protocol
        let port = request.port.unwrap_or_else(|| match request.protocol {
            ConnectionProtocol::Ssh => 22,
            ConnectionProtocol::Rdp => 3389,
            ConnectionProtocol::Vnc => 5900,
            ConnectionProtocol::Spice => 5900,
        });

        // Build credentials
        let auth_method = if let Some(key) = request.private_key {
            AuthMethod::PrivateKey {
                key_path: key,
                passphrase: None,
            }
        } else if let Some(pwd) = request.password {
            AuthMethod::Password { password: pwd }
        } else {
            AuthMethod::None
        };

        let credentials = Credentials {
            username: request.username,
            auth_method,
            save_credentials: true,
        };

        // Default settings based on protocol
        let settings = request.settings.unwrap_or_else(|| {
            ConnectionSettings {
                ssh_settings: if request.protocol == ConnectionProtocol::Ssh {
                    Some(SshSettings {
                        terminal_type: "xterm-256color".to_string(),
                        compression: true,
                        forward_x11: false,
                        keep_alive_interval: 30,
                        port_forwards: Vec::new(),
                        environment_variables: HashMap::new(),
                        command: None,
                    })
                } else {
                    None
                },
                rdp_settings: if request.protocol == ConnectionProtocol::Rdp {
                    Some(RdpSettings {
                        domain: None,
                        security: RdpSecurity::Any,
                        console_session: false,
                        enable_clipboard: true,
                        enable_audio: true,
                        enable_printer: false,
                        enable_drive_redirection: false,
                        shared_folders: Vec::new(),
                        gateway: None,
                    })
                } else {
                    None
                },
                vnc_settings: if request.protocol == ConnectionProtocol::Vnc {
                    Some(VncSettings {
                        password: None,
                        view_only: false,
                        quality: VncQuality::Medium,
                        encoding: VncEncoding::Tight,
                        cursor_mode: CursorMode::Local,
                    })
                } else {
                    None
                },
                display_settings: DisplaySettings {
                    resolution: None,
                    color_depth: ColorDepth::True,
                    fullscreen: false,
                    scaling: ScalingMode::Fit,
                },
                tunnel_settings: None,
            }
        });

        let connection = RemoteConnection {
            id: connection_id.clone(),
            name: request.name,
            protocol: request.protocol,
            host: request.host,
            port,
            credentials,
            settings,
            status: ConnectionStatus::Disconnected,
            last_connected: None,
            favorite: false,
            tags: Vec::new(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection.clone());
        
        self.save_connections().await?;

        Ok(connection)
    }

    /// Update an existing connection
    pub async fn update_connection(&self, id: &str, request: UpdateConnectionRequest) -> Result<RemoteConnection> {
        info!("Updating connection: {}", id);

        let mut connections = self.connections.write().await;
        let connection = connections
            .get_mut(id)
            .ok_or_else(|| anyhow!("Connection not found"))?;

        if let Some(name) = request.name {
            connection.name = name;
        }
        if let Some(host) = request.host {
            connection.host = host;
        }
        if let Some(port) = request.port {
            connection.port = port;
        }
        if let Some(creds) = request.credentials {
            connection.credentials = creds;
        }
        if let Some(settings) = request.settings {
            connection.settings = settings;
        }
        if let Some(fav) = request.favorite {
            connection.favorite = fav;
        }
        if let Some(tags) = request.tags {
            connection.tags = tags;
        }

        let updated = connection.clone();
        drop(connections);
        
        self.save_connections().await?;

        Ok(updated)
    }

    /// Delete a connection
    pub async fn delete_connection(&self, id: &str) -> Result<()> {
        info!("Deleting connection: {}", id);

        // Disconnect if active
        if self.is_connected(id).await {
            self.disconnect(id).await?;
        }

        let mut connections = self.connections.write().await;
        connections.remove(id)
            .ok_or_else(|| anyhow!("Connection not found"))?;
        
        self.save_connections().await?;

        Ok(())
    }

    /// Connect to a remote system
    pub async fn connect(&self, request: ConnectRequest) -> Result<ActiveSession> {
        let connection = self.get_connection(&request.connection_id).await?;
        
        info!("Connecting to {} ({}://{}:{})", 
            connection.name, 
            match connection.protocol {
                ConnectionProtocol::Ssh => "ssh",
                ConnectionProtocol::Rdp => "rdp",
                ConnectionProtocol::Vnc => "vnc",
                ConnectionProtocol::Spice => "spice",
            },
            connection.host, 
            connection.port
        );

        // Use override credentials if provided
        let credentials = request.override_credentials.unwrap_or(connection.credentials.clone());

        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Launch the appropriate client
        let process = match connection.protocol {
            ConnectionProtocol::Ssh => {
                self.launch_ssh_client(&connection, &credentials).await?
            }
            ConnectionProtocol::Rdp => {
                self.launch_rdp_client(&connection, &credentials).await?
            }
            ConnectionProtocol::Vnc => {
                self.launch_vnc_client(&connection, &credentials).await?
            }
            ConnectionProtocol::Spice => {
                return Err(anyhow!("SPICE protocol not yet implemented"));
            }
        };

        // Store the process
        let mut processes = self.session_processes.write().await;
        processes.insert(session_id.clone(), SessionProcess {
            process: Some(process),
            protocol: connection.protocol.clone(),
        });

        // Create session record
        let session = ActiveSession {
            connection_id: request.connection_id.clone(),
            session_id: session_id.clone(),
            started_at: chrono::Utc::now().to_rfc3339(),
            bytes_sent: 0,
            bytes_received: 0,
            latency: None,
        };

        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id.clone(), session.clone());

        // Update connection status
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(&request.connection_id) {
            conn.status = ConnectionStatus::Connected;
            conn.last_connected = Some(chrono::Utc::now().to_rfc3339());
        }

        Ok(session)
    }

    /// Launch SSH client
    async fn launch_ssh_client(&self, connection: &RemoteConnection, credentials: &Credentials) -> Result<Child> {
        let mut args = Vec::new();

        // Add user and host
        args.push(format!("{}@{}", credentials.username, connection.host));

        // Add port
        args.push("-p".to_string());
        args.push(connection.port.to_string());

        // Add authentication
        match &credentials.auth_method {
            AuthMethod::PrivateKey { key_path, passphrase: _ } => {
                args.push("-i".to_string());
                args.push(key_path.clone());
            }
            AuthMethod::Password { password: _ } => {
                // Note: SSH doesn't support password on command line for security reasons
                // In production, use SSH agent or expect script
                warn!("SSH password authentication requires interactive input");
            }
            _ => {}
        }

        // Add SSH-specific settings
        if let Some(ssh_settings) = &connection.settings.ssh_settings {
            if ssh_settings.compression {
                args.push("-C".to_string());
            }
            if ssh_settings.forward_x11 {
                args.push("-X".to_string());
            }

            // Add port forwards
            for forward in &ssh_settings.port_forwards {
                match forward.forward_type {
                    ForwardType::Local => {
                        args.push("-L".to_string());
                        args.push(format!("{}:{}:{}", 
                            forward.local_port, 
                            forward.remote_host, 
                            forward.remote_port
                        ));
                    }
                    ForwardType::Remote => {
                        args.push("-R".to_string());
                        args.push(format!("{}:{}:{}", 
                            forward.remote_port, 
                            forward.remote_host, 
                            forward.local_port
                        ));
                    }
                    ForwardType::Dynamic => {
                        args.push("-D".to_string());
                        args.push(forward.local_port.to_string());
                    }
                }
            }

            // Execute command if specified
            if let Some(cmd) = &ssh_settings.command {
                args.push(cmd.clone());
            }
        }

        debug!("Launching SSH: ssh {}", args.join(" "));

        let child = std::process::Command::new("ssh")
            .args(&args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .context("Failed to launch SSH client")?;

        Ok(child)
    }

    /// Launch RDP client
    async fn launch_rdp_client(&self, connection: &RemoteConnection, credentials: &Credentials) -> Result<Child> {
        #[cfg(target_os = "linux")]
        {
            // Use xfreerdp (FreeRDP)
            let mut args = vec![
                format!("/v:{}:{}", connection.host, connection.port),
                format!("/u:{}", credentials.username),
            ];

            if let AuthMethod::Password { password } = &credentials.auth_method {
                args.push(format!("/p:{}", password));
            }

            if let Some(rdp_settings) = &connection.settings.rdp_settings {
                if let Some(domain) = &rdp_settings.domain {
                    args.push(format!("/d:{}", domain));
                }

                if rdp_settings.enable_clipboard {
                    args.push("+clipboard".to_string());
                }

                if rdp_settings.enable_audio {
                    args.push("/sound:sys:pulse".to_string());
                }

                // Add shared folders
                for folder in &rdp_settings.shared_folders {
                    args.push(format!("/drive:{},{}", folder.remote_name, folder.local_path));
                }
            }

            // Display settings
            let display = &connection.settings.display_settings;
            if display.fullscreen {
                args.push("/f".to_string());
            } else if let Some(res) = &display.resolution {
                args.push(format!("/w:{}", res.width));
                args.push(format!("/h:{}", res.height));
            }

            debug!("Launching RDP: xfreerdp {}", args.join(" "));

            let child = std::process::Command::new("xfreerdp")
                .args(&args)
                .spawn()
                .context("Failed to launch RDP client")?;

            Ok(child)
        }

        #[cfg(target_os = "windows")]
        {
            // Use built-in mstsc.exe
            // Create temporary .rdp file
            let rdp_content = format!(
                "full address:s:{}:{}\nusername:s:{}\n",
                connection.host, connection.port, credentials.username
            );

            let temp_path = std::env::temp_dir().join(format!("stacksight_{}.rdp", uuid::Uuid::new_v4()));
            std::fs::write(&temp_path, rdp_content)?;

            let child = std::process::Command::new("mstsc")
                .arg(temp_path.to_str().unwrap())
                .spawn()
                .context("Failed to launch RDP client")?;

            Ok(child)
        }

        #[cfg(target_os = "macos")]
        {
            // Use Microsoft Remote Desktop if installed
            // Or create .rdp file and open it
            Err(anyhow!("RDP client launch not fully implemented for macOS"))
        }
    }

    /// Launch VNC client
    async fn launch_vnc_client(&self, connection: &RemoteConnection, _credentials: &Credentials) -> Result<Child> {
        let mut args = Vec::new();

        // VNC connection string
        let vnc_url = format!("{}:{}", connection.host, connection.port);
        args.push(vnc_url);

        if let Some(vnc_settings) = &connection.settings.vnc_settings {
            if vnc_settings.view_only {
                args.push("-ViewOnly".to_string());
            }

            // Quality settings
            match vnc_settings.quality {
                VncQuality::Low => args.push("-Quality=0".to_string()),
                VncQuality::Medium => args.push("-Quality=5".to_string()),
                VncQuality::High => args.push("-Quality=8".to_string()),
                VncQuality::Lossless => args.push("-Quality=9".to_string()),
            }
        }

        debug!("Launching VNC: vncviewer {}", args.join(" "));

        let child = std::process::Command::new("vncviewer")
            .args(&args)
            .spawn()
            .context("Failed to launch VNC client")?;

        Ok(child)
    }

    /// Check if a connection is active
    pub async fn is_connected(&self, connection_id: &str) -> bool {
        let sessions = self.active_sessions.read().await;
        sessions.values().any(|s| s.connection_id == connection_id)
    }

    /// Disconnect from a remote system
    pub async fn disconnect(&self, connection_id: &str) -> Result<()> {
        info!("Disconnecting from connection: {}", connection_id);

        // Find and remove active session
        let mut sessions = self.active_sessions.write().await;
        let session_id = sessions
            .iter()
            .find(|(_, s)| s.connection_id == connection_id)
            .map(|(id, _)| id.clone());

        if let Some(sid) = session_id {
            sessions.remove(&sid);

            // Kill the process
            let mut processes = self.session_processes.write().await;
            if let Some(mut session_process) = processes.remove(&sid) {
                if let Some(mut process) = session_process.process.take() {
                    let _ = process.kill();
                }
            }
        }

        // Update connection status
        let mut connections = self.connections.write().await;
        if let Some(conn) = connections.get_mut(connection_id) {
            conn.status = ConnectionStatus::Disconnected;
        }

        Ok(())
    }

    /// Get all active sessions
    pub async fn get_active_sessions(&self) -> Result<Vec<ActiveSession>> {
        let sessions = self.active_sessions.read().await;
        Ok(sessions.values().cloned().collect())
    }

    /// Create a connection group
    pub async fn create_group(&self, name: String, color: Option<String>) -> Result<ConnectionGroup> {
        let group_id = uuid::Uuid::new_v4().to_string();
        
        let group = ConnectionGroup {
            id: group_id.clone(),
            name,
            connections: Vec::new(),
            color,
        };

        let mut groups = self.groups.write().await;
        groups.insert(group_id, group.clone());
        drop(groups);
        
        // Save groups after creating
        if let Err(e) = self.save_groups().await {
            info!("Failed to save groups: {}", e);
        }

        Ok(group)
    }

    /// Add connection to group
    pub async fn add_to_group(&self, group_id: &str, connection_id: &str) -> Result<()> {
        let mut groups = self.groups.write().await;
        let group = groups.get_mut(group_id)
            .ok_or_else(|| anyhow!("Group not found"))?;

        if !group.connections.contains(&connection_id.to_string()) {
            group.connections.push(connection_id.to_string());
        }
        drop(groups);
        
        // Save groups after adding connection
        if let Err(e) = self.save_groups().await {
            info!("Failed to save groups: {}", e);
        }

        Ok(())
    }

    /// Get all groups
    pub async fn get_groups(&self) -> Result<Vec<ConnectionGroup>> {
        let groups = self.groups.read().await;
        Ok(groups.values().cloned().collect())
    }
}

#[async_trait::async_trait]
impl crate::services::Service for RemoteDesktopService {
    async fn start(&mut self) -> Result<()> {
        info!("remote desktop service start");
        self.initialize().await?;
        Ok(())
    }

    async fn run(self) -> Result<()> {
        info!("remote desktop service running");
        let mut command_rx = self.command_rx.lock().await;
        
        loop {
            tokio::select! {
                cmd = command_rx.recv() => {
                    match cmd {
                        Some(Command::RemoteDesktopGetConnections) => {
                            if let Ok(connections) = self.get_connections().await {
                                self.bus.publish(Event::RemoteDesktopConnectionsUpdated { connections });
                            }
                        }
                        Some(Command::RemoteDesktopCreateConnection { request }) => {
                            match self.create_connection(request).await {
                                Ok(_) => {
                                    if let Ok(connections) = self.get_connections().await {
                                        self.bus.publish(Event::RemoteDesktopConnectionsUpdated { connections });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to create connection: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopUpdateConnection { id, request }) => {
                            match self.update_connection(&id, request).await {
                                Ok(_) => {
                                    if let Ok(connections) = self.get_connections().await {
                                        self.bus.publish(Event::RemoteDesktopConnectionsUpdated { connections });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to update connection: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopDeleteConnection { id }) => {
                            match self.delete_connection(&id).await {
                                Ok(_) => {
                                    if let Ok(connections) = self.get_connections().await {
                                        self.bus.publish(Event::RemoteDesktopConnectionsUpdated { connections });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to delete connection: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopConnect { connection_id }) => {
                            match self.connect(ConnectRequest { connection_id, override_credentials: None }).await {
                                Ok(session) => {
                                    if let Ok(sessions) = self.get_active_sessions().await {
                                        self.bus.publish(Event::RemoteDesktopSessionsUpdated { sessions });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to connect: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopDisconnect { connection_id }) => {
                            match self.disconnect(&connection_id).await {
                                Ok(_) => {
                                    if let Ok(sessions) = self.get_active_sessions().await {
                                        self.bus.publish(Event::RemoteDesktopSessionsUpdated { sessions });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to disconnect: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopGetGroups) => {
                            if let Ok(groups) = self.get_groups().await {
                                self.bus.publish(Event::RemoteDesktopGroupsUpdated { groups });
                            }
                        }
                        Some(Command::RemoteDesktopCreateGroup { name, color }) => {
                            match self.create_group(name, color).await {
                                Ok(_) => {
                                    if let Ok(groups) = self.get_groups().await {
                                        self.bus.publish(Event::RemoteDesktopGroupsUpdated { groups });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to create group: {}", e) });
                                }
                            }
                        }
                        Some(Command::RemoteDesktopAddToGroup { group_id, connection_id }) => {
                            match self.add_to_group(&group_id, &connection_id).await {
                                Ok(_) => {
                                    if let Ok(groups) = self.get_groups().await {
                                        self.bus.publish(Event::RemoteDesktopGroupsUpdated { groups });
                                    }
                                }
                                Err(e) => {
                                    self.bus.publish(Event::Error { message: format!("Failed to add to group: {}", e) });
                                }
                            }
                        }
                        Some(_) => {} // other commands handled by other services
                        None => {
                            info!("remote desktop command channel closed");
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
