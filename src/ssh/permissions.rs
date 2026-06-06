use crate::error::AppError;
use std::fs;
use std::path::Path;

pub(crate) const DIRECTORY_MODE: u32 = 0o700;
pub(crate) const PRIVATE_MODE: u32 = 0o600;

pub(crate) fn set_mode(path: &Path, mode: u32) -> Result<(), AppError> {
    let permissions = fs::metadata(path)?.permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = permissions;
        permissions.set_mode(mode);
        fs::set_permissions(path, permissions)?;
    }
    #[cfg(not(unix))]
    {
        let _ = permissions;
        let _ = mode;
    }
    Ok(())
}

#[cfg(unix)]
pub(crate) fn mode(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;
    metadata.permissions().mode() & 0o777
}

#[cfg(unix)]
pub(crate) fn owner(metadata: &fs::Metadata) -> u32 {
    use std::os::unix::fs::MetadataExt;
    metadata.uid()
}
