use crate::error::AppError;
use crate::ssh::host_config::{self, HostConfig};
use crate::ssh::layout::Layout;
use std::fs;

pub(crate) fn execute(
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
    Layout::validate_host(host)?;
    if let Some(hostname) = hostname {
        Layout::validate_hostname(hostname)?;
    }
    let layout = Layout::from_env()?;
    let config_path = layout.host_config(host);
    match fs::symlink_metadata(&config_path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::HostNotFound(host.to_string()));
        }
        Err(error) => return Err(error.into()),
    }
    layout.require_regular_file(&config_path)?;
    let config = HostConfig::parse(&fs::read_to_string(&config_path)?, &layout)?;
    layout.require_host_identity(&config.identity, host)?;
    let key_type = Layout::managed_key_type(&config.identity, host)?;

    let new_hostname =
        hostname.map(str::to_string).or(config.hostname).unwrap_or_else(|| host.to_string());
    let new_user = user.map(str::to_string).or(config.user);
    let new_port = port.or(config.port);

    let rendered =
        HostConfig::render(host, &new_hostname, &key_type, new_user.as_deref(), new_port);
    host_config::write(&config_path, &rendered)?;
    Ok(new_hostname)
}
