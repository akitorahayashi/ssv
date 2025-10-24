use crate::error::AppError;
use crate::ssh_paths::SshPaths;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) struct RemoveHost<'a> {
    pub host: &'a str,
}

impl<'a> RemoveHost<'a> {
    pub(crate) fn execute(&self, paths: &SshPaths) -> Result<(), AppError> {
        paths.ensure_base_dirs()?;
        paths.validate_host(self.host)?;

        let config_path = paths.host_config_path(self.host);
        let mut identity_candidates = Vec::new();

        if config_path.exists()
            && let Ok(config_contents) = fs::read_to_string(&config_path)
        {
            identity_candidates.extend(self.parse_identity_files(&config_contents, paths));
        }

        if config_path.exists()
            && let Err(err) = fs::remove_file(&config_path)
            && err.kind() != std::io::ErrorKind::NotFound
        {
            return Err(AppError::from(err));
        }

        if identity_candidates.is_empty() {
            identity_candidates.extend(self.guess_identity_files(paths));
        }

        for key_path in identity_candidates {
            Self::remove_if_exists(&key_path)?;
            let pub_path = Self::to_public_key_path(&key_path);
            Self::remove_if_exists(&pub_path)?;
        }

        Ok(())
    }

    fn parse_identity_files(&self, contents: &str, paths: &SshPaths) -> Vec<PathBuf> {
        contents
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() {
                    return None;
                }
                let mut parts = trimmed.split_whitespace();
                let directive = parts.next()?;
                if directive.eq_ignore_ascii_case("IdentityFile") {
                    let value = parts.next()?;
                    Some(Self::expand_path(value, paths))
                } else {
                    None
                }
            })
            .collect()
    }

    fn guess_identity_files(&self, paths: &SshPaths) -> Vec<PathBuf> {
        let mut candidates = Vec::new();
        let ssh_dir = paths.ssh_root();
        if !ssh_dir.exists() {
            return candidates;
        }

        if let Ok(entries) = fs::read_dir(&ssh_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str())
                    && file_name.starts_with("id_")
                    && file_name.ends_with(self.host)
                    && !file_name.ends_with(".pub")
                {
                    candidates.push(path.clone());
                }
            }
        }

        candidates
    }

    fn expand_path(value: &str, paths: &SshPaths) -> PathBuf {
        if let Some(stripped) = value.strip_prefix("~/") {
            paths.home().join(stripped)
        } else {
            PathBuf::from(value)
        }
    }

    fn remove_if_exists(path: &Path) -> Result<(), AppError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AppError::from(err)),
        }
    }

    fn to_public_key_path(private: &Path) -> PathBuf {
        let mut file_name = private
            .file_name()
            .and_then(|n| n.to_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| String::from("id_ssh_key"));
        file_name.push_str(".pub");
        private.with_file_name(file_name)
    }
}
