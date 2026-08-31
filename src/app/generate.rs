use crate::context::Context;
use crate::error::{AppError, IoResultExt};
use crate::ssh::bootstrap;
use crate::ssh::host_config;
use crate::ssh::keygen;
use crate::ssh::naming::{HostIdentifier, Hostname, ManagedKeyName, RemoteUser};
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
    let host = HostIdentifier::new(host)?;
    let hostname = Hostname::new(hostname.unwrap_or(host.as_str()))?;
    let user = user.map(RemoteUser::new).transpose()?;
    let key_name = ManagedKeyName::new(key_type, host.clone())?;
    let layout = ctx.layout();
    bootstrap::ensure_bootstrap(layout)?;

    let (private, public) = layout.key_pair(&key_name);
    let config = layout.host_config(&host);
    if layout.artifact_exists(&private)?
        || layout.artifact_exists(&public)?
        || layout.artifact_exists(&config)?
    {
        return Err(AppError::validation(format!(
            "artifacts for host '{host}' already exist; remove them before regenerating"
        )));
    }

    keygen::generate(ctx.keygen(), key_type, &private)?;
    let result = (|| {
        permissions::set_mode(&private, permissions::PRIVATE_MODE)?;
        host_config::write(
            &config,
            &host_config::render(&key_name, &hostname, user.as_ref(), port),
        )?;
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
