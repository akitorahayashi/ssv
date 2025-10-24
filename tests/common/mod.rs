//! Shared testing utilities for the `ssv` CLI and library.

use assert_cmd::Command;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Testing harness providing an isolated HOME/workspace pair and stubbed ssh-keygen.
#[allow(dead_code)]
pub struct TestContext {
    root: TempDir,
    work_dir: PathBuf,
    original_home: Option<OsString>,
    original_keygen: Option<OsString>,
    keygen_stub: PathBuf,
}

#[allow(dead_code)]
impl TestContext {
    /// Create a new isolated environment and point `HOME` to it so the CLI uses local storage.
    pub fn new() -> Self {
        let root = TempDir::new().expect("Failed to create temp directory for tests");
        let work_dir = root.path().join("work");
        fs::create_dir_all(&work_dir).expect("Failed to create test work directory");

        let bin_dir = root.path().join("bin");
        fs::create_dir_all(&bin_dir).expect("Failed to create stub bin directory");
        let keygen_stub = bin_dir.join("ssh-keygen");
        Self::write_keygen_stub(&keygen_stub);

        let original_home = env::var_os("HOME");
        unsafe {
            env::set_var("HOME", root.path());
        }

        let original_keygen = env::var_os("SSV_SSH_KEYGEN_PATH");
        unsafe {
            env::set_var("SSV_SSH_KEYGEN_PATH", &keygen_stub);
        }

        Self { root, work_dir, original_home, original_keygen, keygen_stub }
    }

    /// Absolute path to the emulated `$HOME` directory.
    pub fn home(&self) -> &Path {
        self.root.path()
    }

    /// Path to the workspace directory used for CLI invocations.
    pub fn work_dir(&self) -> &Path {
        &self.work_dir
    }

    /// Build a command for invoking the compiled `ssv` binary within the default workspace.
    pub fn cli(&self) -> Command {
        self.cli_in(self.work_dir())
    }

    /// Build a command for invoking the compiled `ssv` binary within a custom directory.
    pub fn cli_in<P: AsRef<Path>>(&self, dir: P) -> Command {
        let mut cmd = Command::cargo_bin("ssv").expect("Failed to locate ssv binary");
        cmd.current_dir(dir.as_ref())
            .env("HOME", self.home())
            .env("SSV_SSH_KEYGEN_PATH", &self.keygen_stub);
        cmd
    }

    /// Path to the configuration file generated for a host.
    pub fn host_config_path(&self, host: &str) -> PathBuf {
        self.home().join(".ssh").join("conf.d").join(format!("{host}.conf"))
    }

    /// Path to the private key generated for a host (defaults to ed25519).
    pub fn private_key_path(&self, key_type: &str, host: &str) -> PathBuf {
        self.home().join(".ssh").join(format!("id_{}_{}", key_type, host))
    }

    /// Path to the public key generated for a host (defaults to ed25519).
    pub fn public_key_path(&self, key_type: &str, host: &str) -> PathBuf {
        self.home().join(".ssh").join(format!("id_{}_{}.pub", key_type, host))
    }

    /// Assert that a configuration file contains the expected content snippet.
    pub fn assert_config_contains(&self, host: &str, needle: &str) {
        let config = fs::read_to_string(self.host_config_path(host)).expect("Config not readable");
        assert!(
            config.contains(needle),
            "Config for {host} did not contain `{needle}`.\nContents:\n{config}"
        );
    }

    fn write_keygen_stub(path: &Path) {
        let script = r#"#!/usr/bin/env sh
set -eu
outfile=""
keytype="stub"
while [ "$#" -gt 0 ]; do
  arg="$1"
  shift
  case "$arg" in
    -f)
      if [ "$#" -eq 0 ]; then
        echo "missing -f argument" >&2
        exit 1
      fi
      outfile="$1"
      shift
      ;;
    -t)
      if [ "$#" -eq 0 ]; then
        echo "missing -t argument" >&2
        exit 1
      fi
      keytype="$1"
      shift
      ;;
    *)
      ;;
  esac
done
if [ -z "$outfile" ]; then
  echo "missing -f argument" >&2
  exit 1
fi
printf 'PRIVATE-%s\n' "$keytype" > "$outfile"
printf 'ssh-%s AAAATESTKEY %s@ssv\n' "$keytype" "$keytype" > "${outfile}.pub"
"#;
        fs::write(path, script).expect("Failed to create ssh-keygen stub");
        #[cfg(unix)]
        {
            let mut perms = fs::metadata(path).expect("stub metadata").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("stub chmod");
        }
    }

    /// Execute a closure after temporarily switching into the provided directory.
    pub fn with_dir<F, R, P>(&self, dir: P, action: F) -> R
    where
        F: FnOnce() -> R,
        P: AsRef<Path>,
    {
        let original = env::current_dir().expect("Failed to capture current dir");
        env::set_current_dir(dir.as_ref()).expect("Failed to switch current dir");
        let result = action();
        env::set_current_dir(original).expect("Failed to restore current dir");
        result
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        match &self.original_home {
            Some(value) => unsafe {
                env::set_var("HOME", value);
            },
            None => unsafe {
                env::remove_var("HOME");
            },
        }

        match &self.original_keygen {
            Some(value) => unsafe {
                env::set_var("SSV_SSH_KEYGEN_PATH", value);
            },
            None => unsafe {
                env::remove_var("SSV_SSH_KEYGEN_PATH");
            },
        }
    }
}
