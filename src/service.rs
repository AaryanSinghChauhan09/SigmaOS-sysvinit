#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Child;
use tokio::process::Command as TokioCommand;

/// Service status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

/// Service definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service name
    pub name: String,
    /// Path to init script
    pub script_path: PathBuf,
    /// Description
    pub description: String,
    /// Required runlevels
    pub runlevels: Vec<u8>,
    /// Start priority (lower = earlier)
    pub start_priority: u32,
    /// Stop priority (lower = earlier)
    pub stop_priority: u32,
    /// Whether service should be started by default
    pub enabled: bool,
    /// Dependencies (services that must start first)
    pub dependencies: Vec<String>,
    /// Current status
    #[serde(skip)]
    pub status: ServiceStatus,
    /// Process ID if running
    #[serde(skip)]
    pub pid: Option<u32>,
}

impl Service {
    /// Create a new service from an init script
    pub fn from_script(script_path: PathBuf) -> Result<Self> {
        let name = script_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();
        
        // Parse script for metadata
        let (description, runlevels, start_priority, stop_priority) = 
            Self::parse_script_metadata(&script_path)?;
        
        Ok(Self {
            name,
            script_path,
            description,
            runlevels,
            start_priority,
            stop_priority,
            enabled: true,
            dependencies: Vec::new(),
            status: ServiceStatus::Stopped,
            pid: None,
        })
    }

    /// Parse init script for LSB-style metadata
    fn parse_script_metadata(path: &PathBuf) -> Result<(String, Vec<u8>, u32, u32)> {
        let content = std::fs::read_to_string(path)
            .context("Failed to read init script")?;
        
        let mut description = String::new();
        let mut runlevels = vec![2, 3, 4, 5]; // Default runlevels
        let mut start_priority = 50;
        let mut stop_priority = 50;
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("# Provides:") {
                // Could parse service name here
            } else if line.starts_with("# Description:") {
                description = line.trim_start_matches("# Description:").trim().to_string();
            } else if line.starts_with("# Default-Start:") {
                let levels = line.trim_start_matches("# Default-Start:").trim();
                runlevels = levels
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
            } else if line.starts_with("# Required-Start:") {
                // Could parse dependencies here
            } else if line.starts_with("# X-Start-Before:") {
                // Could parse ordering here
            }
        }
        
        if description.is_empty() {
            description = "Service".to_string();
        }
        
        Ok((description, runlevels, start_priority, stop_priority))
    }

    /// Start the service
    pub async fn start(&mut self) -> Result<()> {
        if self.status == ServiceStatus::Running {
            log::info!("Service {} already running", self.name);
            return Ok(());
        }
        
        log::info!("Starting service: {}", self.name);
        self.status = ServiceStatus::Starting;
        
        let output = TokioCommand::new(&self.script_path)
            .arg("start")
            .output()
            .await
            .context("Failed to execute start command")?;
        
        if output.status.success() {
            self.status = ServiceStatus::Running;
            self.pid = self.find_pid().await?;
            log::info!("Service {} started successfully", self.name);
            Ok(())
        } else {
            self.status = ServiceStatus::Failed;
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to start service {}: {}", self.name, stderr);
        }
    }

    /// Stop the service
    pub async fn stop(&mut self) -> Result<()> {
        if self.status == ServiceStatus::Stopped {
            log::info!("Service {} already stopped", self.name);
            return Ok(());
        }
        
        log::info!("Stopping service: {}", self.name);
        self.status = ServiceStatus::Stopping;
        
        let output = TokioCommand::new(&self.script_path)
            .arg("stop")
            .output()
            .await
            .context("Failed to execute stop command")?;
        
        if output.status.success() {
            self.status = ServiceStatus::Stopped;
            self.pid = None;
            log::info!("Service {} stopped successfully", self.name);
            Ok(())
        } else {
            self.status = ServiceStatus::Failed;
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Failed to stop service {}: {}", self.name, stderr);
        }
    }

    /// Restart the service
    pub async fn restart(&mut self) -> Result<()> {
        log::info!("Restarting service: {}", self.name);
        self.stop().await?;
        self.start().await?;
        Ok(())
    }

    /// Get service status by checking the init script
    pub async fn check_status(&mut self) -> Result<ServiceStatus> {
        let output = TokioCommand::new(&self.script_path)
            .arg("status")
            .output()
            .await;
        
        match output {
            Ok(out) => {
                if out.status.success() {
                    self.status = ServiceStatus::Running;
                    self.pid = self.find_pid().await?;
                } else {
                    self.status = ServiceStatus::Stopped;
                    self.pid = None;
                }
            }
            Err(_) => {
                self.status = ServiceStatus::Stopped;
                self.pid = None;
            }
        }
        
        Ok(self.status)
    }

    /// Find the PID of the running service
    async fn find_pid(&self) -> Result<Option<u32>> {
        // Try to read from PID file first
        let pid_file = format!("/var/run/{}.pid", self.name);
        if let Ok(content) = std::fs::read_to_string(&pid_file) {
            if let Ok(pid) = content.trim().parse::<u32>() {
                return Ok(Some(pid));
            }
        }
        
        // Fallback: try to find by process name
        Ok(None)
    }

    /// Check if service should run in a given runlevel
    pub fn should_run_in(&self, runlevel: u8) -> bool {
        self.enabled && self.runlevels.contains(&runlevel)
    }
}

/// Service registry
#[derive(Debug)]
pub struct ServiceRegistry {
    services: HashMap<String, Service>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Load services from init scripts directory
    pub fn load_from_dir(&mut self, dir: &PathBuf) -> Result<()> {
        if !dir.exists() {
            log::warn!("Init scripts directory does not exist: {:?}", dir);
            return Ok(());
        }
        
        let entries = std::fs::read_dir(dir)
            .context("Failed to read init scripts directory")?;
        
        for entry in entries {
            let entry = entry.context("Failed to read directory entry")?;
            let path = entry.path();
            
            if path.is_file() {
                // Check if executable
                if let Ok(metadata) = std::fs::metadata(&path) {
                    // On Unix, check if it's a regular file
                    if metadata.is_file() {
                        match Service::from_script(path) {
                            Ok(service) => {
                                log::debug!("Loaded service: {}", service.name);
                                self.services.insert(service.name.clone(), service);
                            }
                            Err(e) => {
                                log::warn!("Failed to load service from {:?}: {}", entry.path(), e);
                            }
                        }
                    }
                }
            }
        }
        
        log::info!("Loaded {} services", self.services.len());
        Ok(())
    }

    /// Get a service by name
    pub fn get(&self, name: &str) -> Option<&Service> {
        self.services.get(name)
    }

    /// Get a mutable service by name
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Service> {
        self.services.get_mut(name)
    }

    /// Get all services
    pub fn all(&self) -> Vec<&Service> {
        self.services.values().collect()
    }

    /// Get services for a specific runlevel
    pub fn for_runlevel(&self, runlevel: u8) -> Vec<&Service> {
        self.services
            .values()
            .filter(|s| s.should_run_in(runlevel))
            .collect()
    }

    /// Get service names
    pub fn names(&self) -> Vec<String> {
        self.services.keys().cloned().collect()
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
