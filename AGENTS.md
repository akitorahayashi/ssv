# ssv Development Overview

## Project Summary
`ssv` is a Rust CLI for SSH key and host configuration management under `~/.ssh/conf.d/`. It manages SSH layout bootstrapping, key generation via `ssh-keygen`, key deployment via `ssh-copy-id`, managed host setting updates, repository `origin` remote rewrites, host listing, public key output, audit checks, and teardown.

## Plugin Skills
The `plugin/` directory provides Agent Skills for Claude Code and Codex:
- `github-ssh-setup`: SSH host configuration and public key registration for GitHub.
- `ip-ssh-setup`: Direct SSH host configuration using an IP address and username.

## Testing Notes
Integration tests in `tests/` stub `ssh-keygen` and `ssh-copy-id` via `SSV_SSH_KEYGEN_PATH` and `SSV_SSH_COPY_ID_PATH` environment variables. They create Git repositories using `git2` and run sequentially via `serial_test` due to `HOME` manipulation.
