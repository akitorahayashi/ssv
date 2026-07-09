use crate::error::AppError;
use std::path::Path;

/// Standard OpenSSH FIDO/security-key identity filenames. A managed key must never render to
/// one of these, and the orphan scanner must ignore them: they are personal keys, not ssv-owned.
const RESERVED_KEY_NAMES: [&str; 2] = ["id_ecdsa_sk", "id_ed25519_sk"];

/// The single grammar for ssv-managed key filenames: `id_<key_type>_<host>`.
///
/// This is the sole authority for producing and recognizing managed key names. `render`/
/// `identity_directive` produce the on-disk filename and the `IdentityFile` directive value;
/// `parse` recognizes a filename; `new` constructs a name for generation, rejecting collisions
/// with reserved OpenSSH names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ManagedKeyName {
    key_type: String,
    host: String,
}

impl ManagedKeyName {
    /// Construct a managed key name from an already-validated key type and host, rejecting a
    /// `(key_type, host)` pair whose rendered filename collides with a reserved OpenSSH name.
    pub(crate) fn new(key_type: &str, host: &str) -> Result<Self, AppError> {
        let name = Self { key_type: key_type.to_string(), host: host.to_string() };
        if RESERVED_KEY_NAMES.contains(&name.render().as_str()) {
            return Err(AppError::validation(format!(
                "refusing to generate key '{}': it collides with a reserved OpenSSH key name",
                name.render()
            )));
        }
        Ok(name)
    }

    /// Recognize a private-key filename as a managed key name. Returns `None` for reserved
    /// OpenSSH names, non-managed grammar, or invalid key type/host.
    pub(crate) fn parse(filename: &str) -> Option<Self> {
        if RESERVED_KEY_NAMES.contains(&filename) {
            return None;
        }
        let (key_type, host) = filename.strip_prefix("id_")?.split_once('_')?;
        if validate_key_type(key_type).is_err() || validate_host(host).is_err() {
            return None;
        }
        Some(Self { key_type: key_type.to_string(), host: host.to_string() })
    }

    pub(crate) fn key_type(&self) -> &str {
        &self.key_type
    }

    pub(crate) fn host(&self) -> &str {
        &self.host
    }

    /// The key-pair filename, e.g. `id_ed25519_github.com`.
    pub(crate) fn render(&self) -> String {
        format!("id_{}_{}", self.key_type, self.host)
    }

    /// The `IdentityFile` directive value, e.g. `~/.ssh/id_ed25519_github.com`.
    pub(crate) fn identity_directive(&self) -> String {
        format!("~/.ssh/{}", self.render())
    }
}

/// Return the key type of a managed identity path that belongs to `host`, or an error if the
/// filename is not an ssv-owned managed key for that host.
pub(crate) fn managed_key_type(path: &Path, host: &str) -> Result<String, AppError> {
    let filename = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| AppError::validation("IdentityFile has no UTF-8 filename"))?;
    match ManagedKeyName::parse(filename) {
        Some(name) if name.host() == host => Ok(name.key_type().to_string()),
        _ => Err(unmanaged_identity(path, host)),
    }
}

/// Whether a filesystem entry is a candidate ssv-managed private key (used by orphan detection).
pub(crate) fn is_managed_key_candidate(path: &Path) -> bool {
    let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    if filename.ends_with(".pub") {
        return false;
    }
    ManagedKeyName::parse(filename).is_some()
}

fn unmanaged_identity(path: &Path, host: &str) -> AppError {
    AppError::validation(format!(
        "refusing to manage identity '{}' because it does not match host '{host}'",
        path.display()
    ))
}

