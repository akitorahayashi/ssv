use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config;
use crate::ssh::layout::Layout;
use crate::ssh::naming;
use std::fs;

pub(crate) fn execute(host: &str) -> Result<String, AppError> {
    naming::validate_host(host)?;
    let layout = Layout::from_env()?;
    let config = host_config::load(&layout, host)?;
    layout.require_host_identity(&config.identity, host)?;
    let public = layout.public_key(&config.identity)?;
    layout.require_regular_file(&public)?;
    fs::read_to_string(&public).path_ctx(&public)
}
