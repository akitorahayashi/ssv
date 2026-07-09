use crate::context::Context;
use crate::error::{AppError, IoResultExt};
use crate::ssh::bootstrap;
use crate::ssh::host_config::{self, HostConfig};
use crate::ssh::keygen;
use crate::ssh::naming::{self, ManagedKeyName};
use crate::ssh::permissions;
use std::fs;
use std::path::Path;

pub(crate) fn execute(
    ctx: &Context,
    host: &str,
    hostname: Option<&str>,
    key_type: &str,
    user: Option<&str>,
    port: Option<u16>,
) -> Result<String, AppError> {
    naming::validate_host(host)?;
    if let Some(hn) = hostname {
        naming::validate_hostname(hn)?;
    }
    if let Some(user) = user {
        naming::validate_user(user)?;
    }
    naming::validate_key_type(key_type)?;
    let key_name = ManagedKeyName::new(key_type, host)?;
    let layout = ctx.layout();
    bootstrap::ensure_bootstrap(layout)?;

    let (private, public) = layout.key_pair(&key_name);
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

    keygen::generate(ctx.keygen(), key_type, &private)?;
    let result = (|| {
        permissions::set_mode(&private, permissions::PRIVATE_MODE)?;
        host_config::write(&config, &HostConfig::render(&key_name, target_hostname, user, port))?;
        fs::read_to_string(&public).path_ctx(&public)
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
