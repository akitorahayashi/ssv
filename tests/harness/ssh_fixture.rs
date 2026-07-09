use super::TestContext;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

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

    pub fn copy_id_invocation(&self) -> String {
        fs::read_to_string(self.home().join("ssh-copy-id.args")).unwrap_or_default()
    }

    /// Install a keygen stub that writes only the private key (no matching public key), so the
    /// caller can exercise the `generate` rollback path. Returns the stub path.
    pub fn install_private_only_keygen(&self) -> PathBuf {
        let keygen = self.home().join("private-only-keygen");
        fs::write(
            &keygen,
            "#!/usr/bin/env sh\nset -eu\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-f\" ]; then\n    shift\n    outfile=\"$1\"\n  fi\n  shift\ndone\nprintf 'PRIVATE-ed25519\\n' > \"$outfile\"\n",
        )
        .expect("private-only keygen should be written");
        self.set_mode(&keygen, 0o755);
        keygen
    }

    pub fn set_mode(&self, path: &std::path::Path, mode: u32) {
        let mut permissions = fs::metadata(path).expect("metadata should exist").permissions();
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions).expect("mode should be set");
    }
}
