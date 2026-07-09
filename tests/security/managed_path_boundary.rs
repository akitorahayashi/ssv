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
