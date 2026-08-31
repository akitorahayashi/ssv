use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

/// Library-wide error type capturing filesystem, validation, and command execution failures.
#[derive(Debug)]
pub enum AppError {
    /// A filesystem or subprocess I/O failure, tagged with the path (or program) it occurred on.
    Io { path: PathBuf, source: io::Error },
    /// Configuration or environment issue that prevents command execution.
    ConfigError(String),
    /// Raised when a requested host cannot be located in managed assets.
    HostNotFound(String),
    /// Raised when managed SSH bootstrap directories have not been initialized yet.
    BootstrapRequired(PathBuf),
    /// Indicates that a command rolled back partial SSH assets after a failure.
    RolledBack(Box<AppError>),
    /// Indicates that cleanup also failed while handling a primary failure.
    CleanupFailed { primary: Box<AppError>, cleanup: Vec<AppError> },
    /// Indicates a validation problem with user-provided arguments or derived data.
    ValidationError(String),
    /// Indicates a path outside the directory owned by ssv.
    OutsideManagedRoot(PathBuf),
    /// Indicates that a managed document references an identity not owned by its host.
    UnmanagedIdentity(String),
    /// Indicates that a pathname was published but its final durability step failed.
    CommittedIo { path: PathBuf, action: String, source: io::Error },
    /// A spawned command exited with a non-zero status code.
    CommandFailed { program: String, status: ExitStatus },
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io { path, source } => write!(f, "{}: {source}", path.display()),
            AppError::ConfigError(message) => write!(f, "{message}"),
            AppError::HostNotFound(host) => write!(f, "Host '{host}' was not found"),
            AppError::BootstrapRequired(path) => {
                write!(
                    f,
                    "SSH bootstrap directory '{}' is missing; run `ssv init` first",
                    path.display()
                )
            }
            AppError::RolledBack(error) => {
                write!(f, "Rolled back partial SSH assets due to failure: {error}")
            }
            AppError::CleanupFailed { primary, cleanup } => {
                write!(f, "{primary}; cleanup also failed")?;
                for error in cleanup {
                    write!(f, ": {error}")?;
                }
                Ok(())
            }
            AppError::ValidationError(message) => write!(f, "{message}"),
            AppError::OutsideManagedRoot(path) => {
                write!(f, "Path '{}' is outside the managed SSH directory", path.display())
            }
            AppError::UnmanagedIdentity(message) => write!(f, "{message}"),
            AppError::CommittedIo { path, action, source } => {
                write!(f, "{} was published but {action} failed: {source}", path.display())
            }
            AppError::CommandFailed { program, status } => {
                write!(f, "Command '{program}' exited with status {status}")
            }
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Io { source, .. } => Some(source),
            AppError::RolledBack(error) => Some(error.as_ref()),
            AppError::CleanupFailed { primary, .. } => Some(primary.as_ref()),
            AppError::CommittedIo { source, .. } => Some(source),
            AppError::ConfigError(_)
            | AppError::HostNotFound(_)
            | AppError::BootstrapRequired(_)
            | AppError::ValidationError(_)
            | AppError::OutsideManagedRoot(_)
            | AppError::UnmanagedIdentity(_)
            | AppError::CommandFailed { .. } => None,
        }
    }
}

impl AppError {
    pub(crate) fn io(path: &Path, source: io::Error) -> Self {
        AppError::Io { path: path.to_path_buf(), source }
    }

    pub(crate) fn config<S: Into<String>>(message: S) -> Self {
        AppError::ConfigError(message.into())
    }

    pub(crate) fn validation<S: Into<String>>(message: S) -> Self {
        AppError::ValidationError(message.into())
    }

    pub(crate) fn bootstrap_missing(path: PathBuf) -> Self {
        AppError::BootstrapRequired(path)
    }

    pub(crate) fn unmanaged_identity<S: Into<String>>(message: S) -> Self {
        AppError::UnmanagedIdentity(message.into())
    }

    pub(crate) fn rolled_back(error: AppError) -> Self {
        AppError::RolledBack(Box::new(error))
    }

    pub(crate) fn with_cleanup(primary: AppError, cleanup: Vec<AppError>) -> Self {
        if cleanup.is_empty() {
            primary
        } else {
            AppError::CleanupFailed { primary: Box::new(primary), cleanup }
        }
    }

    pub(crate) fn committed_io(path: &Path, action: &str, source: io::Error) -> Self {
        AppError::CommittedIo { path: path.to_path_buf(), action: action.to_string(), source }
    }

    pub(crate) fn is_committed(&self) -> bool {
        matches!(self, AppError::CommittedIo { .. })
    }

    pub(crate) fn command_failed(program: &str, status: ExitStatus) -> Self {
        AppError::CommandFailed { program: program.to_string(), status }
    }
}

/// Attach a path to an `io::Result`, converting its error into `AppError::Io`. Replaces the
/// blanket `From<io::Error>` conversion so no I/O failure reaches the user without a path.
pub(crate) trait IoResultExt<T> {
    fn path_ctx(self, path: &Path) -> Result<T, AppError>;
}

impl<T> IoResultExt<T> for io::Result<T> {
    fn path_ctx(self, path: &Path) -> Result<T, AppError> {
        self.map_err(|source| AppError::io(path, source))
    }
}
