use crate::error::AppError;
use crate::ssh::host_config;
use crate::ssh::layout::Layout;
use crate::ssh::naming::{HostIdentifier, Hostname, ManagedKeyName, RemoteUser};

pub(crate) fn execute(
    layout: &Layout,
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
    let host = HostIdentifier::new(host)?;
    let hostname = hostname.map(Hostname::new).transpose()?;
    let user = user.map(RemoteUser::new).transpose()?;
    let config = host_config::load(layout, &host)?;
    let key_type = crate::ssh::naming::managed_key_type(&config.private_key, &host)?;
    let key_name = ManagedKeyName::new(&key_type, host.clone())?;

    let new_hostname = hostname.unwrap_or(config.hostname);
    let new_user = user.or(config.user);
    let new_port = port.or(config.port);

    let rendered = host_config::render(&key_name, &new_hostname, new_user.as_ref(), new_port);
    host_config::replace(&layout.host_config(&host), &rendered)?;
    Ok(new_hostname.to_string())
}
