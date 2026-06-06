use crate::error::AppError;
use crate::ssh::layout::Layout;
use std::fs;

pub(crate) fn execute() -> Result<Vec<String>, AppError> {
    let hosts_dir = Layout::from_env()?.hosts();
    if !hosts_dir.exists() {
        return Ok(Vec::new());
    }

    let mut hosts = Vec::new();
    for entry in fs::read_dir(hosts_dir)? {
        let path = entry?.path();
        if path.extension().and_then(|extension| extension.to_str()) == Some("conf")
            && let Some(host) = path.file_stem().and_then(|stem| stem.to_str())
        {
            hosts.push(host.to_string());
        }
    }
    hosts.sort();
    Ok(hosts)
}
