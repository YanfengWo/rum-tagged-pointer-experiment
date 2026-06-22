# brief-025: per-module snapshot checkpoint-chain driver

## Goal

A driver `scripts/snapshot-checkpoint-chain.sh` that walks the full mma-hol
library stack ONE module at a time on a SCRATCH copy, and after EACH module
writes a named, profiled snapshot: cold-build a base through `auto/Simp` (the
layer before `Pair`), then `extend_snapshot` each subsequent library in
dependency order, copying `bootstrap.mx` to a per-module checkpoint and recording
that step's load time, snapshot size, contained libraries, and HOL context/name
counts. Per-step timeouts are large (default tuned to ~10x WolframKernel's order).
The driver STOPS at the first module that errors or times out and reports exactly
where rum gets stuck. When done, an interrupted/resumable per-module checkpoint
set + a manifest exist, the pristine clone is untouched, and we know the first
downstream wall.

## Context pointers

- Read `AGENTS.md` (unmodified-`mma-hol` HARD LINE) and the PROGRESS entry
  `2026-06-23 Snapshot reliability harness GREEN` (the round-trip is now
  byte-idempotent + fidelity-preserving, so this chain is trustworthy).
- Match the shape/flags/rum-invocation of `scripts/mma-hol-bounded-smoke.sh` and
  `scripts/snapshot-reliability.sh` (both run `.wls` via
  `rum-cli --eval '$ScriptCommandLine = {...}; Get["<abs>"];'`, bound via
  `scripts/run-bounded.py`, write a `summary.tsv`, and operate on a scratch copy).
- Snapshot tools in the clone (read-only): `tests/build_snapshot.wls <pattern>`
  (cold build through the first library whose filename contains `<pattern>`, ->
  `bootstrap.mx`), `tests/extend_snapshot.wls <relpath>` (restore `bootstrap.mx`,
  load one more file on top, re-dump). extend prints `re-dumping N HOL contexts`
  and timing markers.
- The dependency-ordered library list is the `libs` list in
  `tests/build_snapshot.wls` (read it; do NOT hardcode a divergent copy — parse
  it or mirror it exactly). After `auto/Simp.wl` (the base) the remaining order is:
  `stdlib/Pair.wl, stdlib/Sum.wl, stdlib/Option.wl, stdlib/Set.wl, auto/Set.wl,
  stdlib/Num.wl, auto/Arith.wl, stdlib/List.wl, stdlib/Finite.wl, stdlib/FTA.wl,
  stdlib/Int.wl, stdlib/Rat.wl, stdlib/Real/Cut.wl, stdlib/Real/RatAux.wl,
  stdlib/Real/Field.wl, stdlib/Real/Mul.wl, stdlib/Real/Inv.wl,
  stdlib/Real/Complete.wl, stdlib/Real/Abs.wl, stdlib/Real/MinMax.wl,
  auto/RealArith.wl, stdlib/Real/Seq.wl, stdlib/Real/SeqAux.wl,
  stdlib/Real/Compact.wl, stdlib/Real/Connected.wl, stdlib/Real/Topology.wl,
  stdlib/Real/CompactSet.wl`.

## Deliverable: `scripts/snapshot-checkpoint-chain.sh`

Flags (mirror the existing harness conventions):
- `--mma-hol PATH` — if it points at `/Users/fft/Developments/mma-hol-acceptance`
  (pristine), rsync to a throwaway scratch dir (exclude `.git`, `*.mx`) and use
  THAT; never write into the pristine clone.
- `--output-dir PATH` — checkpoints + `manifest.tsv` + per-snapshot profiles.
- `--base PATTERN` — cold base layer (default `Simp` → through `auto/Simp.wl`).
- `--timeout SECONDS` — per-step wall timeout (default `1800`; this is the
  ~10x-WK-order knob — WK loads through Pair in ~38s, rum ~190s, so 1800s leaves
  ample headroom and a genuine overrun is a real "stuck" signal).
- `--limit N` — stop after N extend steps (for validation; default: all).
- `--rum-cli PATH` / `RUM_CLI_BIN`.
- `--list`, `-h`.

Behavior:
1. Resolve/clone the scratch copy. Build `rum-cli` release if `RUM_CLI_BIN` unset.
2. Cold-build the base: `build_snapshot.wls <PATTERN>`, timed. Copy `bootstrap.mx`
   to `<outdir>/cp00-<base-module-slug>.mx`. Write its profile + manifest row.
3. For each remaining library L (index NN starting at the base index+1), in order:
   - Run `extend_snapshot.wls <relpath-of-L>` under `run-bounded.py --timeout
     <SECONDS>`, capturing wall seconds and stdout/stderr.
   - Classify: `timeout` (exit 124), `error` (nonzero / `$Failed` /
     `TerminatedEvaluation` in output), else `ok`.
   - On `ok`: copy `bootstrap.mx` -> `<outdir>/cpNN-<module-slug>.mx`; record
     load_seconds, cumulative_seconds, size_bytes (`wc -c`), num_contexts +
     num_names (parse extend's `re-dumping N HOL contexts`, and/or a tiny
     `rum-cli --eval 'Length[Names["HOL`*"]]'` against the snapshot — cheap).
     Write the per-snapshot profile; append the manifest row (status ok).
   - On `timeout`/`error`: append the manifest row with that status + the stderr
     path, print `STUCK AT: <module> (<status>, <seconds>s)`, and STOP the chain.
4. Print a final summary: deepest module reached, total cumulative seconds, and
   the stuck point (or "completed all N").

Naming: `cpNN-<module-slug>.mx` where NN is the 2-digit library index from the
build_snapshot list and `module-slug` is the relpath with `/` and `.` -> `_`
(e.g. `cp15-stdlib_Pair_wl.mx`, `cp27-stdlib_Real_Cut_wl.mx`).

Per-snapshot profile `<outdir>/cpNN-<module-slug>.profile.txt` — a small
human-readable record: the index + module just added, the FULL cumulative list of
libraries contained in this snapshot (base list + everything extended so far),
this step's load_seconds, the snapshot size, and the HOL context/name counts.

`manifest.tsv` columns:
`index  module  snapshot_file  load_seconds  cumulative_seconds  size_bytes  num_contexts  num_names  status`
(one header row + one row per step, base included).

## Scope

- Files you MAY create: `scripts/snapshot-checkpoint-chain.sh`. You MAY read
  (not modify) `scripts/run-bounded.py`, `scripts/mma-hol-bounded-smoke.sh`,
  `scripts/snapshot-reliability.sh`.
- MUST NOT touch: any `mma-hol` clone (scratch only; NEVER write to
  `/Users/fft/Developments/mma-hol-acceptance`); any `crates/**`; `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` except your own report; `.git`, git config.

## Pitfalls (read twice)

1. **Scratch only.** Confirm `git -C /Users/fft/Developments/mma-hol-acceptance
   status --short` stays empty after your runs. The build/extend write
   `bootstrap.mx` into the mma-hol root.
2. **The chain accumulates in `bootstrap.mx`.** extend restores + re-dumps the
   SAME `bootstrap.mx`; copy it to the named checkpoint AFTER each step. The next
   extend continues from `bootstrap.mx`.
3. **Real rum invocation.** Mirror the smoke driver's `$ScriptCommandLine = {...};
   Get["<abs>"]` form for build/extend, bounded via `run-bounded.py`.
4. **Dependency order is load-bearing.** Use the build_snapshot `libs` order
   exactly; extend throws if a file's deps are not yet in the snapshot.
5. **Stop at the first failure** — do NOT `--keep-going` past a stuck module (the
   chain state is cumulative; later steps depend on it).
6. **Validate cheaply.** For YOUR verification run, use `--limit 2` (base ->
   Pair -> Sum) so you exercise the cold base, the ~190s Pair extend, one more
   extend, the naming, the profiles, and the manifest WITHOUT running the whole
   multi-hour stack. Do NOT run the full chain — Claude runs that.

## Verification (MANDATORY — run, paste verbatim)

```
bash scripts/snapshot-checkpoint-chain.sh --mma-hol /Users/fft/Developments/mma-hol-acceptance --output-dir /tmp/cdx025-chain --limit 2
ls -l /tmp/cdx025-chain/*.mx
cat /tmp/cdx025-chain/manifest.tsv
echo "--- a sample profile ---"; cat /tmp/cdx025-chain/cp15-*.profile.txt
git -C /Users/fft/Developments/mma-hol-acceptance status --short   # MUST be empty
bash scripts/snapshot-checkpoint-chain.sh -h | head -20
```

Paste verbatim: the `--limit 2` run output, the checkpoint `.mx` listing (expect
`cp00-auto_Simp_wl.mx`, `cp15-stdlib_Pair_wl.mx`, `cp16-stdlib_Sum_wl.mx`), the
manifest (base + Pair + Sum rows with real load_seconds, sizes, statuses), one
profile, and the empty pristine-status check.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and runs the full chain.
- NEVER write to/edit any `mma-hol` clone; scratch copy only.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo (except named `/tmp`
  scratch + output dir), no network. Commands: file edits within Scope, `bash`,
  `rsync`, `grep`, `ls`, `wc`, the release `rum-cli`, `cargo build -p rum-cli` if
  needed, `python3`.
- Minimal, self-contained script matching the existing harness style.
- You MUST paste real verbatim output; if a step is not green, say so — never
  fake it.

## Report format (final message)

1. What changed — the new script, its flags, the naming/profile/manifest scheme.
2. Verification — the `--limit 2` run verbatim, the `.mx` listing, the manifest,
   a profile, the empty pristine check, and `-h`.
3. The verdict — confirm base + Pair + Sum checkpoints + profiles + manifest are
   produced correctly, with real load times, on a scratch copy.
4. Stop-loss / open questions — anything unsure (empty if none).
