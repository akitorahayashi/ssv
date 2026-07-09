use crate::error::{AppError, IoResultExt};
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::path::Path;

pub(crate) const DIRECTORY_MODE: u32 = 0o700;
pub(crate) const PRIVATE_MODE: u32 = 0o600;

pub(crate) fn set_mode(path: &Path, mode: u32) -> Result<(), AppError> {
    let mut permissions = fs::metadata(path).path_ctx(path)?.permissions();
    permissions.set_mode(mode);
    fs::set_permissions(path, permissions).path_ctx(path)
}

pub(crate) fn mode(metadata: &fs::Metadata) -> u32 {
    metadata.permissions().mode() & 0o777
}

pub(crate) fn owner(metadata: &fs::Metadata) -> u32 {
    metadata.uid()
}
