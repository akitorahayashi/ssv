use crate::error::AppError;
use crate::ssh::host_config::HostConfig;
use crate::ssh::layout::Layout;
use std::fs;
use std::path::Path;

pub(crate) fn execute(host: &str) -> Result<(), AppError> {
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

    remove_if_present(&config.identity)?;
    remove_if_present(&public)?;
    fs::remove_file(config_path)?;
    Ok(())
}

fn remove_if_present(path: &Path) -> Result<(), AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(err) => Err(err.into()),
    }
}
