use crate::harness::TestContext;
use predicates::prelude::*;
use std::fs;
use std::process::Command;

#[test]
fn link_updates_remote_url_from_ssh() {
    let context = TestContext::new();
    let host = "github.com-work";

    // Setup ssv host
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    // Setup git repo
    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();

    Command::new("git").arg("init").current_dir(&repo_dir).status().unwrap();
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("git@github.com:org/repo.git")
        .current_dir(&repo_dir)
        .status()
        .unwrap();

    // Run ssv link
    context.cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Linked repository to '{host}'")))
        .stdout(predicate::str::contains(format!("new remote URL: git@{host}:org/repo.git")));

    // Verify git remote
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    let url = String::from_utf8(output.stdout).unwrap();
    assert_eq!(url.trim(), format!("git@{host}:org/repo.git"));
}

#[test]
fn link_updates_remote_url_from_https() {
    let context = TestContext::new();
    let host = "github.com-work";

    // Setup ssv host
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    // Setup git repo
    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();

    Command::new("git").arg("init").current_dir(&repo_dir).status().unwrap();
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("https://github.com/org/repo.git")
        .current_dir(&repo_dir)
        .status()
        .unwrap();

    // Run ssv link
    context.cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Linked repository to '{host}'")))
        .stdout(predicate::str::contains(format!("new remote URL: git@{host}:org/repo.git")));

    // Verify git remote
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .current_dir(&repo_dir)
        .output()
        .unwrap();
    let url = String::from_utf8(output.stdout).unwrap();
    assert_eq!(url.trim(), format!("git@{host}:org/repo.git"));
}

#[test]
fn link_fails_if_host_not_found() {
    let context = TestContext::new();
    context.cli().arg("init").assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    Command::new("git").arg("init").current_dir(&repo_dir).status().unwrap();
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("git@github.com:org/repo.git")
        .current_dir(&repo_dir)
        .status()
        .unwrap();

    context.cli()
        .arg("link")
        .arg("unknown-host")
        .current_dir(&repo_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: Host 'unknown-host' was not found"));
}

#[test]
fn link_fails_if_not_git_repo() {
    let context = TestContext::new();
    let host = "github.com";
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    let non_repo_dir = context.home().join("not-a-repo");
    fs::create_dir(&non_repo_dir).unwrap();

    context.cli()
        .arg("link")
        .arg(host)
        .current_dir(&non_repo_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Command 'git' exited with status"));
}

#[test]
fn link_fails_if_no_origin_remote() {
    let context = TestContext::new();
    let host = "github.com";
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    Command::new("git").arg("init").current_dir(&repo_dir).status().unwrap();

    context.cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Command 'git' exited with status"));
}

#[test]
fn link_fails_on_unsupported_url_format() {
    let context = TestContext::new();
    let host = "github.com";
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    Command::new("git").arg("init").current_dir(&repo_dir).status().unwrap();
    Command::new("git")
        .arg("remote")
        .arg("add")
        .arg("origin")
        .arg("ftp://github.com/org/repo.git")
        .current_dir(&repo_dir)
        .status()
        .unwrap();

    context.cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: unsupported git remote URL format: ftp://github.com/org/repo.git"));
}
