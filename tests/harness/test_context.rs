use super::keygen_stub;
use assert_cmd::Command;
use std::env;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    pub(crate) root: TempDir,
    work_dir: PathBuf,
    original_home: Option<OsString>,
    original_keygen: Option<OsString>,
    keygen_stub: PathBuf,
}

impl TestContext {
    pub fn new() -> Self {
        let root = TempDir::new().expect("test root should be created");
        let work_dir = root.path().join("work");
        fs::create_dir(&work_dir).expect("work directory should be created");
        let bin_dir = root.path().join("bin");
        fs::create_dir(&bin_dir).expect("bin directory should be created");
        let keygen_stub = bin_dir.join("ssh-keygen");
        keygen_stub::write(&keygen_stub);

        let original_home = env::var_os("HOME");
        let original_keygen = env::var_os("SSV_SSH_KEYGEN_PATH");
        unsafe {
            env::set_var("HOME", root.path());
            env::set_var("SSV_SSH_KEYGEN_PATH", &keygen_stub);
        }

        Self { root, work_dir, original_home, original_keygen, keygen_stub }
    }

    pub fn home(&self) -> &Path {
        self.root.path()
    }

    pub fn cli(&self) -> Command {
        let mut command = Command::cargo_bin("ssv").expect("ssv binary should exist");
        command
            .current_dir(&self.work_dir)
            .env("HOME", self.home())
            .env("SSV_SSH_KEYGEN_PATH", &self.keygen_stub);
        command
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        match &self.original_home {
            Some(value) => unsafe { env::set_var("HOME", value) },
            None => unsafe { env::remove_var("HOME") },
        }
        match &self.original_keygen {
            Some(value) => unsafe { env::set_var("SSV_SSH_KEYGEN_PATH", value) },
            None => unsafe { env::remove_var("SSV_SSH_KEYGEN_PATH") },
        }
    }
}
