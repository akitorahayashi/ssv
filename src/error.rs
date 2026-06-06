use std::error::Error;
use std::fmt::{self, Display};
use std::io;
use std::path::PathBuf;
use std::process::ExitStatus;

/// Library-wide error type capturing filesystem, validation, and command execution failures.
#[derive(Debug)]
pub enum AppError {
    Io(io::Error),
    /// Configuration or environment issue that prevents command execution.
    ConfigError(String),
    /// Raised when a requested host cannot be located in managed assets.
    HostNotFound(String),
    /// Raised when managed SSH bootstrap directories have not been initialized yet.
    BootstrapRequired(PathBuf),
    /// Indicates a validation problem with user-provided arguments or derived data.
    ValidationError(String),
    /// Indicates a path outside the directory owned by ssv.
    OutsideManagedRoot(PathBuf),
    /// A spawned command exited with a non-zero status code.
    CommandFailed {
        program: String,
        status: ExitStatus,
    },
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Io(err) => write!(f, "{}", err),
            AppError::ConfigError(message) => write!(f, "{message}"),
            AppError::HostNotFound(host) => write!(f, "Host '{host}' was not found"),
            AppError::BootstrapRequired(path) => {
                write!(
                    f,
                    "SSH bootstrap directory '{}' is missing; run `ssv init` first",
                    path.display()
                )
            }
            AppError::ValidationError(message) => write!(f, "{message}"),
            AppError::OutsideManagedRoot(path) => {
                write!(f, "Path '{}' is outside the managed SSH directory", path.display())
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
            AppError::Io(err) => Some(err),
            AppError::ConfigError(_)
            | AppError::HostNotFound(_)
            | AppError::BootstrapRequired(_)
            | AppError::ValidationError(_)
            | AppError::OutsideManagedRoot(_)
            | AppError::CommandFailed { .. } => None,
        }
    }
}

impl From<io::Error> for AppError {
    fn from(value: io::Error) -> Self {
        AppError::Io(value)
    }
}

impl AppError {
    pub(crate) fn config<S: Into<String>>(message: S) -> Self {
        AppError::ConfigError(message.into())
    }

    pub(crate) fn validation<S: Into<String>>(message: S) -> Self {
        AppError::ValidationError(message.into())
    }

    pub(crate) fn bootstrap_missing(path: PathBuf) -> Self {
        AppError::BootstrapRequired(path)
    }

    pub(crate) fn command_failed(program: &str, status: ExitStatus) -> Self {
        AppError::CommandFailed { program: program.to_string(), status }
    }
}
