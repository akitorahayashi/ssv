use crate::harness::TestContext;
use serial_test::serial;
use ssv::{generate, init, remove};
use std::fs;

#[test]
#[serial]
fn remove_missing_host_does_not_guess_key_names() {
    let context = TestContext::new();
    generate("foo.bar.com", None, "ed25519", None, None).expect("generate should succeed");

    assert!(remove("bar.com").is_err());
    assert!(context.private_key("ed25519", "foo.bar.com").exists());
}

#[test]
#[serial]
fn remove_refuses_identity_not_owned_by_host() {
    let context = TestContext::new();
    init().expect("init should succeed");
    let default_key = context.ssh_root().join("id_ed25519");
    fs::write(&default_key, "personal private key").expect("default key should be written");
    context.write_host_config("personal.test", "~/.ssh/id_ed25519");

    assert!(remove("personal.test").is_err());
    assert_eq!(
        fs::read_to_string(default_key).expect("default key should remain"),
        "personal private key"
    );
    assert!(context.host_config("personal.test").exists());
}

#[test]
#[serial]
fn remove_removes_host_definition_before_key_cleanup_failure() {
    let context = TestContext::new();
    generate("retry.test", None, "ed25519", None, None).expect("generate should succeed");
    fs::remove_file(context.public_key("ed25519", "retry.test"))
        .expect("public key should be removed");
    fs::create_dir(context.public_key("ed25519", "retry.test"))
        .expect("public key path should block removal");

    assert!(remove("retry.test").is_err());

    assert!(!context.host_config("retry.test").exists());
    assert!(!context.private_key("ed25519", "retry.test").exists());
}
