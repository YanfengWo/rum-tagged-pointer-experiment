# Brief 125 — Archive WK/mma-hol scripts; make both microbenches rum-only

Implementation brief. Second step of `docs/dev/release-transformation-plan.md`
(read its Decision + Policy sections first). Predecessor brief-124 removed
`rum-difftest`; this brief cleans `scripts/`.

## Goal

`scripts/` contains no WolframKernel- or mma-hol-facing tooling: the nine
archive items below are copied verbatim into the private archive and deleted
from the tree, and the two microbench scripts are transformed into **rum-only**
tools (no `wolfram` engine, no WK columns/ratios) while preserving their rum
lane byte-for-byte in behavior: same labels, same iteration counts, same
guarded timing method, same TSV-per-label output for rum.

## Context pointers

- `docs/dev/release-transformation-plan.md` — the inventory this brief executes.
- Archive root: `/Users/fft/Developments/Archive/rum-history/` (exists; has
  `MANIFEST.md`, `WLBuiltin/`, `crates/rum-difftest/`, `scripts/`).
- The microbench A/B workflow after this brief: run the script on build A and
  build B separately, compare the two per-label TSVs. The scripts stay the
  project's perf-gate tools — do not degrade their rum measurement path.

## Scope

- DELETE from the repo (after archiving): `scripts/mma-hol-bounded-smoke.sh`,
  `scripts/pair-simp-hotspot-probes.sh`, `scripts/pair-simp-pair-body-phase-tree.sh`,
  `scripts/pair-simp-repetition-probe.sh`, `scripts/perf-baseline-stage0.sh`,
  `scripts/perf-probes/` (whole directory), `scripts/rebuild-snapshot.sh`,
  `scripts/snapshot-checkpoint-chain.sh`, `scripts/snapshot-reliability.sh`.
- MODIFY: `scripts/core-language-microbench.sh`, `scripts/core-memory-microbench.sh`.
- MAY create/modify outside the repo (EXPLICIT EXCEPTION): only under
  `/Users/fft/Developments/Archive/rum-history/` — mirror copies under its
  `scripts/`, and appending to `MANIFEST.md`.
- MUST NOT touch: anything else — no crate source, no `docs/**`, no other
  scripts, no `Cargo.*`, no `.git`. References to the archived scripts inside
  `docs/**` are handled by later briefs: REPORT, do not edit.

## Steps

1. Archive copies FIRST, mirroring paths, then verify:
   ```
   cp <each listed file> /Users/fft/Developments/Archive/rum-history/scripts/
   cp -R scripts/perf-probes /Users/fft/Developments/Archive/rum-history/scripts/perf-probes
   diff -rq scripts/perf-probes /Users/fft/Developments/Archive/rum-history/scripts/perf-probes   # no output
   ```
   (plus a `diff -q` per single file → no output).
2. Append ONE grouped entry to the archive `MANIFEST.md` (follow the existing
   format): 原路径 = the nine paths listed above; 性质 = mma-hol snapshot
   pipeline (rebuild/checkpoint-chain/reliability), mma-hol bounded smoke, WK
   perf probes (pair-simp trio + perf-probes dir incl. a WK-side `.wls`
   profiler), and the mma-hol stage0 perf baseline driver; 移除 commit =
   exactly `（由 Claude 在提交时回填）`; 回收线索 = none under the new policy
   (any future perf macro-benchmarks will be rum-native).
3. `git rm` the nine items.
4. Transform `scripts/core-language-microbench.sh`:
   - remove the `--engine` option entirely (the script is rum-only now), the
     `--wolfram` option, the `wolfram_bin` default, the WK executable check,
     the WK run lane, and the WK side of the report;
   - the per-label output becomes `timings.tsv` with header
     `label<TAB>iterations<TAB>rum_seconds` (drop `wolfram_seconds`/`ratio`;
     rename the old `ratios.tsv` naming accordingly);
   - keep EVERYTHING else identical for the rum lane: label set, statement
     generation, iteration counts, best-of/timing discipline, output dir
     layout, usage text (update wording to rum-only).
5. Transform `scripts/core-memory-microbench.sh` the same way: delete WK
   engine/columns wholesale; keep the rum-side panel (labels, columns, method)
   unchanged; update usage text.
6. Syntax-check both: `bash -n scripts/core-language-microbench.sh` and
   `bash -n scripts/core-memory-microbench.sh`.
7. Smoke-run the language microbench rum-only. Check its usage for a
   label-subset flag; if one exists, run 2-3 cheap labels; otherwise one full
   default run is acceptable (budget ≤15 min). Paste the `timings.tsv` header
   + the first rows. If the memory microbench supports a similarly cheap
   invocation, smoke-run it too; if its cheapest run exceeds ~10 min, skip the
   run, say so, and rely on `bash -n` + a careful diff review.
8. Residue sweep: `rg -in 'wolfram|mathematica|mma[-_]hol|wstp' scripts/` must
   be EMPTY (report `CLEAN`).

## Deliverable

- The deletions + the two transformed scripts, uncommitted in the working tree.
- Archive copies verified; MANIFEST grouped entry appended.
- Verbatim verification output (diff checks, bash -n, smoke-run TSV rows, sweep).

## Verification (MANDATORY — run all, paste verbatim)

```
bash -n scripts/core-language-microbench.sh && echo LANG-SYNTAX-OK
bash -n scripts/core-memory-microbench.sh && echo MEM-SYNTAX-OK
<the smoke run from step 7 — paste timings.tsv header + first rows>
rg -in 'wolfram|mathematica|mma[-_]hol|wstp' scripts/ || echo CLEAN
ls scripts/ | sort
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Outside-repo writes ONLY under `/Users/fft/Developments/Archive/rum-history/`.
  No network. Never touch any `mma-hol` clone.
- Do NOT "improve" the rum measurement path while editing — the rum lane's
  behavior must be preserved exactly (same labels, same timing discipline).
  This is a deletion-only transform plus wording updates.
- Minimal diff outside the WK-removal hunks; no drive-by reformatting.
- Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per path (archived+deleted / transformed), with the transform
   summarized per script (what was removed, what was preserved).
2. Verification — each command with verbatim output.
3. Residue handoff — any references to the archived scripts you noticed in
   `docs/**` (list only).
4. Stop-loss / open questions (empty if none).
