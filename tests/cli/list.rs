use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn list_outputs_managed_hosts() {
    let context = TestContext::new();
    context.cli().args(["generate", "alpha.test"]).assert().success();
    context.cli().args(["generate", "beta.test"]).assert().success();

    context
        .cli()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha.test").and(predicate::str::contains("beta.test")));
}

#[test]
#[serial]
fn list_reports_empty_state_after_init() {
    let context = TestContext::new();

    context.cli().arg("init").assert().success();

    context
        .cli()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("(no hosts managed yet)"));
}
