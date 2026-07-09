use crate::context::Context;
use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config;
use crate::ssh::naming;
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

pub(crate) fn execute(ctx: &Context, host: &str) -> Result<RemovalStatus, AppError> {
    naming::validate_host(host)?;
    let layout = ctx.layout();
    let config = host_config::load(layout, host)?;
    layout.require_host_identity(&config.identity, host)?;
    let public = layout.public_key(&config.identity)?;

    let config_path = layout.host_config(host);
    fs::remove_file(&config_path).path_ctx(&config_path)?;

    let mut missing = 0;
    if !remove_if_present(&config.identity)? {
        missing += 1;
    }
    if !remove_if_present(&public)? {
        missing += 1;
    }

    if missing == 0 { Ok(RemovalStatus::Removed) } else { Ok(RemovalStatus::Partial { missing }) }
}

fn remove_if_present(path: &Path) -> Result<bool, AppError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::io(path, err)),
    }
}
