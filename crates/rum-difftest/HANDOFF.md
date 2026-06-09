# Rum Difftest Handoff

This file is meant to be enough for a fresh agent to restart `rum-difftest`
work without reading older chat history.

## Goal

Build a `rum-difftest` workstream that treats same-name Wolfram features as
compatibility targets by default, while keeping `rum`'s own host/runtime/API
contracts separate from Wolfram-specific transport and embedding behavior.

The oracle side of this workstream is optional and user-owned. `rum` should
not assume a bundled Wolfram license, a project-managed oracle install, or a
single pinned Wolfram version. Instead, when a contributor has access to a
usable local Mathematica/Wolfram environment, `rum-difftest` should help them
drive that environment as an oracle.

## Core Principle

`rum-difftest` should not be one monolithic test bucket. It should be split
into separate lanes with different oracles.

`rum-difftest inventory` is the non-oracle coverage-audit command for default
Lane A/B fixtures. It reports fixture counts, source groups, semantic
categories, normal entries, and known-difference entries, and supports
`--summary-json PATH` for scripts or dashboards.
`--check-baseline --fail-on-regression` compares against
`coverage/inventory-baseline.txt`; baseline rows can target `source=...` or
`category=...`, `min_*` fields are lower bounds, and
`max_known_difference_entries` is an upper bound. The coverage-goal policy is
documented in `coverage/README.md`. `--write-baseline PATH` captures the
current inventory in the same stable baseline format.
`rum-difftest check` is the fast non-oracle aggregate gate over that checked
inventory baseline, Lane C ledger validation, and oracle-artifact hygiene. It
exits non-zero if any default gate is dirty and can write `--summary-json PATH`
for scripts without starting a Wolfram oracle. It intentionally skips Lane D
timing inputs unless `--include-lane-d-smoke` is passed;
`--lane-d-smoke-baseline PATH` runs the same optional gate against an alternate
smoke baseline artifact.
Use `--include-lane-d-readiness` when the aggregate gate should also check the
checked Lane D runner-artifact set.

Every `--summary-json PATH` output includes a top-level `schema_version`; the
current value is `1`. Increment it for intentionally incompatible report-shape
changes. CLI regression tests should parse generated summary files as JSON and
assert the schema version, not only match string fragments.

For the Wolfram-backed lanes, keep these policy constraints explicit:

- the oracle command should be user-configured and may point to
  `WolframKernel`, `MathKernel`, or a thin kernel-compatible wrapper script
- the license belongs to the user running the diff, not to the project
- generated oracle outputs are transient artifacts and should not be committed
- `cargo test -p rum-difftest` rejects committed Lane A/B oracle wrapper,
  case/step, session, message, and print-capture artifacts
- every diff run should regenerate fresh comparison data
- version variance is acceptable and useful because it also probes
  compatibility with users' existing Wolfram environments
- Lane A/B should report the `user-provided/unpinned` oracle-version policy
  and should not accept config keys that pin one required oracle version
- every oracle invocation should have a finite timeout, with timed-out kernel
  processes terminated rather than leaving the diff run hung indefinitely
- oracle stdout/stderr capture should stay bounded so a noisy or broken wrapper
  cannot exhaust agent or CI memory before the timeout fires

And keep the exclusion boundary explicit too:

- `rum-difftest` is for oracle-comparable same-name semantics
- it is not the main home for `rum`-specific runtime/session features
- it is not the main home for suspend/resume, structured diagnostics, host
  APIs, or adapter/protocol tests
- those tests should stay in the crate-local suites that own those contracts

Current implementation status:

- a first-pass `lane-a` runner now exists
- it accepts `--oracle-kernel PATH`, `--oracle-command PATH`,
  `--oracle-timeout-seconds N`, or `--config PATH`
- config currently supports `oracle_kernel = "/path/to/WolframKernel"` and
  `oracle_command = "WolframKernel"`, plus
  `oracle_timeout_seconds = 120`
- oracle resolution also checks `RUM_DIFFTEST_ORACLE_KERNEL`, `PATH`, and
  common local Wolfram install paths
- oracle timeouts default to 120 seconds and resolve from CLI, config,
  `RUM_DIFFTEST_ORACLE_TIMEOUT_SECONDS`, then the default; Lane A cases, Lane B
  fixture sessions, and `$Version` probes use the same timeout executor
- each oracle stdout/stderr stream has a fixed 8 MiB capture limit; the runner
  terminates an over-limit process and reports the retained diagnostic prefix
- it drives a local kernel executable directly and stores generated oracle
  inputs/wrappers only in a temporary run directory
- Lane A/B CLI output records the resolved oracle executable and best-effort
  Wolfram `$Version`, plus the explicit `user-provided/unpinned` version
  policy; version probe failures are reported but do not fail the diff run
- Lane A/B text output reports oracle process, batch, and batch-input counts
  so local runs expose current batching behavior without requiring JSON parsing
