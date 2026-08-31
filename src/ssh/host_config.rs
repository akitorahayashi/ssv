use crate::error::{AppError, IoResultExt};
use crate::ssh::layout::Layout;
use crate::ssh::naming::{HostIdentifier, Hostname, ManagedKeyName, RemoteUser};
use crate::ssh::permissions;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(crate) struct ManagedHost {
    pub(crate) host: HostIdentifier,
    pub(crate) path: PathBuf,
    pub(crate) hostname: Hostname,
    pub(crate) user: Option<RemoteUser>,
    pub(crate) port: Option<u16>,
    pub(crate) private_key: PathBuf,
    pub(crate) public_key: PathBuf,
}

impl ManagedHost {
    pub(crate) fn parse(
        contents: &str,
        layout: &Layout,
        host: HostIdentifier,
        path: PathBuf,
    ) -> Result<Self, AppError> {
        let mut found_host = false;
        let mut hostnames = Vec::new();
        let mut users = Vec::new();
        let mut ports = Vec::new();
        let mut identities = Vec::new();
        let mut identities_only = Vec::new();

        for (line_index, line) in contents.lines().enumerate() {
            let Some((name, value)) = directive(line).map_err(|message| {
                AppError::validation(format!(
                    "managed host config line {}: {message}",
                    line_index + 1
                ))
            })?
            else {
                continue;
            };

            if name.eq_ignore_ascii_case("Match") {
                return Err(AppError::validation("managed host config contains a Match block"));
            }
            if name.eq_ignore_ascii_case("Host") {
                if found_host {
                    return Err(AppError::validation(
                        "managed host config contains multiple Host blocks",
                    ));
                }
                let alias = scalar(value, "Host")?;
                if alias != host.as_str() {
                    return Err(AppError::validation(format!(
                        "managed host config Host '{alias}' does not match filename host '{host}'"
                    )));
                }
                found_host = true;
                continue;
            }
            if !found_host {
                return Err(AppError::validation(format!(
                    "managed host config directive '{name}' appears before Host"
                )));
            }

            if name.eq_ignore_ascii_case("HostName") {
                hostnames.push(scalar(value, "HostName")?.to_string());
            } else if name.eq_ignore_ascii_case("User") {
                users.push(scalar(value, "User")?.to_string());
            } else if name.eq_ignore_ascii_case("Port") {
                ports.push(scalar(value, "Port")?.to_string());
            } else if name.eq_ignore_ascii_case("IdentityFile") {
                identities.push(layout.resolve_identity(scalar(value, "IdentityFile")?)?);
            } else if name.eq_ignore_ascii_case("IdentitiesOnly") {
                identities_only.push(scalar(value, "IdentitiesOnly")?.to_string());
            }
        }

        if !found_host {
            return Err(AppError::validation("managed host config has no Host block"));
        }
        let hostname = Hostname::new(&required_single(hostnames, "HostName")?)?;
        let user =
            optional_single(users, "User")?.map(|value| RemoteUser::new(&value)).transpose()?;
        let port = optional_single(ports, "Port")?
            .map(|value| {
                value.parse::<u16>().map_err(|_| {
                    AppError::validation(format!(
                        "managed host config has an invalid Port value '{value}'"
                    ))
                })
            })
            .transpose()?;
        let private_key = required_single(identities, "IdentityFile")?;
        layout.require_host_identity(&private_key, &host)?;
        let public_key = layout.public_key(&private_key)?;
        let identities_only = required_single(identities_only, "IdentitiesOnly")?;
        if !identities_only.eq_ignore_ascii_case("yes") {
            return Err(AppError::validation("managed host config requires IdentitiesOnly yes"));
        }

        Ok(Self { host, path, hostname, user, port, private_key, public_key })
    }
}

pub(crate) fn render(
    key_name: &ManagedKeyName,
    hostname: &Hostname,
    user: Option<&RemoteUser>,
    port: Option<u16>,
) -> String {
    let mut contents = format!("Host {}\nHostName {hostname}\n", key_name.host());
    if let Some(user) = user {
        contents.push_str(&format!("User {user}\n"));
    }
    if let Some(port) = port {
        contents.push_str(&format!("Port {port}\n"));
    }
    contents.push_str(&format!("IdentityFile {}\n", key_name.identity_directive()));
    contents.push_str("IdentitiesOnly yes\n");
    contents
}

pub(crate) fn load(layout: &Layout, host: &HostIdentifier) -> Result<ManagedHost, AppError> {
    let path = layout.host_config(host);
    match fs::symlink_metadata(&path) {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err(AppError::HostNotFound(host.to_string()));
        }
        Err(error) => return Err(AppError::io(&path, error)),
    }
    layout.require_regular_file(&path)?;
    let contents = fs::read_to_string(&path).path_ctx(&path)?;
    ManagedHost::parse(&contents, layout, host.clone(), path)
}

pub(crate) fn write(path: &Path, contents: &str) -> Result<(), AppError> {
    let mut file = fs::File::create(path).path_ctx(path)?;
    file.write_all(contents.as_bytes()).path_ctx(path)?;
    file.sync_all().path_ctx(path)?;
    permissions::set_mode(path, permissions::PRIVATE_MODE)
}

fn required_single<T>(values: Vec<T>, name: &str) -> Result<T, AppError> {
    match values.len() {
        0 => Err(AppError::validation(format!("managed host config has no {name}"))),
        1 => Ok(values.into_iter().next().expect("one value")),
        _ => Err(AppError::validation(format!("managed host config has multiple {name} entries"))),
    }
}

