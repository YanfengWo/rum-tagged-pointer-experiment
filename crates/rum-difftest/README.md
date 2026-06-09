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
The same hygiene check also rejects stray files under `cases/` that are not
recognized fixture files, README files, or stateful package support files, so
temporary probe fixtures should live outside the crate tree.

## Fast Local Check

Use `check` as the default fast, non-oracle health gate for this crate:

```bash
cargo run -p rum-difftest -- check
cargo run -p rum-difftest -- check --summary-json /tmp/rum-difftest-check.json
cargo run -p rum-difftest -- check --fail-on-open-known-difference
cargo run -p rum-difftest -- check --include-lane-d-smoke
```

The command runs the checked inventory baseline, Lane C known-difference
ledger validation, and artifact hygiene, then exits non-zero if any
default gate is dirty. It does not launch a Wolfram oracle. By default it also
avoids Lane D timing inputs, so it is suitable for quick local preflight and
scripts that only need repository-state checks. Use `--inventory-baseline PATH`
to test an alternate fixture-coverage baseline. Use `--include-lane-d-smoke`
to opt into the local Lane D smoke baseline gate, or
`--lane-d-smoke-baseline PATH` to run that gate against an alternate smoke
baseline. The optional summary JSON includes bounded `issue_samples` for
representative failing locators plus untruncated `issue_check_counts` and
`issue_kind_counts` arrays for scripts that need stable aggregate failure
counts even when samples are capped. Use `--issue-sample-check NAME` and
`--issue-sample-kind KIND` to focus the bounded locator list during
aggregate-gate debugging; the summary still reports the unfiltered issue total
and untruncated check/kind counts. `KIND` is a checked aggregate issue kind
such as `regression`, `missing_ledger_entry`, `oracle_artifact`,
`stray_fixture_file`, `open_known_difference`, `missing_record`, or
`missing_artifact`; invalid kind names fail fast instead of silently producing
an empty locator list. Issue-sample controls require `--summary-json PATH`,
because they only affect the machine-readable locator list. Use
`--fail-on-open-known-difference` when a local or CI script should fail if the
active Lane C ledger contains any `status=open` row; deferred and intentional
rows remain allowed by that gate.

## Machine-Readable Summaries

Every `--summary-json PATH` output includes a top-level `schema_version` field.
The current version is `1`. Increment that value for intentionally
incompatible report-shape changes; additive fields that preserve existing
consumers should stay on the current version.

CLI regression tests parse generated summaries as JSON and assert the schema
version, so machine-readable report changes should not rely only on string
fragment checks. The integration suite also includes a small summary contract
smoke that runs `check`, fake-oracle `lane-a` / `lane-b`, `inventory`,
`lane-c`, and `lane-d` summary paths together, including a failing aggregate
`check`, so schema-versioned JSON emission stays consistent across command
families.
Commands reject issue-sample controls such as `--max-issue-samples`,
`--issue-sample-kind`, `--issue-sample-check`, and `--issue-sample-source`
unless `--summary-json PATH` is also present.

## Fixture Inventory

Use `inventory` to inspect the default non-oracle fixture coverage before
adding or pruning cases:

```bash
cargo run -p rum-difftest -- inventory
cargo run -p rum-difftest -- inventory --check-baseline --fail-on-regression
cargo run -p rum-difftest -- inventory --write-baseline /tmp/rum-difftest-inventory-baseline.txt
cargo run -p rum-difftest -- inventory --summary-json /tmp/rum-difftest-inventory.json
cargo run -p rum-difftest -- inventory --lane lane-a --source curated-core --category threading
```

