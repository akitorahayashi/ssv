use crate::error::AppError;
use crate::ssh::host_config::{HostConfig, has_managed_include};
use crate::ssh::keygen;
use crate::ssh::layout::Layout;
use crate::ssh::permissions;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditSeverity {
    Error,
    Warning,
}

impl Display for AuditSeverity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Error => formatter.write_str("error"),
            Self::Warning => formatter.write_str("warning"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditCode {
    Missing,
    MissingInclude,
    UnsafePermissions,
    NonStandardPermissions,
    OwnerMismatch,
    InvalidFileType,
    ConfigParse,
    OutsideManagedRoot,
    KeyMismatch,
    OrphanedAsset,
    ReadFailure,
    UnsupportedPlatform,
}

impl Display for AuditCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            Self::Missing => "missing",
            Self::MissingInclude => "missing-include",
            Self::UnsafePermissions => "unsafe-permissions",
            Self::NonStandardPermissions => "non-standard-permissions",
            Self::OwnerMismatch => "owner-mismatch",
            Self::InvalidFileType => "invalid-file-type",
            Self::ConfigParse => "config-parse",
            Self::OutsideManagedRoot => "outside-managed-root",
            Self::KeyMismatch => "key-mismatch",
            Self::OrphanedAsset => "orphaned-asset",
            Self::ReadFailure => "read-failure",
            Self::UnsupportedPlatform => "unsupported-platform",
        };
        formatter.write_str(code)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditFinding {
    pub severity: AuditSeverity,
    pub code: AuditCode,
    pub path: PathBuf,
    pub message: String,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct AuditReport {
    pub findings: Vec<AuditFinding>,
}

impl AuditReport {
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|finding| finding.severity == AuditSeverity::Error)
    }

    fn error(&mut self, code: AuditCode, path: &Path, message: impl Into<String>) {
        self.findings.push(AuditFinding {
            severity: AuditSeverity::Error,
            code,
            path: path.to_path_buf(),
            message: message.into(),
        });
    }

    fn warning(&mut self, code: AuditCode, path: &Path, message: impl Into<String>) {
        self.findings.push(AuditFinding {
            severity: AuditSeverity::Warning,
            code,
            path: path.to_path_buf(),
            message: message.into(),
        });
    }
}

pub(crate) fn execute() -> Result<AuditReport, AppError> {
    let layout = Layout::from_env()?;
    #[cfg(unix)]
    let owner = owner_of(layout.home());
    let mut audit = Audit {
        layout,
        report: AuditReport::default(),
        referenced_keys: HashSet::new(),
        #[cfg(unix)]
        owner,
    };
    audit.run();
    Ok(audit.report)
}

struct Audit {
    layout: Layout,
    report: AuditReport,
    referenced_keys: HashSet<PathBuf>,
    #[cfg(unix)]
    owner: Option<u32>,
}

impl Audit {
    fn run(&mut self) {
        self.inspect_directory(&self.layout.root(), permissions::DIRECTORY_MODE);
        self.inspect_directory(&self.layout.hosts(), permissions::DIRECTORY_MODE);
        self.inspect_main_config();
        self.inspect_host_configs();
        self.inspect_orphaned_keys();

        #[cfg(not(unix))]
        self.report.warning(
            AuditCode::UnsupportedPlatform,
            &self.layout.root(),
            "Unix ownership and permission checks were not performed",
        );
    }

    fn inspect_main_config(&mut self) {
        let path = self.layout.config();
        if !self.inspect_config_file(&path) {
            return;
        }
        match fs::read_to_string(&path) {
            Ok(contents) if has_managed_include(&contents) => {}
            Ok(_) => self.report.error(
                AuditCode::MissingInclude,
                &path,
                "top-level Include ~/.ssh/conf.d/*.conf is missing",
            ),
            Err(error) => self.read_failure(&path, error),
        }
    }

