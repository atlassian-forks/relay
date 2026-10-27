# Experimental CI checkpoint executable

`relay-compiler-ci` is a thin caller of Relay's existing `Compiler::compile()` and
`FileSourceKind::External` machinery. The stock `relay` executable, external reader,
and Watchman recovery policy are unchanged. There is no second implementation of
compilation, dependency analysis, persistence, or artifact generation.

This is a draft integration, not a production release or an upstream-agreed CLI.

## Build and use

From `compiler/`:

```sh
cargo build --locked -p relay --bin relay-compiler-ci
```

```sh
# Full validation creates/replaces a working checkpoint (WalkDir, not Watchman).
relay-compiler-ci relay.config.json --validate --state-file state.bin

# Native incremental validation refreshes the same checkpoint after success.
relay-compiler-ci relay.config.json --validate \
  --state-file state.bin --changed-files changes.json
```

Omit `--validate` to generate artifacts. Neither watch mode nor project filtering
is supported by this entry point. `--state-file` without a delta always performs a
full compilation, even when a checkpoint already exists. Without `--state-file`,
the runner performs an ordinary full compilation/validation without saving state.

The executable is Cargo-discovered in the existing package; it adds no dependencies.
Release artifact collection/npm packaging **has not been wired**. Normal stock
release behavior remains unchanged. Do not substitute this executable into AFM's
old standalone benchmark/shadow runner: that prototype uses a different protocol.
No frontend developer command or required CI check changes with this draft.

## Caller contract

The delta is a complete JSON array relative to the **configured Relay root**:

```json
[
  {"name": "src/App.tsx", "exists": true},
  {"name": "src/Removed.tsx", "exists": false},
  {"name": "src/__generated__/AppQuery.graphql.ts", "exists": false}
]
```

- `exists` is required. The caller computes it against the configured root. This
  avoids relying on the native optional-existence lookup's working directory.
- The runner rejects empty, absolute, traversal, backslash and colon-containing
  paths. It checks delta syntax before invoking the compiler, without changing
  shared external-reader behavior.
- Include every source, schema, resolver/extension and artifact change since the
  checkpoint. Represent renames as deletion plus addition. `[]` means no changes.
- State/delta **file arguments** are relative to the process working directory.
- Inputs must remain immutable, including between the caller's checks and native
  compilation. This runner does not verify checkout cleanliness or delta completeness.
- State is trusted, compiler-specific data. Use the exact compiler build, compatible
  evaluated configuration and source/sparse-checkout scope; do not bypass native
  saved-state version checking in CI.

## Checkpoint publication and failures

The runner writes a temporary sibling checkpoint, then atomically renames it after
successful compilation/validation and serialization. Failures preserve the prior
working checkpoint. Atomic replacement does not provide crash durability or writer
coordination: use a private working checkpoint per invocation, outside source roots.
Cache transport, authentication, revision tracking and remote publication belong to
AFM and are not implemented here.

Upstream's external-state setup fallback is unchanged. Missing/corrupt/incompatible
state or other native setup errors can attempt a full Watchman query. That recovery
path does not consult `FORCE_NO_WATCHMAN`; a successful exit alone does not establish
an incremental hit. If the caller knows no snapshot is available, omit the delta
for an explicit full WalkDir run. A strict no-Watchman-on-error API remains an
upstream integration question, not a new global compiler policy.

Actual validation failures are not retried by this runner. It uses the stock
Atlassian persister and extra-artifact hooks; there is no new guarantee of zero
persisted-query network requests.

## Independently testable library fixes

The remaining library changes are confined to `compiler_state.rs`:

1. **Filtered owned artifacts:** when merging an external delta after restore,
   source filters can hide colocated `__generated__` outputs. Re-admit only filtered
   paths owned by the snapshot for enabled/base projects into the existing Generated
   processing. Do not scan/reparse a replacement dependency graph, broaden source
   discovery, or treat another tool's generated output as Relay-owned.
2. **Serialization errors:** explicitly finish the buffered writer and zstd encoder
   so write failures are observable before the caller publishes a checkpoint.

The artifact integration regression was observed failing before the first fix.
A native library test covers edits/deletions, disabled projects, excluded sources,
unknown outputs, and the collect-changed-artifacts switch independently of this bin.
The serialization write-failure regression uses `/dev/full` on Linux.

These fixes still require upstream review; they are not claims of an upstream
supported production embedding API. There are no changes to `compiler.rs`, stock
`main.rs`, `file_source.rs`, or `external_file_source.rs` in the net PR diff.

## Tests and rollout boundary

```sh
cargo test --locked -p relay --test saved_state
cargo test --locked -p relay-compiler --lib
```

Integration tests invoke the actual executables on disposable projects. They cover
checkpoint reuse, semantic failure preservation, artifact-only edits/deletions,
transitive fragment invalidation with only source/own-artifact inputs changed,
foreign generated outputs, caller input checks, upstream recovery, and stock CLI
isolation. Tests remove Watchman from PATH so recovery cannot use a developer daemon.

Current verification is local macOS correctness testing. Linux write-failure tests,
full-AFM/persisted-mock parity, performance, release packaging and authenticated cache
integration remain follow-ups. Historical standalone Linux benchmarks do **not**
verify this thinner executable. Keep existing required validation authoritative.
