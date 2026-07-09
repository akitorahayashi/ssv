use crate::harness::TestContext;
use ssv::AuditCode;
use std::fs;

#[test]
fn mismatched_public_key_is_reported() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("mismatch.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    fs::write(context.public_key("ed25519", "mismatch.test"), "ssh-ed25519 DIFFERENT\n")
        .expect("public key should be replaced");

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::KeyMismatch));
}
