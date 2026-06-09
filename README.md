# ssv

`ssv` is a standalone Rust CLI for managing SSH key pairs and host configuration files under `~/.ssh/conf.d/`. It bootstraps the required SSH layout, generates keys via `ssh-keygen`, relinks repository remotes to managed SSH hosts, lists managed hosts, prints public keys, audits managed assets, and removes credentials when they are no longer needed.

## Features

- Secure bootstrap: `ssv init` (alias: `i`) ensures `~/.ssh`, `~/.ssh/conf.d`, and `~/.ssh/config` are ready for `ssv`-managed hosts.
- Key generation: `ssv generate` (alias: `g`) wraps `ssh-keygen`, writes host-specific configs, and prints the public key so it can be registered immediately.
- Repository relinking: `ssv link <HOST>` (alias: `ln`) rewrites the current repository's `origin` URL to use a managed SSH host.
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
ssv generate github.com -u git

# List all managed hosts
ssv list

# Print a managed host's public key
ssv show github.com

# Rewrite the current repository's origin remote to a managed host
ssv link github.com

# Audit managed SSH assets
ssv audit

# Remove keys/config for github.com
ssv remove github.com
```

All subcommands support short aliases for convenience:

| Subcommand | Alias | Description |
| --- | --- | --- |
| init | i | Bootstrap the SSH configuration layout |
| generate | g | Generate a key pair and host configuration file |
| list | ls | List managed hosts |
| remove | rm | Remove key pairs and configuration for a host |
| show | sw | Print the public key for a managed host |
| link | ln | Rewrite the current repository origin to a managed host |
| audit | au | Inspect managed SSH assets without modifying them |

Configuration files are stored at `~/.ssh/conf.d/<HOST>.conf`, and keys follow the `~/.ssh/id_<TYPE>_<HOST>` naming convention. Optional `-t/--type`, `-u/--user`, `-p/--port`, and `-n/--hostname` flags let you customise the generated configuration.

`list`, `show`, and `audit` are read-only. `audit` writes findings to standard error and exits non-zero when error-level findings exist.

### Managing Multiple Accounts

You can use `ssv` to manage multiple accounts for the same service (e.g., personal and work GitHub accounts) by keeping your main account as the default domain and creating an alias for your secondary account.

```bash
# Personal (Main account): Use the standard domain
ssv generate github.com -u git

# Work (Sub account): Create an alias and specify the HostName
ssv generate github.com-w -n github.com -u git
```

When cloning repositories, use the standard URL for your personal account, and replace the domain with the alias for your work account:

```bash
# Personal
git clone git@github.com:username/repo.git

# Work
git clone git@github.com-w:orgname/repo.git
```

For an existing local checkout, run `ssv link github.com-w` from anywhere inside the repository to rewrite its `origin` remote to the managed host alias.
