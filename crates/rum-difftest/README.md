# rum-difftest

`rum-difftest` is the home for `rum`'s differential and conformance-testing
workstream.

It owns:

- imported seed corpora
- future `rum`-owned curated fixture trees
- fixture parsers and normalization helpers
- future user-configured Wolfram/Mathematica oracle runners
- divergence ledgers
- performance baseline inputs

This crate is intentionally not part of the evaluator dependency chain. Its job
is to keep conformance work organized as a first-class lane without forcing all
verification concerns into `rum-runtime`'s local regression suites.

This crate is also intentionally optional. `rum` does not bundle a Wolfram
license or ship project-owned oracle binaries. Differential runs are meant to
use a contributor's own local Wolfram environment when they have one.

That is a feature, not just a constraint. Users may compare `rum` against an
older or otherwise preexisting Mathematica/Wolfram installation, so a diff run
can act as both a `rum` correctness check and a compatibility check against the
user's real environment.

Oracle-generated outputs are not repository assets. The runner should generate
fresh comparison material for each run and keep those artifacts out of the
checked-in tree.

This crate is also deliberately not the place for every `rum` test. If a
feature is `rum`-specific rather than Wolfram-oracle-comparable, its primary
tests should stay in the crate that owns that contract. That includes:

- `rum`-native runtime/session features
- structured diagnostics and suspension/resume behavior
- host-facing Rust/Python embedding APIs
- CLI/REPL/Jupyter and other adapter/protocol behavior

## First-Pass Lane A Runner

The crate now includes a first-pass `lane-a` runner:

```bash
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel
```

You can also point it at a config file:

```bash
cargo run -p rum-difftest -- lane-a --config /path/to/rum-difftest.conf
```

Supported config keys today:

```text
oracle_kernel = "/path/to/WolframKernel"
```

The current runner accepts a kernel executable path such as `WolframKernel` or
`MathKernel`. It writes its oracle wrapper script and per-case input files to a
temporary directory for each run, compares results in-memory, and removes those
artifacts afterward. It is intentionally a first pass: enough to validate the
oracle workflow, not yet the final word on message capture, top-level display
parity, or every Wolfram transport nuance.

Start here:

- [HANDOFF.md](./HANDOFF.md)
- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

Layout:

- `seed/`: imported or historical assets kept intact as input material
- `cases/`: future `rum`-owned curated fixtures
- `divergences/`: future checked known-difference ledgers
- `perf/`: future performance baseline material

The original imported staging README is preserved in
[README.imported.md](./README.imported.md).
