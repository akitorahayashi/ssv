use crate::context::Context;
use crate::error::AppError;
use crate::ssh::host_config;
use crate::ssh::keycopy;
use crate::ssh::naming;

pub(crate) fn execute(ctx: &Context, host: &str) -> Result<String, AppError> {
    naming::validate_host(host)?;
    let layout = ctx.layout();
    let config = host_config::load(layout, host)?;
    layout.require_host_identity(&config.identity, host)?;
    let public = layout.public_key(&config.identity)?;
    layout.require_regular_file(&public)?;
    let hostname = config
        .hostname
        .ok_or_else(|| AppError::validation("managed host config has no HostName"))?;
    keycopy::install(ctx.copy_id(), &public, config.user.as_deref(), &hostname, config.port)?;
    Ok(keycopy::target(config.user.as_deref(), &hostname))
}
