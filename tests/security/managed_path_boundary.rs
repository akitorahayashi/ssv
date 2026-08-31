use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn outside_identity_is_not_read_or_removed() {
    let context = TestContext::new();
    let ctx = context.ctx();
    let outside = context.home().join("outside.key");
    fs::write(&outside, "outside").expect("outside key should be written");
    context.write_host_config("outside.test", outside.to_str().expect("UTF-8 path"));

    assert!(ctx.show("outside.test").is_err());
    assert!(ctx.remove("outside.test").is_err());
    assert!(outside.exists());

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::OutsideManagedRoot));
}

#[test]
fn default_identity_inside_ssh_root_is_not_managed() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    fs::write(context.ssh_root().join("id_ed25519"), "personal key")
        .expect("personal key should be written");
    context.write_host_config("default.test", "~/.ssh/id_ed25519");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::UnmanagedIdentity));
}

#[test]
fn show_rejects_unmanaged_identity_inside_ssh_root() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    fs::write(context.ssh_root().join("id_ed25519"), "personal key")
        .expect("personal key should be written");
    context.write_host_config("default.test", "~/.ssh/id_ed25519");

    assert!(ctx.show("default.test").is_err());
}

#[test]
fn current_standard_identity_is_not_reported_as_managed() {
    let context = TestContext::new();
    context.prepare_include();
    let private = context.ssh_root().join("id_mldsa44_ed25519");
    let public = context.ssh_root().join("id_mldsa44_ed25519.pub");
    fs::write(&private, "personal key").expect("personal private key should be written");
    fs::write(&public, "personal public key").expect("personal public key should be written");

    let report = context.ctx().audit().expect("audit should succeed");
    assert!(!report.findings.iter().any(|finding| {
        finding.code == AuditCode::OrphanedAsset
            && (finding.path == private || finding.path == public)
    }));
}
