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

#[test]
fn init_creates_private_modes_under_a_permissive_umask() {
    use std::os::unix::fs::PermissionsExt;

    let context = TestContext::new();
    context.cli_with_permissive_umask().arg("init").assert().success();

    for (path, expected) in
        [(context.ssh_root(), 0o700), (context.hosts_dir(), 0o700), (context.main_config(), 0o600)]
    {
        let mode = fs::metadata(path).expect("metadata").permissions().mode() & 0o777;
        assert_eq!(mode, expected);
    }
}
