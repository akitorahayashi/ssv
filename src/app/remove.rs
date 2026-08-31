use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config;
use crate::ssh::layout::Layout;
use crate::ssh::naming::HostIdentifier;
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

pub(crate) fn execute(layout: &Layout, host: &str) -> Result<RemovalStatus, AppError> {
    let host = HostIdentifier::new(host)?;
    let config = host_config::load(layout, &host)?;
    preflight_optional_file(layout, &config.private_key)?;
    preflight_optional_file(layout, &config.public_key)?;

    let mut missing = 0;
    if !remove_if_present(&config.private_key)? {
        missing += 1;
    }
    if !remove_if_present(&config.public_key)? {
        missing += 1;
    }
    fs::remove_file(&config.path).path_ctx(&config.path)?;

    if missing == 0 { Ok(RemovalStatus::Removed) } else { Ok(RemovalStatus::Partial { missing }) }
}

fn preflight_optional_file(layout: &Layout, path: &Path) -> Result<(), AppError> {
    match fs::symlink_metadata(path) {
        Ok(_) => layout.require_regular_file(path),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::io(path, error)),
    }
}

fn remove_if_present(path: &Path) -> Result<bool, AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::io(path, err)),
    }
}