    fn inspect_host_configs(&mut self) {
        let hosts = self.layout.hosts();
        let entries = match fs::read_dir(&hosts) {
            Ok(entries) => entries,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    self.read_failure(&hosts, error);
                }
                return;
            }
        };

        for entry in entries {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(error) => {
                    self.read_failure(&hosts, error);
                    continue;
                }
            };
            if path.extension().and_then(|extension| extension.to_str()) != Some("conf") {
                continue;
            }
            self.inspect_host_config(&path);
        }
    }

    fn inspect_host_config(&mut self, path: &Path) {
        if !self.inspect_config_file(path) {
            return;
        }
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => {
                self.read_failure(path, error);
                return;
            }
        };
        let config = match HostConfig::parse(&contents, &self.layout) {
            Ok(config) => config,
            Err(error) => {
                let code = if matches!(error, AppError::OutsideManagedRoot(_)) {
                    AuditCode::OutsideManagedRoot
                } else {
                    AuditCode::ConfigParse
                };
                self.report.error(code, path, error.to_string());
                return;
            }
        };
        self.referenced_keys.insert(config.identity.clone());
        self.inspect_private_key(&config.identity);
    }

    fn inspect_private_key(&mut self, private: &Path) {
        if !self.inspect_file(private, FilePolicy::Private) {
            return;
        }
        let public = match self.layout.public_key(private) {
            Ok(public) => public,
            Err(error) => {
                self.report.error(AuditCode::ConfigParse, private, error.to_string());
                return;
            }
        };
        if !self.inspect_file(&public, FilePolicy::Public) {
            return;
        }
        self.compare_key_pair(private, &public);
    }

    fn compare_key_pair(&mut self, private: &Path, public: &Path) {
        let expected = match derived_public_key(private) {
            Ok(key) => key,
            Err(message) => {
                self.report.error(AuditCode::KeyMismatch, private, message);
                return;
            }
        };
        let actual = match fs::read_to_string(public) {
            Ok(contents) => comparable_key(&contents),
            Err(error) => {
                self.read_failure(public, error);
                return;
            }
        };
        if expected != actual {
            self.report.error(
                AuditCode::KeyMismatch,
                public,
                "public key does not match the configured private key",
            );
        }
    }

    fn inspect_orphaned_keys(&mut self) {
        let root = self.layout.root();
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries,
            Err(error) => {
                if error.kind() != std::io::ErrorKind::NotFound {
                    self.read_failure(&root, error);
                }
                return;
            }
        };
        for entry in entries {
            let path = match entry {
                Ok(entry) => entry.path(),
                Err(error) => {
                    self.read_failure(&root, error);
                    continue;
                }
            };
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if filename.starts_with("id_")
                && !filename.ends_with(".pub")
                && !self.referenced_keys.contains(&path)
            {
                self.report.warning(
                    AuditCode::OrphanedAsset,
                    &path,
                    "key is not referenced by a managed host config",
                );
            }
        }
    }

    fn inspect_directory(&mut self, path: &Path, expected_mode: u32) -> bool {
        self.inspect_path(path, ExpectedType::Directory, Some(expected_mode), FilePolicy::Directory)
    }

    fn inspect_config_file(&mut self, path: &Path) -> bool {
        self.inspect_path(
            path,
            ExpectedType::File,
            Some(permissions::PRIVATE_MODE),
            FilePolicy::Config,
        )
    }

    fn inspect_file(&mut self, path: &Path, policy: FilePolicy) -> bool {
        let expected_mode = match policy {
            FilePolicy::Private => Some(permissions::PRIVATE_MODE),
            FilePolicy::Public => None,
            FilePolicy::Config | FilePolicy::Directory => unreachable!(),
        };
        self.inspect_path(path, ExpectedType::File, expected_mode, policy)
    }

    fn inspect_path(
        &mut self,
        path: &Path,
        expected_type: ExpectedType,
        expected_mode: Option<u32>,
        policy: FilePolicy,
    ) -> bool {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                self.report.error(AuditCode::Missing, path, "required managed asset is missing");
                return false;
            }
            Err(error) => {
                self.read_failure(path, error);
                return false;
            }
        };
        let correct_type = match expected_type {
            ExpectedType::Directory => metadata.file_type().is_dir(),
            ExpectedType::File => metadata.file_type().is_file(),
        };
        if !correct_type {
            self.report.error(
                AuditCode::InvalidFileType,
                path,
                "managed asset has an unexpected file type",
            );
            return false;
        }
        match self.layout.has_symlink_component(path) {
            Ok(false) => {}
            Ok(true) => {
                self.report.error(
                    AuditCode::InvalidFileType,
                    path,
                    "managed asset path contains a symbolic link",
                );
                return false;
            }
            Err(error) => {
                self.report.error(AuditCode::ReadFailure, path, error.to_string());
                return false;
            }
        }
        self.inspect_owner(path, &metadata);
        self.inspect_mode(path, &metadata, expected_mode, policy);
        true
    }

    fn inspect_owner(&mut self, path: &Path, metadata: &fs::Metadata) {
        #[cfg(unix)]
        if let Some(owner) = self.owner
            && permissions::owner(metadata) != owner
        {
            self.report.error(
                AuditCode::OwnerMismatch,
                path,
                "managed asset owner differs from the HOME directory owner",
            );
        }
        #[cfg(not(unix))]
        let _ = (path, metadata);
    }

    fn inspect_mode(
        &mut self,
        path: &Path,
        metadata: &fs::Metadata,
        expected_mode: Option<u32>,
        policy: FilePolicy,
    ) {
        #[cfg(unix)]
        {
            let mode = permissions::mode(metadata);
            let unsafe_mode = match policy {
                FilePolicy::Directory | FilePolicy::Private => mode & 0o077 != 0,
                FilePolicy::Config => mode & 0o022 != 0,
                FilePolicy::Public => false,
            };
            if unsafe_mode {
                self.report.error(
                    AuditCode::UnsafePermissions,
                    path,
                    format!("permissions {mode:04o} expose the managed asset"),
                );
            } else if let Some(expected) = expected_mode
                && mode != expected
            {
                self.report.warning(
                    AuditCode::NonStandardPermissions,
                    path,
                    format!("permissions {mode:04o} differ from ssv standard {expected:04o}"),
                );
            }
        }
        #[cfg(not(unix))]
        let _ = (path, metadata, expected_mode, policy);
    }

    fn read_failure(&mut self, path: &Path, error: std::io::Error) {
        self.report.error(AuditCode::ReadFailure, path, error.to_string());
    }
}

#[derive(Clone, Copy)]
enum ExpectedType {
    Directory,
    File,
}

#[derive(Clone, Copy)]
enum FilePolicy {
    Directory,
    Config,
    Private,
    Public,
}

fn derived_public_key(private: &Path) -> Result<String, String> {
    keygen::derive_public(private)
        .map(|output| comparable_key(&output))
        .map_err(|error| error.to_string())
}

fn comparable_key(contents: &str) -> String {
    contents.split_whitespace().take(2).collect::<Vec<_>>().join(" ")
}

#[cfg(unix)]
fn owner_of(path: &Path) -> Option<u32> {
    fs::metadata(path).ok().map(|metadata| permissions::owner(&metadata))
}
