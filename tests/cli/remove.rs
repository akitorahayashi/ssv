use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;
use std::fs;

#[test]
#[serial]
fn remove_outputs_result_and_deletes_assets() {
    let context = TestContext::new();
    context.cli().args(["generate", "--host", "cleanup.test"]).assert().success();

    context
        .cli()
        .args(["remove", "--host", "cleanup.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed SSH assets for 'cleanup.test'"));

    assert!(!context.host_config("cleanup.test").exists());
    assert!(!context.private_key("ed25519", "cleanup.test").exists());
}

#[test]
#[serial]
fn remove_reports_when_some_assets_are_already_missing() {
    let context = TestContext::new();
    context.cli().args(["generate", "--host", "partial.test"]).assert().success();
    fs::remove_file(context.private_key("ed25519", "partial.test"))
        .expect("private key should be removed");

    context
        .cli()
        .args(["remove", "--host", "partial.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("already absent"));

    assert!(!context.host_config("partial.test").exists());
}
