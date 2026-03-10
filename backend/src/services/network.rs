use anyhow::{anyhow, Context, Result};
use crate::core::event_bus::EventBus;
use crate::models::commands::Command;
use crate::models::events::Event;
use crate::models::network::*;
use std::collections::HashMap;
use std::net::IpAddr;
use std::process::Command as SysCommand;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tracing::{debug, info};

#[allow(dead_code)]
pub struct NetworkService {
    bus: EventBus,
    interfaces: Arc<RwLock<HashMap<String, NetworkInterface>>>,
    routes: Arc<RwLock<Vec<Route>>>,
    firewall_rules: Arc<RwLock<HashMap<String, FirewallRule>>>,
    command_rx: mpsc::Receiver<Command>,
}

#[allow(dead_code)]
impl NetworkService {
    pub async fn new(bus: EventBus, command_rx: mpsc::Receiver<Command>) -> Result<Self> {
        Ok(Self {
            bus,
            interfaces: Arc::new(RwLock::new(HashMap::new())),
            routes: Arc::new(RwLock::new(Vec::new())),
            firewall_rules: Arc::new(RwLock::new(HashMap::new())),
            command_rx,
        })
    }

    /// Initialize the service by scanning the system
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing network service");
        
        self.refresh_interfaces().await?;
        self.refresh_routes().await?;
        self.refresh_firewall_rules().await?;
        
