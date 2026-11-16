# SigmaOS-sysvinit

SysVinit init system compatibility layer for SigmaOS.

## Overview

This component provides a compatibility layer for SysVinit-style init systems, enabling SigmaOS to work with traditional SysVinit scripts, runlevels, and service management.

## Features

- **Service Management**: Start, stop, restart, and query status of SysVinit services
- **Runlevel Support**: Manage runlevels 0-6 with proper service transitions
- **Init Script Support**: Parse and execute LSB-compliant init scripts
- **Service Supervision**: Monitor and manage service lifecycles
- **Dependency Handling**: Respect service dependencies and start/stop ordering
- **Configuration**: Flexible configuration for script directories and timeouts

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
sigmaos-sysvinit = "0.1"
```

## Usage

### As a Library

```rust
use sigmaos_sysvinit::SysVinit;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let sysvinit = SysVinit::new().await?;
    
    // Initialize the system
    sysvinit.initialize().await?;
    
    // Start a service
    sysvinit.start_service("nginx").await?;
    
    // Change runlevel
    sysvinit.change_runlevel(5).await?;
    
    // Shutdown
    sysvinit.shutdown().await?;
    
    Ok(())
}
```

### As a CLI

```bash
# Initialize the system
sigmaos-sysvinit init

# Start a service
sigmaos-sysvinit start nginx

# Stop a service
sigmaos-sysvinit stop nginx

# Restart a service
sigmaos-sysvinit restart nginx

# Check service status
sigmaos-sysvinit status nginx

# List all services
sigmaos-sysvinit list

# Change runlevel
sigmaos-sysvinit runlevel 5

# Get current runlevel
sigmaos-sysvinit get-runlevel

# Shutdown
sigmaos-sysvinit shutdown
```

## Runlevels

- **0**: Halt
- **1**: Single-user mode
- **2**: Multi-user mode (without NFS)
- **3**: Full multi-user mode
- **4**: User-defined
- **5**: Graphical mode
- **6**: Reboot

## Configuration

Configuration is stored in `$CONFIG_DIR/sigmaos/sysvinit.json` (typically `~/.config/sigmaos/sysvinit.json`).

```json
{
  "default_runlevel": 3,
  "init_scripts_dir": "/etc/init.d",
  "rc_dir": "/etc/rc.d",
  "lock_dir": "/var/lock/subsys",
  "pid_dir": "/var/run",
  "startup_timeout": 30,
  "shutdown_timeout": 30
}
```

## Init Scripts

The component supports LSB-compliant init scripts with the following headers:

```bash
#!/bin/sh
### BEGIN INIT INFO
# Provides:          myservice
# Required-Start:    $network $syslog
# Required-Stop:     $network $syslog
# Default-Start:     2 3 4 5
# Default-Stop:      0 1 6
# Description:       My custom service
### END INIT INFO
```

## Architecture

- **config**: Configuration management
- **service**: Service definitions and registry
- **runlevel**: Runlevel management
- **script**: Init script parsing and execution
- **supervisor**: Service supervision and lifecycle management

## Safety

This crate uses `#![forbid(unsafe_code)]` to ensure memory safety.

## License

MIT

## Contributing

Contributions are welcome! Please open an issue or submit a pull request.
