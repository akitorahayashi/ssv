use crate::error::AppError;
use crate::ssh::host_config::{self, HostConfig};
use crate::ssh::keygen;
use crate::ssh::layout::Layout;
use crate::ssh::permissions;
use std::fs;
use std::path::Path;

pub(crate) fn execute(
    host: &str,
    hostname: Option<&str>,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    Layout::validate_host(host)?;
    if let Some(hn) = hostname {
        Layout::validate_hostname(hn)?;
    }
    if let Some(user) = user {
        Layout::validate_user(user)?;
    }
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

    let target_hostname = hostname.unwrap_or(host);

    keygen::generate(key_type, &private)?;
    let result = (|| {
        permissions::set_mode(&private, permissions::PRIVATE_MODE)?;
        host_config::write(
            &config,
            &HostConfig::render(host, target_hostname, key_type, user, port),
        )?;
        Ok(fs::read_to_string(&public)?)
    })();
    match result {
        Ok(public_key) => Ok(public_key),
        Err(error) => {
            remove_generated_artifacts([config.as_path(), public.as_path(), private.as_path()]);
            Err(AppError::rolled_back(error))
        }
    }
}

fn remove_generated_artifacts<const N: usize>(paths: [&Path; N]) {
    for path in paths {
        let _ = fs::remove_file(path);
    }
}
