use crate::harness::TestContext;
use predicates::prelude::*;
use std::fs;

#[test]
fn generate_outputs_public_key_and_creates_assets() {
    let context = TestContext::new();

    context
        .cli()
        .args(["generate", "github.com", "-u", "git", "-p", "2222"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Generated SSH assets for 'github.com'"))
        .stdout(predicate::str::contains("ssh-ed25519 AAAATESTKEY"));

    assert!(context.host_config("github.com").exists());
    assert!(context.private_key("ed25519", "github.com").exists());
}

#[test]
fn generate_with_custom_hostname() {
    let context = TestContext::new();

    context
        .cli()
        .args(["generate", "github-custom", "-n", "github.com", "-u", "git"])
        .assert()
        .success();

    assert!(context.host_config("github-custom").exists());
    let config_content = fs::read_to_string(context.host_config("github-custom")).unwrap();
    assert!(config_content.contains("Host github-custom"));
    assert!(config_content.contains("HostName github.com"));
    assert!(config_content.contains("User git"));
    assert!(config_content.contains("IdentityFile"));
    assert!(config_content.contains("github-custom"));
}

#[test]
fn generate_rejects_host_colliding_with_reserved_key_name() {
    let context = TestContext::new();

    // Host "sk" with the default ed25519 type renders id_ed25519_sk, a reserved OpenSSH name.
    context.cli().args(["generate", "sk"]).assert().failure();
    assert!(!context.host_config("sk").exists());
    assert!(!context.private_key("ed25519", "sk").exists());

    // A non-colliding key type for the same host is accepted.
    context.cli().args(["generate", "sk", "-t", "rsa"]).assert().success();
    assert!(context.host_config("sk").exists());
}

#[test]
fn generate_rejects_hostname_with_newline() {
    let context = TestContext::new();

    context
        .cli()
        .args([
            "generate",
            "github-inject",
            "-n",
            "github.com\nProxyCommand malicious",
            "-u",
            "git",
        ])
        .assert()
        .failure();
}

#[test]
fn generate_rejects_user_with_newline() {
    let context = TestContext::new();

    context
        .cli()
        .args(["generate", "github-inject", "-u", "git\nProxyCommand malicious"])
        .assert()
        .failure();

    assert!(!context.host_config("github-inject").exists());
}

#[test]
fn generate_reports_rollback_when_public_key_read_fails() {
    let context = TestContext::new();
    let keygen = context.install_private_only_keygen();

    let mut command = context.cli();
    command.env("SSV_SSH_KEYGEN_PATH", &keygen);
    command
        .args(["generate", "rollback.test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Rolled back partial SSH assets due to failure"));

    assert!(!context.host_config("rollback.test").exists());
    assert!(!context.private_key("ed25519", "rollback.test").exists());
    assert!(!context.public_key("ed25519", "rollback.test").exists());
}
