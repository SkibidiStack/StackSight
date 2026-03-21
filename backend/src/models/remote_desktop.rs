use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConnection {
    pub id: String,
    pub name: String,
    pub protocol: ConnectionProtocol,
    pub host: String,
    pub port: u16,
    pub credentials: Credentials,
    pub settings: ConnectionSettings,
    pub status: ConnectionStatus,
    pub last_connected: Option<String>,
    pub favorite: bool,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionProtocol {
    Ssh,
    Vnc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub auth_method: AuthMethod,
    pub save_credentials: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthMethod {
    Password { password: String },
    PrivateKey { key_path: String, passphrase: Option<String> },
    Certificate { cert_path: String },
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionSettings {
    pub ssh_settings: Option<SshSettings>,
    pub vnc_settings: Option<VncSettings>,
    pub display_settings: DisplaySettings,
    pub tunnel_settings: Option<TunnelSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SshSettings {
    pub terminal_type: String,
    pub compression: bool,
    pub forward_x11: bool,
    pub keep_alive_interval: u32,
    pub port_forwards: Vec<PortForward>,
    pub environment_variables: HashMap<String, String>,
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VncSettings {
    pub password: Option<String>,
    pub view_only: bool,
    pub quality: VncQuality,
    pub encoding: VncEncoding,
    pub cursor_mode: CursorMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VncQuality {
    Low,
    Medium,
    High,
    Lossless,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VncEncoding {
    Raw,
    Tight,
    Zrle,
    Hextile,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CursorMode {
    Local,
    Remote,
    Dot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplaySettings {
    pub resolution: Option<Resolution>,
    pub color_depth: ColorDepth,
    pub fullscreen: bool,
    pub scaling: ScalingMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resolution {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ColorDepth {
    Low,      // 8-bit
    Medium,   // 16-bit
    High,     // 24-bit
    True,     // 32-bit
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ScalingMode {
    None,
    Fit,
    Stretch,
    Center,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelSettings {
    pub jump_host: String,
    pub jump_port: u16,
    pub jump_username: String,
    pub jump_auth: AuthMethod,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortForward {
    pub local_port: u16,
    pub remote_host: String,
    pub remote_port: u16,
    pub forward_type: ForwardType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ForwardType {
    Local,
    Remote,
    Dynamic,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
    Failed { error: String },
    Reconnecting,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveSession {
    pub connection_id: String,
    pub session_id: String,
    pub started_at: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency: Option<u32>, // ms
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecording {
    pub session_id: String,
    pub connection_name: String,
    pub started_at: String,
    pub duration: u64, // seconds
    pub file_path: String,
    pub file_size: u64,
}

// Request/Response models
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnectionRequest {
    pub name: String,
    pub protocol: ConnectionProtocol,
    pub host: String,
    pub port: Option<u16>,
    pub username: String,
    pub password: Option<String>,
    pub private_key: Option<String>,
    pub settings: Option<ConnectionSettings>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConnectionRequest {
    pub name: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub credentials: Option<Credentials>,
    pub settings: Option<ConnectionSettings>,
    pub favorite: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    pub connection_id: String,
    pub override_credentials: Option<Credentials>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionGroup {
    pub id: String,
    pub name: String,
    pub connections: Vec<String>, // connection IDs
    pub color: Option<String>,
}
