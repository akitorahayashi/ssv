use crate::error::{AppError, IoResultExt};
use crate::ssh::naming::{self, ManagedKeyName};
use std::fs;
use std::path::{Component, Path, PathBuf};

/// Derives the managed SSH paths from `$HOME` and enforces the managed-root boundary.
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

    pub(crate) fn from_home(home: PathBuf) -> Self {
        Self { home }
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

    pub(crate) fn key_pair(&self, name: &ManagedKeyName) -> (PathBuf, PathBuf) {
        let filename = name.render();
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
            Err(error) => Err(AppError::io(path, error)),
        }
    }

    pub(crate) fn require_host_identity(&self, path: &Path, host: &str) -> Result<(), AppError> {
        self.require_managed(path)?;
        naming::managed_key_type(path, host).map(|_| ())
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
        let metadata = fs::symlink_metadata(path).path_ctx(path)?;
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
}

fn is_symlink_if_present(path: &Path) -> Result<bool, AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(metadata.file_type().is_symlink()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(err) => Err(AppError::io(path, err)),
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

    #[test]
    fn identity_resolution_rejects_paths_outside_root() {
        let layout = Layout::from_home(PathBuf::from("/home/test"));
        assert!(layout.resolve_identity("/outside/key").is_err());
    }
}
