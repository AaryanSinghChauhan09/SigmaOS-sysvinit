#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// SysVinit configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SysVinitConfig {
    /// Default runlevel (0-6)
    pub default_runlevel: u8,
    /// Directory containing init scripts
    pub init_scripts_dir: PathBuf,
    /// Directory for runlevel-specific scripts
    pub rc_dir: PathBuf,
    /// Directory for lock files
    pub lock_dir: PathBuf,
    /// Directory for PID files
    pub pid_dir: PathBuf,
    /// Service startup timeout in seconds
    pub startup_timeout: u64,
    /// Service shutdown timeout in seconds
    pub shutdown_timeout: u64,
}

impl Default for SysVinitConfig {
    fn default() -> Self {
        Self {
            default_runlevel: 3,
            init_scripts_dir: PathBuf::from("/etc/init.d"),
            rc_dir: PathBuf::from("/etc/rc.d"),
            lock_dir: PathBuf::from("/var/lock/subsys"),
            pid_dir: PathBuf::from("/var/run"),
            startup_timeout: 30,
            shutdown_timeout: 30,
        }
    }
}

impl SysVinitConfig {
    /// Load configuration from file or use defaults
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path();
        
        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .context("Failed to read config file")?;
            let config: Self = serde_json::from_str(&content)
                .context("Failed to parse config file")?;
            Ok(config)
        } else {
            // Create default config file
            let default = Self::default();
            default.save()?;
            Ok(default)
        }
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let config_path = Self::config_path();
        
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let content = serde_json::to_string_pretty(self)
            .context("Failed to serialize config")?;
        
        fs::write(&config_path, content)
            .context("Failed to write config file")?;
        
        Ok(())
    }

    /// Get the configuration file path
    fn config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("/etc"))
            .join("sigmaos")
            .join("sysvinit.json")
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        if self.default_runlevel > 6 {
            anyhow::bail!("Invalid default runlevel: {}. Must be 0-6", self.default_runlevel);
        }
        
        if self.startup_timeout == 0 {
            anyhow::bail!("Startup timeout must be greater than 0");
        }
        
        if self.shutdown_timeout == 0 {
            anyhow::bail!("Shutdown timeout must be greater than 0");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SysVinitConfig::default();
        assert_eq!(config.default_runlevel, 3);
        assert_eq!(config.startup_timeout, 30);
    }

    #[test]
    fn test_config_validation() {
        let mut config = SysVinitConfig::default();
        assert!(config.validate().is_ok());
        
        config.default_runlevel = 7;
        assert!(config.validate().is_err());
        
        config.default_runlevel = 3;
        config.startup_timeout = 0;
        assert!(config.validate().is_err());
    }
}
