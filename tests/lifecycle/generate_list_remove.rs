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

#[test]
fn nonzero_keygen_cleanup_permits_retry() {
    for writes_public in [false, true] {
        let context = TestContext::new();
        let failing = context.ctx_with_keygen(context.install_failing_keygen(writes_public));

        assert!(failing.generate("retry.test", None, "ed25519", None, None).is_err());
        assert!(!context.host_config("retry.test").exists());
        assert!(!context.private_key("ed25519", "retry.test").exists());
        assert!(!context.public_key("ed25519", "retry.test").exists());

        context
            .ctx()
            .generate("retry.test", None, "ed25519", None, None)
            .expect("retry should succeed");
    }
}

#[test]
fn config_conflict_does_not_clobber_external_file_and_rolls_back_keys() {
    let context = TestContext::new();
    let host = "conflict.test";
    let ctx = context.ctx_with_keygen(context.install_config_conflict_keygen(host));

    assert!(ctx.generate(host, None, "ed25519", None, None).is_err());

    assert_eq!(
        std::fs::read_to_string(context.host_config(host)).expect("external config"),
        "external config\n"
    );
    assert!(!context.private_key("ed25519", host).exists());
    assert!(!context.public_key("ed25519", host).exists());
}
