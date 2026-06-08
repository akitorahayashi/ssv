use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AuditCode, audit, generate, init};
use std::fs;

#[test]
#[serial]
fn standard_ssh_keys_are_not_reported_as_orphaned() {
    let context = TestContext::new();
    init().expect("init should succeed");
    fs::write(context.ssh_root().join("id_ed25519"), "standard key")
        .expect("key should be written");
    fs::write(context.ssh_root().join("id_ed25519_sk"), "standard sk key")
        .expect("sk key should be written");

    let report = audit().expect("audit should succeed");
    assert!(!report.findings.iter().any(|finding| finding.code == AuditCode::OrphanedAsset));
}

#[test]
#[serial]
fn unreferenced_ssv_key_is_reported_as_orphaned() {
    let context = TestContext::new();
    generate("orphan.test", None, "ed25519", None, None).expect("generate should succeed");
    fs::remove_file(context.host_config("orphan.test")).expect("host config should be removed");

    let report = audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::OrphanedAsset));
}
