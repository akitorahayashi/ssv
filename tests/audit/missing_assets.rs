use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn missing_public_key_is_reported() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("missing.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    fs::remove_file(context.public_key("ed25519", "missing.test")).expect("public key removed");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::Missing));
}
