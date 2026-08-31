# ssv managed-state refactor

## Objective

The refactor preserves healthy-state CLI behavior while making managed SSH state explicit, safe to
mutate, deterministic to inspect, and independently testable. Invalid or unsafe state that was
previously accepted becomes an explicit error or audit finding.

Completion requires:

- One managed-host contract shared by generation, loading, mutation, listing, linking, removal,
  authorization, display, and audit.
- Restrictive, atomic SSH configuration writes with no observable partial content.
- Retryable generation and removal after every returned failure, with cleanup failures reported.
- One managed inventory boundary enforcing root, regular-file, and no-symlink invariants.
- Complete audit findings in an exact deterministic order.
- Application operations depending only on the runtime resources they use.
- Test fixtures derived independently from production renderers outside lifecycle tests.
- Documentation, Agent Skills, Cargo metadata, development commands, and CI describing the same
  implemented behavior and toolchain.
- `just fix`, `just check`, and `just test` passing in that order.

## Baseline

The implementation starts from commit `5df7ab1`. The Rust source and tests are unchanged from
`330410c` (`v0.6.0`); the later commits add the Japanese README, plugin manifests, and the
`github-ssh-setup` and `ip-ssh-setup` Agent Skills.

The current baseline passes 14 unit tests and 55 integration tests with `just test`. The binary is
Unix-only and stores state in ordinary files under a selected home directory:

- `~/.ssh/config`
- `~/.ssh/conf.d/<HOST>.conf`
- `~/.ssh/id_<TYPE>_<HOST>`
- `~/.ssh/id_<TYPE>_<HOST>.pub`

Only `ssv authorize` performs a remote operation in the binary, through `ssh-copy-id`. Plugin
workflows may separately invoke `ssh` for connection verification.

## Retained product contracts

- Command names, aliases, default key type, healthy-state output streams, warning exit behavior,
  and managed filenames remain stable.
- `generate` performs bootstrap and never replaces an existing final artifact.
- `list`, `show`, and `audit` remain read-only.
- `set` retains the key pair and renders only the canonical managed directives.
- Managed operations never read, replace, or remove an identity outside the SSH root.
- Symlinked roots, configuration files, keys, and path components are rejected rather than followed.
- Removal derives targets only from a validated config-to-host relationship; filename guessing is
  not recovery behavior.
- Subprocess input is passed as individual arguments without a shell command string.
- Key algorithms remain accepted dynamically by `ssh-keygen`; the repository does not own an
  algorithm catalogue.

## Failure model

The current refactor guarantees recovery from errors returned to the caller. Multi-file crash
recovery, locking, and protection against an adversarial same-user process swapping paths between
checks and mutations remain outside this change.

For an atomic file operation, publication of the final pathname is the commit point. A pre-commit
failure leaves the old target or absence unchanged. A failure after publication, including a parent
directory sync failure, reports that publication occurred and never triggers cleanup that would
invalidate the published document's dependencies.

## Stage 1: Independent fixtures and managed-host contract

This stage implements TD-01 and the test-fixture portion of TD-07.

### Types and ownership

- A validated host identifier owns the filename and SSH alias grammar: non-empty ASCII letters,
  digits, `.`, `-`, and `_`, without a leading `-`.
- A managed key name validates its key type and host identifier at construction and rejects reserved
  OpenSSH filename collisions.
- Hostnames use the existing ASCII address character set, reject a leading `-`, and reject empty,
  whitespace, control, and directive-injection values.
- Remote users reject empty values, whitespace, control characters, a leading `-`, and `@`.
- A loaded managed-host record owns the host identifier, config path, required hostname, optional
  user and port, private key path, and public key path.

### Managed document syntax

Directive names are case-insensitive. Empty lines and lines whose first non-whitespace character is
`#` are ignored. A directive uses either one or more whitespace characters or optional whitespace
around exactly one `=` as its name/value separator. A scalar value may be enclosed in one pair of
double quotes. The parser does not claim support for additional OpenSSH quoting or escaping forms.

A valid managed document has:

- Exactly one `Host` block with one literal alias exactly matching the filename host.
- Exactly one `HostName`.
- Zero or one `User`.
- Zero or one `Port`.
- Exactly one `IdentityFile` naming the managed key for the filename host.
- Exactly one `IdentitiesOnly yes`.
- No `Match` block or second `Host` block.

