use crate::harness::TestContext;
use predicates::prelude::*;
use serial_test::serial;

#[test]
#[serial]
fn help_lists_show_and_audit() {
    let context = TestContext::new();

    context.cli().arg("--help").assert().success().stdout(
        predicate::str::contains("init")
            .and(predicate::str::contains("[aliases: i]"))
            .and(predicate::str::contains("generate"))
            .and(predicate::str::contains("[aliases: g]"))
            .and(predicate::str::contains("list"))
            .and(predicate::str::contains("[aliases: ls]"))
            .and(predicate::str::contains("remove"))
            .and(predicate::str::contains("[aliases: rm]"))
            .and(predicate::str::contains("show"))
            .and(predicate::str::contains("[aliases: sw]"))
            .and(predicate::str::contains("audit"))
            .and(predicate::str::contains("[aliases: au]")),
    );
}
