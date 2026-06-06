use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AuditCode, audit, remove, show};
use std::fs;

#[test]
#[serial]
fn outside_identity_is_not_read_or_removed() {
    let context = TestContext::new();
    let outside = context.home().join("outside.key");
    fs::write(&outside, "outside").expect("outside key should be written");
    context.write_host_config("outside.test", outside.to_str().expect("UTF-8 path"));

    assert!(show("outside.test").is_err());
    assert!(remove("outside.test").is_err());
    assert!(outside.exists());

    let report = audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::OutsideManagedRoot));
}
