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
        fs::create_dir_all(self.hosts_dir()).expect("hosts directory should exist");
        self.set_mode(&self.ssh_root(), 0o700);
        self.set_mode(&self.hosts_dir(), 0o700);
        fs::write(self.main_config(), "Include ~/.ssh/conf.d/*.conf\n")
            .expect("main config should be written");
        self.set_mode(&self.main_config(), 0o600);
    }

    pub fn write_managed_host(&self, host: &str) {
        self.write_managed_host_with(host, host, "ed25519", None, None);
    }

    pub fn write_managed_host_with(
        &self,
        host: &str,
        hostname: &str,
        key_type: &str,
        user: Option<&str>,
        port: Option<u16>,
    ) {
        self.prepare_include();
        fs::write(self.private_key(key_type, host), format!("PRIVATE-{key_type}\n"))
            .expect("private key should be written");
        fs::write(
            self.public_key(key_type, host),
            format!("ssh-{key_type} AAAATESTKEY {key_type}@fixture\n"),
        )
        .expect("public key should be written");
        self.set_mode(&self.private_key(key_type, host), 0o600);

        let mut config = format!("Host {host}\nHostName {hostname}\n");
        if let Some(user) = user {
            config.push_str(&format!("User {user}\n"));
        }
        if let Some(port) = port {
            config.push_str(&format!("Port {port}\n"));
        }
        config.push_str(&format!("IdentityFile ~/.ssh/id_{key_type}_{host}\nIdentitiesOnly yes\n"));
        fs::write(self.host_config(host), config).expect("host config should be written");
        self.set_mode(&self.host_config(host), 0o600);
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

    pub fn install_failing_keygen(&self, writes_public: bool) -> PathBuf {
        let name = if writes_public { "failing-pair-keygen" } else { "failing-private-keygen" };
        let keygen = self.home().join(name);
        let public_write = if writes_public {
            "printf 'ssh-ed25519 AAAATESTKEY failing@fixture\\n' > \"${outfile}.pub\"\n"
        } else {
            ""
        };
        fs::write(
            &keygen,
            format!(
                "#!/usr/bin/env sh\nset -eu\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-f\" ]; then\n    shift\n    outfile=\"$1\"\n  fi\n  shift\ndone\nprintf 'PRIVATE-ed25519\\n' > \"$outfile\"\n{public_write}echo 'injected keygen failure' >&2\nexit 23\n"
            ),
        )
        .expect("failing keygen should be written");
        self.set_mode(&keygen, 0o755);
        keygen
    }

    pub fn install_config_conflict_keygen(&self, host: &str) -> PathBuf {
        let keygen = self.home().join("config-conflict-keygen");
        fs::write(
            &keygen,
            format!(
                r#"#!/usr/bin/env sh
set -eu
outfile=""
derive="false"
while [ "$#" -gt 0 ]; do
  case "$1" in
    -f)
      shift
      outfile="$1"
      ;;
    -y)
      derive="true"
      ;;
    -P)
      shift
      ;;
  esac
  shift
done
if [ "$derive" = "true" ]; then
  printf 'ssh-ed25519 AAAATESTKEY\n'
  exit 0
fi
printf 'PRIVATE-ed25519\n' > "$outfile"
printf 'ssh-ed25519 AAAATESTKEY conflict@fixture\n' > "${{outfile}}.pub"
root="${{outfile%/*}}"
printf 'external config\n' > "${{root}}/conf.d/{host}.conf"
"#
            ),
        )
        .expect("conflicting keygen should be written");
        self.set_mode(&keygen, 0o755);
        keygen
    }

    pub fn install_failing_derive(&self) -> PathBuf {
        let keygen = self.home().join("failing-derive-keygen");
        fs::write(&keygen, "#!/usr/bin/env sh\necho 'injected derive failure' >&2\nexit 29\n")
            .expect("failing derive should be written");
        self.set_mode(&keygen, 0o755);
        keygen
    }

    pub fn install_failing_copy_id(&self) -> PathBuf {
        let copy_id = self.home().join("failing-copy-id");
        fs::write(&copy_id, "#!/usr/bin/env sh\necho 'injected copy-id failure' >&2\nexit 31\n")
            .expect("failing copy-id should be written");
        self.set_mode(&copy_id, 0o755);
        copy_id
    }

    pub fn set_mode(&self, path: &std::path::Path, mode: u32) {
        let mut permissions = fs::metadata(path).expect("metadata should exist").permissions();
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions).expect("mode should be set");
    }
}
