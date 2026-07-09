use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn standard_ssh_keys_are_not_reported_as_orphaned() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    fs::write(context.ssh_root().join("id_ed25519"), "standard key")
        .expect("key should be written");
    fs::write(context.ssh_root().join("id_ed25519_sk"), "standard sk key")
        .expect("sk key should be written");

    let report = ctx.audit().expect("audit should succeed");
    assert!(!report.findings.iter().any(|finding| finding.code == AuditCode::OrphanedAsset));
}

#[test]
fn unreferenced_ssv_key_is_reported_as_orphaned() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("orphan.test", None, "ed25519", None, None).expect("generate should succeed");
    fs::remove_file(context.host_config("orphan.test")).expect("host config should be removed");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::OrphanedAsset));
}
