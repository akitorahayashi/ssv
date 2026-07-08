//! Library entry point exposing the application operations for `ssv`.

mod app;
mod cli;
pub mod error;
mod ssh;

pub use app::audit::{AuditCode, AuditFinding, AuditReport, AuditSeverity};
pub use app::remove::RemovalStatus;
pub use cli::run as cli;
pub use error::AppError;
pub use ssh::layout::BootstrapStatus;

/// Ensure the SSH bootstrap required for managed host configs exists.
pub fn init() -> Result<BootstrapStatus, AppError> {
    app::init::execute()
}

/// Generate a new SSH key pair and configuration for the provided host.
///
/// `hostname` is an optional override for the SSH `HostName` directive. When `None`, the
/// `HostName` in the generated config defaults to `host`. The key pair and config file are
/// always named after `host`, regardless of the `hostname` override.
pub fn generate(
    host: &str,
    hostname: Option<&str>,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    app::generate::execute(host, hostname, key_type, user, port)
}

/// List all managed hosts underneath ~/.ssh/conf.d.
pub fn list() -> Result<Vec<String>, AppError> {
    app::list::execute()
}

/// Remove the key pair and configuration associated with a host.
pub fn remove(host: &str) -> Result<RemovalStatus, AppError> {
    app::remove::execute(host)
}

/// Return the public key associated with a managed host.
pub fn show(host: &str) -> Result<String, AppError> {
    app::show::execute(host)
}

/// Link a repository to a managed host.
pub fn link(host: &str) -> Result<String, AppError> {
    app::link::execute(host)
}

/// Install a managed host's public key on the remote server via `ssh-copy-id`.
///
/// The connection target (user, hostname, port) is read from the managed host config, so the
/// public key path, credentials, and port never need to be retyped. Returns the resolved target.
pub fn authorize(host: &str) -> Result<String, AppError> {
    app::authorize::execute(host)
}

/// Update the `HostName`, user, or port of an existing managed host without regenerating keys.
///
/// Unspecified directives keep their current values. Returns the resulting `HostName`.
pub fn set(
    host: &str,
    hostname: Option<&str>,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    app::set::execute(host, hostname, user, port)
}

/// Inspect managed SSH assets without modifying them.
pub fn audit() -> Result<AuditReport, AppError> {
    app::audit::execute()
}
