use crate::error::AppError;
use crate::ssh::host_config;
use crate::ssh::keycopy;
use crate::ssh::layout::Layout;
use crate::ssh::naming::HostIdentifier;
use std::path::Path;

pub(crate) fn execute(layout: &Layout, copy_id: &Path, host: &str) -> Result<String, AppError> {
    let host = HostIdentifier::new(host)?;
    let config = host_config::load(layout, &host)?;
    layout.require_regular_file(&config.public_key)?;
    let user = config.user.as_ref().map(|user| user.as_str());
    keycopy::install(copy_id, &config.public_key, user, config.hostname.as_str(), config.port)?;
    Ok(keycopy::target(user, config.hostname.as_str()))
}
