use crate::harness::TestContext;

#[test]
fn managed_host_lifecycle_uses_public_api() {
    let context = TestContext::new();
    let ctx = context.ctx();

    let generated =
        ctx.generate("flow.test", None, "ed25519", None, None).expect("generate should succeed");
    assert_eq!(ctx.show("flow.test").expect("show should succeed"), generated);
    assert_eq!(ctx.list().expect("list should succeed"), vec!["flow.test"]);

    ctx.remove("flow.test").expect("remove should succeed");
    assert!(!context.host_config("flow.test").exists());
    assert!(!context.private_key("ed25519", "flow.test").exists());
}

#[test]
fn generate_removes_artifacts_when_public_key_read_fails() {
    let context = TestContext::new();
    let ctx = context.ctx_with_keygen(context.install_private_only_keygen());

    assert!(ctx.generate("rollback.test", None, "ed25519", None, None).is_err());

    assert!(!context.host_config("rollback.test").exists());
    assert!(!context.private_key("ed25519", "rollback.test").exists());
    assert!(!context.public_key("ed25519", "rollback.test").exists());
}
