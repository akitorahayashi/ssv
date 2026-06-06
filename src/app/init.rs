use crate::error::AppError;
use crate::ssh::layout::Layout;

pub(crate) fn execute() -> Result<(), AppError> {
    Layout::from_env()?.ensure_bootstrap()
}
