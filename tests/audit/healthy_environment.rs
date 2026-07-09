use crate::harness::TestContext;

#[test]
fn healthy_environment_has_no_findings() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("healthy.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();

    assert!(ctx.audit().expect("audit should succeed").findings.is_empty());
}
