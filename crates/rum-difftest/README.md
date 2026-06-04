# rum-difftest

`rum-difftest` is the home for `rum`'s differential and conformance-testing
workstream.

The project-wide testing policy lives in
[docs/testing-strategy.md](../../docs/testing-strategy.md). This crate owns the
Wolfram-oracle-comparable lanes from that policy; crate-local regression,
host-contract, and adapter/protocol tests should stay in their owning crates.

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

## Lane B Runner

The crate also includes a `lane-b` runner for ordered stateful session
fixtures:

```bash
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel
```

Lane B keeps one `rum` session per fixture file and evaluates the matching
Wolfram steps in one local kernel process for that fixture. This avoids the
older replay-based state simulation and gives definition, option, message,
attribute, and precedence checks a real ordered-session oracle. The runner
compares result values plus per-step message and print streams. Long-lived
interactive process reuse remains a future throughput refinement.

## Lane C Runner

Lane C checks the known-difference ledger:

```bash
cargo run -p rum-difftest -- lane-c
```

The runner validates that every checked fixture `!!! id=...` is present in
[divergences/known-differences.txt](./divergences/known-differences.txt), and
that the ledger does not contain unreferenced IDs. This keeps deliberate or
deferred Wolfram divergences visible as data instead of scattered comments.

## Lane D Runner

Lane D runs local performance baseline inputs through `rum` without a Wolfram
oracle:

```bash
cargo run -p rum-difftest -- lane-d
cargo run -p rum-difftest -- lane-d --suite memory --limit 5
cargo run -p rum-difftest -- lane-d --suite all --profile full
```

The default `smoke` profile skips imported heavy / very-large cases so the
runner is safe for routine validation. Use `--profile full` for explicit
baseline collection over every imported performance input. The runner reports
elapsed microseconds, result node count, message count, and print count for
each executed expression. It is a baseline input runner, not yet a
historical-threshold or regression-decision system.

Start here:

- [HANDOFF.md](./HANDOFF.md)
- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

Layout:

- `seed/`: imported or historical assets kept intact as input material
- `cases/`: `rum`-owned curated fixtures layered on top of the imported seeds
- `divergences/`: checked known-difference ledgers
- `perf/`: Lane D performance baseline policy and follow-up notes

The first curated Lane A files now live under
[cases/wolfram/core](./cases/wolfram/core/README.md). They currently cover:

- arithmetic and structural smoke checks
- loops, control objects, and failure/status flows
- associations and keyed-data basics
- CL-04 functional combinators
- rules, patterns, replacement, and traversal
- CL-07 string surface/pattern/template/similarity subsets

The original imported staging README is preserved in
[README.imported.md](./README.imported.md).
