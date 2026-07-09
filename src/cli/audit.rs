use crate::cli::{Exit, Result};
use crate::context::Context;

pub(crate) fn run(ctx: &Context) -> Result {
    let report = ctx.audit()?;
    for finding in &report.findings {
        eprintln!(
            "{} [{}] {}: {}",
            finding.severity,
            finding.code,
            finding.path.display(),
            finding.message
        );
    }
    if report.has_errors() {
        Ok(Exit::Failure)
    } else if report.has_warnings() {
        println!("SSH assets have warnings");
        Ok(Exit::Success)
    } else {
        println!("SSH assets are healthy");
        Ok(Exit::Success)
    }
}
