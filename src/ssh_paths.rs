use crate::error::AppError;
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Helper struct for resolving and preparing SSH asset paths.
pub(crate) struct SshPaths {
    home: PathBuf,
}

const SSV_INCLUDE_LINE: &str = "Include ~/.ssh/conf.d/*.conf";

impl SshPaths {
    /// Resolve the paths using the `HOME` environment variable.
    pub(crate) fn from_env() -> Result<Self, AppError> {
        let home = std::env::var_os("HOME")
            .ok_or_else(|| AppError::config_error("HOME environment variable not set"))?;
        Ok(Self { home: PathBuf::from(home) })
    }

    /// Ensure the ~/.ssh and ~/.ssh/conf.d directories exist with secure permissions.
    pub(crate) fn ensure_base_dirs(&self) -> Result<(), AppError> {
        self.ensure_dir_with_mode(&self.ssh_root())?;
        self.ensure_dir_with_mode(&self.conf_dir())?;
        Ok(())
    }

    /// Ensure the bootstrap config required to activate managed host configs exists.
    pub(crate) fn ensure_bootstrap(&self) -> Result<(), AppError> {
        self.ensure_base_dirs()?;
        self.ensure_main_config_include()
    }

    pub(crate) fn ssh_root(&self) -> PathBuf {
        self.home.join(".ssh")
    }

    pub(crate) fn conf_dir(&self) -> PathBuf {
        self.ssh_root().join("conf.d")
    }

    pub(crate) fn main_config_path(&self) -> PathBuf {
        self.ssh_root().join("config")
    }

    pub(crate) fn home(&self) -> &Path {
        &self.home
    }

    pub(crate) fn host_config_path(&self, host: &str) -> PathBuf {
        self.conf_dir().join(format!("{host}.conf"))
    }

    pub(crate) fn key_paths(&self, key_type: &str, host: &str) -> (PathBuf, PathBuf) {
        let filename = format!("id_{}_{}", key_type, host);
        let private = self.ssh_root().join(&filename);
        let public = self.ssh_root().join(format!("{filename}.pub"));
        (private, public)
    }

    pub(crate) fn validate_host(&self, host: &str) -> Result<(), AppError> {
        if host.is_empty() {
            return Err(AppError::validation_error("host must not be empty"));
        }

        let is_valid =
            host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
                && !host.contains('/')
                && !host.contains(' ');

        if !is_valid {
            return Err(AppError::validation_error(format!(
                "invalid host identifier '{host}'; allowed characters are alphanumeric, '.', '-', '_'"
            )));
        }

        Ok(())
    }

    pub(crate) fn validate_key_type(&self, key_type: &str) -> Result<(), AppError> {
        if key_type.is_empty() {
            return Err(AppError::validation_error("key type must not be empty"));
        }

        let is_valid = key_type.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit());

        if !is_valid {
            return Err(AppError::validation_error(format!(
                "invalid key type '{key_type}'; expected lowercase letters or digits"
            )));
        }

        Ok(())
    }

    fn ensure_dir_with_mode(&self, path: &Path) -> Result<(), AppError> {
        if !path.exists() {
            fs::create_dir_all(path)?;
        }
        self.apply_permissions(path)?;
        Ok(())
    }

    fn ensure_main_config_include(&self) -> Result<(), AppError> {
        let path = self.main_config_path();

        if !path.exists() {
            fs::write(&path, format!("{SSV_INCLUDE_LINE}\n"))?;
            self.apply_file_permissions(&path)?;
            return Ok(());
        }

        let contents = fs::read_to_string(&path)?;
        if contents.lines().any(Self::is_ssv_include_line) {
            self.apply_file_permissions(&path)?;
            return Ok(());
        }

        let mut updated = contents;
        if !updated.is_empty() && !updated.ends_with('\n') {
            updated.push('\n');
        }
        updated.push_str(SSV_INCLUDE_LINE);
        updated.push('\n');

        fs::write(&path, updated)?;
        self.apply_file_permissions(&path)?;
        Ok(())
    }

    fn apply_permissions(&self, path: &Path) -> Result<(), AppError> {
        let permissions = fs::metadata(path)?.permissions();
        #[cfg(unix)]
        {
            let mut perms = permissions;
            perms.set_mode(0o700);
            fs::set_permissions(path, perms)?;
        }
        #[cfg(not(unix))]
        {
            let _ = permissions;
        }
        Ok(())
    }

    fn apply_file_permissions(&self, path: &Path) -> Result<(), AppError> {
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path)?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(path, perms)?;
        }
        Ok(())
    }

    fn is_ssv_include_line(line: &str) -> bool {
        let mut parts = line.split('#').next().unwrap_or("").split_whitespace();
        matches!(
            (parts.next(), parts.next(), parts.next()),
            (Some(command), Some(value), None)
                if command.eq_ignore_ascii_case("include") && value == "~/.ssh/conf.d/*.conf"
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn paths() -> SshPaths {
        SshPaths { home: PathBuf::from(".") }
    }

    fn temp_paths() -> (TempDir, SshPaths) {
        let temp = TempDir::new().expect("temp dir");
        let paths = SshPaths { home: temp.path().to_path_buf() };
        (temp, paths)
    }

    #[test]
    fn validate_host_accepts_simple_names() {
        let paths = paths();
        assert!(paths.validate_host("github.com").is_ok());
        assert!(paths.validate_host("internal-host_01").is_ok());
    }

    #[test]
    fn validate_host_rejects_invalid_characters() {
        let paths = paths();
        assert!(paths.validate_host("bad/host").is_err());
        assert!(paths.validate_host("spaces host").is_err());
    }

    #[test]
    fn validate_key_type_restricts_charset() {
        let paths = paths();
        assert!(paths.validate_key_type("ed25519").is_ok());
        assert!(paths.validate_key_type("RSA").is_err());
    }

    #[test]
    fn ensure_bootstrap_creates_main_config_with_include() {
        let (_temp, paths) = temp_paths();

        paths.ensure_bootstrap().expect("bootstrap should succeed");

        let config = fs::read_to_string(paths.main_config_path()).expect("main config");
        assert_eq!(config, "Include ~/.ssh/conf.d/*.conf\n");
    }

    #[test]
    fn ensure_bootstrap_appends_include_once() {
        let (_temp, paths) = temp_paths();
        paths.ensure_base_dirs().expect("base dirs");
        fs::write(paths.main_config_path(), "Host *\n  ServerAliveInterval 60\n")
            .expect("seed config");

        paths.ensure_bootstrap().expect("bootstrap should succeed");
        paths.ensure_bootstrap().expect("bootstrap should remain idempotent");

        let config = fs::read_to_string(paths.main_config_path()).expect("main config");
        assert_eq!(config.matches(SSV_INCLUDE_LINE).count(), 1);
        assert!(config.contains("Host *\n  ServerAliveInterval 60\n"));
    }

    #[test]
    fn ensure_bootstrap_recognizes_equivalent_include_line() {
        let (_temp, paths) = temp_paths();
        paths.ensure_base_dirs().expect("base dirs");
        fs::write(
            paths.main_config_path(),
            "Host *\n  ServerAliveInterval 60\ninclude   ~/.ssh/conf.d/*.conf   # managed elsewhere\n",
        )
        .expect("seed config");

        paths.ensure_bootstrap().expect("bootstrap should succeed");

        let config = fs::read_to_string(paths.main_config_path()).expect("main config");
        assert_eq!(config.matches("~/.ssh/conf.d/*.conf").count(), 1);
    }
}
