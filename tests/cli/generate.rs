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
        .stderr(predicate::str::contains("operation-owned files were rolled back"));

    assert!(!context.host_config("rollback.test").exists());
    assert!(!context.private_key("ed25519", "rollback.test").exists());
    assert!(!context.public_key("ed25519", "rollback.test").exists());
}

#[test]
fn generate_reports_captured_keygen_stderr() {
    let context = TestContext::new();
    let keygen = context.install_failing_keygen(true);
    let mut command = context.cli();
    command.env("SSV_SSH_KEYGEN_PATH", keygen);

    command
        .args(["generate", "failure.test"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("injected keygen failure"));
}

#[test]
fn generate_rejects_option_like_connection_values_before_bootstrap() {
    for (host, hostname, user) in [
        ("-host", None, None),
        ("safe.test", Some("-hostname"), None),
        ("safe.test", None, Some("-user")),
        ("safe.test", None, Some("user@host")),
    ] {
        let context = TestContext::new();
        assert!(context.ctx().generate(host, hostname, "ed25519", user, None).is_err());
        assert!(!context.ssh_root().exists());
    }
}
