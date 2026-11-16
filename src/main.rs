#![forbid(unsafe_code)]

use anyhow::Result;
use clap::{Parser, Subcommand};
use log::info;
use sigmaos_sysvinit::SysVinit;

#[derive(Parser)]
#[command(name = "sigmaos-sysvinit")]
#[command(about = "SysVinit compatibility layer for SigmaOS", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the SysVinit system
    Init,
    /// Shutdown the SysVinit system
    Shutdown,
    /// Change runlevel
    Runlevel {
        /// Target runlevel (0-6)
        runlevel: u8,
    },
    /// Get current runlevel
    GetRunlevel,
    /// Start a service
    Start {
        /// Service name
        name: String,
    },
    /// Stop a service
    Stop {
        /// Service name
        name: String,
    },
    /// Restart a service
    Restart {
        /// Service name
        name: String,
    },
    /// Get service status
    Status {
        /// Service name
        name: String,
    },
    /// List all services
    List,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let cli = Cli::parse();
    let sysvinit = SysVinit::new().await?;
    
    match cli.command {
        Commands::Init => {
            sysvinit.initialize().await?;
            println!("SysVinit initialized successfully");
        }
        Commands::Shutdown => {
            sysvinit.shutdown().await?;
            println!("SysVinit shutdown successfully");
        }
        Commands::Runlevel { runlevel } => {
            if runlevel > 6 {
                anyhow::bail!("Invalid runlevel: {}. Must be 0-6", runlevel);
            }
            sysvinit.change_runlevel(runlevel).await?;
            println!("Changed to runlevel {}", runlevel);
        }
        Commands::GetRunlevel => {
            let current = sysvinit.current_runlevel().await?;
            println!("Current runlevel: {}", current);
        }
        Commands::Start { name } => {
            sysvinit.start_service(&name).await?;
            println!("Service '{}' started successfully", name);
        }
        Commands::Stop { name } => {
            sysvinit.stop_service(&name).await?;
            println!("Service '{}' stopped successfully", name);
        }
        Commands::Restart { name } => {
            sysvinit.restart_service(&name).await?;
            println!("Service '{}' restarted successfully", name);
        }
        Commands::Status { name } => {
            let status = sysvinit.service_status(&name).await?;
            println!("Service '{}' status: {:?}", name, status);
        }
        Commands::List => {
            let services = sysvinit.list_services().await?;
            println!("Available services:");
            for service in services {
                println!("  - {}", service);
            }
        }
    }
    
    Ok(())
}
