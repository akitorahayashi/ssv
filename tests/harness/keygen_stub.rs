use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

pub(crate) fn write(path: &Path) {
    let script = r#"#!/usr/bin/env sh
set -eu
outfile=""
keytype="stub"
derive="false"
while [ "$#" -gt 0 ]; do
  arg="$1"
  shift
  case "$arg" in
    -f)
      outfile="$1"
      shift
      ;;
    -t)
      keytype="$1"
      shift
      ;;
    -y)
      derive="true"
      ;;
    -P)
      shift
      ;;
    *)
      ;;
  esac
done
if [ -z "$outfile" ]; then
  echo "missing -f argument" >&2
  exit 1
fi
if [ "$derive" = "true" ]; then
  keytype=$(sed -n 's/^PRIVATE-//p' "$outfile")
  printf 'ssh-%s AAAATESTKEY\n' "$keytype"
  exit 0
fi
printf 'PRIVATE-%s\n' "$keytype" > "$outfile"
printf 'ssh-%s AAAATESTKEY %s@ssv\n' "$keytype" "$keytype" > "${outfile}.pub"
"#;
    fs::write(path, script).expect("keygen stub should be written");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(path).expect("stub metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).expect("stub should be executable");
    }
}
