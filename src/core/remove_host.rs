use crate::error::AppError;
use crate::ssh_paths::SshPaths;
use std::fs;
use std::path::{Component, Path, PathBuf};

pub(crate) struct RemoveHost<'a> {
    pub host: &'a str,
}

impl<'a> RemoveHost<'a> {
    pub(crate) fn execute(&self, paths: &SshPaths) -> Result<(), AppError> {
        paths.ensure_base_dirs()?;
        paths.validate_host(self.host)?;

        let config_path = paths.host_config_path(self.host);
        let mut identity_candidates = Vec::new();

        if let Ok(config_contents) = fs::read_to_string(&config_path) {
            identity_candidates.extend(self.parse_identity_files(&config_contents, paths));
        }

        Self::remove_if_exists(&config_path)?;

        if identity_candidates.is_empty() {
            identity_candidates.extend(self.guess_identity_files(paths));
        }

        identity_candidates.retain(|p| p.starts_with(paths.ssh_root()));

        for key_path in identity_candidates {
            Self::remove_if_exists(&key_path)?;
            if let Some(pub_path) = Self::to_public_key_path(&key_path) {
                Self::remove_if_exists(&pub_path)?;
            }
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
                    let value = Self::unquote(value);
                    Self::expand_path(value, paths)
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
                    && file_name.ends_with(&format!("_{}", self.host))
                    && !file_name.ends_with(".pub")
                {
                    candidates.push(path.clone());
                }
            }
        }

        candidates
    }

    fn expand_path(value: &str, paths: &SshPaths) -> Option<PathBuf> {
        let candidate = if let Some(stripped) = value.strip_prefix("~/") {
            paths.home().join(stripped)
        } else if Path::new(value).is_absolute() {
            PathBuf::from(value)
        } else {
            paths.ssh_root().join(value)
        };

        let normalized_root = normalize_path(&paths.ssh_root());
        let normalized_candidate = normalize_path(&candidate);

        if normalized_candidate.starts_with(&normalized_root) {
            Some(normalized_candidate)
        } else {
            None
        }
    }

    fn remove_if_exists(path: &Path) -> Result<(), AppError> {
        match fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(err) => Err(AppError::from(err)),
        }
    }

    fn to_public_key_path(private: &Path) -> Option<PathBuf> {
        let mut file_name = private.file_name()?.to_os_string();
        file_name.push(".pub");
        Some(private.with_file_name(file_name))
    }

    fn unquote(s: &str) -> &str {
        s.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(s)
    }
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::Normal(part) => normalized.push(part),
        }
    }

    normalized
}
