use crate::harness::TestContext;
use serial_test::serial;
use ssv::{generate, list, remove, show};

#[test]
#[serial]
fn managed_host_lifecycle_uses_public_api() {
    let context = TestContext::new();

    let generated =
        generate("flow.test", None, "ed25519", None, None).expect("generate should succeed");
    assert_eq!(show("flow.test").expect("show should succeed"), generated);
    assert_eq!(list().expect("list should succeed"), vec!["flow.test"]);

    remove("flow.test").expect("remove should succeed");
    assert!(!context.host_config("flow.test").exists());
    assert!(!context.private_key("ed25519", "flow.test").exists());
}

#[test]
#[serial]
#[cfg(unix)]
fn generate_removes_artifacts_when_public_key_read_fails() {
    let context = TestContext::new();
    let keygen = context.install_private_only_keygen();
    unsafe { std::env::set_var("SSV_SSH_KEYGEN_PATH", keygen) };

    assert!(generate("rollback.test", None, "ed25519", None, None).is_err());

    assert!(!context.host_config("rollback.test").exists());
    assert!(!context.private_key("ed25519", "rollback.test").exists());
    assert!(!context.public_key("ed25519", "rollback.test").exists());
}
