//! Library entry point exposing the core command handlers for `ssv`.

mod commands;
pub mod error;
mod ssh_paths;

use commands::{
    generate_host::GenerateHost, init::Init, list_hosts::ListHosts, remove_host::RemoveHost,
};
use error::AppError;
use ssh_paths::SshPaths;

/// Ensure the SSH bootstrap required for managed host configs exists.
pub fn init() -> Result<(), AppError> {
    let paths = SshPaths::from_env()?;
    let command = Init;
    command.execute(&paths)
}

/// Generate a new SSH key pair and configuration for the provided host.
pub fn generate(
    host: &str,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    let paths = SshPaths::from_env()?;
    paths.ensure_bootstrap()?;
    let command = GenerateHost { host, key_type, user, port };
    command.execute(&paths)
}

/// List all managed hosts underneath ~/.ssh/conf.d.
pub fn list() -> Result<Vec<String>, AppError> {
    let paths = SshPaths::from_env()?;
    paths.ensure_base_dirs()?;

    let command = ListHosts;
    command.execute(&paths)
}

/// Remove the key pair and configuration associated with a host.
pub fn remove(host: &str) -> Result<(), AppError> {
    let paths = SshPaths::from_env()?;
    let command = RemoveHost { host };
    command.execute(&paths)?;

    println!("🗑️  Removed SSH assets for '{host}'");
    Ok(())
}
