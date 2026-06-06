use crate::error::AppError;
use crate::ssh::layout::Layout;
use std::path::PathBuf;

#[derive(Debug)]
pub(crate) struct HostConfig {
    pub(crate) identity: PathBuf,
}

impl HostConfig {
    pub(crate) fn parse(contents: &str, layout: &Layout) -> Result<Self, AppError> {
        let identities = contents
            .lines()
            .filter_map(directive)
            .filter(|(name, _)| name.eq_ignore_ascii_case("IdentityFile"))
            .map(|(_, value)| layout.resolve_identity(unquote(value)))
            .collect::<Result<Vec<_>, _>>()?;

        match identities.as_slice() {
            [identity] => Ok(Self { identity: identity.clone() }),
            [] => Err(AppError::validation("managed host config has no IdentityFile")),
            _ => Err(AppError::validation("managed host config has multiple IdentityFile entries")),
        }
    }

    pub(crate) fn render(
        host: &str,
        key_type: &str,
        user: Option<&str>,
        port: Option<u16>,
    ) -> String {
        let mut contents = format!("Host {host}\nHostName {host}\n");
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

pub(crate) fn has_managed_include(contents: &str) -> bool {
    let mut top_level = true;
    for line in contents.lines() {
        if let Some((name, value)) = directive(line) {
            if name.eq_ignore_ascii_case("Host") || name.eq_ignore_ascii_case("Match") {
                top_level = false;
            } else if top_level
                && name.eq_ignore_ascii_case("Include")
                && value.split_whitespace().any(|path| path == "~/.ssh/conf.d/*.conf")
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
        assert!(!has_managed_include("Host example\nInclude ~/.ssh/conf.d/*.conf\n"));
    }
}
