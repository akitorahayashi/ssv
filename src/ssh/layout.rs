use crate::error::AppError;
use crate::ssh::host_config::has_managed_include;
use crate::ssh::permissions;
use std::fs;
use std::path::{Component, Path, PathBuf};

const MANAGED_INCLUDE_LINE: &str = "Include ~/.ssh/conf.d/*.conf";

#[derive(Debug, Clone)]
pub(crate) struct Layout {
    home: PathBuf,
}

impl Layout {
    pub(crate) fn from_env() -> Result<Self, AppError> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| AppError::config("HOME environment variable not set"))?;
        Ok(Self { home: PathBuf::from(home) })
    }

    pub(crate) fn home(&self) -> &Path {
        &self.home
    }

    pub(crate) fn root(&self) -> PathBuf {
        self.home.join(".ssh")
    }

    pub(crate) fn config(&self) -> PathBuf {
        self.root().join("config")
    }

    pub(crate) fn hosts(&self) -> PathBuf {
        self.root().join("conf.d")
    }

    pub(crate) fn host_config(&self, host: &str) -> PathBuf {
        self.hosts().join(format!("{host}.conf"))
    }

    pub(crate) fn key_pair(&self, key_type: &str, host: &str) -> (PathBuf, PathBuf) {
        let filename = format!("id_{key_type}_{host}");
        let private = self.root().join(&filename);
        let public = self.root().join(format!("{filename}.pub"));
        (private, public)
    }

    pub(crate) fn public_key(&self, private: &Path) -> Result<PathBuf, AppError> {
        let mut filename = private
            .file_name()
            .ok_or_else(|| AppError::validation("IdentityFile has no filename"))?
            .to_os_string();
        filename.push(".pub");
        Ok(private.with_file_name(filename))
    }

    pub(crate) fn artifact_exists(&self, path: &Path) -> Result<bool, AppError> {
        match fs::symlink_metadata(path) {
            Ok(_) => Ok(true),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(error) => Err(error.into()),
        }
    }

    pub(crate) fn require_host_identity(&self, path: &Path, host: &str) -> Result<(), AppError> {
        self.require_managed(path)?;
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| AppError::validation("IdentityFile has no UTF-8 filename"))?;
        let Some(name) = filename.strip_prefix("id_") else {
            return Err(unmanaged_identity(path, host));
        };
        let Some((key_type, identity_host)) = name.split_once('_') else {
            return Err(unmanaged_identity(path, host));
        };
        if Self::validate_key_type(key_type).is_err() || identity_host != host {
            return Err(unmanaged_identity(path, host));
        }
        Ok(())
    }

    pub(crate) fn is_managed_key_candidate(path: &Path) -> bool {
        let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };
        if filename.ends_with(".pub") || filename.ends_with("_sk") {
            return false;
        }
        let Some(name) = filename.strip_prefix("id_") else {
            return false;
        };
        let Some((key_type, host)) = name.split_once('_') else {
            return false;
        };
        Self::validate_key_type(key_type).is_ok() && Self::validate_host(host).is_ok()
    }

    pub(crate) fn prepare_for_generate(&self) -> Result<(), AppError> {
        self.ensure_bootstrap()
    }

    pub(crate) fn ensure_bootstrap(&self) -> Result<(), AppError> {
        self.prepare_dir(&self.root())?;
        self.prepare_dir(&self.hosts())?;
        self.ensure_main_config_include()
    }

    pub(crate) fn resolve_identity(&self, value: &str) -> Result<PathBuf, AppError> {
        if value == "none" || value.contains('%') || value.contains('$') {
            return Err(AppError::validation(format!("unsupported IdentityFile value '{value}'")));
        }

        let candidate = if let Some(relative) = value.strip_prefix("~/") {
            self.home.join(relative)
        } else if Path::new(value).is_absolute() {
            PathBuf::from(value)
        } else {
            self.root().join(value)
        };
        let normalized = normalize(&candidate);
        self.require_managed(&normalized)?;
        Ok(normalized)
    }

    pub(crate) fn require_managed(&self, path: &Path) -> Result<(), AppError> {
        if normalize(path).starts_with(normalize(&self.root())) {
            Ok(())
        } else {
            Err(AppError::OutsideManagedRoot(path.to_path_buf()))
        }
    }

    pub(crate) fn require_regular_file(&self, path: &Path) -> Result<(), AppError> {
        self.require_managed(path)?;
        if self.has_symlink_component(path)? {
            return Err(AppError::validation(format!(
                "managed path '{}' contains a symbolic link",
                path.display()
            )));
        }
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_file() {
            Ok(())
        } else {
            Err(AppError::validation(format!(
                "managed path '{}' is not a regular file",
                path.display()
            )))
        }
    }

    pub(crate) fn has_symlink_component(&self, path: &Path) -> Result<bool, AppError> {
        self.require_managed(path)?;
        let relative = path
            .strip_prefix(self.root())
            .map_err(|_| AppError::validation("managed path could not be made relative"))?;
        let mut candidate = self.root();
        if is_symlink_if_present(&candidate)? {
            return Ok(true);
        }
        for component in relative.components() {
            candidate.push(component.as_os_str());
            if is_symlink_if_present(&candidate)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub(crate) fn validate_host(host: &str) -> Result<(), AppError> {
        if host.is_empty() {
            return Err(AppError::validation("host must not be empty"));
        }
        if !host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')) {
            return Err(AppError::validation(format!(
                "invalid host identifier '{host}'; allowed characters are alphanumeric, '.', '-', '_'"
            )));
        }
        Ok(())
    }

    pub(crate) fn validate_key_type(key_type: &str) -> Result<(), AppError> {
        if key_type.is_empty()
            || !key_type.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        {
            return Err(AppError::validation(format!(
                "invalid key type '{key_type}'; expected lowercase letters or digits"
            )));
        }
        Ok(())
    }

    fn prepare_dir(&self, path: &Path) -> Result<(), AppError> {
        match fs::symlink_metadata(path) {
            Ok(metadata) if metadata.file_type().is_dir() => {}
            Ok(_) => {
                return Err(AppError::validation(format!(
                    "refusing to prepare non-directory path '{}'",
                    path.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir_all(path)?;
            }
            Err(error) => return Err(error.into()),
        }
        permissions::set_mode(path, permissions::DIRECTORY_MODE)
    }

    fn ensure_main_config_include(&self) -> Result<(), AppError> {
        let path = self.config();
        match fs::symlink_metadata(&path) {
            Ok(metadata) if metadata.file_type().is_file() => {}
            Ok(_) => {
                return Err(AppError::validation(format!(
                    "refusing to prepare non-file path '{}'",
                    path.display()
                )));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::write(&path, format!("{MANAGED_INCLUDE_LINE}\n"))?;
                permissions::set_mode(&path, permissions::PRIVATE_MODE)?;
                return Ok(());
            }
            Err(error) => return Err(error.into()),
        }

        let contents = fs::read_to_string(&path)?;
        if has_managed_include(&contents) {
            permissions::set_mode(&path, permissions::PRIVATE_MODE)?;
            return Ok(());
        }

        let insertion = format!("{MANAGED_INCLUDE_LINE}\n");
        let updated = match first_block_offset(&contents) {
            Some(offset) => {
                let mut updated = String::with_capacity(contents.len() + insertion.len());
                updated.push_str(&contents[..offset]);
                updated.push_str(&insertion);
                updated.push_str(&contents[offset..]);
                updated
            }
            None => format!("{insertion}{contents}"),
        };
        fs::write(&path, updated)?;
        permissions::set_mode(&path, permissions::PRIVATE_MODE)
    }
}

fn first_block_offset(contents: &str) -> Option<usize> {
    let mut offset = 0;
    for line in contents.split_inclusive('\n') {
        let name = line.split_whitespace().next();
        if name.is_some_and(|name| {
            name.eq_ignore_ascii_case("Host") || name.eq_ignore_ascii_case("Match")
        }) {
            return Some(offset);
        }
        offset += line.len();
    }
    None
}

fn unmanaged_identity(path: &Path, host: &str) -> AppError {
    AppError::validation(format!(
        "refusing to manage identity '{}' because it does not match host '{host}'",
        path.display()
    ))
}

fn is_symlink_if_present(path: &Path) -> Result<bool, AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_symlink()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(err.into()),
    }
}

fn normalize(path: &Path) -> PathBuf {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn layout() -> Layout {
        Layout { home: PathBuf::from("/home/test") }
    }

    #[test]
    fn host_validation_accepts_managed_identifiers() {
        assert!(Layout::validate_host("github.com").is_ok());
        assert!(Layout::validate_host("internal-host_01").is_ok());
    }

    #[test]
    fn host_validation_rejects_path_components() {
        assert!(Layout::validate_host("bad/host").is_err());
        assert!(Layout::validate_host("spaces host").is_err());
    }

    #[test]
    fn identity_resolution_rejects_paths_outside_root() {
        assert!(layout().resolve_identity("/outside/key").is_err());
    }

    #[test]
    fn include_detection_is_idempotent() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout { home: temp.path().to_path_buf() };
        layout.ensure_bootstrap().expect("bootstrap should succeed");
        layout.ensure_bootstrap().expect("bootstrap should remain idempotent");
        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config.matches("~/.ssh/conf.d/*.conf").count(), 1);
    }

    #[test]
    fn quoted_include_is_not_duplicated() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout { home: temp.path().to_path_buf() };
        layout.prepare_dir(&layout.root()).expect("root should exist");
        layout.prepare_dir(&layout.hosts()).expect("hosts should exist");
        fs::write(layout.config(), "Include \"~/.ssh/conf.d/*.conf\"\n")
            .expect("config should be written");

        layout.ensure_bootstrap().expect("bootstrap should succeed");

        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config.matches("~/.ssh/conf.d/*.conf").count(), 1);
    }

    #[test]
    fn managed_include_is_inserted_before_first_block() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout { home: temp.path().to_path_buf() };
        layout.prepare_dir(&layout.root()).expect("root should exist");
        layout.prepare_dir(&layout.hosts()).expect("hosts should exist");
        fs::write(layout.config(), "# settings\nHost example\n  User test\n")
            .expect("config should be written");

        layout.ensure_bootstrap().expect("bootstrap should succeed");

        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config, "# settings\nInclude ~/.ssh/conf.d/*.conf\nHost example\n  User test\n");
        assert!(has_managed_include(&config));
    }

    #[test]
    fn managed_key_candidates_exclude_standard_keys() {
        assert!(!Layout::is_managed_key_candidate(Path::new("id_ed25519")));
        assert!(!Layout::is_managed_key_candidate(Path::new("id_ed25519_sk")));
        assert!(Layout::is_managed_key_candidate(Path::new("id_ed25519_github.com")));
    }
}
