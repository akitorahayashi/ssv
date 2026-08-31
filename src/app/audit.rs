use crate::context::Context;
use crate::error::AppError;
use crate::ssh::host_config::{ManagedHost, has_managed_include};
use crate::ssh::inventory::{self, HostCandidate, IssueKind, KeyCandidate};
use crate::ssh::keygen;
use crate::ssh::layout::Layout;
use crate::ssh::naming::KeyFileKind;
use crate::ssh::permissions;
use std::collections::HashSet;
use std::fmt::{self, Display};
use std::fs;
use std::os::unix::ffi::OsStrExt;
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
    UnmanagedIdentity,
    KeyMismatch,
    KeyVerification,
    OrphanedAsset,
    ReadFailure,
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
            Self::UnmanagedIdentity => "unmanaged-identity",
            Self::KeyMismatch => "key-mismatch",
            Self::KeyVerification => "key-verification",
            Self::OrphanedAsset => "orphaned-asset",
            Self::ReadFailure => "read-failure",
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

    pub fn has_warnings(&self) -> bool {
        self.findings.iter().any(|finding| finding.severity == AuditSeverity::Warning)
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

    fn sort(&mut self) {
        self.findings.sort_by(|left, right| {
            left.path
                .as_os_str()
                .as_bytes()
                .cmp(right.path.as_os_str().as_bytes())
                .then_with(|| severity_rank(left.severity).cmp(&severity_rank(right.severity)))
                .then_with(|| left.code.to_string().cmp(&right.code.to_string()))
                .then_with(|| left.message.as_bytes().cmp(right.message.as_bytes()))
        });
    }
}

pub(crate) fn execute(ctx: &Context) -> Result<AuditReport, AppError> {
    let layout = ctx.layout().clone();
    let mut report = AuditReport::default();
    let owner = match fs::metadata(layout.home()) {
        Ok(metadata) => Some(permissions::owner(&metadata)),
        Err(error) => {
            report.error(AuditCode::ReadFailure, layout.home(), error.to_string());
            None
        }
    };
    let mut audit = Audit {
        layout,
        keygen: ctx.keygen().to_path_buf(),
        report,
        referenced_keys: HashSet::new(),
        owner,
    };
    audit.run();
    audit.report.sort();
    Ok(audit.report)
}

struct Audit {
    layout: Layout,
    keygen: PathBuf,
    report: AuditReport,
    referenced_keys: HashSet<PathBuf>,
    owner: Option<u32>,
}

impl Audit {
    fn run(&mut self) {
        if !self.inspect_directory(&self.layout.root(), permissions::DIRECTORY_MODE) {
            return;
        }
        self.inspect_main_config();
        if self.inspect_directory(&self.layout.hosts(), permissions::DIRECTORY_MODE) {
            self.inspect_host_configs();
        }
        self.inspect_orphaned_keys();
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
        let candidates = match inventory::hosts(&self.layout) {
            Ok(candidates) => candidates,
            Err(error) => {
                self.candidate_failure(&hosts, IssueKind::Read, error);
                return;
            }
        };
        for candidate in candidates {
            match candidate {
                HostCandidate::Managed(config) => self.inspect_managed_host(config),
                HostCandidate::Invalid { path, kind, error } => {
                    self.candidate_failure(&path, kind, error);
                }
            }
        }
    }

    fn inspect_managed_host(&mut self, config: ManagedHost) {
        self.inspect_config_file(&config.path);
        self.referenced_keys.insert(config.private_key.clone());
        self.referenced_keys.insert(config.public_key.clone());
        self.inspect_key_pair(&config.private_key, &config.public_key);
    }

    fn inspect_key_pair(&mut self, private: &Path, public: &Path) {
        let private_valid = self.inspect_path(
            private,
            ExpectedType::File,
            Some(permissions::PRIVATE_MODE),
            SECRET_UNSAFE_MASK,
        );
        let public_valid = self.inspect_path(public, ExpectedType::File, None, PUBLIC_UNSAFE_MASK);
        if private_valid && public_valid {
            self.compare_key_pair(private, public);
        }
    }

