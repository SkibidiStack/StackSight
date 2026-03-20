use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub display_name: String,
    pub mac_address: Option<String>,
    pub ip_addresses: Vec<IpConfiguration>,
    pub status: InterfaceStatus,
    pub mtu: u32,
    pub speed: Option<u64>, // Mbps
    pub interface_type: InterfaceType,
    pub vlans: Vec<VlanConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpConfiguration {
    pub address: IpAddr,
    pub netmask: String,
    pub gateway: Option<IpAddr>,
    pub version: IpVersion,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IpVersion {
    V4,
    V6,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceStatus {
    Up,
    Down,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterfaceType {
    Ethernet,
    Wireless,
    Virtual,
    Loopback,
    Bridge,
    Vlan,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VlanConfig {
    pub id: u16,
    pub name: String,
    pub parent_interface: String,
    pub ip_config: Option<String>,
    pub enabled: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Icmp,
    Any,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsConfiguration {
    pub servers: Vec<IpAddr>,
    pub search_domains: Vec<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStats {
    pub interface: String,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors_in: u64,
    pub errors_out: u64,
    pub dropped_in: u64,
    pub dropped_out: u64,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeConfig {
    pub name: String,
    pub interfaces: Vec<String>,
    pub stp_enabled: bool,
    pub ip_config: Option<IpConfiguration>,
}

// ── Network topology / connected-device discovery models ──────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum NetworkDeviceType {
    Gateway,
    LocalMachine,
    Host,
    Unknown,
}

impl Default for NetworkDeviceType {
    fn default() -> Self {
        NetworkDeviceType::Unknown
    }
}

/// A device discovered on the local network (via ARP table, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkDevice {
    pub ip: String,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub interface: String,
    pub device_type: NetworkDeviceType,
    pub is_reachable: bool,
    pub vendor: Option<String>,
}

/// Full topology snapshot emitted after a scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopologyData {
    pub devices: Vec<NetworkDevice>,
    pub gateway: Option<String>,
    pub local_ip: Option<String>,
    pub scan_time: String,
}

// ── Request/Response models ────────────────────────────────────────────────────

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateVlanRequest {
    pub vlan_id: u16,
    pub name: String,
    pub parent_interface: String,
    pub ip_address: Option<String>,
    pub netmask: Option<String>,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInterfaceRequest {
    pub interface: String,
    pub ip_address: Option<String>,
    pub netmask: Option<String>,
    pub gateway: Option<String>,
    pub dns_servers: Option<Vec<String>>,
    pub mtu: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBridgeRequest {
    pub name: String,
    pub interfaces: Vec<String>,
    pub ip_config: Option<String>,
}
