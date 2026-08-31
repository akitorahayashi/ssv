//! Library entry point exposing the application operations for `ssv`.

// ssv manages Unix file permissions on private keys as a security-critical invariant. It has no
// correct behavior on platforms without Unix permission semantics, so it is Unix-only by design.
#[cfg(not(unix))]
compile_error!("ssv requires a Unix platform");

mod app;
mod cli;
mod context;
pub mod error;
mod ssh;

pub use app::audit::{AuditCode, AuditFinding, AuditReport, AuditSeverity};
pub use app::remove::RemovalStatus;
pub use cli::run as cli;
pub use context::Context;
pub use error::{AppError, GitOperation};
pub use ssh::bootstrap::BootstrapStatus;
