use crate::harness::TestContext;
use serial_test::serial;
use ssv::{AppError, audit, generate, list, show};
use std::fs;

#[test]
#[serial]
fn list_requires_bootstrap_when_managed_directory_is_missing() {
    let context = TestContext::new();

    assert!(matches!(list(), Err(AppError::BootstrapRequired(_))));
    assert!(audit().expect("audit should return findings").has_errors());
    assert!(!context.ssh_root().exists());
}

#[test]
#[serial]
#[cfg(unix)]
fn list_surfaces_permission_errors() {
    let context = TestContext::new();
    ssv::init().expect("init should succeed");
    context.set_mode(&context.ssh_root(), 0o000);

    let result = list();

    context.set_mode(&context.ssh_root(), 0o700);
    assert!(result.is_err());
}

#[test]
#[serial]
fn show_and_audit_do_not_repair_permissions() {
    let context = TestContext::new();
    generate("readonly.test", "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    context.set_mode(&context.private_key("ed25519", "readonly.test"), 0o644);

    show("readonly.test").expect("show should still read the public key");
    audit().expect("audit should return findings");

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
