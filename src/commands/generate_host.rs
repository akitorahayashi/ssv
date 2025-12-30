use crate::error::AppError;
use crate::ssh_paths::SshPaths;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Command object that provisions keys and configuration for a host.
pub(crate) struct GenerateHost<'a> {
    pub host: &'a str,
    pub key_type: &'a str,
    pub user: Option<&'a str>,
    pub port: Option<u16>,
}

impl<'a> GenerateHost<'a> {
    pub(crate) fn execute(&self, paths: &SshPaths) -> Result<String, AppError> {
        paths.ensure_base_dirs()?;
        paths.validate_host(self.host)?;
        paths.validate_key_type(self.key_type)?;

        let (private_key, public_key) = paths.key_paths(self.key_type, self.host);
        let config_path = paths.host_config_path(self.host);

        if private_key.exists() || public_key.exists() || config_path.exists() {
            return Err(AppError::validation_error(format!(
                "Artifacts for host '{}' already exist; remove them before regenerating",
                self.host
            )));
        }

        self.run_keygen(&private_key)?;
        self.write_config(&config_path)?;

        let public_key_contents = fs::read_to_string(&public_key)?;
        Ok(public_key_contents)
    }

    fn run_keygen(&self, private_key: &Path) -> Result<(), AppError> {
        let keygen = std::env::var("SSV_SSH_KEYGEN_PATH").unwrap_or_else(|_| "ssh-keygen".into());

        let status = Command::new(&keygen)
            .arg("-t")
            .arg(self.key_type)
            .arg("-f")
            .arg(private_key)
            .arg("-q")
            .arg("-N")
            .arg("")
            .status()
            .map_err(AppError::from)?;

        if status.success() { Ok(()) } else { Err(AppError::command_failed(&keygen, status)) }
    }

    fn write_config(&self, config_path: &Path) -> Result<(), AppError> {
        let mut contents = format!("Host {}\nHostName {}\n", self.host, self.host);
        if let Some(user) = self.user {
            contents.push_str(&format!("User {}\n", user));
        }
        if let Some(port) = self.port {
            contents.push_str(&format!("Port {}\n", port));
        }
        contents.push_str(&format!("IdentityFile ~/.ssh/id_{}_{}\n", self.key_type, self.host));
        contents.push_str("IdentitiesOnly yes\n");

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = fs::File::create(config_path)?;
        file.write_all(contents.as_bytes())?;
        file.sync_all()?;

        #[cfg(unix)]
        {
            let mut perms = file.metadata()?.permissions();
            perms.set_mode(0o600);
            fs::set_permissions(config_path, perms)?;
        }

        Ok(())
    }
}
