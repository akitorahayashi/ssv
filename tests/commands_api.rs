mod common;

use common::TestContext;
use serial_test::serial;
use ssv::{generate, list, remove};

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
