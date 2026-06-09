use crate::error::AppError;
use crate::ssh::layout::Layout;
use std::process::Command;

pub(crate) fn execute(host: &str) -> Result<String, AppError> {
    Layout::validate_host(host)?;
    let layout = Layout::from_env()?;
    let config_path = layout.host_config(host);

    if !layout.artifact_exists(&config_path)? {
        return Err(AppError::HostNotFound(host.to_string()));
    }

    let current_url = get_git_remote_url()?;
    let repo_path = extract_repo_path(&current_url)?;
    let new_url = format!("git@{host}:{repo_path}");

    set_git_remote_url(&new_url)?;

    Ok(new_url)
}

fn get_git_remote_url() -> Result<String, AppError> {
    let output = Command::new("git")
        .arg("remote")
        .arg("get-url")
        .arg("origin")
        .output()
        .map_err(|_| AppError::config("Command 'git' not found"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(AppError::validation(format!(
            "git remote get-url origin failed: {}",
            stderr.trim()
        )));
    }

    String::from_utf8(output.stdout)
        .map(|s| s.trim().to_string())
        .map_err(|_| AppError::validation("Error: git remote URL is not valid UTF-8"))
}

fn extract_repo_path(url: &str) -> Result<String, AppError> {
    // SSH: git@github.com:org/repo.git
    if url.starts_with("git@") {
        return url.find(':').map(|colon_pos| url[colon_pos + 1..].to_string()).ok_or_else(|| {
            AppError::validation(format!("Error: unsupported git remote URL format: {url}"))
        });
    }

    // HTTPS: https://github.com/org/repo.git
    if let Some(path_part) = url.strip_prefix("https://") {
        if let Some(slash_pos) = path_part.find('/') {
            return Ok(path_part[slash_pos + 1..].to_string());
        }
    }

    Err(AppError::validation(format!("Error: unsupported git remote URL format: {url}")))
}

fn set_git_remote_url(new_url: &str) -> Result<(), AppError> {
    let status = Command::new("git")
        .arg("remote")
        .arg("set-url")
        .arg("origin")
        .arg(new_url)
        .status()
        .map_err(|_| AppError::config("Command 'git' not found"))?;

    if !status.success() {
        return Err(AppError::command_failed("git", status));
    }

    Ok(())
}
