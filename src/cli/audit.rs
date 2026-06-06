use crate::cli::{Exit, Result};

pub(crate) fn run() -> Result {
    let report = crate::audit()?;
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
    } else {
        println!("SSH assets are healthy");
        Ok(Exit::Success)
    }
}
