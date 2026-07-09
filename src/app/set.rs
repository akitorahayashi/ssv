use crate::context::Context;
use crate::error::AppError;
use crate::ssh::host_config::{self, HostConfig};
use crate::ssh::naming::{self, ManagedKeyName};

pub(crate) fn execute(
    ctx: &Context,
    host: &str,
    hostname: Option<&str>,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    if hostname.is_none() && user.is_none() && port.is_none() {
        return Err(AppError::validation(
            "specify at least one of --hostname, --user, or --port to update",
        ));
    }
    naming::validate_host(host)?;
    if let Some(hostname) = hostname {
        naming::validate_hostname(hostname)?;
    }
    if let Some(user) = user {
        naming::validate_user(user)?;
    }
    let layout = ctx.layout();
    let config = host_config::load(layout, host)?;
    layout.require_host_identity(&config.identity, host)?;
    let key_type = naming::managed_key_type(&config.identity, host)?;
    let key_name = ManagedKeyName::new(&key_type, host)?;

    let new_hostname =
        hostname.map(str::to_string).or(config.hostname).unwrap_or_else(|| host.to_string());
    let new_user = user.map(str::to_string).or(config.user);
    let new_port = port.or(config.port);

    let rendered = HostConfig::render(&key_name, &new_hostname, new_user.as_deref(), new_port);
    host_config::write(&layout.host_config(host), &rendered)?;
    Ok(new_hostname)
}
