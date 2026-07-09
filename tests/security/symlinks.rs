use crate::harness::TestContext;
use ssv::AuditCode;

#[test]
#[cfg(unix)]
fn public_key_symlink_is_rejected_and_reported() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("symlink.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    let public = context.public_key("ed25519", "symlink.test");
    let outside = context.home().join("outside.pub");
    fs::write(&outside, "ssh-ed25519 AAAATESTKEY\n").expect("outside public key written");
    fs::remove_file(&public).expect("generated public key removed");
    symlink(outside, &public).expect("symlink should be created");

    assert!(ctx.show("symlink.test").is_err());
    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::InvalidFileType));
}

#[test]
#[cfg(unix)]
fn generate_rejects_broken_key_symlink() {
    use std::os::unix::fs::symlink;

    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    let outside = context.home().join("outside-private-key");
    symlink(&outside, context.private_key("ed25519", "broken.test"))
        .expect("broken symlink should be created");

    assert!(ctx.generate("broken.test", None, "ed25519", None, None).is_err());
    assert!(!outside.exists());
}

#[test]
#[cfg(unix)]
fn init_rejects_symlinked_ssh_root() {
    use std::fs;
    use std::os::unix::fs::symlink;

    let context = TestContext::new();
    let ctx = context.ctx();
    let outside = context.home().join("outside-ssh");
    fs::create_dir(&outside).expect("outside directory should be created");
    symlink(&outside, context.ssh_root()).expect("ssh root symlink should be created");

    assert!(ctx.init().is_err());
    assert!(!outside.join("conf.d").exists());
}
