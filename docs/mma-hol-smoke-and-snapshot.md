# `mma-hol` Smoke Driver And Snapshot Audit

This note records the current `rum`-side harness contract for the sibling
`mma-hol` pressure lane. It intentionally separates cold semantic smoke from
stable snapshot workflow support.

## Bounded Cold Smoke Driver

Use:

```bash
scripts/mma-hol-bounded-smoke.sh --mma-hol /path/to/mma-hol --mode loads
```

The driver is diagnostic harness engineering, not a performance benchmark. It
builds `rum-cli` once unless `RUM_CLI_BIN` is set, then runs selected stages
with a per-stage wall timeout through `scripts/run-bounded.py`. The bounded
runner starts each probe in a fresh process group and terminates the whole
group on timeout, so child processes such as external kernels do not survive a
failed probe. Each stage writes:

- a generated expression in `*.expr`
- captured stdout in `*.stdout`
- captured stderr in `*.stderr`
- a tab-separated `summary.tsv`

Modes:

- `loads`: cumulative package-load probes in the same order as
  `mma-hol/tests/run_all.wls`
- `tests`: independent harness-backed `*_tests.wl` probes
- `all`: load probes followed by test probes

Useful controls:

- `--timeout SECONDS` bounds each stage.
- `--only LABEL` runs one selected stage such as `test/proptaut_tests`.
- `--limit N` stops after the first `N` selected stages.
- `--keep-going` continues after failures so later independent test probes can
  report their own status.
- `--list` and `--dry-run` inspect the selected plan without running `rum`.

The load stages deliberately restart `rum-cli` and replay the dependency prefix
for each stage. That makes the first failing load boundary explicit and avoids
keeping long-lived session state hidden inside the driver. It should not be
used to compare elapsed time across commits.

The test stages preload the package dependency prefix needed by the selected
test file, then load `tests/harness.wl` and run one test file. That keeps
`Needs[...]` package lookup noise out of performance triage; for example,
`test/proptaut_tests` preloads through `auto/PropTaut.wl` before running the
test. Test stages classify `List[n, 0]` as green and keep the full captured
output for failed-test triage.

## Stable Snapshot Workflow Audit

`mma-hol` now has two relevant execution modes:

- Strict mode is the authoritative cold semantic path. It uses the original
  closure / gensym kernel encapsulation and is not a cross-session snapshot
  target.
- Stable mode uses fixed private kernel-state names so a bootstrapped world can
  be dumped and restored during development.

The current `rum` target is not Wolfram `.mx` compatibility. `DumpSave` is a
textual `rum` snapshot format that must round-trip through `Get`.

### Script Surfaces

`tests/build_snapshot.wls` needs:

- `$HOLEncapsulationMode = "Stable"` before loading `Kernel.wl`
- ordered `Get` of library files
- `StringContainsQ` for partial `uptoPattern` selection
- `AbsoluteTime` and `Round` for timing reports
- `Contexts[]` to enumerate `HOL` contexts
- `DumpSave[snapshotFile, Evaluate[holContexts]]`
- `FileByteCount` for the final size report

`tests/run_fast.wls` needs:

- `FileExistsQ`
- `FileNames[..., ..., Infinity]`
- `FileDate[..., "Modification"]`
- `AbsoluteTime[date]` comparisons
- `Get[snapshotFile]`
- `Contexts[]` after restore
- mutation of `$ContextPath` and `$Packages`
- harness load plus selected test-file execution

`tests/extend_snapshot.wls` needs:

- `FileExistsQ`
- `FileByteCount`
- `Get[snapshotFile]`
- `Contexts[]`
- `$ContextPath` restore
- ordered load of new relative files
- `DumpSave[snapshotFile, Evaluate[holContexts]]`

### Current `rum` Status

Already covered enough for cold or partial snapshot work:

- `Get`, `Needs`, `$Path`, `$InputFileName`
- `FileNameJoin`, `DirectoryName`, `ParentDirectory`, `FileNameTake`,
  first-pass `FileNames`, including `FileNames[pattern, dir, Infinity]`
- `StringContainsQ` and current `StringMatchQ` subset
- `StringMatchQ` string wildcards such as `"HOL`*"` for context filtering
- `SelectFirst` for partial snapshot layer selection
- `Contexts[]` first-pass session context enumeration
- `FileExistsQ`, `FileDate[..., "Modification"]`, `FileByteCount`,
  `AbsoluteTime`, and `Round` for snapshot freshness and timing reports
- simple `digits.digits` source literals lower to exact rationals, covering
  timing intervals such as `0.1` / `0.01` without introducing machine-real
  semantics
- textual `Save` / `DumpSave` object persistence and `Get` restore for the
  current saved-definition subset
- `Get` propagates `Print` output from loaded files, so snapshot progress logs
  remain visible when scripts are invoked through a wrapper expression
- CLI `Exit[n]` handling preserves accumulated prints/messages before returning
  the requested process status, so `run_fast.wls` logs are visible even though
  it ends through `HOLTest`testExit[]`
- `DumpSave[file, Evaluate[contexts]]` where `contexts` is a list of context
  strings, plus restore-time `$ContextPath` / `$Packages` session-field
  assignment, covered by a tiny public/private context regression

Dedicated bounded harness:

```bash
scripts/mma-hol-bounded-smoke.sh --mma-hol /path/to/mma-hol --mode snapshot
```

The snapshot mode runs `snapshot/build-parser`, `snapshot/run-fast-parser`, and
`snapshot/extend-proptaut` as separate process-group-bounded stages and writes
the same `summary.tsv`, `*.expr`, `*.stdout`, and `*.stderr` artifacts as the
cold smoke lanes.

Validated on 2026-06-12 against the fixed clean clone at
`/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`:

- `tests/build_snapshot.wls Parser` succeeds in Stable mode, loading 11 of 32
  libraries and writing a partial `bootstrap.mx` of about `11904 KB`.
- `tests/run_fast.wls parser` exits with status `0` against that snapshot.
  The dedicated snapshot harness now captures the visible parser test log,
  ending with `passed: 78` and `failed: 0`.
- `tests/extend_snapshot.wls auto/PropTaut.wl` exits with status `0` and
  re-dumps the snapshot at about `11940 KB`. The fixed probe clone restores
  `$Packages` alongside `$ContextPath`, so this stage is quiet; carry the same
  script cleanup forward when refreshing from the live `mma-hol` tree.

Not yet closed for stable snapshot scripts:

- full `tests/build_snapshot.wls` validation still runs into the known
  downstream `Pair.wl` / `Simp` performance gate

### Recommended Next Order

1. Promote the snapshot mode into the regular `mma-hol` smoke checklist once
   the live `mma-hol` tree carries the `$Packages` restore cleanup.
2. Add a heavier snapshot stage only after it is worth spending time on the
   `PropTaut` / `Simp` downstream gate.
3. Keep any private-context lint probe below the cold and stable snapshot lanes.

Keep this lane below the cold semantic smoke while `PropTaut` / `Simp` remains
the main `mma-hol` blocker.
