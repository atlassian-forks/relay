# One-shot compilation with saved state

The existing `relay` executable can reuse native compiler state with a caller-supplied
file delta. This exposes the existing `FileSourceKind::External` implementation;
there is no additional executable, npm package, or build pipeline. Successful external
state loading does not require Watchman; upstream's recovery path still does.

## CLI

```sh
# Full validation and initial state creation.
FORCE_NO_WATCHMAN=1 relay relay.config.json --validate --state-file state.bin

# Incremental validation and refreshed state, using the same executable/config.
relay relay.config.json --validate \
  --state-file state.bin \
  --changed-files changes.json
```

Omit `--validate` for normal artifact generation.

- `--state-file` alone performs a **full compilation**, without loading any existing
  checkpoint, and creates or replaces that checkpoint after success.
- Adding `--changed-files` loads the checkpoint and applies the complete delta. This
  needs an existing, compatible checkpoint for an incremental run. Upstream's full
  Watchman fallback remains unchanged if external-state setup fails.
- Successful runs serialize to a temporary sibling directory, finish all buffered
  writes, and atomically rename the completed file over the checkpoint. Validation,
  serialization, or replacement failures leave the previous checkpoint unchanged.
  State write failures return a failing exit status. Atomic replacement does not
  provide crash durability or coordination between concurrent writers.

`--changed-files` requires `--state-file`. Both paths are relative to the process
working directory (unlike names inside the delta). Use a private working checkpoint
per invocation; download/copy the original into it and publish the updated checkpoint
only after success. Keep it outside Relay's source directories. To retain an original
snapshot for repeated benchmarks, restore it before each run.

The state flags cannot be combined with `--watch`, `--project`, or `--repersist`. State should cover
the whole configured project set; project filtering and repersistence are not part
of this incremental CLI contract.

## Changed files

The input is a JSON array, not the entire Watchman protocol response:

```json
[
  {"name": "src/App.tsx", "exists": true},
  {"name": "src/Removed.tsx", "exists": false},
  {"name": "src/__generated__/AppQuery.graphql.ts", "exists": false}
]
```

- `name` is relative to the **configured Relay root**, not the process working directory.
  Absolute, drive-prefixed, parent-traversal, and empty paths are rejected by the
  external reader (which triggers upstream's recovery path).
- If `exists` is omitted, the compiler checks the path under that root.
- Include **every** relevant change since the saved state: sources, schema,
  extensions/resolvers, and generated artifacts, including edits and deletions.
  Represent renames as deletion plus addition. Supply `[]` only when nothing changed.
- Known saved-state artifacts are invalidated even under default `__generated__`
  directories, which the non-Watchman source filter ordinarily excludes.
- Inputs must remain stable while compilation runs. The CLI does not compute Git
  deltas, verify checkout cleanliness, or discover missing entries in the delta.

## Compatibility and failures

Saved states are trusted, compiler-specific binary data, not a portable or untrusted
interchange format. Use the same compiler build, compatible evaluated configuration,
project selection, and source/sparse-checkout scope. The native saved-state version
check remains enabled by default; do not bypass it in CI. Callers must separately
key snapshots by compiler build and checkout scope.

Upstream's `FileSource::External` recovery policy is unchanged: failures while
creating state from external inputs (including missing/invalid state, version
mismatches or invalid deltas) log a warning and attempt a full Watchman query.
This fallback can succeed if Watchman is available. It does not consult
`FORCE_NO_WATCHMAN`, which only controls normal CLI file-source selection. Therefore,
a successful command is not by itself evidence of an incremental hit.

If a caller knows its snapshot is unavailable, it can deliberately request full
compilation by omitting `--changed-files` (with `FORCE_NO_WATCHMAN=1` to use WalkDir).
A strict no-Watchman/error-on-invalid-input CI policy is not implemented by these
flags. Resolve that integration contract before rollout; do not change the shared
fallback policy for other callers. **Do not blindly retry nonzero validation exits**:
failed compilation/validation does not write output state.

Ordinary invocations without these flags keep their current Watchman/WalkDir selection,
artifact generation, validation, and persistence behavior. In particular, this change
adds no new persisted-query registration policy: `--validate` retains its existing
persister behavior and is not a guarantee of zero network requests.

This is compiler plumbing, not an AFM cache rollout. Authenticated snapshot transport,
revision bookkeeping, cache retention, stable configuration evaluation, full/incremental
parity for product-specific artifacts and deletions, and shadow/rollback policy remain
caller responsibilities. Do not replace authoritative CI validation until those are
verified on real checkouts.

## Tests

From `compiler/`:

```sh
cargo test --locked -p relay --test saved_state
```

The normal workspace test job discovers these integration tests. They invoke the actual
`relay` executable on disposable projects, without Watchman or remote services.
