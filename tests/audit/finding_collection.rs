use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn audit_collects_findings_across_assets() {
    let context = TestContext::new();
    context.write_managed_host("first.test");
    context.write_managed_host("second.test");
    let ctx = context.ctx();
    fs::remove_file(context.public_key("ed25519", "first.test")).expect("public key removed");
    fs::remove_file(context.private_key("ed25519", "second.test")).expect("private key removed");

    let report = ctx.audit().expect("audit should succeed");
    let missing =
        report.findings.iter().filter(|finding| finding.code == AuditCode::Missing).count();
    assert!(missing >= 2, "missing key assets should be reported for both hosts");
}
