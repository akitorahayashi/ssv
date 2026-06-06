use crate::error::AppError;
use crate::ssh::host_config::HostConfig;
use crate::ssh::keygen;
use crate::ssh::layout::Layout;
use crate::ssh::permissions;
use std::fs;
use std::io::Write;
use std::path::Path;

pub(crate) fn execute(
    host: &str,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    Layout::validate_host(host)?;
    Layout::validate_key_type(key_type)?;
    let layout = Layout::from_env()?;
    layout.prepare_for_generate()?;

    let (private, public) = layout.key_pair(key_type, host);
    let config = layout.host_config(host);
    if layout.artifact_exists(&private)?
        || layout.artifact_exists(&public)?
        || layout.artifact_exists(&config)?
    {
        return Err(AppError::validation(format!(
            "artifacts for host '{host}' already exist; remove them before regenerating"
        )));
    }

    keygen::generate(key_type, &private)?;
    let result = (|| {
        permissions::set_mode(&private, permissions::PRIVATE_MODE)?;
        write_config(&config, &HostConfig::render(host, key_type, user, port))?;
        Ok(fs::read_to_string(&public)?)
    })();
    if result.is_err() {
        remove_generated_artifacts([config.as_path(), public.as_path(), private.as_path()]);
    }
    result
}

fn write_config(path: &Path, contents: &str) -> Result<(), AppError> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    permissions::set_mode(path, permissions::PRIVATE_MODE)
}

fn remove_generated_artifacts<const N: usize>(paths: [&Path; N]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}
