# ssv Development Overview

## Project Summary
`ssv` is a Rust CLI that manages SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It wraps `ssh-keygen` to generate keys, lists managed hosts, and safely removes keys/configs when a host is retired.

## Tech Stack
- Rust 2024 edition
- `clap` for CLI parsing
- Test dependencies: `assert_cmd`, `predicates`, `serial_test`, `tempfile`

## Workflow
- Build: `cargo build` / `cargo build --release`
- Format & lint: `cargo fmt` and `cargo clippy --all-targets --all-features -- -D warnings`
- Tests: `RUST_TEST_THREADS=1 cargo test --all-targets --all-features`

## Testing Notes
Integration tests in `tests/` stub `ssh-keygen` and run sequentially via `serial_test` because they manipulate `HOME`. Keep new tests consistent with that pattern.
