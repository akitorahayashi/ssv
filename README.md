# ssv

`ssv` is a standalone Rust CLI for managing SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It bootstraps the required SSH layout, generates keys via `ssh-keygen`, lists managed hosts, prints public keys, audits managed assets, and removes credentials when they are no longer needed.

## Features

- Secure bootstrap: `ssv init` (alias: `i`) ensures `~/.ssh`, `~/.ssh/conf.d`, and `~/.ssh/config` are ready for `ssv`-managed hosts.
- Key generation: `ssv generate` (alias: `g`) wraps `ssh-keygen`, writes host-specific configs, and prints the public key so it can be registered immediately.
- Inventory awareness: `ssv list` (alias: `ls`) scans managed configs and shows the hostnames under management.
- Public key lookup: `ssv show <HOST>` (alias: `sw`) prints the public key referenced by a managed host config.
- Read-only audit: `ssv audit` (alias: `au`) reports missing assets, unsafe permissions, key mismatches, and other inconsistencies without modifying files.
- Safe teardown: `ssv remove` (alias: `rm`) deletes only the key pair referenced by the managed host config.
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

All subcommands support short aliases for convenience:

| Subcommand | Alias | Description |
| --- | --- | --- |
| init | i | Bootstrap the SSH configuration layout |
| generate | g | Generate a key pair and host configuration file |
| list | ls | List managed hosts |
| remove | rm | Remove key pairs and configuration for a host |
| show | sw | Print the public key for a managed host |
| audit | au | Inspect managed SSH assets without modifying them |

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
