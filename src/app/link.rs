use crate::error::{AppError, GitOperation};
use crate::ssh::host_config;
use crate::ssh::layout::Layout;
use crate::ssh::naming::HostIdentifier;
use git2::{ErrorCode, Repository};
use std::path::Path;

pub(crate) fn execute(
    layout: &Layout,
    repository_start: &Path,
    host: &str,
) -> Result<String, AppError> {
    let host = HostIdentifier::new(host)?;
    host_config::load(layout, &host)?;

    let repository = open_repository(repository_start)?;
    let current_url = origin_url(&repository)?;
    let repo_path = extract_repo_path(&current_url)?;
    let new_url = format!("git@{host}:{repo_path}");

    repository.remote_set_url("origin", &new_url).map_err(|error| {
        AppError::git(
            GitOperation::UpdateOrigin,
            format!("failed to update origin remote URL: {error}"),
        )
    })?;

    Ok(new_url)
}

fn open_repository(start: &Path) -> Result<Repository, AppError> {
    Repository::discover(start).map_err(|error| match error.code() {
        ErrorCode::NotFound => AppError::git(
            GitOperation::DiscoverRepository,
            format!("'{}' is not inside a Git repository", start.display()),
        ),
        _ => AppError::git(
            GitOperation::DiscoverRepository,
            format!("failed to open Git repository from '{}': {error}", start.display()),
        ),
    })
}

fn origin_url(repository: &Repository) -> Result<String, AppError> {
    let remote = repository.find_remote("origin").map_err(|error| match error.code() {
        ErrorCode::NotFound => {
            AppError::git(GitOperation::ReadOrigin, "origin remote was not found")
        }
        _ => AppError::git(
            GitOperation::ReadOrigin,
            format!("failed to read origin remote: {error}"),
        ),
    })?;
    remote.url().map(str::to_owned).ok_or_else(|| {
        AppError::git(GitOperation::ParseOrigin, "origin remote URL is missing or not valid UTF-8")
    })
}

fn extract_repo_path(url: &str) -> Result<String, AppError> {
    if url.starts_with("git@") {
        return url
            .find(':')
            .map(|colon| url[colon + 1..].to_string())
            .ok_or_else(|| unsupported_remote(url));
    }
    if let Some(path) = url.strip_prefix("https://")
        && let Some(slash) = path.find('/')
    {
        return Ok(path[slash + 1..].to_string());
    }
    Err(unsupported_remote(url))
}

fn unsupported_remote(url: &str) -> AppError {
    AppError::git(GitOperation::ParseOrigin, format!("unsupported git remote URL format: {url}"))
}
