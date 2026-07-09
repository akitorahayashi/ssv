use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config::has_managed_include;
use crate::ssh::layout::Layout;
use crate::ssh::permissions;
use std::fmt::{self, Display};
use std::fs;
use std::path::Path;

const MANAGED_INCLUDE_LINE: &str = "Include ~/.ssh/conf.d/*.conf";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BootstrapStatus {
    Created,
    Repaired,
    UpToDate,
}

impl BootstrapStatus {
    fn combine(self, other: Self) -> Self {
        use BootstrapStatus::{Created, Repaired, UpToDate};
        match (self, other) {
            (Created, _) | (_, Created) => Created,
            (Repaired, _) | (_, Repaired) => Repaired,
            (UpToDate, UpToDate) => UpToDate,
        }
    }
}

impl Display for BootstrapStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BootstrapStatus::Created => write!(f, "Created SSH bootstrap directories and config"),
            BootstrapStatus::Repaired => {
                write!(f, "Repaired SSH bootstrap permissions and includes")
            }
            BootstrapStatus::UpToDate => write!(f, "SSH bootstrap is already up-to-date"),
        }
    }
}

/// Create or repair `~/.ssh`, `~/.ssh/conf.d`, and the managed `Include` line in `~/.ssh/config`,
/// leaving each at its expected mode.
pub(crate) fn ensure_bootstrap(layout: &Layout) -> Result<BootstrapStatus, AppError> {
    let mut status = BootstrapStatus::UpToDate;
    status = status.combine(prepare_dir(&layout.root())?);
    status = status.combine(prepare_dir(&layout.hosts())?);
    status = status.combine(ensure_main_config_include(layout)?);
    Ok(status)
}

fn prepare_dir(path: &Path) -> Result<BootstrapStatus, AppError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            if requires_mode_update(&metadata, permissions::DIRECTORY_MODE) {
                permissions::set_mode(path, permissions::DIRECTORY_MODE)?;
                Ok(BootstrapStatus::Repaired)
            } else {
                Ok(BootstrapStatus::UpToDate)
            }
        }
        Ok(_) => Err(AppError::validation(format!(
            "refusing to prepare non-directory path '{}'",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::create_dir_all(path).path_ctx(path)?;
            permissions::set_mode(path, permissions::DIRECTORY_MODE)?;
            Ok(BootstrapStatus::Created)
        }
        Err(error) => Err(AppError::io(path, error)),
    }
}

fn ensure_main_config_include(layout: &Layout) -> Result<BootstrapStatus, AppError> {
    let path = layout.config();
    match fs::symlink_metadata(&path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            let mut status = BootstrapStatus::UpToDate;
            if requires_mode_update(&metadata, permissions::PRIVATE_MODE) {
                permissions::set_mode(&path, permissions::PRIVATE_MODE)?;
                status = BootstrapStatus::Repaired;
            }

            let contents = fs::read_to_string(&path).path_ctx(&path)?;
            if has_managed_include(&contents) {
                return Ok(status);
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
            fs::write(&path, updated).path_ctx(&path)?;
            if status == BootstrapStatus::UpToDate {
                Ok(BootstrapStatus::Repaired)
            } else {
                Ok(status)
            }
        }
        Ok(_) => Err(AppError::validation(format!(
            "refusing to prepare non-file path '{}'",
            path.display()
        ))),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            fs::write(&path, format!("{MANAGED_INCLUDE_LINE}\n")).path_ctx(&path)?;
            permissions::set_mode(&path, permissions::PRIVATE_MODE)?;
            Ok(BootstrapStatus::Created)
        }
        Err(error) => Err(AppError::io(&path, error)),
    }
}

fn requires_mode_update(metadata: &fs::Metadata, expected: u32) -> bool {
    permissions::mode(metadata) != expected
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_detection_is_idempotent() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout::with_home(temp.path().to_path_buf());
        ensure_bootstrap(&layout).expect("bootstrap should succeed");
        ensure_bootstrap(&layout).expect("bootstrap should remain idempotent");
        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config.matches("~/.ssh/conf.d/*.conf").count(), 1);
    }

    #[test]
    fn quoted_include_is_not_duplicated() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout::with_home(temp.path().to_path_buf());
        prepare_dir(&layout.root()).expect("root should exist");
        prepare_dir(&layout.hosts()).expect("hosts should exist");
        fs::write(layout.config(), "Include \"~/.ssh/conf.d/*.conf\"\n")
            .expect("config should be written");

        ensure_bootstrap(&layout).expect("bootstrap should succeed");

        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config.matches("~/.ssh/conf.d/*.conf").count(), 1);
    }

    #[test]
    fn managed_include_is_inserted_before_first_block() {
        let temp = tempfile::TempDir::new().expect("temp dir");
        let layout = Layout::with_home(temp.path().to_path_buf());
        prepare_dir(&layout.root()).expect("root should exist");
        prepare_dir(&layout.hosts()).expect("hosts should exist");
        fs::write(layout.config(), "# settings\nHost example\n  User test\n")
            .expect("config should be written");

        ensure_bootstrap(&layout).expect("bootstrap should succeed");

        let config = fs::read_to_string(layout.config()).expect("config should exist");
        assert_eq!(config, "# settings\nInclude ~/.ssh/conf.d/*.conf\nHost example\n  User test\n");
        assert!(has_managed_include(&config));
    }
}
