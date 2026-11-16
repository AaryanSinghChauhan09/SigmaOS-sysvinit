#![forbid(unsafe_code)]

pub mod config;
pub mod service;
pub mod runlevel;
pub mod script;
pub mod supervisor;

use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main SysVinit compatibility layer structure
#[derive(Clone)]
pub struct SysVinit {
    config: Arc<RwLock<config::SysVinitConfig>>,
    supervisor: supervisor::ServiceSupervisor,
}

impl SysVinit {
    /// Create a new SysVinit compatibility layer instance
    pub async fn new() -> Result<Self> {
        let config = config::SysVinitConfig::load()?;
        let supervisor = supervisor::ServiceSupervisor::new(config.clone()).await?;
        
        Ok(Self {
            config: Arc::new(RwLock::new(config)),
            supervisor,
        })
    }

    /// Initialize the SysVinit system
    pub async fn initialize(&self) -> Result<()> {
        log::info!("Initializing SysVinit compatibility layer");
        
        let config = self.config.read().await;
        runlevel::set_runlevel(config.default_runlevel).await?;
        
        self.supervisor.start_all_services().await?;
        
        log::info!("SysVinit initialization complete");
        Ok(())
    }

    /// Shutdown the SysVinit system
    pub async fn shutdown(&self) -> Result<()> {
        log::info!("Shutting down SysVinit compatibility layer");
        
        self.supervisor.stop_all_services().await?;
        
        log::info!("SysVinit shutdown complete");
        Ok(())
    }

    /// Get the current runlevel
    pub async fn current_runlevel(&self) -> Result<u8> {
        runlevel::get_runlevel().await
    }

    /// Change to a different runlevel
    pub async fn change_runlevel(&self, runlevel: u8) -> Result<()> {
        log::info!("Changing to runlevel {}", runlevel);
        
        let current = self.current_runlevel().await?;
        if current == runlevel {
            log::info!("Already at runlevel {}", runlevel);
            return Ok(());
        }

        // Stop services for current runlevel
        self.supervisor.stop_runlevel_services(current).await?;
        
        // Start services for new runlevel
        self.supervisor.start_runlevel_services(runlevel).await?;
        
        runlevel::set_runlevel(runlevel).await?;
        
        log::info!("Changed to runlevel {}", runlevel);
        Ok(())
    }

    /// Start a specific service
    pub async fn start_service(&self, name: &str) -> Result<()> {
        log::info!("Starting service: {}", name);
        self.supervisor.start_service(name).await
    }

    /// Stop a specific service
    pub async fn stop_service(&self, name: &str) -> Result<()> {
        log::info!("Stopping service: {}", name);
        self.supervisor.stop_service(name).await
    }

    /// Restart a specific service
    pub async fn restart_service(&self, name: &str) -> Result<()> {
        log::info!("Restarting service: {}", name);
        self.supervisor.restart_service(name).await
    }

    /// Get service status
    pub async fn service_status(&self, name: &str) -> Result<service::ServiceStatus> {
        self.supervisor.service_status(name).await
    }

    /// List all services
    pub async fn list_services(&self) -> Result<Vec<String>> {
        self.supervisor.list_services().await
    }
}

impl Default for SysVinit {
    fn default() -> Self {
        // Note: This is a convenience constructor that may fail
        // In production, use new() which returns Result
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(Self::new()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sysvinit_creation() {
        let sysvinit = SysVinit::new().await;
        assert!(sysvinit.is_ok());
    }
}