Unrecognized directives inside the single block remain readable. `set` discards them when it renders
the canonical managed document. Top-level include recognition belongs to bootstrap configuration
logic, not to the managed-host record.

The normal load boundary returns only a fully validated record. Audit uses the same parser and
converts a per-file contract failure to a finding.

### Verification

- Handwritten fixtures cover missing, duplicate, mismatched, and malformed required fields; duplicate
  optional fields; invalid loaded values; second blocks; comments; quotes; whitespace; and `=` syntax.
- `show`, `authorize`, `set`, `link`, and `remove` reject the same invalid record before external
  effects.
- Audit reports the shared contract failure.
- The fixture harness can write a valid managed document and matching stub key pair without invoking
  `Context::generate` or the production renderer.

## Stage 2: Atomic persistence and recoverable lifecycle

This stage implements TD-02 and TD-03.

### Atomic files and directories

A narrowly scoped same-directory persistence boundary provides atomic no-clobber creation and atomic
replacement. It owns unpredictable temporary names, `create_new`, restrictive mode at file creation,
complete writes, file sync, publication, parent-directory sync, and temporary cleanup.

- New SSH files have mode `0600` before content is written.
- New SSH directories have mode `0700` at creation and are verified afterward.
- Replacement refuses an existing symlink or non-regular target.
- No-clobber creation fails if any final target already exists.
- Every temporary cleanup error is combined with the primary error and identifies its path.
- Bootstrap insertion is idempotent and precedes the first `Host` or `Match`, including block
  directives written with `=`.
- No generic filesystem abstraction is introduced.

### Generation

- `ssh-keygen` writes only to operation-owned same-directory staging paths.
- A nonzero generator status triggers cleanup of both possible staged key paths.
- The staged public key exists and its algorithm/key fields match `ssh-keygen -y` output before
  publication.
- Private key, public key, and config publication are no-clobber.
- The keys become final before the config; the config is the multi-file commit marker.
- Before config publication, rollback removes only paths successfully created by the operation.
- After config publication, no failure removes the published keys.
- Primary, rollback, and cleanup failures remain observable together.

### Removal

- Config, private key, and public key are validated and classified before mutation. A directory,
  symlink violation, outside-root path, or other preflight failure causes no deletion.
- Keys are removed before the config. The config remains the retry metadata until both key removals
  have succeeded or been classified as already absent.
- A key-removal error leaves the config in place; a later `ssv remove <HOST>` retries safely.
- A final config-removal error also leaves a retryable config whose absent keys use the existing
  partial-status behavior.

### Verification

- Atomic create/replace tests cover exact content, modes under a permissive umask, symlink and
  non-file refusal, pre-commit write failure, and temporary cleanup.
- Generator stubs that write one or both keys and exit nonzero leave no final assets and permit retry.
- Existing or concurrently introduced final paths are never replaced or removed.
- A config publication failure rolls back operation-owned published keys.
- Removal preflight failure produces zero deletions; an injected key-removal failure leaves the config
  and permits a second successful removal.

## Stage 3: Shared inventory and deterministic audit

This stage implements TD-04.

- One inventory boundary validates `~/.ssh` and `conf.d` before traversal and never traverses a
  rejected root.
- Inventory entries are sorted by raw Unix path bytes.
- Unrelated non-`.conf` files remain outside the managed-host inventory.
- `list` returns names from fully validated managed-host records. Invalid `.conf` hostnames, file
  types, symlinks, or documents cause an explicit error.
- Audit uses the same candidate classification, converts per-entry failures to findings, and
  continues with independent safe entries.
- A missing, unreadable, symlinked, or invalid root becomes a finding and prevents only the unsafe
  traversal. Layout construction errors occur before report collection.
- Failure to determine the home owner becomes a finding and never disables checks silently.
- Managed private and public filenames are paired by the naming authority. Lone or unreferenced
  private and public files are reported without claiming standard OpenSSH keys.
- Failure to derive a public key is distinct from a derived/public key mismatch.
- Final findings are sorted by raw path bytes, then severity with errors first, then audit-code text,
  then message bytes.