- it now compares result values, messages, and printed output for each case
- fixture `expected` fields are seed annotations only; Lane A/B pass/fail
  compares `rum` against fresh oracle output and reports seed-vs-oracle drift
  separately
- `rum` values use a shared InputForm-like renderer that covers common list,
  rule, delayed-rule, and association output shapes
- it does not yet claim full top-level display parity for broader Wolfram
  syntax sugar, though the shared renderer now covers common
  `Plus` / `Times` / `Power` InputForm-style infix output needed by simple
  polynomial and rational-expression diffs
- a first pass of `rum`-owned curated Lane A files now exists under
  [cases/wolfram/core](./cases/wolfram/core/README.md)
- that curated set currently covers arithmetic/structures, exact-arithmetic
  edge cases, a small polynomial/rational-expression subset, loops/control,
  associations, functional combinators, failure/status, and multiple string
  subfamilies
- the current `lane-a` comparison still has display limits beyond the shared
  renderer's common structural forms, so curated files should avoid cases whose
  only useful signal is obscure Wolfram `InputForm` sugar

### Lane A: Single-Expression Wolfram Diff

Use a Wolfram oracle for one-shot expression checks:

- input text
- kernel-visible result
- optional messages
- optional printed output

This lane should start from the imported `ham` golden files under
[seed/ham-import/golden](./seed/ham-import/golden).
The default Lane A runner now includes both those imported seed files and the
curated project-owned files under
[cases/wolfram/core](./cases/wolfram/core/README.md); pass `--fixture PATH`
when a run should target one file.

Run `lane-a --include-known-diffs` when auditing imported one-shot
divergences. Included known-difference cases carry their fixture metadata into
the report, and the summary counts confirmed vs resolved known differences so
fixed rows can be removed from the ledger deliberately. Resolved rows are
listed by ID and fixture location. For imported seed rows, treat this as a
curation signal; keep `seed/` unchanged and migrate promoted coverage into
`cases/` before pruning seed-linked ledger rows.
`lane-a --summary-json PATH` writes a stable machine-readable summary with
oracle configuration, run options, comparison counts, known-difference counts,
seed-drift counts, and oracle process/batch/input counts without freezing
per-case report rows. The text report mirrors the same oracle process, batch,
and batch-input counts. The current Lane A `OracleKernel` transport reports
one oracle process for a non-empty run by submitting compared cases as isolated
single-expression batches, and the JSON oracle section reports the concrete
batch transport name plus process count.

Priority areas:

- arithmetic normalization and exact numbers
- rules, patterns, traversal, replacement
- held evaluation and scoping
- definition precedence and messages
- strings once the string subsystem is mature enough

### Lane B: Stateful Session Diff

Some kernel semantics are about a sequence of evaluations, not one input line.
This lane should cover:

- assignment / redefinition order
- context evolution
- `Begin` / `End` / `BeginPackage` / `Needs`
- iterator leakage and scoping restoration
- message accumulation and clearing

Current curated Lane B fixtures cover definitions/messages, package defaults,
stateful scoping/control restoration, and fixture-boundary cleanup for the
multi-batch oracle transport.

Run `lane-b --include-known-diffs` when auditing current divergences. Included
known-difference steps carry their fixture metadata into the report, and the
summary counts confirmed vs resolved known differences so fixed rows can be
removed from the ledger deliberately. Resolved rows are listed by ID and
fixture location.
`lane-b --summary-json PATH` writes the same stable machine-readable summary
shape as Lane A, including oracle process/batch/input counts, without freezing
per-step report rows. Lane B also reports the concrete session-batch transport
and process count in text and JSON output so the current single-process batch
path is visible before any future interactive reuse transport is added.

Recommended format:

```text
# @name: specific-value-overrides-general-rule
>>> f[1] = 10
<<< 10
>>> f[x_] := x + 1
<<< x + 1
>>> f[1]
<<< 10
```

### Lane C: Known-Difference Ledger

Do not hide mismatches in comments. Track them as data.

The imported `ham` fixture format is a good seed:

```text
input !!! id=name | type=temporary | rum=... | wolfram=... | reason=...
```

For `rum`, this should become the divergence ledger for same-name features.
Current implementation has `divergences/known-differences.txt` plus
`divergences/retired-seed-differences.txt` plus `rum-difftest lane-c` to
validate fixture IDs against the active ledger or retired imported-seed
manifest. The check reports missing rows, unreferenced active rows,
unreferenced retired seed rows, stale fixture metadata when a known-difference
ID moves between fixture files, and active fixture/ledger `type=...` metadata
mismatches. Each active ledger row must include machine-checked
`tracked_by=path:needle` and `close_when` metadata so divergences remain owned
by a live roadmap item and have explicit removal criteria.
`rum-difftest lane-c --summary-json PATH` writes a stable machine-readable
summary with lane status, checked artifact paths, and issue counts for scripts
or CI dashboards without freezing the full internal ledger row shape.

