use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

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
