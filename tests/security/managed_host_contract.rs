use crate::harness::TestContext;
use git2::Repository;
use ssv::AuditCode;
use std::fs;

#[test]
fn invalid_managed_document_is_rejected_before_external_effects() {
    let context = TestContext::new();
    let host = "contract.test";
    context.write_managed_host(host);
    fs::write(
        context.host_config(host),
        format!("Host {host}\nHostName {host}\nIdentityFile ~/.ssh/id_ed25519_{host}\n"),
    )
    .expect("invalid config should be written");
    let ctx = context.ctx();

    assert!(ctx.show(host).is_err());
    assert!(ctx.authorize(host).is_err());
    assert!(ctx.set(host, Some("new.test"), None, None).is_err());
    assert!(ctx.remove(host).is_err());
    assert!(context.copy_id_invocation().is_empty());
    assert!(context.host_config(host).exists());
    assert!(context.private_key("ed25519", host).exists());
    assert!(context.public_key("ed25519", host).exists());

    let report = ctx.audit().expect("audit should collect the invalid document");
    assert!(report.findings.iter().any(|finding| finding.code == AuditCode::ConfigParse));
}

#[test]
fn link_does_not_change_origin_for_an_invalid_managed_document() {
    let context = TestContext::new();
    let host = "contract.test";
    context.write_managed_host(host);
    fs::write(
        context.host_config(host),
        format!(
            "Host other.test\nHostName {host}\nIdentityFile ~/.ssh/id_ed25519_{host}\nIdentitiesOnly yes\n"
        ),
    )
    .expect("invalid config should be written");
    let repository_path = context.home().join("repository");
    fs::create_dir(&repository_path).expect("repository directory should exist");
    let repository = Repository::init(&repository_path).expect("repository should initialize");
    let original = "https://example.test/owner/project.git";
    repository.remote("origin", original).expect("origin should exist");

    context.cli().args(["link", host]).current_dir(&repository_path).assert().failure();

    let repository = Repository::open(&repository_path).expect("repository should open");
    assert_eq!(repository.find_remote("origin").expect("origin").url(), Some(original));
}
