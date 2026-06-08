use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AuditCode, audit, generate};
use std::fs;

#[test]
#[serial]
fn mismatched_public_key_is_reported() {
    let context = TestContext::new();
    generate("mismatch.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    fs::write(context.public_key("ed25519", "mismatch.test"), "ssh-ed25519 DIFFERENT\n")
        .expect("public key should be replaced");

    let report = audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::KeyMismatch));
}
