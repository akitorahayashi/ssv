use crate::error::AppError;
use crate::ssh_paths::SshPaths;
use std::fs;

pub(crate) struct ListHosts;

impl ListHosts {
    pub(crate) fn execute(&self, paths: &SshPaths) -> Result<Vec<String>, AppError> {
        let conf_dir = paths.conf_dir();
        if !conf_dir.exists() {
            return Ok(Vec::new());
        }

        let mut hosts = Vec::new();
        for entry in fs::read_dir(&conf_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("conf")
                && let Some(stem) = path.file_stem().and_then(|stem| stem.to_str())
            {
                hosts.push(stem.to_string());
            }
        }

        hosts.sort();
        Ok(hosts)
    }
}