fn optional_single<T>(values: Vec<T>, name: &str) -> Result<Option<T>, AppError> {
    match values.len() {
        0 => Ok(None),
        1 => Ok(values.into_iter().next()),
        _ => Err(AppError::validation(format!("managed host config has multiple {name} entries"))),
    }
}

pub(crate) fn has_managed_include(contents: &str) -> bool {
    let mut top_level = true;
    for line in contents.lines() {
        let Ok(Some((name, value))) = directive(line) else {
            continue;
        };
        if name.eq_ignore_ascii_case("Host") || name.eq_ignore_ascii_case("Match") {
            top_level = false;
        } else if top_level
            && name.eq_ignore_ascii_case("Include")
            && value.split_whitespace().any(|path| {
                scalar(path, "Include").is_ok_and(|path| path == "~/.ssh/conf.d/*.conf")
            })
        {
            return true;
        }
    }
    false
}

pub(crate) fn directive_name(line: &str) -> Option<&str> {
    directive(line).ok().flatten().map(|(name, _)| name)
}

fn directive(line: &str) -> Result<Option<(&str, &str)>, &'static str> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return Ok(None);
    }
    let Some(separator) =
        line.find(|character: char| character == '=' || character.is_whitespace())
    else {
        return Err("directive has no value separator");
    };
    let name = &line[..separator];
    let mut value = &line[separator..];
    if let Some(rest) = value.strip_prefix('=') {
        value = rest.trim_start();
    } else {
        value = value.trim_start();
        if let Some(rest) = value.strip_prefix('=') {
            value = rest.trim_start();
        }
    }
    if name.is_empty() || value.is_empty() {
        return Err("directive name and value must not be empty");
    }
    Ok(Some((name, value.trim_end())))
}

fn scalar<'a>(value: &'a str, name: &str) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.starts_with('"') || value.ends_with('"') {
        let Some(inner) = value.strip_prefix('"').and_then(|value| value.strip_suffix('"')) else {
            return Err(AppError::validation(format!(
                "managed host config has malformed quotes for {name}"
            )));
        };
        if inner.contains('"') {
            return Err(AppError::validation(format!(
                "managed host config has unsupported quoting for {name}"
            )));
        }
        return Ok(inner);
    }
    if value.contains(char::is_whitespace) {
        return Err(AppError::validation(format!(
            "managed host config has multiple values for {name}"
        )));
    }
    Ok(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(contents: &str) -> Result<ManagedHost, AppError> {
        let layout = Layout::from_home(PathBuf::from("/home/test"));
        let host = HostIdentifier::new("example.test").expect("host");
        let path = layout.host_config(&host);
        ManagedHost::parse(contents, &layout, host, path)
    }

    fn valid() -> &'static str {
        "Host example.test\nHostName destination.test\nUser deploy\nPort 2222\nIdentityFile ~/.ssh/id_ed25519_example.test\nIdentitiesOnly yes\n"
    }

    #[test]
    fn managed_document_accepts_supported_lexical_forms() {
        let config = parse(
            "  # managed\nHost = \"example.test\"\nHostName=destination.test\nUser deploy\nPort=2222\nIdentityFile = ~/.ssh/id_ed25519_example.test\nIdentitiesOnly=yes\nProxyJump bastion\n",
        )
        .expect("valid config");
        assert_eq!(config.hostname.as_str(), "destination.test");
        assert_eq!(config.user.as_ref().map(RemoteUser::as_str), Some("deploy"));
        assert_eq!(config.port, Some(2222));
    }

    #[test]
    fn managed_document_rejects_invalid_host_blocks() {
        for contents in [
            valid().replacen("Host example.test\n", "", 1),
            valid().replacen("Host example.test", "Host other.test", 1),
            format!("{}Host example.test\n", valid()),
            format!("{}Match all\n", valid()),
            valid().replacen("Host example.test\n", "Host example.test other.test\n", 1),
        ] {
            assert!(parse(&contents).is_err(), "accepted {contents:?}");
        }
    }

    #[test]
    fn managed_document_rejects_missing_and_duplicate_fields() {
        for directive in [
            "HostName destination.test\n",
            "IdentityFile ~/.ssh/id_ed25519_example.test\n",
            "IdentitiesOnly yes\n",
        ] {
            assert!(parse(&valid().replacen(directive, "", 1)).is_err());
            assert!(parse(&format!("{}{directive}", valid())).is_err());
        }
        for directive in ["User deploy\n", "Port 2222\n"] {
            assert!(parse(&format!("{}{directive}", valid())).is_err());
        }
    }

    #[test]
    fn managed_document_rejects_invalid_values_and_scope() {
        for contents in [
            valid().replacen("destination.test", "-option", 1),
            valid().replacen("User deploy", "User bad@user", 1),
            valid().replacen("Port 2222", "Port invalid", 1),
            valid().replacen("IdentitiesOnly yes", "IdentitiesOnly no", 1),
            valid().replacen("id_ed25519_example.test", "id_ed25519_other.test", 1),
            format!("HostName destination.test\n{contents}", contents = valid()),
        ] {
            assert!(parse(&contents).is_err(), "accepted {contents:?}");
        }
    }

    #[test]
    fn include_must_be_top_level_and_supports_equals() {
        assert!(has_managed_include("Include=~/.ssh/conf.d/*.conf\nHost example\n"));
        assert!(has_managed_include("Include \"~/.ssh/conf.d/*.conf\"\nHost example\n"));
        assert!(!has_managed_include("Host example\nInclude ~/.ssh/conf.d/*.conf\n"));
    }
}
