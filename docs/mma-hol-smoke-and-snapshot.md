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
- `--limit N` stops after the first `N` selected stages.
- `--keep-going` continues after failures so later independent test probes can
  report their own status.
- `--list` and `--dry-run` inspect the selected plan without running `rum`.

The load stages deliberately restart `rum-cli` and replay the dependency prefix
for each stage. That makes the first failing load boundary explicit and avoids
keeping long-lived session state hidden inside the driver. It should not be
used to compare elapsed time across commits.

The test stages load the current minimal harness prelude
(`ErrorUtil.wl`, `Types.wl`, `Terms.wl`, `Kernel.wl`, and
`tests/harness.wl`) and then run one test file. They classify `List[n, 0]` as
green and keep the full captured output for failed-test triage.

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
- `Contexts[]` first-pass session context enumeration
- `FileExistsQ`, `FileDate[..., "Modification"]`, `FileByteCount`,
  `AbsoluteTime`, and `Round` for snapshot freshness and timing reports
- simple `digits.digits` source literals lower to exact rationals, covering
  timing intervals such as `0.1` / `0.01` without introducing machine-real
  semantics
- textual `Save` / `DumpSave` object persistence and `Get` restore for the
  current saved-definition subset

Not yet closed for stable snapshot scripts:

- assignment to `$ContextPath` / `$Packages` as runtime session fields, not
  merely inert own values
- a regression proving
  `DumpSave[snapshotFile, Evaluate[holContexts]]; Get[snapshotFile]` restores
  enough public/private `HOL` contexts for `run_fast.wls`

### Recommended Next Order

1. Add a runtime snapshot regression with a tiny synthetic package tree before
   trying the full `mma-hol` stable scripts.
2. Teach assignment to `$ContextPath` / `$Packages` to mutate the runtime
   session fields needed after snapshot restore.
3. Try `tests/build_snapshot.wls` only through the current green cold frontier,
   then use `tests/run_fast.wls` on one small test pattern.

Keep this lane below the cold semantic smoke while `PropTaut` / `Simp` remains
the main `mma-hol` blocker.
