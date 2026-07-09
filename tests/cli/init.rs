use crate::harness::TestContext;
use predicates::prelude::*;
use std::fs;

#[test]
fn init_bootstraps_ssh_layout() {
    let context = TestContext::new();

    context
        .cli()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("Created SSH bootstrap directories and config"));

    context
        .cli()
        .arg("init")
        .assert()
        .success()
        .stdout(predicate::str::contains("SSH bootstrap is already up-to-date"));

    let config = fs::read_to_string(context.main_config()).expect("config should exist");
    assert!(config.contains("Include ~/.ssh/conf.d/*.conf"));
}
