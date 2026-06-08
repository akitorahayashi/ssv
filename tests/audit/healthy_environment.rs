use crate::harness::TestContext;
use serial_test::serial;
use ssv::{audit, generate};

#[test]
#[serial]
fn healthy_environment_has_no_findings() {
    let context = TestContext::new();
    generate("healthy.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();

    assert!(audit().expect("audit should succeed").findings.is_empty());
}
