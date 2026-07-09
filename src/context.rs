use crate::app;
use crate::app::audit::AuditReport;
use crate::app::remove::RemovalStatus;
use crate::error::AppError;
use crate::ssh::bootstrap::BootstrapStatus;
use crate::ssh::layout::Layout;
use std::path::{Path, PathBuf};

/// The resolved runtime environment for every operation: the managed-root layout derived from a
/// home directory, plus the external tool binaries (`ssh-keygen`, `ssh-copy-id`).
///
/// The CLI builds one via [`Context::from_env`]; tests build one via [`Context::new`] pointing at
/// a temporary home and stub binaries. Threading it explicitly keeps the environment out of
/// process-global state, so operations are isolated and tests need no global mutation.
pub struct Context {
    layout: Layout,
    keygen: PathBuf,
    copy_id: PathBuf,
}

impl Context {
    /// Resolve the context from the process environment: `HOME` for the managed root and the
    /// `SSV_SSH_KEYGEN_PATH` / `SSV_SSH_COPY_ID_PATH` overrides for the external tools.
    pub fn from_env() -> Result<Self, AppError> {
        Ok(Self {
            layout: Layout::from_env()?,
            keygen: tool_path("SSV_SSH_KEYGEN_PATH", "ssh-keygen"),
            copy_id: tool_path("SSV_SSH_COPY_ID_PATH", "ssh-copy-id"),
        })
    }

    /// Build a context from explicit paths, reading no environment variables.
    pub fn new(home: PathBuf, keygen: PathBuf, copy_id: PathBuf) -> Self {
        Self { layout: Layout::from_home(home), keygen, copy_id }
    }

    pub(crate) fn layout(&self) -> &Layout {
        &self.layout
    }

    pub(crate) fn keygen(&self) -> &Path {
        &self.keygen
    }

    pub(crate) fn copy_id(&self) -> &Path {
        &self.copy_id
    }

    /// Ensure the SSH bootstrap required for managed host configs exists.
    pub fn init(&self) -> Result<BootstrapStatus, AppError> {
        app::init::execute(self)
    }

    /// Generate a new SSH key pair and configuration for `host`.
    ///
    /// `hostname` overrides the SSH `HostName`; when `None` it defaults to `host`. The key pair
    /// and config file are always named after `host`.
    pub fn generate(
        &self,
        host: &str,
        hostname: Option<&str>,
        key_type: &str,
        user: Option<&str>,
        port: Option<u16>,
    ) -> Result<String, AppError> {
        app::generate::execute(self, host, hostname, key_type, user, port)
    }

    /// List all managed hosts underneath `~/.ssh/conf.d`.
    pub fn list(&self) -> Result<Vec<String>, AppError> {
        app::list::execute(self)
    }

    /// Remove the key pair and configuration associated with `host`.
    pub fn remove(&self, host: &str) -> Result<RemovalStatus, AppError> {
        app::remove::execute(self, host)
    }

    /// Return the public key associated with a managed host.
    pub fn show(&self, host: &str) -> Result<String, AppError> {
        app::show::execute(self, host)
    }

    /// Rewrite the current repository's `origin` remote to the managed host.
    pub fn link(&self, host: &str) -> Result<String, AppError> {
        app::link::execute(self, host)
    }

    /// Install a managed host's public key on the remote server via `ssh-copy-id`.
    pub fn authorize(&self, host: &str) -> Result<String, AppError> {
        app::authorize::execute(self, host)
    }

    /// Update the `HostName`, user, or port of an existing managed host without regenerating keys.
    pub fn set(
        &self,
        host: &str,
        hostname: Option<&str>,
        user: Option<&str>,
        port: Option<u16>,
    ) -> Result<String, AppError> {
        app::set::execute(self, host, hostname, user, port)
    }

    /// Inspect managed SSH assets without modifying them.
    pub fn audit(&self) -> Result<AuditReport, AppError> {
        app::audit::execute(self)
    }
}

fn tool_path(var: &str, default: &str) -> PathBuf {
    std::env::var_os(var).map(PathBuf::from).unwrap_or_else(|| PathBuf::from(default))
}
