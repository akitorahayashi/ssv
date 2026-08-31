use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn mismatched_public_key_is_reported() {
    let context = TestContext::new();
    context.write_managed_host("mismatch.test");
    let ctx = context.ctx();
    fs::write(context.public_key("ed25519", "mismatch.test"), "ssh-ed25519 DIFFERENT\n")
        .expect("public key should be replaced");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::KeyMismatch));
}
