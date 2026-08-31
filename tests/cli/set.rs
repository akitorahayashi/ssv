use crate::harness::TestContext;
use predicates::prelude::*;
use std::fs;

#[test]
fn set_updates_hostname_and_preserves_key() {
    let context = TestContext::new();
    context.write_managed_host_with("mmn", "mmn.local", "ed25519", None, None);

    context
        .cli()
        .args(["set", "mmn", "-n", "100.78.35.98"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Updated 'mmn' (HostName 100.78.35.98)"));

    let config = fs::read_to_string(context.host_config("mmn")).unwrap();
    assert!(config.contains("HostName 100.78.35.98"));
    assert!(!config.contains("mmn.local"));
    assert!(config.contains("IdentityFile ~/.ssh/id_ed25519_mmn"));
    assert!(config.contains("IdentitiesOnly yes"));
    assert!(context.private_key("ed25519", "mmn").exists());
    assert!(context.public_key("ed25519", "mmn").exists());
}

#[test]
fn set_updates_user_and_port_and_keeps_hostname() {
    let context = TestContext::new();
    context.write_managed_host_with("mmn", "mmn.local", "ed25519", None, None);

    context.cli().args(["set", "mmn", "-u", "admin", "-p", "2022"]).assert().success();

    let config = fs::read_to_string(context.host_config("mmn")).unwrap();
    assert!(config.contains("HostName mmn.local"));
    assert!(config.contains("User admin"));
    assert!(config.contains("Port 2022"));
}

#[test]
fn set_rejects_user_with_newline() {
    let context = TestContext::new();
    context.write_managed_host_with("mmn", "mmn.local", "ed25519", None, None);

    context.cli().args(["set", "mmn", "-u", "admin\nProxyCommand malicious"]).assert().failure();

    let config = fs::read_to_string(context.host_config("mmn")).unwrap();
    assert!(!config.contains("ProxyCommand"));
}

#[test]
fn set_requires_at_least_one_field() {
    let context = TestContext::new();
    context.write_managed_host("mmn");

    context
        .cli()
        .args(["set", "mmn"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("specify at least one"));
}

#[test]
fn set_fails_for_unknown_host() {
    let context = TestContext::new();
    context.cli().arg("init").assert().success();

    context
        .cli()
        .args(["set", "ghost", "-n", "example.com"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Host 'ghost' was not found"));
}
