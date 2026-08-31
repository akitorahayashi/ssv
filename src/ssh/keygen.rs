use crate::error::{AppError, IoResultExt};
use std::path::Path;
use std::process::Command;

pub(crate) fn generate(keygen: &Path, key_type: &str, private: &Path) -> Result<(), AppError> {
    let output = Command::new(keygen)
        .arg("-t")
        .arg(key_type)
        .arg("-f")
        .arg(private)
        .arg("-q")
        .arg("-N")
        .arg("")
        .output()
        .path_ctx(keygen)?;
    if output.status.success() {
        Ok(())
    } else {
        Err(AppError::external_command(
            "generating an SSH key pair",
            keygen,
            output.status,
            Some(&output.stderr),
        ))
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
        return Err(AppError::external_command(
            "deriving an SSH public key",
            keygen,
            output.status,
            Some(&output.stderr),
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| {
        AppError::invalid_external_output(
            "deriving an SSH public key",
            format!("stdout was not UTF-8: {error}"),
        )
    })
}
