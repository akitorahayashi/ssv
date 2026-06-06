# ssv

`ssv` is a standalone Rust CLI for managing SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It bootstraps the required SSH layout, generates keys via `ssh-keygen`, lists managed hosts, prints public keys, audits managed assets, and removes credentials when they are no longer needed.

## Features

- Secure bootstrap: `ssv init` ensures `~/.ssh`, `~/.ssh/conf.d`, and `~/.ssh/config` are ready for `ssv`-managed hosts.
- Key generation: `ssv generate` wraps `ssh-keygen`, writes host-specific configs, and prints the public key so it can be registered immediately.
- Inventory awareness: `ssv list` scans managed configs and shows the hostnames under management.
- Public key lookup: `ssv show <HOST>` prints the public key referenced by a managed host config.
- Read-only audit: `ssv audit` reports missing assets, unsafe permissions, key mismatches, and other inconsistencies without modifying files.
- Safe teardown: `ssv remove` deletes only the key pair referenced by the managed host config.
- Agentless: generated configurations use explicit `IdentityFile` paths, so `ssh-agent` and reboots are not required.

## Setup

```bash
ssv init
```

`ssv init` ensures `~/.ssh`, `~/.ssh/conf.d`, and `~/.ssh/config` are ready for `ssv`-managed hosts.

## Usage

```bash
# Bootstrap SSH config integration
ssv init

# Generate keys/config for github.com
ssv generate --host github.com --user git

# List all managed hosts
ssv list

# Print a managed host's public key
ssv show github.com

# Audit managed SSH assets
ssv audit

# Remove keys/config for github.com
ssv remove --host github.com
```

Configuration files are stored at `~/.ssh/conf.d/<HOST>.conf`, and keys follow the `~/.ssh/id_<TYPE>_<HOST>` naming convention. Optional `--type`, `--user`, and `--port` flags let you customise the generated configuration.

`list`, `show`, and `audit` are read-only. `audit` writes findings to standard error and exits non-zero when error-level findings exist.

## Development

```bash
cargo build         # debug build
cargo build --release
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
RUST_TEST_THREADS=1 cargo test --all-targets --all-features
```

### Testing

Integration tests in `tests/` exercise the CLI and library API with a stubbed `ssh-keygen`. They rely on `serial_test` because the fixtures manipulate the `HOME` environment variable. Run the full suite with `RUST_TEST_THREADS=1 cargo test --all-targets --all-features` before committing changes.

## License

This project is distributed under the MIT license.