The command summarizes Lane A and Lane B fixture files, source groups, semantic
categories, normal entries, and known-difference entries. It does not execute
`rum` or a Wolfram oracle. The JSON output is intended for scripts or
dashboards that need stable coverage counts without parsing runner logs; its
top-level `clean` field is `true` when no baseline comparison was requested
or when the requested baseline comparison is clean, and `baseline_checked`
states whether that clean result included a baseline gate. Use
`--lane NAME`, `--source NAME`, and `--category NAME` for a focused coverage
audit; the filters apply consistently to text output, summary JSON, baseline
comparison, and `--write-baseline`, and a filter set that matches no fixtures
fails fast. Baseline comparison narrows the checked baseline records to the
active filter scope as well, and fails if no baseline records match that scope.
Filtered `--write-baseline` artifacts include comment lines recording the
active filters so they are not confused with the repository-wide baseline.
`--check-baseline` compares the current inventory against
[coverage/inventory-baseline.txt](./coverage/inventory-baseline.txt).
The coverage policy is documented in
[coverage/README.md](./coverage/README.md). Baseline records can target source
groups with `source=...` or semantic categories with `category=...`. `min_*`
counts are lower bounds;
`max_known_difference_entries` is an upper bound so resolved known differences
can decrease without failing the gate. Use `--fail-on-regression` to make a
non-clean baseline comparison exit non-zero. `--write-baseline PATH` captures
the current inventory in the same stable baseline format for intentional
fixture additions or reshuffles.

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
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-a-summary.json
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --source curated-core --category context-namespace
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --fail-on-seed-drift
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --fail-on-project-mismatch
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --fail-on-project-seed-drift
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --include-known-diffs --fail-on-resolved-known-difference
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-a-seed-drift.json --issue-sample-kind seed-drift --max-issue-samples 50
cargo run -p rum-difftest -- lane-a --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-a-project-issues.json --issue-sample-source curated-core
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
in-memory, and removes those artifacts afterward. By default, Lane A runs both
the imported seed golden files under `seed/ham-import/golden/` and the curated
project-owned fixtures under `cases/wolfram/core/`; use `--fixture PATH` for a
single file. Use `--source NAME` and `--category NAME` to run a focused slice
of the default fixture set, using the same source labels and semantic category
names reported by `inventory`; a filter set that matches no fixtures fails
fast. The CLI reports the resolved oracle executable, the effective
per-invocation timeout, the explicit
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
comparison. It also covers common `Plus` / `Times` / `Power` infix forms used
by simple polynomial and rational-expression diffs. It is intentionally not
yet a complete Wolfram display implementation or transport model. Fixture
`expected` fields are seed
annotations, not the pass/fail oracle: the runner compares `rum` against the
fresh oracle result on every run and reports seed-vs-oracle drift separately.
Use `--fail-on-seed-drift` when intentionally auditing fixture seed snapshots
against the current oracle. Use `--fail-on-project-seed-drift` when a default
run should fail only for non-`seed/ham-import` fixture drift, preserving the
imported seed snapshot as historical input.
Use `--fail-on-project-mismatch` when a default run should fail for
project-owned fixture mismatches or failures while still reporting imported
seed mismatches as historical-reference noise.
When `--include-known-diffs` is used, the report includes known-difference
ID/type/reason metadata and counts confirmed vs resolved known differences.
Resolved known differences are listed by ID and fixture location so stale
imported divergences are visible before curation. Keep `seed/` unchanged;
migrate promoted cases under `cases/` before pruning seed-linked ledger rows.
The text report lists oracle process, batch, and batch-input counts for quick
local inspection. `--summary-json PATH` writes a stable machine-readable
summary with oracle configuration, run options, comparison counts,
known-difference counts, and seed-drift counts, including
`counts.oracle_processes`, `counts.oracle_batches`, and
`counts.oracle_batch_inputs` so scripts can see that the current Lane A
`OracleKernel` transport runs compared cases through a single case-isolated
batch process. The JSON options section records failure gate flags such as
`fail_on_mismatch`, `fail_on_project_mismatch`, `fail_on_seed_drift`,
`fail_on_project_seed_drift`, and
`fail_on_resolved_known_difference`, plus optional `source` / `category`
filters, so scripts can interpret non-zero exits without inferring which gate
or fixture slice was active. The JSON oracle section also reports the concrete
batch transport name and process count. It also includes
an unfiltered `issue_counts` breakdown plus a bounded `issue_samples` locator
list for the first non-matching, seed-drifted, or resolved known-difference
cases, including each sample's `fixture_source` and
`project_owned_fixture`. The sibling `issue_source_counts` array keeps the
same unfiltered issue breakdown grouped by fixture source. It does not freeze
the full per-case report shape. Use
`--issue-sample-kind non-match`, `seed-drift`, or
`resolved-known-difference` to focus the bounded locator list during targeted
audits while still retaining the full issue-count breakdown. Use
`--issue-sample-source seed-golden|curated-core|lane-a-custom` to focus those
bounded locators by fixture source without changing the unfiltered issue
counts or source-count breakdown. These issue-sample controls require
`--summary-json PATH`.
Use `--fail-on-resolved-known-difference` with `--include-known-diffs` when
an oracle audit should fail if an active known-difference row now matches and
is ready for ledger cleanup.

## Lane B Runner

The crate also includes a `lane-b` runner for ordered stateful session
fixtures:

```bash
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel
cargo run -p rum-difftest -- lane-b --oracle-command WolframKernel
cargo run -p rum-difftest -- lane-b --oracle-command WolframKernel --oracle-timeout-seconds 60
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --include-known-diffs
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-b-summary.json
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --source stateful --category mutation-updates
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --fail-on-seed-drift
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --fail-on-project-mismatch
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --fail-on-project-seed-drift
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --include-known-diffs --fail-on-resolved-known-difference
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-b-seed-drift.json --issue-sample-kind seed-drift --max-issue-samples 50
cargo run -p rum-difftest -- lane-b --oracle-kernel /path/to/WolframKernel --summary-json /tmp/rum-lane-b-stateful-issues.json --issue-sample-source stateful
```

