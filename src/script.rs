#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::process::Command as TokioCommand;

/// Init script handler
pub struct InitScript {
    path: PathBuf,
}

impl InitScript {
    /// Create a new init script handler
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    /// Execute the script with a given action
    pub async fn execute(&self, action: &str) -> Result<ScriptOutput> {
        log::debug!("Executing {} for script {:?}", action, self.path);
        
        let output = TokioCommand::new(&self.path)
            .arg(action)
            .output()
            .await
            .context("Failed to execute init script")?;
        
        Ok(ScriptOutput {
            status: output.status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
        })
    }

    /// Start the service
    pub async fn start(&self) -> Result<ScriptOutput> {
        self.execute("start").await
    }

    /// Stop the service
    pub async fn stop(&self) -> Result<ScriptOutput> {
        self.execute("stop").await
    }

    /// Restart the service
    pub async fn restart(&self) -> Result<ScriptOutput> {
        self.execute("restart").await
    }

    /// Reload the service
    pub async fn reload(&self) -> Result<ScriptOutput> {
        self.execute("reload").await
    }

    /// Get service status
    pub async fn status(&self) -> Result<ScriptOutput> {
        self.execute("status").await
    }

    /// Check if the script supports an action
    pub async fn supports_action(&self, action: &str) -> bool {
        // Try to parse the script for the action
        if let Ok(content) = std::fs::read_to_string(&self.path) {
            let pattern = format!("{}()", action);
            content.contains(&pattern) || content.contains(&format!("case $1 in")) && content.contains(action)
        } else {
            false
        }
    }

    /// Get script metadata (LSB headers)
    pub fn get_metadata(&self) -> Result<ScriptMetadata> {
        let content = std::fs::read_to_string(&self.path)
            .context("Failed to read script")?;
        
        let mut metadata = ScriptMetadata::default();
        
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("### BEGIN INIT INFO") {
                break; // Stop at LSB block start
            }
            if line.starts_with("# Provides:") {
                metadata.provides = line
                    .trim_start_matches("# Provides:")
                    .trim()
                    .to_string();
            } else if line.starts_with("# Required-Start:") {
                metadata.required_start = line
                    .trim_start_matches("# Required-Start:")
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if line.starts_with("# Required-Stop:") {
                metadata.required_stop = line
                    .trim_start_matches("# Required-Stop:")
                    .trim()
                    .split_whitespace()
                    .map(|s| s.to_string())
                    .collect();
            } else if line.starts_with("# Default-Start:") {
                metadata.default_start = line
                    .trim_start_matches("# Default-Start:")
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
            } else if line.starts_with("# Default-Stop:") {
                metadata.default_stop = line
                    .trim_start_matches("# Default-Stop:")
                    .trim()
                    .split_whitespace()
                    .filter_map(|s| s.parse().ok())
                    .collect();
            } else if line.starts_with("# Description:") {
                metadata.description = line
                    .trim_start_matches("# Description:")
                    .trim()
                    .to_string();
            }
        }
        
        Ok(metadata)
    }
}

/// Script execution output
#[derive(Debug)]
pub struct ScriptOutput {
    pub status: std::process::ExitStatus,
    pub stdout: String,
    pub stderr: String,
}

impl ScriptOutput {
    /// Check if the script succeeded
    pub fn success(&self) -> bool {
        self.status.success()
    }
}

/// Script metadata (LSB headers)
#[derive(Debug, Default, Clone)]
pub struct ScriptMetadata {
    pub provides: String,
    pub required_start: Vec<String>,
    pub required_stop: Vec<String>,
    pub default_start: Vec<u8>,
    pub default_stop: Vec<u8>,
    pub description: String,
}

/// Validate that a script is a valid init script
pub fn validate_init_script(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        anyhow::bail!("Script does not exist: {:?}", path);
    }
    
    let metadata = std::fs::metadata(path)
        .context("Failed to read script metadata")?;
    
    if !metadata.is_file() {
        anyhow::bail!("Path is not a file: {:?}", path);
    }
    
    // Check if it's executable (on Unix systems)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        if mode & 0o111 == 0 {
            log::warn!("Script is not executable: {:?}", path);
        }
    }
    
    // Try to read and parse the script
    let content = std::fs::read_to_string(path)
        .context("Failed to read script content")?;
    
    // Basic checks for init script structure
    if !content.contains("case") && !content.contains("start") {
        log::warn!("Script may not be a valid init script: {:?}", path);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_metadata_default() {
        let metadata = ScriptMetadata::default();
        assert!(metadata.provides.is_empty());
        assert!(metadata.required_start.is_empty());
    }
}
