use crate::harness::TestContext;
use ssv::{AuditCode, AuditSeverity};

#[test]
#[cfg(unix)]
fn unsafe_and_non_standard_permissions_are_distinguished() {
    let context = TestContext::new();
    let ctx = context.ctx();
    ctx.generate("permissions.test", None, "ed25519", None, None).expect("generate should succeed");
    context.prepare_include();
    context.set_mode(&context.private_key("ed25519", "permissions.test"), 0o644);
    context.set_mode(&context.host_config("permissions.test"), 0o400);

    let report = ctx.audit().expect("audit should succeed");
    assert!(report.findings.iter().any(|finding| {
        finding.code == AuditCode::UnsafePermissions && finding.severity == AuditSeverity::Error
    }));
    assert!(report.findings.iter().any(|finding| {
        finding.code == AuditCode::NonStandardPermissions
            && finding.severity == AuditSeverity::Warning
    }));
}
