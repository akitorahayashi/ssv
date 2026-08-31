use crate::error::{AppError, IoResultExt};
use crate::ssh::host_config::ManagedHost;
use crate::ssh::layout::Layout;
use crate::ssh::naming::{HostIdentifier, KeyFileKind, ManagedKeyName};
use std::ffi::OsStr;
use std::fs;
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum IssueKind {
    Read,
    FileType,
    Contract,
}

#[derive(Debug)]
pub(crate) enum HostCandidate {
    Managed(ManagedHost),
    Invalid { path: PathBuf, kind: IssueKind, error: AppError },
}

#[derive(Debug)]
pub(crate) enum KeyCandidate {
    Managed { path: PathBuf, name: ManagedKeyName, kind: KeyFileKind },
    Invalid { path: PathBuf, kind: IssueKind, error: AppError },
}

pub(crate) fn hosts(layout: &Layout) -> Result<Vec<HostCandidate>, AppError> {
    require_inventory_root(layout, &layout.root())?;
    require_inventory_root(layout, &layout.hosts())?;
    let mut paths = read_paths(&layout.hosts())?;
    paths.retain(|path| path.extension() == Some(OsStr::new("conf")));
    Ok(paths.into_iter().map(|path| classify_host(layout, path)).collect())
}

pub(crate) fn keys(layout: &Layout) -> Result<Vec<KeyCandidate>, AppError> {
    require_inventory_root(layout, &layout.root())?;
    let paths = read_paths(&layout.root())?;
    Ok(paths.into_iter().filter_map(|path| classify_key(layout, path)).collect())
}

fn require_inventory_root(layout: &Layout, path: &Path) -> Result<(), AppError> {
    match layout.require_directory(path) {
        Err(AppError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            Err(AppError::bootstrap_missing(path.to_path_buf()))
        }
        result => result,
    }
}

fn read_paths(directory: &Path) -> Result<Vec<PathBuf>, AppError> {
    let entries = fs::read_dir(directory).path_ctx(directory)?;
    let mut paths = entries
        .map(|entry| entry.path_ctx(directory).map(|entry| entry.path()))
        .collect::<Result<Vec<_>, _>>()?;
    paths.sort_by(|left, right| left.as_os_str().as_bytes().cmp(right.as_os_str().as_bytes()));
    Ok(paths)
}

fn classify_host(layout: &Layout, path: PathBuf) -> HostCandidate {
    if let Err(error) = layout.require_regular_file(&path) {
        let kind = if matches!(error, AppError::Io { .. }) {
            IssueKind::Read
        } else {
            IssueKind::FileType
        };
        return HostCandidate::Invalid { path, kind, error };
    }
    let Some(host) = path.file_stem().and_then(|stem| stem.to_str()) else {
        return invalid_host(
            path,
            IssueKind::Contract,
            AppError::validation("host config has no UTF-8 filename"),
        );
    };
    let host = match HostIdentifier::new(host) {
        Ok(host) => host,
        Err(error) => return invalid_host(path, IssueKind::Contract, error),
    };
    let contents = match fs::read_to_string(&path) {
        Ok(contents) => contents,
        Err(error) => {
            return invalid_host(path.clone(), IssueKind::Read, AppError::io(&path, error));
        }
    };
    match ManagedHost::parse(&contents, layout, host, path.clone()) {
        Ok(host) => HostCandidate::Managed(host),
        Err(error) => invalid_host(path, IssueKind::Contract, error),
    }
}

fn invalid_host(path: PathBuf, kind: IssueKind, error: AppError) -> HostCandidate {
    HostCandidate::Invalid { path, kind, error }
}

fn classify_key(layout: &Layout, path: PathBuf) -> Option<KeyCandidate> {
    let filename = path.file_name()?.to_str()?;
    let (name, kind) = ManagedKeyName::parse_file(filename)?;
    if let Err(error) = layout.require_regular_file(&path) {
        let kind = if matches!(error, AppError::Io { .. }) {
            IssueKind::Read
        } else {
            IssueKind::FileType
        };
        return Some(KeyCandidate::Invalid { path, kind, error });
    }
    Some(KeyCandidate::Managed { path, name, kind })
}
