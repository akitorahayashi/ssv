use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[test]
#[serial]
fn generate_outputs_public_key_and_creates_assets() {
    let context = TestContext::new();

    context
        .cli()
        .args(["generate", "--host", "github.com", "--user", "git", "--port", "2222"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated SSH assets for 'github.com'"))
        .stdout(predicate::str::contains("ssh-ed25519 AAAATESTKEY"));

    assert!(context.host_config("github.com").exists());
    assert!(context.private_key("ed25519", "github.com").exists());
}

#[test]
#[serial]
fn generate_reports_rollback_when_public_key_read_fails() {
    let context = TestContext::new();
    let keygen = context.home().join("private-only-keygen");
    fs::write(
        &keygen,
        "#!/usr/bin/env sh\nset -eu\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-f\" ]; then\n    shift\n    outfile=\"$1\"\n  fi\n  shift\ndone\nprintf 'PRIVATE-ed25519\\n' > \"$outfile\"\n",
    )
    .expect("keygen should be written");
    let mut permissions = fs::metadata(&keygen).expect("keygen metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&keygen, permissions).expect("keygen should be executable");
    unsafe { std::env::set_var("SSV_SSH_KEYGEN_PATH", &keygen) };

    let mut command = context.cli();
    command.env("SSV_SSH_KEYGEN_PATH", &keygen);
    command
        .args(["generate", "--host", "rollback.test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Rolled back partial SSH assets due to failure"));

    assert!(!context.host_config("rollback.test").exists());
    assert!(!context.private_key("ed25519", "rollback.test").exists());
    assert!(!context.public_key("ed25519", "rollback.test").exists());
}