pub(crate) fn validate_host(host: &str) -> Result<(), AppError> {
    if host.is_empty() {
        return Err(AppError::validation("host must not be empty"));
    }
    if !host.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_')) {
        return Err(AppError::validation(format!(
            "invalid host identifier '{host}'; allowed characters are alphanumeric, '.', '-', '_'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_key_type(key_type: &str) -> Result<(), AppError> {
    if key_type.is_empty()
        || !key_type.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
    {
        return Err(AppError::validation(format!(
            "invalid key type '{key_type}'; expected lowercase letters or digits"
        )));
    }
    Ok(())
}

pub(crate) fn validate_hostname(hostname: &str) -> Result<(), AppError> {
    if hostname.is_empty() {
        return Err(AppError::validation("hostname must not be empty"));
    }
    let allowed =
        |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_' | '[' | ']' | ':');
    if !hostname.chars().all(allowed) {
        return Err(AppError::validation(format!(
            "invalid hostname '{hostname}'; allowed characters are alphanumeric, '.', '-', '_', '[', ']', ':'"
        )));
    }
    Ok(())
}

pub(crate) fn validate_user(user: &str) -> Result<(), AppError> {
    if user.is_empty() {
        return Err(AppError::validation("user must not be empty"));
    }
    if user.chars().any(|c| c.is_control() || c.is_whitespace()) {
        return Err(AppError::validation(
            "invalid user; control characters and whitespace are not allowed",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_and_directive_round_trip() {
        let name = ManagedKeyName::new("ed25519", "github.com").expect("valid name");
        assert_eq!(name.render(), "id_ed25519_github.com");
        assert_eq!(name.identity_directive(), "~/.ssh/id_ed25519_github.com");

        let parsed = ManagedKeyName::parse(&name.render()).expect("round-trip parse");
        assert_eq!(parsed.key_type(), "ed25519");
        assert_eq!(parsed.host(), "github.com");
    }

    #[test]
    fn host_with_underscore_round_trips() {
        let name = ManagedKeyName::new("ed25519", "foo_bar").expect("valid name");
        assert_eq!(name.render(), "id_ed25519_foo_bar");
        let parsed = ManagedKeyName::parse("id_ed25519_foo_bar").expect("parse");
        assert_eq!(parsed.key_type(), "ed25519");
        assert_eq!(parsed.host(), "foo_bar");
    }

    #[test]
    fn reserved_names_are_rejected_only_when_they_collide() {
        // The exact FIDO key names collide and are refused.
        assert!(ManagedKeyName::new("ed25519", "sk").is_err());
        assert!(ManagedKeyName::new("ecdsa", "sk").is_err());
        // A non-FIDO key type or a longer host does not collide and is accepted.
        assert_eq!(ManagedKeyName::new("rsa", "sk").expect("valid").render(), "id_rsa_sk");
        assert_eq!(
            ManagedKeyName::new("ed25519", "foo_sk").expect("valid").render(),
            "id_ed25519_foo_sk"
        );
    }

    #[test]
    fn parse_ignores_reserved_and_non_managed_names() {
        assert!(ManagedKeyName::parse("id_ed25519_sk").is_none());
        assert!(ManagedKeyName::parse("id_ecdsa_sk").is_none());
        assert!(ManagedKeyName::parse("id_ed25519").is_none());
        assert!(ManagedKeyName::parse("random").is_none());
    }

    #[test]
    fn parse_recognizes_legitimate_sk_suffixed_host() {
        // A `*_sk` host is ssv-owned and must be recognizable so audit can flag it as orphaned.
        let parsed = ManagedKeyName::parse("id_ed25519_deploy_sk").expect("parse");
        assert_eq!(parsed.host(), "deploy_sk");
    }

    #[test]
    fn host_validation_accepts_managed_identifiers() {
        assert!(validate_host("github.com").is_ok());
        assert!(validate_host("internal-host_01").is_ok());
    }

    #[test]
    fn host_validation_rejects_path_components() {
        assert!(validate_host("bad/host").is_err());
        assert!(validate_host("spaces host").is_err());
    }

    #[test]
    fn managed_key_candidates_exclude_standard_keys() {
        assert!(!is_managed_key_candidate(Path::new("id_ed25519")));
        assert!(!is_managed_key_candidate(Path::new("id_ed25519_sk")));
        assert!(is_managed_key_candidate(Path::new("id_ed25519_github.com")));
    }
}
