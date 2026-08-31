use crate::context::Context;
use crate::error::AppError;
use crate::ssh::host_config;
use crate::ssh::naming::HostIdentifier;
use git2::{ErrorCode, Repository};

pub(crate) fn execute(ctx: &Context, host: &str) -> Result<String, AppError> {
    let host = HostIdentifier::new(host)?;
    // `link` never dereferences the identity; it only confirms the host is a managed,
    // well-formed config (regular file, no symlink) before rewriting the git remote.
    host_config::load(ctx.layout(), &host)?;

    let repository = open_repository()?;
    let current_url = origin_url(&repository)?;
    let repo_path = extract_repo_path(&current_url)?;
    let new_url = format!("git@{host}:{repo_path}");

    repository.remote_set_url("origin", &new_url).map_err(|error| {
        AppError::config(format!("failed to update origin remote URL: {error}"))
    })?;

    Ok(new_url)
}

fn open_repository() -> Result<Repository, AppError> {
    Repository::discover(".").map_err(|error| match error.code() {
        ErrorCode::NotFound => {
            AppError::validation("current directory is not inside a Git repository")
        }
        _ => AppError::config(format!("failed to open Git repository: {error}")),
    })
}

fn origin_url(repository: &Repository) -> Result<String, AppError> {
    let remote = repository.find_remote("origin").map_err(|error| match error.code() {
        ErrorCode::NotFound => AppError::validation("origin remote was not found"),
        _ => AppError::config(format!("failed to read origin remote: {error}")),
    })?;
    remote
        .url()
        .map(str::to_owned)
        .ok_or_else(|| AppError::validation("origin remote URL is missing or not valid UTF-8"))
}

fn extract_repo_path(url: &str) -> Result<String, AppError> {
    // SSH: git@github.com:org/repo.git
    if url.starts_with("git@") {
        return url.find(':').map(|colon_pos| url[colon_pos + 1..].to_string()).ok_or_else(|| {
            AppError::validation(format!("unsupported git remote URL format: {url}"))
        });
    }

    // HTTPS: https://github.com/org/repo.git
    if let Some(path_part) = url.strip_prefix("https://")
        && let Some(slash_pos) = path_part.find('/')
    {
        return Ok(path_part[slash_pos + 1..].to_string());
    }

    Err(AppError::validation(format!("unsupported git remote URL format: {url}")))
}
