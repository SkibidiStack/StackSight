use crate::core::event_bus::EventBus;
use crate::models::commands::Command;
use crate::models::events::Event;
use crate::models::system::{SystemSnapshot, LoadAvg, DiskInfo, NetworkInfo, Alert, AlertLevel, ProcessInfo};
use anyhow::Result;
use sysinfo::{System, Networks, RefreshKind, CpuRefreshKind, MemoryRefreshKind, ProcessRefreshKind};
use tokio::sync::mpsc;
use tracing::{info, error};
use std::time::{Duration, Instant};

pub struct SystemService {
    bus: EventBus,
    sys: System,
    networks: Networks,
    command_rx: mpsc::Receiver<Command>,
    last_alert_check: Instant,
}

impl SystemService {
    pub async fn new(bus: EventBus, command_rx: mpsc::Receiver<Command>) -> Result<Self> {
        let refresh_kind = RefreshKind::new()
            .with_cpu(CpuRefreshKind::everything())
            .with_memory(MemoryRefreshKind::everything());
        
        let mut sys = System::new_with_specifics(refresh_kind);
        sys.refresh_all();
        
        Ok(Self { 
            bus, 
            sys, 
            networks: Networks::new_with_refreshed_list(),
            command_rx,
            last_alert_check: Instant::now(),
        })
    }

    async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::SystemGetProcessList => {
                self.sys.refresh_processes_specifics(ProcessRefreshKind::everything());
                
                // Limit to top 100 processes by CPU usage to avoid huge payloads
                let mut procs: Vec<_> = self.sys.processes().values().collect();
                procs.sort_by(|a, b| b.cpu_usage().partial_cmp(&a.cpu_usage()).unwrap_or(std::cmp::Ordering::Equal));
                
                let processes = procs.into_iter().take(100).map(|proc| {
                    ProcessInfo {
                        pid: proc.pid().to_string(),
                        name: proc.name().to_string(),
                        cpu_usage: proc.cpu_usage(),
                        memory: proc.memory(),
                        status: format!("{:?}", proc.status()),
                        cmd: Some(proc.cmd().iter().map(|s| s.to_string()).collect()),
                        parent_pid: proc.parent().map(|p| p.to_string()),
                    }
                }).collect();
                
                self.bus.publish(Event::SystemProcessList(processes));
            },
            Command::SystemKillProcess { pid } => {
                let pid_str = pid.clone();
                // Try parsing as simple number first (most platforms)
                // sysinfo::Pid is usually a wrapper around integer
                // We'll iterate processes and find string match if direct parsing is tricky across platforms
                // But generally sysinfo::Pid implements FromStr or similar
                
                // Hack: Sysinfo 0.30+ uses Pid struct. Let's try to find process by string PID if parsing is hard
                // Or better, assume Pid is u32/i32 wrapper
                
                let pid_parsed = if let Ok(val) = pid.parse::<usize>() {
                     sysinfo::Pid::from(val)
                } else {
                    // Fallback or error
                    error!("Invalid PID format: {}", pid);
                    return;
                };

                if let Some(proc) = self.sys.process(pid_parsed) {
                    if proc.kill() {
                        info!("Killed process {}", pid_str);
                        self.bus.publish(Event::SystemAlert(Alert {
                            level: AlertLevel::Info,
                            title: "Process Killed".to_string(),
                            message: format!("Process {} ({}) killed successfully", proc.name().to_string(), pid_str),
                            timestamp: chrono::Local::now().to_rfc3339(),
                        }));
                    } else {
                        error!("Failed to kill process {}", pid_str);
                            self.bus.publish(Event::SystemAlert(Alert {
                            level: AlertLevel::Warning,
                            title: "Kill Failed".to_string(),
                            message: format!("Failed to kill process {}", pid_str),
                            timestamp: chrono::Local::now().to_rfc3339(),
                        }));
                    }
                } else {
                    error!("Process {} not found", pid_str);
                }
            },
            _ => {}
        }
    }

    fn collect_metrics(&mut self) -> SystemSnapshot {
        self.sys.refresh_cpu();
        self.sys.refresh_memory();
        self.networks.refresh();
        // refresh disks occasionally? Or maybe every time? Sysinfo disks refresh is cheap?
        // Let's do disks refresh
        
        let cpu_usage = self.sys.global_cpu_info().cpu_usage();
        let memory_used = self.sys.used_memory();
        let memory_total = self.sys.total_memory();
        let swap_used = self.sys.used_swap();
        let swap_total = self.sys.total_swap();
        let uptime = System::uptime();
        
        let load = System::load_average();
        let load_avg = LoadAvg {
            one: load.one,
            five: load.five,
            fifteen: load.fifteen,
        };
        
        let disks_list = sysinfo::Disks::new_with_refreshed_list();
        let disks = disks_list.list().iter().map(|disk| {
            DiskInfo {
                name: disk.name().to_string_lossy().into_owned(),
                mount_point: disk.mount_point().to_string_lossy().into_owned(),
                total_space: disk.total_space(),
                available_space: disk.available_space(),
                file_system: disk.file_system().to_string_lossy().into_owned(),
            }
        }).collect();

        let networks = self.networks.iter().map(|(name, data)| {
            NetworkInfo {
                name: name.to_string(),
                received: data.total_received(),
                transmitted: data.total_transmitted(),
                packets_recv: data.total_packets_received(),
                packets_sent: data.total_packets_transmitted(),
                errors_on_recv: data.total_errors_on_received(),
                errors_on_sent: data.total_errors_on_transmitted(),
            }
        }).collect();

        SystemSnapshot {
            cpu_usage,
            memory_used,
            memory_total,
            swap_used,
            swap_total,
            uptime,
            load_avg,
            disks,
            networks,
        }
    }
    
    fn check_alerts(&mut self, snapshot: &SystemSnapshot) {
        // Simple alert logic
        if snapshot.cpu_usage > 90.0 {
            self.bus.publish(Event::SystemAlert(Alert {
                level: AlertLevel::Warning,
                title: "High CPU Usage".to_string(),
                message: format!("CPU usage is at {:.1}%", snapshot.cpu_usage),
                timestamp: chrono::Local::now().to_rfc3339(),
            }));
        }
        
        let mem_percent = (snapshot.memory_used as f64 / snapshot.memory_total as f64) * 100.0;
        if mem_percent > 90.0 {
             self.bus.publish(Event::SystemAlert(Alert {
                level: AlertLevel::Warning,
                title: "High Memory Usage".to_string(),
                message: format!("Memory usage is at {:.1}%", mem_percent),
                timestamp: chrono::Local::now().to_rfc3339(),
            }));
        }
    }
}

#[async_trait::async_trait]
impl crate::services::Service for SystemService {
    async fn start(&mut self) -> Result<()> {
        info!("system service started");
        Ok(())
    }

    async fn run(mut self) -> Result<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(2));
        
        loop {
            tokio::select! {
                _ = interval.tick() => {
                    let snapshot = self.collect_metrics();
                    self.bus.publish(Event::SystemSnapshot(snapshot.clone()));
                    
                    if self.last_alert_check.elapsed() > Duration::from_secs(60) {
                        self.check_alerts(&snapshot);
                        self.last_alert_check = Instant::now();
                    }
                }
                Some(cmd) = self.command_rx.recv() => {
                    self.handle_command(cmd).await;
                }
            }
        }
    }
}
