use crate::error::AppError;
use crate::ssh::host_config::HostConfig;
use crate::ssh::keycopy;
use crate::ssh::layout::Layout;
use std::fs;

pub(crate) fn execute(host: &str) -> Result<String, AppError> {
    Layout::validate_host(host)?;
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
    let public = layout.public_key(&config.identity)?;
    layout.require_regular_file(&public)?;
    let hostname = config
        .hostname
        .ok_or_else(|| AppError::validation("managed host config has no HostName"))?;
    keycopy::install(&public, config.user.as_deref(), &hostname, config.port)?;
    Ok(keycopy::target(config.user.as_deref(), &hostname))
}
