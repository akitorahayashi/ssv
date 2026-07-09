use crate::harness::TestContext;
use ssv::AppError;
use std::fs;

#[test]
fn list_requires_bootstrap_when_managed_directory_is_missing() {
    let context = TestContext::new();
    let ctx = context.ctx();

    assert!(matches!(ctx.list(), Err(AppError::BootstrapRequired(_))));
    assert!(ctx.audit().expect("audit should return findings").has_errors());
    assert!(!context.ssh_root().exists());
}

#[test]
#[cfg(unix)]
fn list_surfaces_permission_errors() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    context.set_mode(&context.ssh_root(), 0o000);

    let result = ctx.list();

    context.set_mode(&context.ssh_root(), 0o700);
    assert!(result.is_err());
}

#[test]
fn show_and_audit_do_not_repair_permissions() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("readonly.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    context.set_mode(&context.private_key("ed25519", "readonly.test"), 0o644);

    ctx.show("readonly.test").expect("show should still read the public key");
    ctx.audit().expect("audit should return findings");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = fs::metadata(context.private_key("ed25519", "readonly.test"))
            .expect("private key metadata")
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o644);
    }
}
