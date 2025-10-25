mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn generate_command_provisions_assets() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["generate", "--host", "github.com", "--user", "git", "--port", "2222"])
        .assert()
        .success()
        .stdout(predicate::str::contains("✅ Generated SSH assets for 'github.com'"))
        .stdout(predicate::str::contains("ssh-ed25519 AAAATESTKEY ed25519@ssv"));

    let config = ctx.host_config_path("github.com");
    assert!(config.exists(), "Config file should be created");
    ctx.assert_config_contains("github.com", "Host github.com");
    ctx.assert_config_contains("github.com", "User git");
    ctx.assert_config_contains("github.com", "Port 2222");

    let private_key = ctx.private_key_path("ed25519", "github.com");
    assert!(private_key.exists(), "Private key should be created");
    let contents = fs::read_to_string(private_key).expect("Failed to read private key");
    assert!(contents.contains("PRIVATE-ed25519"));
}

#[test]
#[serial]
fn list_command_outputs_hosts() {
    let ctx = TestContext::new();

    ctx.cli().args(["generate", "--host", "alpha.test"]).assert().success();
    ctx.cli().args(["generate", "--host", "beta.test", "--type", "rsa"]).assert().success();

    ctx.cli()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha.test").and(predicate::str::contains("beta.test")));
}

#[test]
#[serial]
fn remove_command_cleans_up_assets() {
    let ctx = TestContext::new();

    ctx.cli().args(["generate", "--host", "cleanup.test"]).assert().success();

    let config = ctx.host_config_path("cleanup.test");
    assert!(config.exists(), "Config should exist before removal");

    ctx.cli()
        .args(["remove", "--host", "cleanup.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed SSH assets for 'cleanup.test'"));

    assert!(!config.exists(), "Config should be removed");
    let private_key = ctx.private_key_path("ed25519", "cleanup.test");
    assert!(!private_key.exists(), "Private key should be removed");
}
