use crate::harness::TestContext;
use std::fs;

#[test]
fn remove_missing_host_does_not_guess_key_names() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("foo.bar.com", None, "ed25519", None, None).expect("generate should succeed");

    assert!(ctx.remove("bar.com").is_err());
    assert!(context.private_key("ed25519", "foo.bar.com").exists());
}

#[test]
fn remove_refuses_identity_not_owned_by_host() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.init().expect("init should succeed");
    let default_key = context.ssh_root().join("id_ed25519");
    fs::write(&default_key, "personal private key").expect("default key should be written");
    context.write_host_config("personal.test", "~/.ssh/id_ed25519");

    assert!(ctx.remove("personal.test").is_err());
    assert_eq!(
        fs::read_to_string(default_key).expect("default key should remain"),
        "personal private key"
    );
    assert!(context.host_config("personal.test").exists());
}

#[test]
fn remove_preflights_all_targets_and_remains_retryable() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("retry.test", None, "ed25519", None, None).expect("generate should succeed");
    fs::remove_file(context.public_key("ed25519", "retry.test"))
        .expect("public key should be removed");
    fs::create_dir(context.public_key("ed25519", "retry.test"))
        .expect("public key path should block removal");

    assert!(ctx.remove("retry.test").is_err());

    assert!(context.host_config("retry.test").exists());
    assert!(context.private_key("ed25519", "retry.test").exists());

    fs::remove_dir(context.public_key("ed25519", "retry.test"))
        .expect("blocking directory should be removed");
    ctx.remove("retry.test").expect("retry should succeed");
    assert!(!context.host_config("retry.test").exists());
    assert!(!context.private_key("ed25519", "retry.test").exists());
}
