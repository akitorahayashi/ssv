use crate::error::AppError;
use crate::ssh::bootstrap::{self, BootstrapStatus};
use crate::ssh::layout::Layout;

pub(crate) fn execute() -> Result<BootstrapStatus, AppError> {
    bootstrap::ensure_bootstrap(&Layout::from_env()?)
}
