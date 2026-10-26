# One-shot compilation with saved state

The existing `relay` executable can reuse native compiler state with a caller-supplied
file delta. This exposes the existing `FileSourceKind::External` implementation;
there is no additional executable, npm package, build pipeline, or Watchman service.

## CLI

```sh
# Full validation and initial state creation.
FORCE_NO_WATCHMAN=1 relay relay.config.json --validate --save-state state.bin

# Incremental validation and refreshed state, using the same executable/config.
relay relay.config.json --validate \
  --load-saved-state state.bin \
  --changed-files changes.json \
  --save-state next-state.bin
```

Omit `--validate` for normal artifact generation. `--save-state` is optional, and is
executed only after compilation/validation succeeds. A state write error is reported
as a command failure. Prefer a distinct output path; the caller is responsible for
atomically publishing it after the command succeeds.

`--load-saved-state` and `--changed-files` must be supplied together. The state flags
cannot be combined with `--watch`, `--project`, or `--repersist`. State should cover
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
  Absolute, drive-prefixed, parent-traversal, and empty paths are rejected.
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

Invalid/missing state, version mismatches, or invalid changed-files input fail the
command. External mode no longer silently attempts a full Watchman query. The caller
can deliberately choose a full run when its snapshot is unavailable/incompatible;
**do not blindly retry every nonzero validation exit**, since artifact/GraphQL errors
must remain failures. Failed compilation/validation does not write output state.

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
