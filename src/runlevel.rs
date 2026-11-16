#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

/// Runlevel file path
const RUNLEVEL_FILE: &str = "/var/run/sigmaos-runlevel";

/// Set the current runlevel
pub async fn set_runlevel(runlevel: u8) -> Result<()> {
    if runlevel > 6 {
        anyhow::bail!("Invalid runlevel: {}. Must be 0-6", runlevel);
    }
    
    log::info!("Setting runlevel to {}", runlevel);
    
    // Ensure parent directory exists
    if let Some(parent) = PathBuf::from(RUNLEVEL_FILE).parent() {
        fs::create_dir_all(parent)
            .context("Failed to create runlevel directory")?;
    }
    
    fs::write(RUNLEVEL_FILE, runlevel.to_string())
        .context("Failed to write runlevel file")?;
    
    Ok(())
}

/// Get the current runlevel
pub async fn get_runlevel() -> Result<u8> {
    if !PathBuf::from(RUNLEVEL_FILE).exists() {
        // Default to runlevel 3 if file doesn't exist
        return Ok(3);
    }
    
    let content = fs::read_to_string(RUNLEVEL_FILE)
        .context("Failed to read runlevel file")?;
    
    let runlevel: u8 = content
        .trim()
        .parse()
        .context("Failed to parse runlevel")?;
    
    Ok(runlevel)
}

/// Get runlevel name
pub fn runlevel_name(runlevel: u8) -> &'static str {
    match runlevel {
        0 => "Halt",
        1 => "Single-user",
        2 => "Multi-user (no NFS)",
        3 => "Multi-user",
        4 => "User-defined",
        5 => "Graphical",
        6 => "Reboot",
        _ => "Unknown",
    }
}

/// Check if runlevel is valid
pub fn is_valid_runlevel(runlevel: u8) -> bool {
    runlevel <= 6
}

/// Check if runlevel is operational (not halt or reboot)
pub fn is_operational_runlevel(runlevel: u8) -> bool {
    matches!(runlevel, 1 | 2 | 3 | 4 | 5)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runlevel_names() {
        assert_eq!(runlevel_name(0), "Halt");
        assert_eq!(runlevel_name(3), "Multi-user");
        assert_eq!(runlevel_name(5), "Graphical");
        assert_eq!(runlevel_name(6), "Reboot");
    }

    #[test]
    fn test_valid_runlevels() {
        assert!(is_valid_runlevel(0));
        assert!(is_valid_runlevel(3));
        assert!(is_valid_runlevel(6));
        assert!(!is_valid_runlevel(7));
    }

    #[test]
    fn test_operational_runlevels() {
        assert!(!is_operational_runlevel(0));
        assert!(is_operational_runlevel(1));
        assert!(is_operational_runlevel(3));
        assert!(is_operational_runlevel(5));
        assert!(!is_operational_runlevel(6));
    }
}
