use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn help_lists_show_and_audit() {
    let context = TestContext::new();

    context
        .cli()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("show").and(predicate::str::contains("audit")));
}
