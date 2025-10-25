mod common;

use common::TestContext;
use serial_test::serial;
use ssv::{generate, list, remove};
use std::fs;

#[test]
#[serial]
fn generate_creates_assets_via_library_api() {
    let ctx = TestContext::new();

    ctx.with_dir(ctx.work_dir(), || {
        generate("code.test", "ed25519", Some("git"), Some(2222)).expect("generate should succeed");
    });

    assert!(ctx.host_config_path("code.test").exists());
    assert!(ctx.private_key_path("ed25519", "code.test").exists());
}

#[test]
#[serial]
fn list_returns_hosts_via_library_api() {
    let ctx = TestContext::new();

    ctx.with_dir(ctx.work_dir(), || {
        generate("first.test", "ed25519", None, None).unwrap();
        generate("second.test", "rsa", None, None).unwrap();
        let mut hosts = list().expect("list should succeed");
        hosts.sort();
        assert_eq!(hosts, vec!["first.test".to_string(), "second.test".to_string()]);
    });
}

#[test]
#[serial]
fn remove_deletes_assets_via_library_api() {
    let ctx = TestContext::new();

    ctx.with_dir(ctx.work_dir(), || {
        generate("remove.test", "ed25519", None, None).unwrap();
    });

    ctx.with_dir(ctx.work_dir(), || {
        remove("remove.test").expect("remove should succeed");
    });

    assert!(!ctx.host_config_path("remove.test").exists());
    assert!(!ctx.private_key_path("ed25519", "remove.test").exists());
}

#[test]
#[serial]
fn remove_does_not_delete_other_hosts_when_guessing() {
    let ctx = TestContext::new();

    ctx.with_dir(ctx.work_dir(), || {
        generate("foo.bar.com", "ed25519", None, None).unwrap();
    });

    let other_key = ctx.private_key_path("ed25519", "foo.bar.com");
    assert!(other_key.exists(), "expected foo.bar.com key to exist before removal");

    ctx.with_dir(ctx.work_dir(), || {
        remove("bar.com").expect("remove should tolerate missing config");
    });

    assert!(other_key.exists(), "remove for bar.com should not delete foo.bar.com key");
}

#[test]
#[serial]
fn remove_ignores_identity_files_outside_ssh_root() {
    let ctx = TestContext::new();

    let outside = ctx.home().join("outside.key");
    fs::write(&outside, "outside").expect("failed to write outside file");

    let config_path = ctx.host_config_path("danger");
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).expect("failed to create conf.d directory");
    }
    fs::write(
        &config_path,
        format!("Host danger\nIdentityFile {}\nIdentitiesOnly yes\n", outside.display()),
    )
    .expect("failed to write config");

    ctx.with_dir(ctx.work_dir(), || {
        remove("danger").expect("remove should succeed");
    });

    assert!(outside.exists(), "outside identity file should not be removed");
    assert!(!config_path.exists(), "config file should be removed");
}
