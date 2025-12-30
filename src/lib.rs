//! Library entry point exposing the core command handlers for `ssv`.

mod commands;
pub mod error;
mod ssh_paths;

use commands::{generate_host::GenerateHost, list_hosts::ListHosts, remove_host::RemoveHost};
use error::AppError;
use ssh_paths::SshPaths;

/// Generate a new SSH key pair and configuration for the provided host.
pub fn generate(
    host: &str,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<(), AppError> {
    let paths = SshPaths::from_env()?;
    let command = GenerateHost { host, key_type, user, port };
    let public_key = command.execute(&paths)?;

    println!("✅ Generated SSH assets for '{host}'");
    println!("{public_key}");
    Ok(())
}

/// List all managed hosts underneath ~/.ssh/conf.d.
pub fn list() -> Result<Vec<String>, AppError> {
    let paths = SshPaths::from_env()?;
    paths.ensure_base_dirs()?;

    let command = ListHosts;
    let hosts = command.execute(&paths)?;

    if hosts.is_empty() {
        println!("(no hosts managed yet)");
    } else {
        for host in &hosts {
            println!("{host}");
        }
    }

    Ok(hosts)
}

/// Remove the key pair and configuration associated with a host.
pub fn remove(host: &str) -> Result<(), AppError> {
    let paths = SshPaths::from_env()?;
    let command = RemoveHost { host };
    command.execute(&paths)?;

    println!("🗑️  Removed SSH assets for '{host}'");
    Ok(())
}
