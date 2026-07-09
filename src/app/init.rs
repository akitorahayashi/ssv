use crate::context::Context;
use crate::error::AppError;
use crate::ssh::bootstrap::{self, BootstrapStatus};

pub(crate) fn execute(ctx: &Context) -> Result<BootstrapStatus, AppError> {
    bootstrap::ensure_bootstrap(ctx.layout())
}
