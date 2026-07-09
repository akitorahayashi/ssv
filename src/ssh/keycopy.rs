use crate::error::{AppError, IoResultExt};
use std::path::Path;
use std::process::Command;

pub(crate) fn install(
    public_key: &Path,
    user: Option<&str>,
    hostname: &str,
    port: Option<u16>,
) -> Result<(), AppError> {
    let program = program();
    let mut command = Command::new(&program);
    command.arg("-i").arg(public_key);
    if let Some(port) = port {
        command.arg("-p").arg(port.to_string());
    }
    command.arg(target(user, hostname));
    let status = command.status().path_ctx(Path::new(&program))?;
    if status.success() { Ok(()) } else { Err(AppError::command_failed(&program, status)) }
}

pub(crate) fn target(user: Option<&str>, hostname: &str) -> String {
    match user {
        Some(user) => format!("{user}@{hostname}"),
        None => hostname.to_string(),
    }
}

fn program() -> String {
    std::env::var("SSV_SSH_COPY_ID_PATH").unwrap_or_else(|_| "ssh-copy-id".into())
}
