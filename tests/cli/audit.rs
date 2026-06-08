use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn audit_succeeds_for_healthy_assets() {
    let context = TestContext::new();
    context.cli().args(["generate", "healthy.test"]).assert().success();
    context.prepare_include();

    context.cli().arg("audit").assert().success().stdout("SSH assets are healthy\n").stderr("");
}

#[test]
#[serial]
fn audit_reports_findings_to_stderr_and_fails() {
    let context = TestContext::new();

    context.cli().arg("audit").assert().failure().stderr(predicate::str::contains("[missing]"));
}

#[test]
#[serial]
#[cfg(unix)]
fn audit_reports_warning_only_assets_without_claiming_health() {
    let context = TestContext::new();
    context.cli().args(["generate", "warning.test"]).assert().success();
    context.set_mode(&context.host_config("warning.test"), 0o400);

    context
        .cli()
        .arg("audit")
        .assert()
        .success()
        .stdout("SSH assets have warnings\n")
        .stderr(predicate::str::contains("[non-standard-permissions]"));
}
