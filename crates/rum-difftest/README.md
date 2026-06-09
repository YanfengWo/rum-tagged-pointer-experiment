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
- user-configured Wolfram/Mathematica oracle runners
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
`cargo test -p rum-difftest` includes an artifact hygiene check that rejects
the Lane A/B temporary wrapper scripts, case/step input files, session
directories, and message/print capture files if they appear under this crate.

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
cargo run -p rum-difftest -- lane-a --oracle-command WolframKernel
cargo run -p rum-difftest -- lane-a --oracle-command WolframKernel --oracle-timeout-seconds 60
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --include-known-diffs
```

You can also point it at a config file:

```bash
cargo run -p rum-difftest -- lane-a --config /path/to/rum-difftest.conf
```

Supported config keys today:

```text
oracle_kernel = "/path/to/WolframKernel"
oracle_command = "WolframKernel"
oracle_timeout_seconds = 120
```

The current runner accepts a kernel-compatible executable path or command such
as `WolframKernel`, `MathKernel`, or a wrapper with the same `-noprompt
-script` interface. Oracle resolution prefers the CLI flag, then config, then
`RUM_DIFFTEST_ORACLE_KERNEL`, then `PATH`, then common local install paths. It
writes its oracle wrapper script and per-case input files to a temporary
directory for each run, compares values, messages, and printed output
in-memory, and removes those artifacts afterward. The CLI reports the resolved
oracle executable, the effective per-invocation timeout, the explicit
`user-provided/unpinned` version policy, and best-effort Wolfram `$Version`;
version probe failures are reported but do not fail the diff run, and config
does not support version pin keys. The timeout defaults to 120 seconds and can
be overridden by `--oracle-timeout-seconds`, `oracle_timeout_seconds`, or
`RUM_DIFFTEST_ORACLE_TIMEOUT_SECONDS`, in that precedence order. Timed-out
kernel processes are terminated and reported as oracle failures. Each oracle
stdout/stderr stream is also capped at 8 MiB; a process that exceeds the cap is
terminated while the retained diagnostic prefix is reported. `rum` values
are rendered through the shared difftest InputForm-like renderer, which covers
common list, rule, delayed-rule, and association output shapes before
comparison. It is intentionally not yet a complete Wolfram display
implementation or transport model. Fixture `expected` fields are seed
annotations, not the pass/fail oracle: the runner compares `rum` against the
fresh oracle result on every run and reports seed-vs-oracle drift separately.
When `--include-known-diffs` is used, the report includes known-difference
ID/type/reason metadata and counts confirmed vs resolved known differences.
Resolved known differences are listed by ID and fixture location so stale
imported divergences are visible before curation. Keep `seed/` unchanged;
migrate promoted cases under `cases/` before pruning seed-linked ledger rows.

## Lane B Runner

The crate also includes a `lane-b` runner for ordered stateful session
fixtures:

```bash
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel
cargo run -p rum-difftest -- lane-b --oracle-command WolframKernel
cargo run -p rum-difftest -- lane-b --oracle-command WolframKernel --oracle-timeout-seconds 60
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --include-known-diffs
```

Lane B keeps one `rum` session per fixture file and evaluates the matching
Wolfram steps in one local kernel process for that fixture. This avoids the
older replay-based state simulation and gives definition, option, message,
attribute, and precedence checks a real ordered-session oracle. The runner
compares InputForm-like result values plus per-step message and print streams.
Like Lane A, its CLI reports the resolved oracle executable and best-effort
Wolfram `$Version` without making version probing a pass/fail gate, and applies
the same per-session oracle timeout.
Fixture `expected` fields are treated as seed annotations and are checked only
for seed-vs-oracle drift, not as the comparison oracle.
When `--include-known-diffs` is used, the report includes the known-difference
ID/type/reason for non-matching steps and counts confirmed vs resolved known
differences. Resolved known differences are listed by ID and fixture location
so fixed divergences are visible before removing ledger rows.
Long-lived interactive process reuse remains a future throughput refinement.

## Lane C Runner

Lane C checks the known-difference ledger:

```bash
cargo run -p rum-difftest -- lane-c
cargo run -p rum-difftest -- lane-c --summary-json /tmp/rum-lane-c-summary.json
```

The runner validates that every checked fixture `!!! id=...` is present in
[divergences/known-differences.txt](./divergences/known-differences.txt), and
that the ledger does not contain unreferenced IDs, stale `fixture=...`
metadata, or active fixture/ledger `type=...` mismatches. Resolved imported
seed IDs can move to
[divergences/retired-seed-differences.txt](./divergences/retired-seed-differences.txt)
after promotion into `cases/`, preserving `seed/` as historical input without
keeping resolved seed rows in the active ledger. Ledger rows must also carry
`tracked_by` and `close_when` metadata. Active `tracked_by` values use a
machine-checked `path:needle` shape, keeping deliberate or deferred Wolfram
divergences tied to live roadmap ownership and explicit removal criteria
instead of scattered comments.
`--summary-json PATH` writes a stable machine-readable summary for scripts or
CI dashboards. That JSON intentionally exposes lane status, checked artifact
paths, and issue counts only; it does not freeze the full internal ledger row
shape.

## Lane D Runner

Lane D runs local performance baseline inputs through `rum` without a Wolfram
oracle:

```bash
cargo run -p rum-difftest -- lane-d
cargo run -p rum-difftest -- lane-d --suite memory --limit 5
cargo run -p rum-difftest -- lane-d --suite all --profile full
cargo run -p rum-difftest -- lane-d --baseline crates/rum-difftest/perf/baselines/smoke.txt --fail-on-regression
cargo run -p rum-difftest -- lane-d --write-baseline /tmp/rum-lane-d-smoke.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --capture-runs 5 --write-baseline /tmp/rum-lane-d-smoke.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --threshold-profile ci-debug --capture-runs 5 --write-baseline /tmp/rum-lane-d-ci-debug.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --check-baseline-readiness
```

The default `smoke` profile skips imported heavy / very-large cases so the
runner is safe for routine validation. Use `--profile full` for explicit
baseline collection over every imported performance input. The runner reports
elapsed microseconds, result node count, message count, and print count for
each executed expression. `--baseline PATH` compares the run against a checked
baseline artifact; `--fail-on-regression` turns missing rows, stale rows,
elapsed guardrail failures, and result-node guardrail failures into a non-zero
exit. `--write-baseline PATH` captures the current run as a baseline artifact,
refusing to write if any executed case fails. `--capture-runs N` repeats the
same run before writing and uses per-case median elapsed samples plus the
slowest observed sample for guardrails. `--threshold-profile` selects the
baseline threshold record set and capture defaults: `local-debug` for local dev
runs, `ci-debug` for debug builds on CI machines, and `ci-release` for release
builds on CI/release machines. `--check-baseline-readiness` checks the default
checked smoke baseline artifacts for all threshold profiles and exits non-zero
until the `ci-debug` / `ci-release` artifacts have been captured on the
intended runners. The default expected paths are
`perf/baselines/smoke.txt`, `perf/baselines/smoke-ci-debug.txt`, and
`perf/baselines/smoke-ci-release.txt`.

Start here:

- [HANDOFF.md](./HANDOFF.md)
- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

Layout:

- `seed/`: imported or historical assets kept intact as input material
- `cases/`: `rum`-owned curated fixtures layered on top of the imported seeds
- `divergences/`: checked known-difference ledgers
- `perf/`: Lane D performance baseline policy and checked baseline artifacts

## Lane E/F Boundaries

Lane E and Lane F are intentionally not Wolfram oracle runners. Host-contract
coverage belongs in the host crate, and adapter/protocol coverage belongs in
the adapter crate that owns the transport contract:

```bash
cargo test -p rum-host
cargo test -p rum-jupyter
```

Future adapters should follow the same split. For example, a future Python
host crate should carry its own host-contract tests instead of adding a
Wolfram diff lane for Python binding behavior.

The first curated Lane A files now live under
[cases/wolfram/core](./cases/wolfram/core/README.md). They currently cover:

- arithmetic and structural smoke checks
- loops, control objects, and failure/status flows
- associations and keyed-data basics
- definition precedence promoted from resolved imported seed coverage
- CL-04 functional combinators
- rules, patterns, replacement, and traversal
- CL-07 string surface/pattern/template/similarity subsets

The original imported staging README is preserved in
[README.imported.md](./README.imported.md).
