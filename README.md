# ssv

`ssv` is a standalone Rust CLI for managing SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It replaces ad-hoc scripts with a single binary that can generate keys via `ssh-keygen`, list managed hosts, and clean up credentials when they are no longer needed.

## Features

- **Secure bootstrap** – every subcommand ensures `~/.ssh` and `~/.ssh/conf.d` exist with `0700` permissions before continuing.
- **Key generation** – `ssv generate` wraps `ssh-keygen`, writes host-specific configs, and prints the public key so it can be registered immediately.
- **Inventory awareness** – `ssv list` scans managed configs and shows the hostnames under management.
- **Safe teardown** – `ssv remove` deletes matching configs and key pairs without erroring if files were already removed manually.

## Usage

```bash
# Generate keys/config for github.com
ssv generate --host github.com --user git

# List all managed hosts
ssv list

# Remove keys/config for github.com
ssv remove --host github.com
```

Configuration files are stored at `~/.ssh/conf.d/<HOST>.conf`, and keys follow the `~/.ssh/id_<TYPE>_<HOST>` naming convention. Optional `--type`, `--user`, and `--port` flags let you customise the generated configuration.

## Development

```bash
cargo build         # debug build
cargo build --release
cargo fmt
cargo clippy --all-targets --all-features -- -D warnings
RUST_TEST_THREADS=1 cargo test --all-targets --all-features
```

### Testing

Integration tests in `tests/` exercise the CLI and library API with a stubbed `ssh-keygen`. They rely on `serial_test` because the fixtures manipulate the `HOME` environment variable. Run the full suite with `cargo test` before committing changes.

## License

This project is distributed under the MIT license.
