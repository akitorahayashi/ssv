# Remote machine login with ssv

This guide covers the end-to-end lifecycle of logging in from a client machine to a
remote machine over SSH using `ssv`. Each step names the machine it runs on.

## Roles

- Client: the machine you connect from. `ssv` runs here and owns the key pair and host config.
- Server: the machine you connect to. It only needs an SSH server and the client's public key.

## Server preparation (on the server)

Enable the SSH server so it accepts connections on port 22:

```bash
sudo systemsetup -setremotelogin on   # macOS; or enable "Remote Login" in Settings > General > Sharing
whoami                                 # note the login user; it is the -u value used on the client
```

Without a running SSH server, connection attempts fail with `Connection refused` before any
key or password is checked.

## Client setup (on the client)

```bash
ssv init                               # once per machine; prepares ~/.ssh, ~/.ssh/conf.d, ~/.ssh/config
ssv generate <HOST> -n <ADDRESS> -u <SERVER_USER>
ssv authorize <HOST>                   # installs the public key on the server (prompts for the server password once)
ssh <HOST>                             # key-based login; no password
```

- `<HOST>` is the alias written to `~/.ssh/conf.d/<HOST>.conf` and used by `ssh <HOST>`.
- `-n <ADDRESS>` sets the `HostName` actually dialed. It is the server's reachable address
  (a `.local` mDNS name on the same LAN, a Tailscale address, or a raw IP), not the alias.
- `-u <SERVER_USER>` is the login user on the server, from `whoami` above.

`ssv authorize` reads the address, user, and port back from the managed config and runs
`ssh-copy-id` with the generated public key, so none of those values are retyped. It replaces
the manual `ssh-copy-id -i ~/.ssh/id_<TYPE>_<HOST>.pub <user>@<address>` step.

## Updating the address (on the client)

When the server's address changes (for example a reassigned IP), update the config in place
instead of regenerating keys:

```bash
ssv set <HOST> -n <NEW_ADDRESS>        # also accepts -u and -p
```

`ssv set` re-renders the host config, keeping the existing key pair and any directives that are
not overridden. `ssv audit` verifies the managed assets afterward.

## Tailscale

Tailscale assigns each device a stable `100.x.x.x` address and, with MagicDNS enabled, a name
that resolves across the tailnet. Two options:

- Use the Tailscale address as `HostName`: `ssv generate <HOST> -n <100.x.x.x | magicdns-name> -u <SERVER_USER>`.
- Rely on MagicDNS search domains: name the managed host after the Tailscale device and omit
  `-n`, so `HostName` equals the device name and MagicDNS resolves it.

`.local` (mDNS) resolution only works on the same LAN; Tailscale addresses work from anywhere on
the tailnet. When the Tailscale address changes (device re-registration, OS reinstall), use
`ssv set <HOST> -n <NEW_ADDRESS>`.

## Connecting from a phone

`ssv` is a desktop CLI and does not run on phones. To log in from a phone SSH app:

1. Generate a key pair inside the phone SSH app (keep the private key on the phone).
2. Append the phone's public key to the server's `~/.ssh/authorized_keys`. From an already
   authorized client: `ssh <HOST> "cat >> ~/.ssh/authorized_keys"` and paste the key.
3. In the phone app, set the host address (Tailscale address off-LAN, `.local`/IP on-LAN),
   the server user, and the generated private key.
