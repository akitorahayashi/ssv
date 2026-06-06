use crate::error::AppError;
use std::path::Path;
use std::process::Command;

pub(crate) fn generate(key_type: &str, private: &Path) -> Result<(), AppError> {
    let program = program();
    let status = Command::new(&program)
        .arg("-t")
        .arg(key_type)
        .arg("-f")
        .arg(private)
        .arg("-q")
        .arg("-N")
        .arg("")
        .status()?;
    if status.success() { Ok(()) } else { Err(AppError::command_failed(&program, status)) }
}

pub(crate) fn derive_public(private: &Path) -> Result<String, AppError> {
    let program = program();
    let output =
        Command::new(&program).arg("-y").arg("-P").arg("").arg("-f").arg(private).output()?;
    if !output.status.success() {
        return Err(AppError::command_failed(&program, output.status));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::config(format!("derived public key was not UTF-8: {error}")))
}

fn program() -> String {
    std::env::var("SSV_SSH_KEYGEN_PATH").unwrap_or_else(|_| "ssh-keygen".into())
}
