use crate::harness::TestContext;
use git2::Repository;
use ssv::{AppError, GitOperation};
use std::fs;

#[test]
fn link_uses_the_supplied_repository_start_path() {
    let context = TestContext::new();
    let host = "work.test";
    context.write_managed_host(host);
    let repository_path = context.home().join("repository");
    fs::create_dir(&repository_path).expect("repository directory");
    let repository = Repository::init(&repository_path).expect("repository");
    repository.remote("origin", "https://example.test/owner/project.git").expect("origin");

    let url = context.ctx().link(&repository_path, host).expect("link should use explicit path");

    assert_eq!(url, "git@work.test:owner/project.git");
}

#[test]
fn link_errors_retain_git_operation_categories() {
    let context = TestContext::new();
    context.write_managed_host("work.test");
    let ctx = context.ctx();
    let non_repository = context.home().join("not-a-repository");
    fs::create_dir(&non_repository).expect("directory");

    assert!(matches!(
        ctx.link(&non_repository, "work.test"),
        Err(AppError::Git { operation: GitOperation::DiscoverRepository, .. })
    ));

    let repository_path = context.home().join("repository");
    fs::create_dir(&repository_path).expect("repository directory");
    let repository = Repository::init(&repository_path).expect("repository");
    assert!(matches!(
        ctx.link(&repository_path, "work.test"),
        Err(AppError::Git { operation: GitOperation::ReadOrigin, .. })
    ));

    repository.remote("origin", "ftp://example.test/owner/project.git").expect("origin");
    assert!(matches!(
        ctx.link(&repository_path, "work.test"),
        Err(AppError::Git { operation: GitOperation::ParseOrigin, .. })
    ));
}

#[test]
fn link_update_failure_has_its_own_category() {
    use std::os::unix::fs::PermissionsExt;

    let context = TestContext::new();
    context.write_managed_host("work.test");
    let repository_path = context.home().join("read-only-repository");
    fs::create_dir(&repository_path).expect("repository directory");
    let repository = Repository::init(&repository_path).expect("repository");
    repository.remote("origin", "https://example.test/owner/project.git").expect("origin");
    let git_directory = repository.path().to_path_buf();
    fs::set_permissions(&git_directory, fs::Permissions::from_mode(0o500))
        .expect("git directory should become read-only");

    let result = context.ctx().link(&repository_path, "work.test");

    fs::set_permissions(&git_directory, fs::Permissions::from_mode(0o700))
        .expect("git directory mode should be restored");
    assert!(matches!(result, Err(AppError::Git { operation: GitOperation::UpdateOrigin, .. })));
}
