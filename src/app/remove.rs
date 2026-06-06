use crate::error::AppError;
use crate::ssh::host_config::HostConfig;
use crate::ssh::layout::Layout;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemovalStatus {
    Removed,
    Partial { missing: usize },
}

impl RemovalStatus {
    pub(crate) fn message(self, host: &str) -> String {
        match self {
            Self::Removed => format!("Removed SSH assets for '{host}'"),
            Self::Partial { missing } => format!(
                "Removed SSH assets for '{host}' ({missing} {} already absent)",
                if missing == 1 { "asset was" } else { "assets were" }
            ),
        }
    }
}

pub(crate) fn execute(host: &str) -> Result<RemovalStatus, AppError> {
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

    let mut missing = 0;
    if !remove_if_present(&config.identity)? {
        missing += 1;
    }
    if !remove_if_present(&public)? {
        missing += 1;
    }
    fs::remove_file(config_path)?;

    if missing == 0 { Ok(RemovalStatus::Removed) } else { Ok(RemovalStatus::Partial { missing }) }
}

fn remove_if_present(path: &Path) -> Result<bool, AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}
