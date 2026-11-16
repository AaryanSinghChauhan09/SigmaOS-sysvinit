#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use crate::config::SysVinitConfig;
use crate::service::{Service, ServiceRegistry, ServiceStatus};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Service supervisor
pub struct ServiceSupervisor {
    config: Arc<RwLock<SysVinitConfig>>,
    registry: Arc<RwLock<ServiceRegistry>>,
}

impl ServiceSupervisor {
    /// Create a new service supervisor
    pub async fn new(config: Arc<RwLock<SysVinitConfig>>) -> Result<Self> {
        let config_read = config.read().await;
        let mut registry = ServiceRegistry::new();
        registry.load_from_dir(&config_read.init_scripts_dir)?;
        
        Ok(Self {
            config,
            registry: Arc::new(RwLock::new(registry)),
        })
    }

    /// Start all services for the current runlevel
    pub async fn start_all_services(&self) -> Result<()> {
        let config = self.config.read().await;
        let runlevel = config.default_runlevel;
        drop(config);
        
        self.start_runlevel_services(runlevel).await
    }

    /// Start services for a specific runlevel
    pub async fn start_runlevel_services(&self, runlevel: u8) -> Result<()> {
        log::info!("Starting services for runlevel {}", runlevel);
        
        let mut registry = self.registry.write().await;
        let services = registry.for_runlevel(runlevel);
        
        // Sort by start priority
        let mut sorted_services: Vec<_> = services.iter().collect();
        sorted_services.sort_by_key(|s| s.start_priority);
        
        for service_ref in sorted_services {
            let name = service_ref.name.clone();
            if let Some(service) = registry.get_mut(&name) {
                if service.status != ServiceStatus::Running {
                    if let Err(e) = service.start().await {
                        log::error!("Failed to start service {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Stop services for a specific runlevel
    pub async fn stop_runlevel_services(&self, runlevel: u8) -> Result<()> {
        log::info!("Stopping services for runlevel {}", runlevel);
        
        let mut registry = self.registry.write().await;
        let services = registry.for_runlevel(runlevel);
        
        // Sort by stop priority (reverse order of start)
        let mut sorted_services: Vec<_> = services.iter().collect();
        sorted_services.sort_by_key(|s| std::cmp::Reverse(s.stop_priority));
        
        for service_ref in sorted_services {
            let name = service_ref.name.clone();
            if let Some(service) = registry.get_mut(&name) {
                if service.status == ServiceStatus::Running {
                    if let Err(e) = service.stop().await {
                        log::error!("Failed to stop service {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Stop all services
    pub async fn stop_all_services(&self) -> Result<()> {
        log::info!("Stopping all services");
        
        let mut registry = self.registry.write().await;
        let services = registry.all();
        
        // Sort by stop priority
        let mut sorted_services: Vec<_> = services.iter().collect();
        sorted_services.sort_by_key(|s| std::cmp::Reverse(s.stop_priority));
        
        for service_ref in sorted_services {
            let name = service_ref.name.clone();
            if let Some(service) = registry.get_mut(&name) {
                if service.status == ServiceStatus::Running {
                    if let Err(e) = service.stop().await {
                        log::error!("Failed to stop service {}: {}", name, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Start a specific service
    pub async fn start_service(&self, name: &str) -> Result<()> {
        let mut registry = self.registry.write().await;
        
        if let Some(service) = registry.get_mut(name) {
            service.start().await?;
            Ok(())
        } else {
            anyhow::bail!("Service not found: {}", name);
        }
    }

    /// Stop a specific service
    pub async fn stop_service(&self, name: &str) -> Result<()> {
        let mut registry = self.registry.write().await;
        
        if let Some(service) = registry.get_mut(name) {
            service.stop().await?;
            Ok(())
        } else {
            anyhow::bail!("Service not found: {}", name);
        }
    }

    /// Restart a specific service
    pub async fn restart_service(&self, name: &str) -> Result<()> {
        let mut registry = self.registry.write().await;
        
        if let Some(service) = registry.get_mut(name) {
            service.restart().await?;
            Ok(())
        } else {
            anyhow::bail!("Service not found: {}", name);
        }
    }

    /// Get service status
    pub async fn service_status(&self, name: &str) -> Result<ServiceStatus> {
        let mut registry = self.registry.write().await;
        
        if let Some(service) = registry.get_mut(name) {
            service.check_status().await?;
            Ok(service.status)
        } else {
            anyhow::bail!("Service not found: {}", name);
        }
    }

    /// List all services
    pub async fn list_services(&self) -> Result<Vec<String>> {
        let registry = self.registry.read().await;
        Ok(registry.names())
    }

    /// Reload service registry
    pub async fn reload_registry(&self) -> Result<()> {
        let config = self.config.read().await;
        let mut registry = self.registry.write().await;
        
        *registry = ServiceRegistry::new();
        registry.load_from_dir(&config.init_scripts_dir)?;
        
        log::info!("Reloaded service registry");
        Ok(())
    }
}