### Lane D: Performance Regression

Timing and memory suites should exist, but they are not primarily Wolfram
compatibility checks. Their primary oracle is `rum`'s own baseline.

The imported `ham` timing and memory suites under
[seed/ham-import/bench](./seed/ham-import/bench)
and
[seed/ham-import/mem](./seed/ham-import/mem)
are seed inputs, not final `rum` policy. Current implementation has
`rum-difftest lane-d`, which executes those seed inputs locally and reports raw
metrics. Its default `smoke` profile skips imported heavy / very-large
categories for daily validation; use `--profile full` for explicit all-input
baseline collection.

Checked baseline artifacts live under [perf/baselines](./perf/baselines).
`perf/baselines/smoke.txt` covers the default smoke profile. `--baseline PATH`
enables comparison, and `--fail-on-regression` exits non-zero for missing
baseline rows, stale rows, elapsed guardrail failures, or result-node guardrail
failures. `--write-baseline PATH` captures the current successful run as a
baseline artifact and can tune elapsed guardrails with
`--baseline-elapsed-multiplier` and `--baseline-min-elapsed-micros`.
`--capture-runs N` repeats the same run, using per-case median elapsed samples
and slowest-sample guardrails. `--threshold-profile` selects the baseline
threshold record set and capture defaults:

- `local-debug`: local developer debug runs
- `ci-debug`: debug builds on shared CI machines
- `ci-release`: release builds on CI/release machines

Baseline rows without `threshold_profile` are treated as `local-debug` for
compatibility with older artifacts. The current checked-in elapsed thresholds
are intentionally broad smoke guardrails; actual CI/release artifacts still
need to be captured on the intended runners before tightening thresholds.
`lane-d --check-baseline-readiness` now checks the default checked smoke
artifact set: `perf/baselines/smoke.txt`,
`perf/baselines/smoke-ci-debug.txt`, and
`perf/baselines/smoke-ci-release.txt`.
`lane-d --summary-json PATH` writes a stable machine-readable summary for
scripts or CI dashboards, covering selected suite/profile/threshold profile,
run counts, clean status, and optional baseline/readiness/write-baseline
summary counts without freezing per-case result rows.

### Lane E: Host Contract Tests

Rust/Python host adapters and runtime embedding boundaries are important, but
they are not Wolfram-diff targets.

These should be tested against `rum`'s own public contract:

- session request/result types
- structured output/messages
- suspension and resume
- host callbacks / input requests
- Rust and Python adapter semantics

Current Rust host coverage lives in `cargo test -p rum-host`. Future Python
host coverage should live in the future Python host crate rather than in
`rum-difftest`.

### Lane F: Adapter / Protocol Tests

Jupyter and other protocols should be tested against protocol specs and `rum`'s
own regression expectations, not against Wolfram transport behavior.

Current Jupyter adapter/protocol coverage lives in `cargo test -p rum-jupyter`.

## What Should Not Be Forced Into Wolfram Diff

Do not use Wolfram as the oracle for:

- Rust host API shape
- Python host API shape
- suspension payload formats
- Jupyter framing details
- any `rum`-specific host capability mechanism
- any `rum`-specific kernel extension that does not have a meaningful
  Wolfram-side oracle

The right split is:

- Wolfram diff for kernel semantics
- contract tests for `rum`'s own host and protocol surfaces

## Imported Seed Pack

The imported `ham` assets are documented in:

- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

Raw counts:

- 9 golden correctness categories
- 166 normal correctness cases
- 3 known-difference cases
- 20 timing expressions
- 13 memory expressions

## Immediate Work Queue

1. Keep the imported `ham` corpus intact under `seed/`.
2. Continue extending the curated `cases/wolfram/core/` tree page by page from
   the matrix-guided kernel priorities.
3. Keep promoting resolved imported seed differences into `cases/` and moving
   their historical IDs into the retired seed manifest instead of editing
   `seed/`.
4. Extend the shared InputForm-like renderer beyond the current common
   structural forms when broader Wolfram display syntax becomes a real fixture
   blocker.
5. Continue extending Lane A/B curated fixtures from matrix-guided priorities.
6. Capture checked `ci-debug` / `ci-release` Lane D artifacts on the intended
   runners before tightening elapsed thresholds beyond broad smoke guardrails.
7. Keep Lane E/F boundaries explicit: host-contract and adapter/protocol tests
   stay in their owning crates rather than becoming Wolfram diffs.

## If This Folder Was Copied Standalone

If the receiving checkout only copied `difftest/` and not the accompanying
docs or TODO edits from another environment, that is fine. The next agent
should treat this folder as the canonical handoff package and recreate any
repository-level TODO or documentation updates locally as needed.
