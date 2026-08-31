use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GitOperation {
    DiscoverRepository,
    ReadOrigin,
    ParseOrigin,
    UpdateOrigin,
}

#[derive(Debug)]
pub enum AppError {
    Io {
        path: PathBuf,
        source: io::Error,
    },
    Environment(String),
    Validation(String),
    ManagedDocument(String),
    ManagedIdentity(String),
    HostNotFound(String),
    BootstrapRequired(PathBuf),
    OutsideManagedRoot(PathBuf),
    Git {
        operation: GitOperation,
        message: String,
    },
    ExternalCommand {
        operation: String,
        program: PathBuf,
        status: ExitStatus,
        stderr: Option<String>,
    },
    InvalidExternalOutput {
        operation: String,
        message: String,
    },
    Rollback {
        operation: String,
        primary: Box<AppError>,
        cleanup: Vec<AppError>,
    },
    CleanupFailed {
        primary: Box<AppError>,
        cleanup: Vec<AppError>,
    },
    CommittedIo {
        path: PathBuf,
        action: String,
        source: io::Error,
    },
}

impl Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(formatter, "{}: {source}", path.display()),
            Self::Environment(message)
            | Self::Validation(message)
            | Self::ManagedDocument(message)
            | Self::ManagedIdentity(message) => formatter.write_str(message),
            Self::HostNotFound(host) => write!(formatter, "Host '{host}' was not found"),
            Self::BootstrapRequired(path) => write!(
                formatter,
                "SSH bootstrap directory '{}' is missing; run `ssv init` first",
                path.display()
            ),
            Self::OutsideManagedRoot(path) => {
                write!(formatter, "Path '{}' is outside the managed SSH directory", path.display())
            }
            Self::Git { message, .. } => formatter.write_str(message),
            Self::ExternalCommand { operation, program, status, stderr } => {
                write!(
                    formatter,
                    "Command '{}' failed while {operation} with status {status}",
                    program.display()
                )?;
                if let Some(stderr) = stderr {
                    write!(formatter, ": {stderr}")?;
                }
                Ok(())
            }
            Self::InvalidExternalOutput { operation, message } => {
                write!(formatter, "Invalid output while {operation}: {message}")
            }
            Self::Rollback { operation, primary, cleanup } => {
                write!(
                    formatter,
                    "{operation} failed; operation-owned files were rolled back: {primary}"
                )?;
                write_cleanup(formatter, cleanup)
            }
            Self::CleanupFailed { primary, cleanup } => {
                write!(formatter, "{primary}")?;
                write_cleanup(formatter, cleanup)
            }
            Self::CommittedIo { path, action, source } => {
                write!(formatter, "{} was published but {action} failed: {source}", path.display())
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::CommittedIo { source, .. } => Some(source),
            Self::Rollback { primary, .. } | Self::CleanupFailed { primary, .. } => {
                Some(primary.as_ref())
            }
            Self::Environment(_)
            | Self::Validation(_)
            | Self::ManagedDocument(_)
            | Self::ManagedIdentity(_)
            | Self::HostNotFound(_)
            | Self::BootstrapRequired(_)
            | Self::OutsideManagedRoot(_)
            | Self::Git { .. }
            | Self::ExternalCommand { .. }
            | Self::InvalidExternalOutput { .. } => None,
        }
    }
}

impl AppError {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        Self::Io { path: path.to_path_buf(), source }
    }

    pub(crate) fn environment(message: impl Into<String>) -> Self {
        Self::Environment(message.into())
    }

    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation(message.into())
    }

    pub(crate) fn managed_document(message: impl Into<String>) -> Self {
        Self::ManagedDocument(message.into())
    }

    pub(crate) fn managed_identity(message: impl Into<String>) -> Self {
        Self::ManagedIdentity(message.into())
    }

    pub(crate) fn bootstrap_missing(path: PathBuf) -> Self {
        Self::BootstrapRequired(path)
    }

    pub(crate) fn git(operation: GitOperation, message: impl Into<String>) -> Self {
        Self::Git { operation, message: message.into() }
    }

    pub(crate) fn external_command(
        operation: &str,
        program: &Path,
        status: ExitStatus,
        stderr: Option<&[u8]>,
    ) -> Self {
        let stderr = stderr
            .map(String::from_utf8_lossy)
            .map(|stderr| stderr.trim().to_string())
            .filter(|stderr| !stderr.is_empty());
        Self::ExternalCommand {
            operation: operation.to_string(),
            program: program.to_path_buf(),
            status,
            stderr,
        }
    }

    pub(crate) fn invalid_external_output(operation: &str, message: impl Into<String>) -> Self {
        Self::InvalidExternalOutput { operation: operation.to_string(), message: message.into() }
    }

    pub(crate) fn rollback(operation: &str, primary: AppError, cleanup: Vec<AppError>) -> Self {
        Self::Rollback { operation: operation.to_string(), primary: Box::new(primary), cleanup }
    }

    pub(crate) fn with_cleanup(primary: AppError, cleanup: Vec<AppError>) -> Self {
        if cleanup.is_empty() {
            primary
        } else {
            Self::CleanupFailed { primary: Box::new(primary), cleanup }
        }
    }

    pub(crate) fn committed_io(path: &Path, action: &str, source: io::Error) -> Self {
        Self::CommittedIo { path: path.to_path_buf(), action: action.to_string(), source }
    }

    pub(crate) fn is_committed(&self) -> bool {
        matches!(self, Self::CommittedIo { .. })
    }
}

fn write_cleanup(formatter: &mut fmt::Formatter<'_>, cleanup: &[AppError]) -> fmt::Result {
    if cleanup.is_empty() {
        return Ok(());
    }
    formatter.write_str("; cleanup also failed")?;
    for error in cleanup {
        write!(formatter, ": {error}")?;
    }
    Ok(())
}

pub(crate) trait IoResultExt<T> {
    fn path_ctx(self, path: &Path) -> Result<T, AppError>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn path_ctx(self, path: &Path) -> Result<T, AppError> {
        self.map_err(|source| AppError::io(path, source))
    }
}
