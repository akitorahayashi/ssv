# ssv Development Overview

## Project Summary
`ssv` is a Rust CLI for SSH key and host configuration management under `~/.ssh/conf.d/`. It manages SSH layout bootstrapping, key generation via `ssh-keygen`, key deployment via `ssh-copy-id`, managed host setting updates, repository `origin` remote rewrites, host listing, public key output, audit checks, and teardown.

## Plugin Skills

The `plugin/` directory provides Agent Skills for Claude Code and Codex:

- `github-ssh-setup`: SSH host configuration and public key registration for GitHub.
- `ip-ssh-setup`: Direct SSH host configuration using an IP address and username.

## Workflow

- Setup: `just setup`
- Format: `just fix`
- Static checks: `just check`
- Tests: `just test`

## Testing Notes

Integration tests stub `ssh-keygen` and `ssh-copy-id` through
`SSV_SSH_KEYGEN_PATH` and `SSV_SSH_COPY_ID_PATH`. CLI tests set `HOME` only on child
processes, while library tests use explicit `Context` paths. Tests run concurrently without a
serialization dependency.
