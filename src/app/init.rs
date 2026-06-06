use crate::error::AppError;
use crate::ssh::layout::{BootstrapStatus, Layout};

pub(crate) fn execute() -> Result<BootstrapStatus, AppError> {
    Layout::from_env()?.ensure_bootstrap()
}
