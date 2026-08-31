use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn missing_public_key_is_reported() {
    let context = TestContext::new();
    context.write_managed_host("missing.test");
    let ctx = context.ctx();
    fs::remove_file(context.public_key("ed25519", "missing.test")).expect("public key removed");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::Missing));
}