        Ok(())
    }

    /// Get all network interfaces
    pub async fn get_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        let interfaces = self.interfaces.read().await;
        Ok(interfaces.values().cloned().collect())
    }

    /// Get a specific network interface by name
    pub async fn get_interface(&self, name: &str) -> Result<NetworkInterface> {
        let interfaces = self.interfaces.read().await;
        interfaces
            .get(name)
            .cloned()
            .ok_or_else(|| anyhow!("Interface '{}' not found", name))
    }

    /// Refresh all network interfaces from system
    pub async fn refresh_interfaces(&self) -> Result<()> {
        debug!("Refreshing network interfaces");
        
        let discovered = self.discover_interfaces().await?;
        let mut interfaces = self.interfaces.write().await;
        
        for iface in discovered {
            interfaces.insert(iface.name.clone(), iface);
        }
        
        info!("Discovered {} network interfaces", interfaces.len());
        Ok(())
    }

    /// Discover network interfaces from the system
    async fn discover_interfaces(&self) -> Result<Vec<NetworkInterface>> {
        // Use `ip` command on Linux to discover interfaces
        #[cfg(target_os = "linux")]
        {
            self.discover_interfaces_linux().await
        }
        
        // Use native APIs on Windows
        #[cfg(target_os = "windows")]
        {
            self.discover_interfaces_windows().await
        }
        
        // Use `ifconfig` on macOS
        #[cfg(target_os = "macos")]
        {
            self.discover_interfaces_macos().await
        }
    }

    #[cfg(target_os = "linux")]
    async fn discover_interfaces_linux(&self) -> Result<Vec<NetworkInterface>> {
        let output = SysCommand::new("ip")
            .args(&["--json", "addr", "show"])
            .output()
            .context("Failed to execute ip command")?;

        if !output.status.success() {
            return Err(anyhow!("ip command failed: {}", String::from_utf8_lossy(&output.stderr)));
        }

        let json_str = String::from_utf8(output.stdout)?;
        let parsed: Vec<serde_json::Value> = serde_json::from_str(&json_str)
            .context("Failed to parse ip output")?;

        let mut interfaces = Vec::new();
        
        for iface_data in parsed {
            if let Some(iface) = self.parse_linux_interface(&iface_data) {
                interfaces.push(iface);
            }
        }

        Ok(interfaces)
    }

    #[cfg(target_os = "linux")]
    fn parse_linux_interface(&self, data: &serde_json::Value) -> Option<NetworkInterface> {
        let name = data["ifname"].as_str()?.to_string();
        let mac_address = data["address"].as_str().map(|s| s.to_string());
        
        let operstate = data["operstate"].as_str().unwrap_or("unknown");
        let status = match operstate {
            "UP" => InterfaceStatus::Up,
            "DOWN" => InterfaceStatus::Down,
            _ => InterfaceStatus::Unknown,
        };

        let mtu = data["mtu"].as_u64().unwrap_or(1500) as u32;

        let link_type = data["link_type"].as_str().unwrap_or("ether");
        let interface_type = match link_type {
            "ether" => InterfaceType::Ethernet,
            "loopback" => InterfaceType::Loopback,
            "bridge" => InterfaceType::Bridge,
            _ => InterfaceType::Other(link_type.to_string()),
        };

        let mut ip_addresses = Vec::new();
        if let Some(addr_info) = data["addr_info"].as_array() {
            for addr in addr_info {
                if let Some(ip_str) = addr["local"].as_str() {
                    if let Ok(ip_addr) = ip_str.parse::<IpAddr>() {
                        let version = if ip_addr.is_ipv4() { IpVersion::V4 } else { IpVersion::V6 };
                        let prefixlen = addr["prefixlen"].as_u64().unwrap_or(24);
                        let netmask = format!("/{}", prefixlen);
                        
                        ip_addresses.push(IpConfiguration {
                            address: ip_addr,
                            netmask,
                            gateway: None,
                            version,
                        });
                    }
                }
            }
        }

        Some(NetworkInterface {
            name: name.clone(),
            display_name: name,
            mac_address,
            ip_addresses,
            status,
            mtu,
            speed: None,
            interface_type,
            vlans: Vec::new(),
        })
    }

    #[cfg(target_os = "windows")]
    async fn discover_interfaces_windows(&self) -> Result<Vec<NetworkInterface>> {
        // Use PowerShell to get network adapter information
        let output = SysCommand::new("powershell")
            .args(&[
                "-Command",
                "Get-NetAdapter | ConvertTo-Json"
            ])
            .output()
            .context("Failed to execute PowerShell command")?;

        if !output.status.success() {
            return Err(anyhow!("PowerShell command failed"));
        }

        let json_str = String::from_utf8(output.stdout)?;
        // Parse Windows network adapter JSON
        // (Implementation would parse PowerShell output)
        
        Ok(Vec::new()) // Placeholder
    }

    #[cfg(target_os = "macos")]
    async fn discover_interfaces_macos(&self) -> Result<Vec<NetworkInterface>> {
        // Use `ifconfig` on macOS
        let output = SysCommand::new("ifconfig")
            .arg("-a")
            .output()
            .context("Failed to execute ifconfig command")?;

        if !output.status.success() {
            return Err(anyhow!("ifconfig command failed"));
        }

        // Parse ifconfig output
        // (Implementation would parse ifconfig text output)
        
        Ok(Vec::new()) // Placeholder
    }

    /// Create a new VLAN
    pub async fn create_vlan(&self, request: CreateVlanRequest) -> Result<VlanConfig> {
        info!("Creating VLAN {} on interface {}", request.vlan_id, request.parent_interface);

        #[cfg(target_os = "linux")]
        {
            let vlan_name = format!("{}.{}", request.parent_interface, request.vlan_id);
            
            // Create VLAN interface
            let output = SysCommand::new("ip")
                .args(&[
                    "link",
                    "add",
                    "link",
                    &request.parent_interface,
                    "name",
                    &vlan_name,
                    "type",
                    "vlan",
                    "id",
                    &request.vlan_id.to_string(),
                ])
                .output()
                .context("Failed to create VLAN")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to create VLAN: {}", String::from_utf8_lossy(&output.stderr)));
            }

            // Configure IP if provided
            if let (Some(ip), Some(netmask)) = (request.ip_address, request.netmask) {
                SysCommand::new("ip")
                    .args(&["addr", "add", &format!("{}/{}", ip, netmask), "dev", &vlan_name])
                    .output()
                    .context("Failed to configure VLAN IP")?;
            }

            // Bring interface up
            SysCommand::new("ip")
                .args(&["link", "set", &vlan_name, "up"])
                .output()
                .context("Failed to bring up VLAN")?;

            Ok(VlanConfig {
                id: request.vlan_id,
                name: vlan_name,
                parent_interface: request.parent_interface,
                ip_config: None,
                enabled: true,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(anyhow!("VLAN creation not implemented for this platform"))
        }
    }

    /// Delete a VLAN
    pub async fn delete_vlan(&self, parent_interface: &str, vlan_id: u16) -> Result<()> {
        info!("Deleting VLAN {} from interface {}", vlan_id, parent_interface);

        #[cfg(target_os = "linux")]
        {
            let vlan_name = format!("{}.{}", parent_interface, vlan_id);
            
            let output = SysCommand::new("ip")
                .args(&["link", "delete", &vlan_name])
                .output()
                .context("Failed to delete VLAN")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to delete VLAN: {}", String::from_utf8_lossy(&output.stderr)));
            }

            Ok(())
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(anyhow!("VLAN deletion not implemented for this platform"))
        }
    }

    /// Update interface configuration
    pub async fn update_interface(&self, request: UpdateInterfaceRequest) -> Result<()> {
        info!("Updating interface {}", request.interface);

        if let (Some(ip), Some(netmask)) = (&request.ip_address, &request.netmask) {
            #[cfg(target_os = "linux")]
            {
                SysCommand::new("ip")
                    .args(&["addr", "add", &format!("{}/{}", ip, netmask), "dev", &request.interface])
                    .output()
                    .context("Failed to configure IP")?;
            }
        }

        if let Some(mtu) = request.mtu {
            #[cfg(target_os = "linux")]
            {
                SysCommand::new("ip")
                    .args(&["link", "set", &request.interface, "mtu", &mtu.to_string()])
                    .output()
                    .context("Failed to set MTU")?;
            }
        }

        self.refresh_interfaces().await?;
        Ok(())
    }

    /// Get all routes
    pub async fn get_routes(&self) -> Result<Vec<Route>> {
        let routes = self.routes.read().await;
        Ok(routes.clone())
    }

    /// Refresh routing table
    pub async fn refresh_routes(&self) -> Result<()> {
        debug!("Refreshing routing table");
        
        #[cfg(target_os = "linux")]
        {
            let output = SysCommand::new("ip")
                .args(&["--json", "route", "show"])
                .output()
                .context("Failed to get routes")?;

            if output.status.success() {
                let _json_str = String::from_utf8(output.stdout)?;
                // Parse route JSON
                let mut routes = self.routes.write().await;
                routes.clear();
                // (Full parsing implementation here)
            }
        }

        Ok(())
    }

    /// Add a new route
    pub async fn add_route(&self, request: AddRouteRequest) -> Result<()> {
        info!("Adding route to {}", request.destination);

        #[cfg(target_os = "linux")]
        {
            let mut args = vec!["route", "add", &request.destination, "via", &request.gateway];
            
            let metric_str;
            if let Some(iface) = &request.interface {
                args.extend_from_slice(&["dev", iface]);
            }
            
            if let Some(metric) = request.metric {
                metric_str = metric.to_string();
                args.extend_from_slice(&["metric", &metric_str]);
            }

            let output = SysCommand::new("ip")
                .args(&args)
                .output()
                .context("Failed to add route")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to add route: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }

        self.refresh_routes().await?;
        Ok(())
    }

    /// Delete a route
    pub async fn delete_route(&self, destination: &str) -> Result<()> {
        info!("Deleting route to {}", destination);

        #[cfg(target_os = "linux")]
        {
            let output = SysCommand::new("ip")
                .args(&["route", "del", destination])
                .output()
                .context("Failed to delete route")?;

            if !output.status.success() {
                return Err(anyhow!("Failed to delete route: {}", String::from_utf8_lossy(&output.stderr)));
            }
        }

        self.refresh_routes().await?;
        Ok(())
    }

    /// Get all firewall rules
    pub async fn get_firewall_rules(&self) -> Result<Vec<FirewallRule>> {
        let rules = self.firewall_rules.read().await;
        Ok(rules.values().cloned().collect())
    }

    /// Refresh firewall rules
    pub async fn refresh_firewall_rules(&self) -> Result<()> {
        debug!("Refreshing firewall rules");
        
        #[cfg(target_os = "linux")]
        {
            // Use iptables or nftables to get rules
            let output = SysCommand::new("iptables")
                .args(&["-L", "-n", "-v"])
                .output();
                
            if let Ok(result) = output {
                if result.status.success() {
                    // Parse iptables output
                    // (Full parsing implementation here)
                }
            }
        }

        Ok(())
    }

    /// Create a firewall rule
    pub async fn create_firewall_rule(&self, request: CreateFirewallRuleRequest) -> Result<FirewallRule> {
        info!("Creating firewall rule: {}", request.name);

        let rule_id = uuid::Uuid::new_v4().to_string();
        
        #[cfg(target_os = "linux")]
        {
            let chain = match request.direction {
                TrafficDirection::Inbound => "INPUT",
                TrafficDirection::Outbound => "OUTPUT",
                TrafficDirection::Both => "FORWARD",
            };

            let action_str = match request.action {
                FirewallAction::Allow => "ACCEPT",
                FirewallAction::Deny => "DROP",
                FirewallAction::Log => "LOG",
            };

            let mut args = vec!["-A", chain];
            
            if let Some(proto) = &request.protocol {
                let proto_str = match proto {
                    Protocol::Tcp => "tcp",
                    Protocol::Udp => "udp",
                    Protocol::Icmp => "icmp",
                    Protocol::Any => "all",
                };
                args.extend_from_slice(&["-p", proto_str]);
            }

            if let Some(src_ip) = &request.source_ip {
                args.extend_from_slice(&["-s", src_ip]);
            }

            if let Some(dst_ip) = &request.destination_ip {
                args.extend_from_slice(&["-d", dst_ip]);
            }

            args.extend_from_slice(&["-j", action_str]);

            SysCommand::new("iptables")
                .args(&args)
                .output()
                .context("Failed to create firewall rule")?;
        }

        let rule = FirewallRule {
            id: rule_id,
            name: request.name,
            enabled: true,
            action: request.action,
            direction: request.direction,
            protocol: request.protocol,
            source_ip: request.source_ip,
            source_port: request.source_port,
            destination_ip: request.destination_ip,
            destination_port: request.destination_port,
            interface: request.interface,
            priority: 100,
        };

        let mut rules = self.firewall_rules.write().await;
        rules.insert(rule.id.clone(), rule.clone());

        Ok(rule)
    }

    /// Delete a firewall rule
    pub async fn delete_firewall_rule(&self, rule_id: &str) -> Result<()> {
        info!("Deleting firewall rule: {}", rule_id);

        let mut rules = self.firewall_rules.write().await;
        rules.remove(rule_id)
            .ok_or_else(|| anyhow!("Firewall rule not found"))?;

        Ok(())
    }

    /// Get network statistics for an interface
    pub async fn get_interface_stats(&self, interface: &str) -> Result<NetworkStats> {
        #[cfg(target_os = "linux")]
        {
            let rx_bytes = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/rx_bytes", interface))?
                .trim().parse().unwrap_or(0);
            let tx_bytes = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/tx_bytes", interface))?
                .trim().parse().unwrap_or(0);
            let rx_packets = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/rx_packets", interface))?
                .trim().parse().unwrap_or(0);
            let tx_packets = std::fs::read_to_string(format!("/sys/class/net/{}/statistics/tx_packets", interface))?
                .trim().parse().unwrap_or(0);

            Ok(NetworkStats {
                interface: interface.to_string(),
                bytes_sent: tx_bytes,
                bytes_received: rx_bytes,
                packets_sent: tx_packets,
                packets_received: rx_packets,
                errors_in: 0,
                errors_out: 0,
                dropped_in: 0,
                dropped_out: 0,
            })
        }

        #[cfg(not(target_os = "linux"))]
        {
            Err(anyhow!("Interface stats not implemented for this platform"))
        }
    }

    // ── Device discovery / ARP scan ────────────────────────────────────────────

    /// Read the default gateway from the kernel routing table.
    fn get_default_gateway() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            // Parse /proc/net/route: find entry with Destination == 00000000
            if let Ok(content) = std::fs::read_to_string("/proc/net/route") {
                for line in content.lines().skip(1) {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    if fields.len() >= 3 && fields[1] == "00000000" {
                        // Gateway is little-endian hex in field[2]
                        if let Ok(gw_hex) = u32::from_str_radix(fields[2], 16) {
                            let b1 = (gw_hex & 0xFF) as u8;
                            let b2 = ((gw_hex >> 8) & 0xFF) as u8;
                            let b3 = ((gw_hex >> 16) & 0xFF) as u8;
                            let b4 = ((gw_hex >> 24) & 0xFF) as u8;
                            return Some(format!("{}.{}.{}.{}", b1, b2, b3, b4));
                        }
                    }
                }
            }
            None
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Get the primary non-loopback local IPv4 address.
    fn get_local_ip() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            if let Ok(content) = std::fs::read_to_string("/proc/net/if_inet6") {
                // Not what we want — fall through
                let _ = content;
            }
            // Use `ip route get 1.1.1.1` to find the preferred source
            if let Ok(out) = SysCommand::new("ip")
                .args(["route", "get", "1.1.1.1"])
                .output()
            {
                let text = String::from_utf8_lossy(&out.stdout);
                for part in text.split_whitespace().collect::<Vec<_>>().windows(2) {
                    if part[0] == "src" {
                        return Some(part[1].to_string());
                    }
                }
            }
            None
        }
        #[cfg(not(target_os = "linux"))]
        {
            None
        }
    }

    /// Parse the kernel ARP table (/proc/net/arp on Linux) and return discovered devices.
    fn read_arp_table() -> Vec<(String, String, String)> {
        // Returns Vec<(ip, mac, iface)>
        let mut devices = Vec::new();
        #[cfg(target_os = "linux")]
        {
            // First try reading /proc/net/arp
            if let Ok(content) = std::fs::read_to_string("/proc/net/arp") {
                for line in content.lines().skip(1) {
                    let fields: Vec<&str> = line.split_whitespace().collect();
                    // IP address, HW type, Flags, HW address, Mask, Device
                    if fields.len() >= 6 {
                        let ip = fields[0].to_string();
                        let flags = fields[2]; // Check flags field
                        let mac = fields[3].to_string();
                        let iface = fields[5].to_string();
                        
                        // Skip incomplete entries
                        // Flags: 0x0 = incomplete, 0x2 = complete, 0x6 = complete+permanent
                        if mac != "00:00:00:00:00:00" && flags != "0x0" {
                            devices.push((ip, mac, iface));
                        }
                    }
                }
            }
            
            // Also try using `ip neigh` command for a more complete view
            if let Ok(out) = SysCommand::new("ip").args(["neigh", "show"]).output() {
                if out.status.success() {
                    let text = String::from_utf8_lossy(&out.stdout);
                    for line in text.lines() {
                        // Format: "192.168.1.10 dev wlan0 lladdr aa:bb:cc:dd:ee:ff REACHABLE"
                        let parts: Vec<&str> = line.split_whitespace().collect();
                        if parts.len() >= 5 {
                            let ip = parts[0].to_string();
                            // Find "lladdr" keyword
                            if let Some(idx) = parts.iter().position(|&x| x == "lladdr") {
                                if idx + 1 < parts.len() {
                                    let mac = parts[idx + 1].to_string();
                                    let iface = if let Some(dev_idx) = parts.iter().position(|&x| x == "dev") {
                                        if dev_idx + 1 < parts.len() {
                                            parts[dev_idx + 1].to_string()
                                        } else {
                                            String::new()
                                        }
                                    } else {
                                        String::new()
                                    };
                                    
                                    // Only add if not already present
                                    if !devices.iter().any(|(existing_ip, _, _)| existing_ip == &ip) {
                                        devices.push((ip, mac, iface));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        #[cfg(target_os = "windows")]
        {
            // arp -a output: "  <ip>  <mac>  dynamic"
            if let Ok(out) = SysCommand::new("arp").arg("-a").output() {
                let text = String::from_utf8_lossy(&out.stdout);
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 2 && parts[0].contains('.') {
                        devices.push((parts[0].to_string(), parts[1].to_string(), String::new()));
                    }
                }
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(out) = SysCommand::new("arp").args(["-a", "-n"]).output() {
                let text = String::from_utf8_lossy(&out.stdout);
                // "? (192.168.1.1) at aa:bb:cc:dd:ee:ff on en0 ifscope ..."
                for line in text.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 {
                        let ip = parts[1].trim_matches(|c| c == '(' || c == ')').to_string();
                        let mac = parts[3].to_string();
                        let iface = if parts.len() > 5 { parts[5].to_string() } else { String::new() };
                        if mac != "(incomplete)" {
                            devices.push((ip, mac, iface));
                        }
                    }
                }
            }
        }
        devices
    }

    /// Trigger a ping sweep of the local subnet so the ARP table is populated.
    /// We fan-out async pings but don't wait for them — the ARP table read afterwards
    /// will catch everything that responded during the sweep.
    async fn ping_sweep(gateway: &str) {
        // Derive subnet prefix from gateway (e.g. 192.168.1. from 192.168.1.1)
        let prefix = {
            let parts: Vec<&str> = gateway.splitn(4, '.').collect();
            if parts.len() == 4 {
                format!("{}.{}.{}.", parts[0], parts[1], parts[2])
            } else {
                return;
            }
        };
        let mut handles = Vec::new();
        
        // Strategy: Use multiple discovery techniques since phones often ignore ICMP
        for i in 1u8..=254 {
            let ip = format!("{}{}", prefix, i);
            let ip_clone = ip.clone();
            
            handles.push(tokio::spawn(async move {
                // 1. Try ICMP ping first (fast but often blocked by mobile devices)
                #[cfg(unix)]
                let _ = tokio::process::Command::new("ping")
                    .args(["-c", "1", "-W", "1", &ip])
                    .output()
                    .await;
                #[cfg(windows)]
                let _ = tokio::process::Command::new("ping")
                    .args(["-n", "1", "-w", "500", &ip])
                    .output()
                    .await;
                
                // 2. Try ARP ping (more reliable for discovering active hosts on local network)
                #[cfg(target_os = "linux")]
                {
                    // Use arping if available - it sends ARP requests directly
                    let _ = tokio::process::Command::new("arping")
                        .args(["-c", "1", "-w", "1", "-I", "any", &ip_clone])
                        .output()
                        .await;
                }
            }));
        }
        // Wait for all pings — we cap total wait with a timeout at the call site
        futures_util::future::join_all(handles).await;
    }

    /// Full network topology scan: ping sweep + ARP read + classify devices.
    async fn scan_devices(bus: &EventBus) {
        info!("Starting network topology scan");
        let gateway = Self::get_default_gateway();
        let local_ip = Self::get_local_ip();

        // Run a ping sweep if we know the gateway
        if let Some(ref gw) = gateway {
            // Increased timeout to 15s to allow arping to work
            let _ = tokio::time::timeout(
                std::time::Duration::from_secs(15),
                Self::ping_sweep(gw),
            )
            .await;
            
            // Give the system a moment to update the ARP cache
            tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        }

        let arp_entries = Self::read_arp_table();
        let scan_time = chrono::Local::now().to_rfc3339();
        let mut devices: Vec<NetworkDevice> = Vec::new();

        for (ip, mac, iface) in &arp_entries {
            let device_type = if Some(ip.as_str()) == gateway.as_deref() {
                NetworkDeviceType::Gateway
            } else if Some(ip.as_str()) == local_ip.as_deref() {
                NetworkDeviceType::LocalMachine
            } else {
                NetworkDeviceType::Host
            };

            devices.push(NetworkDevice {
                ip: ip.clone(),
                mac: if mac.is_empty() || mac == "00:00:00:00:00:00" {
                    None
                } else {
                    Some(mac.clone())
                },
                hostname: None,
                interface: iface.clone(),
                device_type,
                is_reachable: true,
                vendor: None,
            });
        }

        // Always include ourselves if we know our IP
        if let Some(ref lip) = local_ip {
            if !devices.iter().any(|d| &d.ip == lip) {
                devices.push(NetworkDevice {
                    ip: lip.clone(),
                    mac: None,
                    hostname: Some("localhost".to_string()),
                    interface: String::new(),
                    device_type: NetworkDeviceType::LocalMachine,
                    is_reachable: true,
                    vendor: None,
                });
            }
        }

        info!("Network scan complete: {} devices found", devices.len());
        debug!("Devices discovered: {:?}", devices.iter().map(|d| &d.ip).collect::<Vec<_>>());

        bus.publish(Event::NetworkTopology(NetworkTopologyData {
            devices,
            gateway,
            local_ip,
            scan_time,
        }));
    }
}

#[async_trait::async_trait]
impl crate::services::Service for NetworkService {
    async fn start(&mut self) -> Result<()> {
        info!("network service start");
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        info!("network service running");
        loop {
            tokio::select! {
                cmd = self.command_rx.recv() => {
                    match cmd {
                        Some(Command::NetworkScanDevices) => {
                            let bus = self.bus.clone();
                            tokio::spawn(async move {
                                NetworkService::scan_devices(&bus).await;
                            });
                        }
                        Some(_) => {} // other commands handled by other services
                        None => {
                            info!("network command channel closed");
                            break;
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
