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
pub fn generate(
    host: &str,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    app::generate::execute(host, key_type, user, port)
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

/// Inspect managed SSH assets without modifying them.
pub fn audit() -> Result<AuditReport, AppError> {
    app::audit::execute()
}
