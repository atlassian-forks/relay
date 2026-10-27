# Experimental CI checkpoint runner

`relay-compiler-ci` calls native `Compiler::compile()` with WalkDir (full) or
`ExternalFileSource` (incremental). The stock executable, persistence hooks and
Watchman recovery policy are unchanged. No new dependencies are required.

```sh
# From compiler/:
cargo build --locked -p relay --bin relay-compiler-ci

# Full validation creates/replaces a checkpoint; adding a delta enables reuse.
relay-compiler-ci relay.config.json --validate --state-file state.bin
relay-compiler-ci relay.config.json --validate \
  --state-file state.bin --changed-files changes.json
```

Omit `--validate` to generate artifacts. Delta entries require explicit existence:
`[{"name":"src/App.tsx","exists":true}]`. Names are relative to the configured
root; state/delta file arguments are relative to the working directory. Include
all relevant source, schema, resolver and artifact changes, including deletions.

Use trusted state from the same compiler/configuration/source scope, immutable
inputs and a complete delta. Use a private working checkpoint outside source roots.
Replacement is atomic after success, not crash-durable or coordinated across writers.
Git discovery, cache transport/authentication and remote publication belong to AFM.

Native setup failures may fall back to Watchman, even with `FORCE_NO_WATCHMAN=1`.
A successful exit alone does not prove incremental reuse. Omit the delta for a known
cache miss; do not blindly retry validation failures. Persistence may use the network.

Tests: `cargo test --locked -p relay --test saved_state`.

**Draft only:** depends on [compiler fixes #31](https://github.com/atlassian-forks/relay/pull/31). Release
packaging, upstream API agreement and full-AFM Linux verification remain pending.
The earlier AFM prototype uses a different protocol; do not substitute this binary
into its runner. Existing required CI validation remains authoritative.
