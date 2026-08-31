use crate::error::{AppError, IoResultExt};
use crate::ssh::atomic_file;
use crate::ssh::bootstrap;
use crate::ssh::host_config;
use crate::ssh::keygen;
use crate::ssh::layout::Layout;
use crate::ssh::naming::{HostIdentifier, Hostname, ManagedKeyName, RemoteUser};
use crate::ssh::permissions;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn execute(
    layout: &Layout,
    keygen_path: &Path,
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

    let staged_private = atomic_file::reserve_path(&layout.root(), ".ssv-key-")?;
    let staged_public = layout.public_key(&staged_private)?;
    if let Err(error) = keygen::generate(keygen_path, key_type, &staged_private) {
        return Err(rollback(error, [staged_public.as_path(), staged_private.as_path()]));
    }

    let prepared = prepare_key_pair(layout, keygen_path, &staged_private, &staged_public);
    let public_key = match prepared {
        Ok(public_key) => public_key,
        Err(error) => {
            return Err(rollback(error, [staged_public.as_path(), staged_private.as_path()]));
        }
    };

    if let Err(error) = atomic_file::publish_noclobber(&staged_private, &private) {
        let mut paths = vec![staged_public.clone(), staged_private.clone()];
        if error.is_committed() {
            paths.push(private.clone());
        }
        return Err(rollback(error, paths.iter().map(PathBuf::as_path)));
    }
    if let Err(error) = atomic_file::publish_noclobber(&staged_public, &public) {
        let mut paths = vec![private.clone(), staged_public.clone()];
        if error.is_committed() {
            paths.push(public.clone());
        }
        return Err(rollback(error, paths.iter().map(PathBuf::as_path)));
    }

    let rendered = host_config::render(&key_name, &hostname, user.as_ref(), port);
    match host_config::create(&config, &rendered) {
        Ok(()) => Ok(public_key),
        Err(error) if error.is_committed() => Err(error),
        Err(error) => Err(rollback(error, [public.as_path(), private.as_path()])),
    }
}

fn prepare_key_pair(
    layout: &Layout,
    keygen_path: &Path,
    staged_private: &Path,
    staged_public: &Path,
) -> Result<String, AppError> {
    layout.require_regular_file(staged_private)?;
    layout.require_regular_file(staged_public)?;
    permissions::set_mode(staged_private, permissions::PRIVATE_MODE)?;
    let public_key = fs::read_to_string(staged_public).path_ctx(staged_public)?;
    let derived = keygen::derive_public(keygen_path, staged_private)?;
    let actual = key_fields(&public_key).ok_or_else(|| {
        AppError::invalid_external_output(
            "reading the generated SSH public key",
            "output does not contain algorithm and key fields",
        )
    })?;
    let expected = key_fields(&derived).ok_or_else(|| {
        AppError::invalid_external_output(
            "deriving an SSH public key",
            "output does not contain algorithm and key fields",
        )
    })?;
    if actual != expected {
        return Err(AppError::validation(
            "generated public key does not match the generated private key",
        ));
    }
    Ok(public_key)
}

fn key_fields(contents: &str) -> Option<(&str, &str)> {
    let mut fields = contents.split_whitespace();
    Some((fields.next()?, fields.next()?))
}

fn rollback<'a>(primary: AppError, paths: impl IntoIterator<Item = &'a Path>) -> AppError {
    let mut cleanup = Vec::new();
    for path in paths {
        match fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => cleanup.push(AppError::io(path, error)),
        }
    }
    AppError::rollback("SSH asset generation", primary, cleanup)
}
