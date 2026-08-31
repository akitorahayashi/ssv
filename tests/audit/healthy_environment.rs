use crate::harness::TestContext;

#[test]
fn healthy_environment_has_no_findings() {
    let context = TestContext::new();
    context.write_managed_host("healthy.test");
    let ctx = context.ctx();

    assert!(ctx.audit().expect("audit should succeed").findings.is_empty());
}
