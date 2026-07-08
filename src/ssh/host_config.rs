use crate::error::AppError;
use crate::ssh::layout::Layout;
use crate::ssh::permissions;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct HostConfig {
    pub(crate) hostname: Option<String>,
    pub(crate) user: Option<String>,
    pub(crate) port: Option<u16>,
    pub(crate) identity: PathBuf,
}

impl HostConfig {
    pub(crate) fn parse(contents: &str, layout: &Layout) -> Result<Self, AppError> {
        let mut hostnames = Vec::new();
        let mut users = Vec::new();
        let mut ports = Vec::new();
        let mut identities = Vec::new();
        for (name, value) in contents.lines().filter_map(directive) {
            if name.eq_ignore_ascii_case("HostName") {
                hostnames.push(unquote(value).to_string());
            } else if name.eq_ignore_ascii_case("User") {
                users.push(unquote(value).to_string());
            } else if name.eq_ignore_ascii_case("Port") {
                ports.push(unquote(value).to_string());
            } else if name.eq_ignore_ascii_case("IdentityFile") {
                identities.push(layout.resolve_identity(unquote(value))?);
            }
        }

        let identity = match identities.as_slice() {
            [identity] => identity.clone(),
            [] => return Err(AppError::validation("managed host config has no IdentityFile")),
            _ => {
                return Err(AppError::validation(
                    "managed host config has multiple IdentityFile entries",
                ));
            }
        };
        let hostname = single(hostnames, "HostName")?;
        let user = single(users, "User")?;
        let port = match single(ports, "Port")? {
            Some(value) => Some(value.parse::<u16>().map_err(|_| {
                AppError::validation(format!(
                    "managed host config has an invalid Port value '{value}'"
                ))
            })?),
            None => None,
        };

        Ok(Self { hostname, user, port, identity })
    }

    pub(crate) fn render(
        host: &str,
        hostname: &str,
        key_type: &str,
        user: Option<&str>,
        port: Option<u16>,
    ) -> String {
        let mut contents = format!("Host {host}\nHostName {hostname}\n");
        if let Some(user) = user {
            contents.push_str(&format!("User {user}\n"));
        }
        if let Some(port) = port {
            contents.push_str(&format!("Port {port}\n"));
        }
        contents.push_str(&format!("IdentityFile ~/.ssh/id_{key_type}_{host}\n"));
        contents.push_str("IdentitiesOnly yes\n");
        contents
    }
}

pub(crate) fn write(path: &Path, contents: &str) -> Result<(), AppError> {
    let mut file = fs::File::create(path)?;
    file.write_all(contents.as_bytes())?;
    file.sync_all()?;
    permissions::set_mode(path, permissions::PRIVATE_MODE)
}

fn single(values: Vec<String>, name: &str) -> Result<Option<String>, AppError> {
    match values.len() {
        0 => Ok(None),
        1 => Ok(values.into_iter().next()),
        _ => Err(AppError::validation(format!("managed host config has multiple {name} entries"))),
    }
}

pub(crate) fn has_managed_include(contents: &str) -> bool {
    let mut top_level = true;
    for line in contents.lines() {
        if let Some((name, value)) = directive(line) {
            if name.eq_ignore_ascii_case("Host") || name.eq_ignore_ascii_case("Match") {
                top_level = false;
            } else if top_level
                && name.eq_ignore_ascii_case("Include")
                && value.split_whitespace().any(|path| unquote(path) == "~/.ssh/conf.d/*.conf")
            {
                return true;
            }
        }
    }
    false
}

fn directive(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    line.split_once(char::is_whitespace).map(|(name, value)| (name, value.trim()))
}

fn unquote(value: &str) -> &str {
    value.strip_prefix('"').and_then(|value| value.strip_suffix('"')).unwrap_or(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn include_must_be_top_level() {
        assert!(has_managed_include("Include ~/.ssh/conf.d/*.conf\nHost example\n"));
        assert!(has_managed_include("Include \"~/.ssh/conf.d/*.conf\"\nHost example\n"));
        assert!(!has_managed_include("Host example\nInclude ~/.ssh/conf.d/*.conf\n"));
    }
}
