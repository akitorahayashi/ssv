mod common;

use common::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn end_to_end_generate_list_remove_cycle() {
    let ctx = TestContext::new();

    ctx.cli()
        .args(["generate", "--host", "workflow.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("workflow.test"));

    ctx.cli().arg("list").assert().success().stdout(predicate::str::contains("workflow.test"));

    let pub_key = ctx.public_key_path("ed25519", "workflow.test");
    assert!(pub_key.exists(), "Public key should exist prior to removal");

    ctx.cli()
        .args(["remove", "--host", "workflow.test"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed SSH assets for 'workflow.test'"));

    assert!(!ctx.host_config_path("workflow.test").exists());
    assert!(!pub_key.exists());
    assert!(!ctx.private_key_path("ed25519", "workflow.test").exists());
}
