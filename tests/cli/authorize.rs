use crate::harness::TestContext;
use predicates::prelude::*;
use ssv::AppError;

#[test]
fn authorize_invokes_ssh_copy_id_with_config_values() {
    let context = TestContext::new();
    context.write_managed_host_with("mmn", "mmn.local", "ed25519", Some("admin"), Some(2022));

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
fn authorize_targets_hostname_only_without_user() {
    let context = TestContext::new();
    context.write_managed_host_with("box", "box.example", "ed25519", None, None);

    context
        .cli()
        .args(["authorize", "box"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Authorized 'box' public key on box.example"));

    assert!(context.copy_id_invocation().contains("box.example"));
}

#[test]
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

#[test]
fn authorize_reports_copy_id_failure_category() {
    let context = TestContext::new();
    context.write_managed_host("failure.test");
    let error = context
        .ctx_with_copy_id(context.install_failing_copy_id())
        .authorize("failure.test")
        .expect_err("copy-id should fail");

    assert!(matches!(&error, AppError::ExternalCommand { .. }));
    assert!(error.to_string().contains("installing an SSH public key"));
}

#[test]
fn authorize_rejects_ambiguous_loaded_targets_before_copy_id() {
    for (hostname, user) in [("-option", None), ("host.test", Some("bad@user"))] {
        let context = TestContext::new();
        context.write_managed_host_with("unsafe.test", hostname, "ed25519", user, None);

        assert!(context.ctx().authorize("unsafe.test").is_err());
        assert!(context.copy_id_invocation().is_empty());
    }
}
