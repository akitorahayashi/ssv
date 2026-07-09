# ssv Development Overview

## Project Summary
`ssv` is a Rust CLI that owns the SSH bootstrap and manages SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It ensures the required `~/.ssh/config` include entry exists, wraps `ssh-keygen` to generate keys, rewrites repository `origin` remotes to managed hosts, lists managed hosts, and safely removes keys/configs when a host is retired.

## Workflow
- Setup: `just setup`
- Format & lint: `just check`
- Tests: `just test`

## Testing Notes
Integration tests in `tests/` stub `ssh-keygen` and `ssh-copy-id` through the `SSV_SSH_KEYGEN_PATH` and `SSV_SSH_COPY_ID_PATH` environment variables, which override the subprocess binaries so the suite runs without real key material. They create Git repositories through `git2` and run sequentially via `serial_test` because they manipulate `HOME`. Keep new tests consistent with that pattern.
