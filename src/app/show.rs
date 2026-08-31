use crate::context::Context;
use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config;
use crate::ssh::naming::HostIdentifier;
use std::fs;

pub(crate) fn execute(ctx: &Context, host: &str) -> Result<String, AppError> {
    let host = HostIdentifier::new(host)?;
    let layout = ctx.layout();
    let config = host_config::load(layout, &host)?;
    layout.require_regular_file(&config.public_key)?;
    fs::read_to_string(&config.public_key).path_ctx(&config.public_key)
}
