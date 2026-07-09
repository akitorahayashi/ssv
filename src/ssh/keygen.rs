use crate::error::{AppError, IoResultExt};
use std::path::Path;
use std::process::Command;

pub(crate) fn generate(keygen: &Path, key_type: &str, private: &Path) -> Result<(), AppError> {
    let status = Command::new(keygen)
        .arg("-t")
        .arg(key_type)
        .arg("-f")
        .arg(private)
        .arg("-q")
        .arg("-N")
        .arg("")
        .status()
        .path_ctx(keygen)?;
    if status.success() {
        Ok(())
    } else {
        Err(AppError::command_failed(&keygen.to_string_lossy(), status))
    }
}

pub(crate) fn derive_public(keygen: &Path, private: &Path) -> Result<String, AppError> {
    let output = Command::new(keygen)
        .arg("-y")
        .arg("-P")
        .arg("")
        .arg("-f")
        .arg(private)
        .output()
        .path_ctx(keygen)?;
    if !output.status.success() {
        return Err(AppError::command_failed(&keygen.to_string_lossy(), output.status));
    }
    String::from_utf8(output.stdout)
        .map_err(|error| AppError::config(format!("derived public key was not UTF-8: {error}")))
}
