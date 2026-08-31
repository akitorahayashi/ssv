use crate::error::{AppError, IoResultExt};
use crate::ssh::permissions;
use std::fs::{self, File, Permissions};
use std::io::{self, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tempfile::{Builder, NamedTempFile, PersistError};

enum Publication {
    Create,
    Replace,
}

pub(crate) fn create(path: &Path, contents: &str) -> Result<(), AppError> {
    persist(path, Publication::Create, |file| file.write_all(contents.as_bytes()))
}

pub(crate) fn replace(path: &Path, contents: &str) -> Result<(), AppError> {
    let metadata = fs::symlink_metadata(path).path_ctx(path)?;
    if !metadata.file_type().is_file() {
        return Err(AppError::validation(format!(
            "refusing to replace non-regular file '{}'",
            path.display()
        )));
    }
    persist(path, Publication::Replace, |file| file.write_all(contents.as_bytes()))
}

pub(crate) fn reserve_path(parent: &Path, prefix: &str) -> Result<PathBuf, AppError> {
    let temporary = Builder::new().prefix(prefix).tempfile_in(parent).path_ctx(parent)?;
    let path = temporary.path().to_path_buf();
    temporary.close().path_ctx(&path)?;
    Ok(path)
}

pub(crate) fn publish_noclobber(staged: &Path, final_path: &Path) -> Result<(), AppError> {
    if staged.parent() != final_path.parent() {
        return Err(AppError::validation("staged and final files must share a parent directory"));
    }
    let metadata = fs::symlink_metadata(staged).path_ctx(staged)?;
    if !metadata.file_type().is_file() {
        return Err(AppError::validation(format!(
            "staged path '{}' is not a regular file",
            staged.display()
        )));
    }
    File::open(staged).path_ctx(staged)?.sync_all().path_ctx(staged)?;
    fs::hard_link(staged, final_path).path_ctx(final_path)?;
    if let Err(source) = fs::remove_file(staged) {
        return Err(AppError::committed_io(
            final_path,
            &format!("removing staging link '{}'", staged.display()),
            source,
        ));
    }
    sync_parent(final_path).map_err(|source| {
        AppError::committed_io(final_path, "syncing the parent directory", source)
    })
}

fn persist(
    path: &Path,
    publication: Publication,
    writer: impl FnOnce(&mut File) -> io::Result<()>,
) -> Result<(), AppError> {
    let parent = path
        .parent()
        .ok_or_else(|| AppError::validation("managed file has no parent directory"))?;
    let mut temporary = Builder::new().prefix(".ssv-").tempfile_in(parent).path_ctx(parent)?;
    let temporary_path = temporary.path().to_path_buf();
    if let Err(source) =
        temporary.as_file().set_permissions(Permissions::from_mode(permissions::PRIVATE_MODE))
    {
        return Err(cleanup_temporary(AppError::io(&temporary_path, source), temporary));
    }
    if let Err(source) = writer(temporary.as_file_mut()) {
        return Err(cleanup_temporary(AppError::io(&temporary_path, source), temporary));
    }
    if let Err(source) = temporary.as_file().sync_all() {
        return Err(cleanup_temporary(AppError::io(&temporary_path, source), temporary));
    }

    let persisted = match publication {
        Publication::Create => temporary.persist_noclobber(path),
        Publication::Replace => temporary.persist(path),
    };
    match persisted {
        Ok(file) => {
            drop(file);
            sync_parent(path).map_err(|source| {
                AppError::committed_io(path, "syncing the parent directory", source)
            })
        }
        Err(PersistError { error, file }) => {
            Err(cleanup_temporary(AppError::io(path, error), file))
        }
    }
}

fn cleanup_temporary(primary: AppError, temporary: NamedTempFile) -> AppError {
    let path = temporary.path().to_path_buf();
    match temporary.close() {
        Ok(()) => primary,
        Err(source) => AppError::with_cleanup(primary, vec![AppError::io(&path, source)]),
    }
}

fn sync_parent(path: &Path) -> io::Result<()> {
    let parent = path.parent().ok_or_else(|| io::Error::other("file has no parent directory"))?;
    File::open(parent)?.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_replace_publish_complete_private_files() {
        let directory = tempfile::tempdir().expect("directory");
        let path = directory.path().join("config");

        create(&path, "old\n").expect("create");
        assert_eq!(fs::read_to_string(&path).expect("read"), "old\n");
        assert_eq!(fs::metadata(&path).expect("metadata").permissions().mode() & 0o777, 0o600);

        replace(&path, "new\n").expect("replace");
        assert_eq!(fs::read_to_string(&path).expect("read"), "new\n");
        assert_eq!(fs::metadata(&path).expect("metadata").permissions().mode() & 0o777, 0o600);
    }

    #[test]
    fn create_is_no_clobber() {
        let directory = tempfile::tempdir().expect("directory");
        let path = directory.path().join("config");
        fs::write(&path, "original\n").expect("original");

        assert!(create(&path, "replacement\n").is_err());
        assert_eq!(fs::read_to_string(&path).expect("read"), "original\n");
    }

    #[test]
    fn failed_write_leaves_original_unchanged() {
        let directory = tempfile::tempdir().expect("directory");
        let path = directory.path().join("config");
        create(&path, "original\n").expect("create");

        let error = persist(&path, Publication::Replace, |file| {
            file.write_all(b"partial")?;
            Err(io::Error::other("injected write failure"))
        });

        assert!(error.is_err());
        assert_eq!(fs::read_to_string(&path).expect("read"), "original\n");
        let entries = fs::read_dir(directory.path()).expect("entries").count();
        assert_eq!(entries, 1);
    }

    #[test]
    fn replace_rejects_non_regular_targets() {
        let directory = tempfile::tempdir().expect("directory");
        let target = directory.path().join("config");
        fs::create_dir(&target).expect("target directory");
        assert!(replace(&target, "content").is_err());
        assert!(target.is_dir());
    }

    #[test]
    fn replace_rejects_symlink_targets() {
        use std::os::unix::fs::symlink;

        let directory = tempfile::tempdir().expect("directory");
        let outside = directory.path().join("outside");
        let target = directory.path().join("config");
        fs::write(&outside, "outside").expect("outside");
        symlink(&outside, &target).expect("symlink");

        assert!(replace(&target, "replacement").is_err());
        assert_eq!(fs::read_to_string(&outside).expect("read"), "outside");
    }
}
