use crate::harness::TestContext;
use git2::Repository;
use predicates::prelude::*;
use std::fs;
use std::path::Path;

#[test]
fn link_updates_remote_url_from_ssh() {
    let context = TestContext::new();
    let host = "github.com-work";

    // Setup ssv host
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    // Setup repository
    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    init_repository_with_origin(&repo_dir, "git@github.com:org/repo.git");

    // Run ssv link
    context
        .cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Linked repository to '{host}'")))
        .stdout(predicate::str::contains(format!("new remote URL: git@{host}:org/repo.git")));

    assert_eq!(origin_url(&repo_dir), format!("git@{host}:org/repo.git"));
}

#[test]
fn link_updates_remote_url_from_https() {
    let context = TestContext::new();
    let host = "github.com-work";

    // Setup ssv host
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    // Setup repository
    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    init_repository_with_origin(&repo_dir, "https://github.com/org/repo.git");

    // Run ssv link
    context
        .cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .success()
        .stdout(predicate::str::contains(format!("Linked repository to '{host}'")))
        .stdout(predicate::str::contains(format!("new remote URL: git@{host}:org/repo.git")));

    assert_eq!(origin_url(&repo_dir), format!("git@{host}:org/repo.git"));
}

#[test]
fn link_fails_if_host_not_found() {
    let context = TestContext::new();
    context.cli().arg("init").assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    init_repository_with_origin(&repo_dir, "git@github.com:org/repo.git");

    context
        .cli()
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

    context.cli().arg("link").arg(host).current_dir(&non_repo_dir).assert().failure().stderr(
        predicate::str::contains("Error: current directory is not inside a Git repository"),
    );
}

#[test]
fn link_fails_if_no_origin_remote() {
    let context = TestContext::new();
    let host = "github.com";
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    Repository::init(&repo_dir).unwrap();

    context
        .cli()
        .arg("link")
        .arg(host)
        .current_dir(&repo_dir)
        .assert()
        .failure()
        .stderr(predicate::str::contains("Error: origin remote was not found"));
}

#[test]
fn link_fails_on_unsupported_url_format() {
    let context = TestContext::new();
    let host = "github.com";
    context.cli().arg("init").assert().success();
    context.cli().arg("generate").arg(host).assert().success();

    let repo_dir = context.home().join("repo");
    fs::create_dir(&repo_dir).unwrap();
    init_repository_with_origin(&repo_dir, "ftp://github.com/org/repo.git");

    context.cli().arg("link").arg(host).current_dir(&repo_dir).assert().failure().stderr(
        predicate::str::starts_with(
            "Error: unsupported git remote URL format: ftp://github.com/org/repo.git",
        ),
    );
}

fn init_repository_with_origin(path: &Path, url: &str) {
    let repository = Repository::init(path).unwrap();
    repository.remote("origin", url).unwrap();
}

fn origin_url(path: &Path) -> String {
    let repository = Repository::open(path).unwrap();
    repository.find_remote("origin").unwrap().url().unwrap().to_string()
}
