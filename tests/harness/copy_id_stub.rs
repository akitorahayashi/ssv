use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(crate) fn write(path: &Path) {
    let script = r#"#!/usr/bin/env sh
set -eu
: > "${HOME}/ssh-copy-id.args"
for arg in "$@"; do
  printf '%s\n' "$arg" >> "${HOME}/ssh-copy-id.args"
done
"#;
    fs::write(path, script).expect("copy-id stub should be written");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path).expect("stub metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("stub should be executable");
    }
}