Verification covers symlinked roots, `.conf` directories and symlinks, invalid candidates, lone
public keys, one-sided key pairs, owner lookup failure, derive failure diagnostics, and identical
finding order across repeated runs.

## Stage 4: Dependency direction and errors

This stage implements TD-05 and TD-06.

- `Context` remains the public facade and owns resolved runtime resources.
- Each app operation accepts only the layout, executable path, repository start path, and command
  values it uses. No module under `src/app/` imports `Context`.
- The CLI resolves the current directory with a path-aware error and supplies it to linking.
- The public link operation becomes `link(repository_start, host)`; the hidden-current-directory
  signature is removed without a compatibility duplicate.
- Pass-through modules without command-specific orchestration or presentation ownership are removed.
- Public errors distinguish validation, managed documents, Git operations, path-aware I/O, external
  commands, invalid external output, pre-commit cleanup failure, and committed-but-not-confirmed
  durability failure.
- Captured subprocess failures retain operation, executable path, status, and safely decoded stderr.
  Arguments, private material, and key contents are not included.
- `ssh-copy-id` safety is portable and validation-based: option-like leading values and multi-`@`
  targets fail before process creation; no end-of-options behavior is assumed.
- CLI errors retain the `Error: ...` prefix and nonzero status.

Verification includes source residue searches, an explicit-path library link test, distinguishable
Git failures, nonzero generate/derive/copy-id stubs, captured derive stderr, and hostile values that
never start `ssh-copy-id`.

## Stage 5: Documentation and toolchain consistency

This stage completes TD-07 and implements TD-08.

- `AGENTS.md` states that child-process environments and explicit contexts isolate tests and that no
  serialization dependency is required.
- README changes remain Japanese, the two plugin skills remain Japanese, and
  `docs/remote-login.md` remains English.
- README, remote-login documentation, and the plugin skills describe corrected invalid-state and
  recovery behavior. Proposal-only link semantics remain unchanged.
- Cargo package metadata declares Rust `1.90.0`.
- `rust-toolchain.toml` is the CI installation authority; workflows do not override it independently.
- The exact mise runtime used by CI remains an explicit setup-action input. `mise.toml` owns its
  minimum version, while `mise.lock` owns installed project-tool versions.
- `actionlint` is added to the locked mise tools and `just check` provides the reproducible workflow
  syntax verification command.
- Every third-party `uses:` reference is a reviewed immutable full commit SHA with a concise version
  comment. Local actions remain local, and `akitorahayashi/*` actions retain reviewed release or
  major tags.

Verification searches `.github` for mutable third-party actions, confirms the owner exception, runs
workflow syntax validation, and confirms every CI job uses the repository toolchain.

## Implementation sequence and commits

1. This corrected brief is committed before source changes.
2. Independent fixtures, failing contract tests, and the managed-host schema form one completed
   stage and commit.
3. Atomic persistence plus recoverable generation and removal form one completed stage and commit.
4. Shared inventory and deterministic audit form one completed stage and commit.
5. Dependency direction and error/subprocess corrections form one completed stage and commit.
6. Documentation, Agent Skills, toolchain metadata, and CI form one completed stage and commit.
7. Final formatting, checks, tests, and comprehensive residue searches amend only defects found by
   verification; the final verified state receives a final commit when changes are required.

Renamed or removed structures receive comprehensive residue searches. Compatibility shims,
deprecated duplicates, old parsers, and stale terminology remain only when they are explicit public
contracts.

## Out of scope

- Product changes to Git remote URL parsing or SSH-user precedence.
- Descriptor-relative filesystem APIs, locking, crash journals, and same-user adversarial race
  protection.
- Release checksum, provenance, and tag/version enforcement changes.
- CodeRabbit or Gemini configuration reduction.
- Passphrases, ssh-agent integration, remote revocation, server provisioning, Windows support, JSON
  output, interactive configuration, logging frameworks, or audit subprocess optimization.

## Handoff

The implementation report states completed behavior, ownership changes, stage commits, verification
commands and results, unmet criteria with concrete blockers, and the unchanged out-of-scope items.
An inventory-only status or diff is not verification evidence.
