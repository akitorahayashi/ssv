use super::TestContext;
use std::fs;
use std::path::PathBuf;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

impl TestContext {
    pub fn ssh_root(&self) -> PathBuf {
        self.home().join(".ssh")
    }

    pub fn hosts_dir(&self) -> PathBuf {
        self.ssh_root().join("conf.d")
    }

    pub fn main_config(&self) -> PathBuf {
        self.ssh_root().join("config")
    }

    pub fn host_config(&self, host: &str) -> PathBuf {
        self.hosts_dir().join(format!("{host}.conf"))
    }

    pub fn private_key(&self, key_type: &str, host: &str) -> PathBuf {
        self.ssh_root().join(format!("id_{key_type}_{host}"))
    }

    pub fn public_key(&self, key_type: &str, host: &str) -> PathBuf {
        self.ssh_root().join(format!("id_{key_type}_{host}.pub"))
    }

    pub fn prepare_include(&self) {
        fs::write(self.main_config(), "Include ~/.ssh/conf.d/*.conf\n")
            .expect("main config should be written");
        self.set_mode(&self.main_config(), 0o600);
    }

    pub fn write_host_config(&self, host: &str, identity: &str) {
        fs::create_dir_all(self.hosts_dir()).expect("hosts directory should exist");
        fs::write(
            self.host_config(host),
            format!("Host {host}\nHostName {host}\nIdentityFile {identity}\nIdentitiesOnly yes\n"),
        )
        .expect("host config should be written");
        self.set_mode(&self.host_config(host), 0o600);
    }

    pub fn set_mode(&self, path: &std::path::Path, mode: u32) {
        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(path).expect("metadata should exist").permissions();
            permissions.set_mode(mode);
            fs::set_permissions(path, permissions).expect("mode should be set");
        }
        #[cfg(not(unix))]
        let _ = (path, mode);
    }
}
