use crate::harness::TestContext;
use ssv::{AuditCode, AuditSeverity, Context};
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::symlink;
use std::path::PathBuf;

#[test]
fn symlinked_hosts_directory_is_not_traversed() {
    let context = TestContext::new();
    context.prepare_include();
    fs::remove_dir(context.hosts_dir()).expect("empty hosts directory should be removed");
    let outside = context.home().join("outside-hosts");
    fs::create_dir(&outside).expect("outside directory should exist");
    fs::write(outside.join("outside.test.conf"), "outside").expect("outside config");
    symlink(&outside, context.hosts_dir()).expect("hosts symlink");
    let ctx = context.ctx();

    assert!(ctx.list().is_err());
    let report = ctx.audit().expect("audit should report the rejected root");
    assert!(report.findings.iter().any(|finding| {
        finding.code == AuditCode::InvalidFileType && finding.path == context.hosts_dir()
    }));
    assert!(!report.findings.iter().any(|finding| finding.path.starts_with(&outside)));
}

#[test]
fn conf_directories_and_symlinks_are_invalid_candidates() {
    let context = TestContext::new();
    context.prepare_include();
    let directory = context.hosts_dir().join("directory.test.conf");
    fs::create_dir(&directory).expect("candidate directory");
    let outside = context.home().join("outside.conf");
    fs::write(&outside, "outside").expect("outside file");
    let link = context.hosts_dir().join("link.test.conf");
    symlink(&outside, &link).expect("candidate symlink");
    let ctx = context.ctx();

    assert!(ctx.list().is_err());
    let report = ctx.audit().expect("audit should collect candidates");
    assert!(report.findings.iter().any(|finding| {
        finding.code == AuditCode::InvalidFileType && finding.path == directory
    }));
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.code == AuditCode::InvalidFileType && finding.path == link })
    );
}

#[test]
fn invalid_config_is_an_error_for_list_and_a_finding_for_audit() {
    let context = TestContext::new();
    context.write_managed_host("invalid.test");
    fs::write(
        context.host_config("invalid.test"),
        "Host invalid.test\nHostName invalid.test\nIdentityFile ~/.ssh/id_ed25519_invalid.test\n",
    )
    .expect("invalid config");
    let ctx = context.ctx();

    assert!(ctx.list().is_err());
    let report = ctx.audit().expect("audit should collect invalid config");
    assert!(report.findings.iter().any(|finding| {
        finding.code == AuditCode::ConfigParse
            && finding.path == context.host_config("invalid.test")
    }));
}

#[test]
fn private_and_public_one_sided_assets_are_reported() {
    let context = TestContext::new();
    context.prepare_include();
    let private = context.private_key("ed25519", "private.test");
    let public = context.public_key("ed25519", "public.test");
    fs::write(&private, "private").expect("private candidate");
    fs::write(&public, "public").expect("public candidate");

    let report = context.ctx().audit().expect("audit should collect orphans");
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.code == AuditCode::OrphanedAsset && finding.path == private })
    );
    assert!(
        report
            .findings
            .iter()
            .any(|finding| { finding.code == AuditCode::OrphanedAsset && finding.path == public })
    );
}

#[test]
fn audit_findings_have_a_repeatable_total_order() {
    let context = TestContext::new();
    context.write_managed_host("z.test");
    context.write_managed_host("a.test");
    fs::remove_file(context.private_key("ed25519", "z.test")).expect("private removed");
    fs::remove_file(context.public_key("ed25519", "a.test")).expect("public removed");
    context.set_mode(&context.host_config("a.test"), 0o400);

    let first = context.ctx().audit().expect("first audit");
    let second = context.ctx().audit().expect("second audit");
    assert_eq!(first, second);

    let actual = first
        .findings
        .iter()
        .map(|finding| {
            (
                finding.path.as_os_str().as_bytes().to_vec(),
                severity_rank(finding.severity),
                finding.code.to_string(),
                finding.message.as_bytes().to_vec(),
            )
        })
        .collect::<Vec<_>>();
    let mut expected = actual.clone();
    expected.sort();
    assert_eq!(actual, expected);
}

#[test]
fn owner_lookup_failure_is_a_finding() {
    let context = TestContext::new();
    let missing_home = context.home().join("missing-home");
    let ctx = Context::new(
        missing_home.clone(),
        PathBuf::from("ssh-keygen"),
        PathBuf::from("ssh-copy-id"),
    );

    let report = ctx.audit().expect("audit should return findings");
    assert!(
        report.findings.iter().any(|finding| {
            finding.code == AuditCode::ReadFailure && finding.path == missing_home
        })
    );
}

fn severity_rank(severity: AuditSeverity) -> u8 {
    match severity {
        AuditSeverity::Error => 0,
        AuditSeverity::Warning => 1,
    }
}
