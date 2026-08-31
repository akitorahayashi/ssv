use crate::harness::TestContext;
use predicates::prelude::*;

#[test]
fn list_outputs_managed_hosts() {
    let context = TestContext::new();
    context.write_managed_host("alpha.test");
    context.write_managed_host("beta.test");

    context
        .cli()
        .arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("alpha.test").and(predicate::str::contains("beta.test")));
}

#[test]
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
