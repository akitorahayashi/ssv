use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn authorize_invokes_ssh_copy_id_with_config_values() {
    let context = TestContext::new();
    context
        .cli()
        .args(["generate", "mmn", "-n", "mmn.local", "-u", "admin", "-p", "2022"])
        .assert()
        .success();

    context
        .cli()
        .args(["authorize", "mmn"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Authorized 'mmn' public key on admin@mmn.local"));

    let public_key = context.public_key("ed25519", "mmn");
    let invocation = context.copy_id_invocation();
    let argv: Vec<&str> = invocation.lines().collect();
    assert_eq!(
        argv,
        vec!["-i", public_key.to_str().unwrap(), "-p", "2022", "admin@mmn.local"],
        "unexpected ssh-copy-id argv"
    );
}

#[test]
#[serial]
fn authorize_targets_hostname_only_without_user() {
    let context = TestContext::new();
    context.cli().args(["generate", "box", "-n", "box.example"]).assert().success();

    context
        .cli()
        .args(["authorize", "box"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Authorized 'box' public key on box.example"));

    assert!(context.copy_id_invocation().contains("box.example"));
}

#[test]
#[serial]
fn authorize_fails_for_unknown_host() {
    let context = TestContext::new();
    context.cli().arg("init").assert().success();

    context
        .cli()
        .args(["authorize", "ghost"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Host 'ghost' was not found"));
}