Lane B keeps one `rum` session per fixture file and evaluates the matching
Wolfram steps in one local kernel process for that fixture. This avoids the
older replay-based state simulation and gives definition, option, message,
attribute, precedence, mutation/update, scoping, and control-flow restoration
checks a real ordered-session oracle. The runner compares InputForm-like
result values plus per-step message and print streams. Like Lane A, its CLI
reports the resolved oracle executable and best-effort Wolfram `$Version`
without making version probing a pass/fail gate, and applies the same
per-session oracle timeout.
Fixture `expected` fields are treated as seed annotations and are checked only
for seed-vs-oracle drift, not as the comparison oracle. Use
`--fail-on-seed-drift` when intentionally auditing stateful fixture snapshots
against the current oracle. `--fail-on-project-seed-drift` is also available
and ignores only fixtures under the imported `seed/ham-import` snapshot.
`--fail-on-project-mismatch` is available as the same project-owned non-match
gate as Lane A, mainly for symmetry and custom/imported fixture workflows.
Use `--source NAME` and `--category NAME` to run a focused slice of the
default stateful fixture set, using the same source labels and semantic
category names reported by `inventory`; a filter set that matches no fixtures
fails fast.
When `--include-known-diffs` is used, the report includes the known-difference
ID/type/reason for non-matching steps and counts confirmed vs resolved known
differences. Resolved known differences are listed by ID and fixture location
so fixed divergences are visible before removing ledger rows. `--summary-json
PATH` writes the same stable machine-readable summary shape as Lane A, with
oracle configuration, run options, comparison counts, known-difference counts,
seed-drift counts, failure gate options, oracle process/batch/input counts,
unfiltered issue-count/source breakdowns, optional `source` / `category`
filters, and bounded `issue_samples` locators, without freezing the full
per-step report rows. The same `--issue-sample-kind` filters available in
Lane A apply here, including each sample's `fixture_source` and
`project_owned_fixture` fields. `--issue-sample-source
seed-golden|stateful|lane-b-custom` narrows bounded locators by fixture source
while retaining unfiltered issue/source counts. These issue-sample controls
require `--summary-json PATH`. The same
`--fail-on-resolved-known-difference` cleanup gate applies to included
stateful known differences. Lane B also reports oracle batch/input counts and
the concrete session-batch
transport name/process count in text output, so the current single-process
batch transport is distinguishable from a future interactive reuse transport
without requiring JSON parsing.
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
cargo run --release -p rum-difftest -- lane-d --threshold-profile ci-release --capture-runs 5 --write-baseline /tmp/rum-lane-d-ci-release.txt --fail-on-error
cargo run -p rum-difftest -- lane-d --check-baseline-readiness
cargo run -p rum-difftest -- lane-d --summary-json /tmp/rum-lane-d-summary.json
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
slowest observed sample for guardrails. Capture-only controls such as
`--capture-runs`, `--baseline-elapsed-multiplier`, and
`--baseline-min-elapsed-micros` require `--write-baseline PATH` so scripts do
not silently discard calibration settings. `--threshold-profile` selects the
baseline threshold record set and capture defaults: `local-debug` for local dev
runs, `ci-debug` for debug builds on CI machines, and `ci-release` for release
builds on CI/release machines. `--check-baseline-readiness` checks the default
checked smoke baseline artifacts for all threshold profiles and exits non-zero
until the `ci-debug` / `ci-release` artifacts have been captured on the
intended runners. The default expected paths are
`perf/baselines/smoke.txt`, `perf/baselines/smoke-ci-debug.txt`, and
`perf/baselines/smoke-ci-release.txt`; readiness suggestions use
`cargo run --release` for the `ci-release` artifact so it is not accidentally
captured from a debug build. Readiness JSON entries also expose a
`cargo_profile` field (`debug` or `release`) so dashboards do not need to parse
the suggested command. `--summary-json PATH` writes a stable
machine-readable summary for scripts or CI dashboards. That JSON exposes the
selected suite/profile/threshold profile, run counts, clean status, and
optional baseline/readiness/write-baseline summary counts; it intentionally
does not freeze per-case result rows.

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

- arithmetic and structural smoke checks, plus focused exact-arithmetic edge
  cases
- a small polynomial/rational-expression suite for renderer-supported
  `Expand` / `Together` outputs
- loops, control objects, and failure/status flows
- associations and keyed-data basics
- definition precedence promoted from resolved imported seed coverage
- CL-04 functional combinators
- rules, patterns, replacement, and traversal
- CL-07 string surface/pattern/template/similarity subsets

The original imported staging README is preserved in
[README.imported.md](./README.imported.md).
