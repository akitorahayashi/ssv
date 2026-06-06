use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AuditCode, audit, generate, show};

#[test]
#[serial]
#[cfg(unix)]
fn public_key_symlink_is_rejected_and_reported() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let context = TestContext::new();
    generate("symlink.test", "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    let public = context.public_key("ed25519", "symlink.test");
    let outside = context.home().join("outside.pub");
    fs::write(&outside, "ssh-ed25519 AAAATESTKEY\n").expect("outside public key written");
    fs::remove_file(&public).expect("generated public key removed");
    symlink(outside, &public).expect("symlink should be created");

    assert!(show("symlink.test").is_err());
    let report = audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::InvalidFileType));
}
