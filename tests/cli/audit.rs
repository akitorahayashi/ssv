use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn audit_succeeds_for_healthy_assets() {
    let context = TestContext::new();
    context.cli().args(["generate", "--host", "healthy.test"]).assert().success();
    context.prepare_include();

    context.cli().arg("audit").assert().success().stdout("SSH assets are healthy\n").stderr("");
}

#[test]
#[serial]
fn audit_reports_findings_to_stderr_and_fails() {
    let context = TestContext::new();

    context.cli().arg("audit").assert().failure().stderr(predicate::str::contains("[missing]"));
}
