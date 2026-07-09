use crate::error::{AppError, IoResultExt};
use crate::ssh::layout::Layout;
use std::fs;

pub(crate) fn execute() -> Result<Vec<String>, AppError> {
    let hosts_dir = Layout::from_env()?.hosts();
    let entries = match fs::read_dir(&hosts_dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::bootstrap_missing(hosts_dir));
        }
        Err(error) => return Err(AppError::io(&hosts_dir, error)),
    };

    let mut hosts = Vec::new();
    for entry in entries {
        let path = entry.path_ctx(&hosts_dir)?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("conf")
            && let Some(host) = path.file_stem().and_then(|stem| stem.to_str())
        {
            hosts.push(host.to_string());
        }
    }
    hosts.sort();
    Ok(hosts)
}
