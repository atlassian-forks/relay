# Experimental CI checkpoint runner

`relay-compiler-ci` is an additive caller of native `Compiler::compile()`.
**The compiler library, stock CLI, serialization and recovery behavior are unchanged.**

```sh
# From compiler/:
cargo build --locked -p relay --bin relay-compiler-ci

relay-compiler-ci relay.config.json --validate --state-file state.bin
relay-compiler-ci relay.config.json --validate \
  --state-file state.bin --changed-files changes.json
```

Omit `--validate` to generate artifacts. Delta entries require explicit existence:
`[{"name":"src/App.tsx","exists":true}]`. Names are relative to the configured
root; state/delta file arguments are relative to the working directory. Include
all relevant source, schema, resolver and artifact changes, including deletions.
Use normalized root-relative names (no leading `./`).

## Conservative selection

- No delta: full WalkDir compilation.
- Any change under `__generated__` or configured output/extra-artifact directories:
  full WalkDir compilation, without loading the checkpoint. This avoids native
  external-delta filtering gaps without changing artifact ownership/invalidation.
- Non-normalized or absolute output roots also force full compilation.
- Otherwise: native external-state compilation, including empty deltas.

Logs report the selected path. Native external setup errors may still fall back to
Watchman, even with `FORCE_NO_WATCHMAN=1`; selection is not proof of an incremental hit.
Ordinary PRs include regenerated artifacts, so many will take the full path. Previous
prototype speedups do not apply; representative AFM performance remains unmeasured.

Use trusted compatible state, immutable inputs, a complete delta and a private
working checkpoint outside source roots. Git discovery and authenticated caching
belong to AFM. Do not blindly retry validation failures. Persistence may use the network.

Checkpoint rename is atomic, **not verified or crash-durable**: unchanged upstream
serialization can discard buffered-write errors, so a successful run can replace the
checkpoint with an incomplete file. Treat it as disposable cache data, not evidence
of successful state publication. This runner does not implement cache recovery.

Tests: `cargo test --locked -p relay --test saved_state`.

**Draft only:** release packaging and full-AFM Linux correctness/performance checks
remain pending. The old AFM prototype uses a different protocol. Existing required
CI validation stays authoritative. No library-fix prerequisite is needed.
