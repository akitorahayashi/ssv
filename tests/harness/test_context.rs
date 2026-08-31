use super::{copy_id_stub, keygen_stub};
use assert_cmd::Command;
use ssv::Context;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

pub struct TestContext {
    pub(crate) root: TempDir,
    work_dir: PathBuf,
    keygen_stub: PathBuf,
    copy_id_stub: PathBuf,
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
        let copy_id_stub = bin_dir.join("ssh-copy-id");
        copy_id_stub::write(&copy_id_stub);

        Self { root, work_dir, keygen_stub, copy_id_stub }
    }

    pub fn home(&self) -> &Path {
        self.root.path()
    }

    /// A library context bound to this test's temporary home and stub binaries, for in-process
    /// calls. It reads no process environment, so tests need no global mutation or serialization.
    pub fn ctx(&self) -> Context {
        self.ctx_with_keygen(self.keygen_stub.clone())
    }

    /// Like [`TestContext::ctx`] but with a caller-supplied keygen stub (e.g. the private-only
    /// stub used to drive the `generate` rollback path).
    pub fn ctx_with_keygen(&self, keygen: PathBuf) -> Context {
        Context::new(self.home().to_path_buf(), keygen, self.copy_id_stub.clone())
    }

    pub fn cli(&self) -> Command {
        let mut command = Command::cargo_bin("ssv").expect("ssv binary should exist");
        command
            .current_dir(&self.work_dir)
            .env("HOME", self.home())
            .env("SSV_SSH_KEYGEN_PATH", &self.keygen_stub)
            .env("SSV_SSH_COPY_ID_PATH", &self.copy_id_stub);
        command
    }

    pub fn cli_with_permissive_umask(&self) -> Command {
        let binary = assert_cmd::cargo::cargo_bin("ssv");
        let mut command = Command::new("sh");
        command
            .args(["-c", "umask 000; exec \"$@\"", "sh"])
            .arg(binary)
            .current_dir(&self.work_dir)
            .env("HOME", self.home())
            .env("SSV_SSH_KEYGEN_PATH", &self.keygen_stub)
            .env("SSV_SSH_COPY_ID_PATH", &self.copy_id_stub);
        command
    }
}
