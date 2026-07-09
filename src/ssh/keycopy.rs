use crate::error::{AppError, IoResultExt};
use std::path::Path;
use std::process::Command;

pub(crate) fn install(
    copy_id: &Path,
    public_key: &Path,
    user: Option<&str>,
    hostname: &str,
    port: Option<u16>,
) -> Result<(), AppError> {
    let mut command = Command::new(copy_id);
    command.arg("-i").arg(public_key);
    if let Some(port) = port {
        command.arg("-p").arg(port.to_string());
    }
    command.arg(target(user, hostname));
    let status = command.status().path_ctx(copy_id)?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::command_failed(&copy_id.to_string_lossy(), status))
    }
}

pub(crate) fn target(user: Option<&str>, hostname: &str) -> String {
    match user {
        Some(user) => format!("{user}@{hostname}"),
        None => hostname.to_string(),
    }
}