    fn compare_key_pair(&mut self, private: &Path, public: &Path) {
        let expected = match derived_public_key(&self.keygen, private) {
            Ok(key) => key,
            Err(message) => {
                self.report.error(AuditCode::KeyVerification, private, message);
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
        let candidates = match inventory::keys(&self.layout) {
            Ok(candidates) => candidates,
            Err(error) => {
                self.candidate_failure(&root, IssueKind::Read, error);
                return;
            }
        };
        for candidate in candidates {
            match candidate {
                KeyCandidate::Managed { path, name, kind } => {
                    if self.referenced_keys.contains(&path) {
                        continue;
                    }
                    let file_kind = match kind {
                        KeyFileKind::Private => "private key",
                        KeyFileKind::Public => "public key",
                    };
                    self.report.warning(
                        AuditCode::OrphanedAsset,
                        &path,
                        format!(
                            "managed {file_kind} for host '{}' is not referenced by a managed host config",
                            name.host()
                        ),
                    );
                }
                KeyCandidate::Invalid { path, kind, error } => {
                    if !self.referenced_keys.contains(&path) {
                        self.candidate_failure(&path, kind, error);
                    }
                }
            }
        }
    }

    fn inspect_directory(&mut self, path: &Path, expected_mode: u32) -> bool {
        self.inspect_path(path, ExpectedType::Directory, Some(expected_mode), SECRET_UNSAFE_MASK)
    }

    fn inspect_config_file(&mut self, path: &Path) -> bool {
        self.inspect_path(
            path,
            ExpectedType::File,
            Some(permissions::PRIVATE_MODE),
            CONFIG_UNSAFE_MASK,
        )
    }

    fn inspect_path(
        &mut self,
        path: &Path,
        expected_type: ExpectedType,
        expected_mode: Option<u32>,
        unsafe_mask: u32,
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
        self.inspect_mode(path, &metadata, expected_mode, unsafe_mask);
        true
    }

    fn inspect_owner(&mut self, path: &Path, metadata: &fs::Metadata) {
        if let Some(owner) = self.owner
            && permissions::owner(metadata) != owner
        {
            self.report.error(
                AuditCode::OwnerMismatch,
                path,
                "managed asset owner differs from the HOME directory owner",
            );
        }
    }

    fn inspect_mode(
        &mut self,
        path: &Path,
        metadata: &fs::Metadata,
        expected_mode: Option<u32>,
        unsafe_mask: u32,
    ) {
        let mode = permissions::mode(metadata);
        if mode & unsafe_mask != 0 {
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

    fn read_failure(&mut self, path: &Path, error: std::io::Error) {
        self.report.error(AuditCode::ReadFailure, path, error.to_string());
    }

    fn candidate_failure(&mut self, path: &Path, kind: IssueKind, error: AppError) {
        let code = match (&error, kind) {
            (AppError::OutsideManagedRoot(_), _) => AuditCode::OutsideManagedRoot,
            (AppError::UnmanagedIdentity(_), _) => AuditCode::UnmanagedIdentity,
            (_, IssueKind::Read) => AuditCode::ReadFailure,
            (_, IssueKind::FileType) => AuditCode::InvalidFileType,
            (_, IssueKind::Contract) => AuditCode::ConfigParse,
        };
        self.report.error(code, path, error.to_string());
    }
}

#[derive(Clone, Copy)]
enum ExpectedType {
    Directory,
    File,
}

const SECRET_UNSAFE_MASK: u32 = 0o077;
const CONFIG_UNSAFE_MASK: u32 = 0o022;
const PUBLIC_UNSAFE_MASK: u32 = 0o000;

fn derived_public_key(keygen: &Path, private: &Path) -> Result<String, String> {
    keygen::derive_public(keygen, private)
        .map(|output| comparable_key(&output))
        .map_err(|error| error.to_string())
}

fn comparable_key(contents: &str) -> String {
    contents.split_whitespace().take(2).collect::<Vec<_>>().join(" ")
}

fn severity_rank(severity: AuditSeverity) -> u8 {
    match severity {
        AuditSeverity::Error => 0,
        AuditSeverity::Warning => 1,
    }
}
