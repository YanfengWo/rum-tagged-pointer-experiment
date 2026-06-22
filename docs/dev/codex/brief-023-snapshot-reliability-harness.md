# brief-023: snapshot-reliability harness (dimension A cycle + dimension B gensym assertion)

## Goal

A permanent, reusable harness `scripts/snapshot-reliability.sh` that stress-tests
rum's `DumpSave`/`Get` snapshot round-trip on the FAST mma-hol layers and turns
the two now-fixed reliability properties into standing regression guards:

- **Dimension A — round-trip cycle stability:** for each selected layer, cold-
  build a Stable snapshot, then loop `restore -> re-dump` N times; assert the
  snapshot is BYTE-IDENTICAL across every cycle (guards BUG-RT, the negative-
  literal round-trip rot, fixed in commit `20458b9`) and that the layer's test
  file gives the SAME pass/fail after a restored cycle as on the cold build.
- **Dimension A — cold-vs-incremental fidelity:** building a layer cold vs
  reaching it by `extend_snapshot` from the previous layer must yield the same
  HOL name count and the same test result.
- **Dimension B — gensym hygiene:** the dumped snapshot contains ZERO Module
  gensyms (no `$<digit>` matches), guarding the #2 fix (commit `ae7e92d`).

When done, `scripts/snapshot-reliability.sh` runs GREEN on a scratch copy of the
acceptance standard and exits non-zero if any assertion fails, with a
`summary.tsv` and human-readable per-check lines. This is an integration harness
(diagnostic engineering), NOT a perf benchmark and NOT an mma-hol patch.

## Context pointers

- Read `AGENTS.md` (the unmodified-`mma-hol` HARD LINE) and the PROGRESS entry
  `2026-06-22 Snapshot reliability test (dimension A)` — it has the diagnosis,
  the cycle method, and the two bugs this harness guards.
- Reference harness to match in shape/flags/output: `scripts/mma-hol-bounded-
  smoke.sh` (it already builds `rum-cli`, runs `.wls` via `rum-cli --eval` with
  `$ScriptCommandLine = {...}; Get["<abs>.wls"]`, bounds stages via
  `scripts/run-bounded.py`, and writes a `summary.tsv`). Reuse the same rum
  invocation pattern and `RUM_CLI_BIN`/`--mma-hol`/`--output-dir`/`--timeout`
  conventions.
- The mma-hol snapshot tools (in the clone, read-only): `tests/build_snapshot.wls
  <uptoPattern>` (cold build through a layer -> `bootstrap.mx`),
  `tests/extend_snapshot.wls <file...>` (restore + load on top + re-dump),
  `tests/run_fast.wls <pat>` (restore + run a test file). `holContexts =
  Select[Contexts[], StringMatchQ[#, "HOL`*"] &]`; a pure re-dump is
  `Get[mx]; DumpSave[mx, Evaluate[holContexts]]`.
- Pristine clone: `/Users/fft/Developments/mma-hol-acceptance` (HEAD `200f13a`).
  HARD LINE: never write into it — snapshot builds write `bootstrap.mx` into the
  root, so the harness MUST operate on a writable scratch copy.

## Deliverable

- `scripts/snapshot-reliability.sh` with `--mma-hol PATH`, `--layers "Bool
  Parser ..."` (default a small fast set, see below), `--cycles N` (default 3),
  `--output-dir`, `--timeout`, `--rum-cli`/`RUM_CLI_BIN`, `--list`, `-h`. It must
  itself make (or accept) a writable scratch copy and NEVER touch the pristine
  clone; if `--mma-hol` points at the pristine acceptance path, it rsyncs to a
  throwaway scratch dir first (exclude `.git`, `*.mx`) and uses that.
- A `summary.tsv` and per-check stdout lines.
- No source change to `crates/**`. This is a script-only brief.

## The checks (precise)

Default layers (all load in ≤ ~2s; each has a test file): `Bool Drule Tactics
Printer Parser`. Map layer -> `build_snapshot` pattern and `run_fast` pattern
(lowercased), e.g. `Parser`/`parser`, `Bool`/`bool`.

For each layer L:

1. **Cold build:** `build_snapshot.wls L` -> `bootstrap.mx`. Record `bytes0`,
   the HOL name count (`Length[Names["HOL`*"]]`), and `gensym_count`
   (`Length[Select[Names["HOL`*"], StringMatchQ[#, "*$" ~~ DigitCharacter ~~ "*"]&]]`
   — or grep the dumped file for `$<digit>`). Run `run_fast.wls l` -> record the
   test summary line (`passed: P  failed: F`).
2. **Dimension B:** assert `gensym_count == 0` in the dumped snapshot (grep
   `'\$[0-9]'` on the `.mx` is acceptable). FAIL the layer otherwise.
3. **Dimension A cycle:** repeat N times: `Get[mx]; DumpSave[mx, Evaluate[
   holContexts]]` in place; record `FileByteCount`. Assert every cycle's byte
   count equals `bytes0` (BYTE-IDENTICAL). After the cycles, run `run_fast.wls l`
   again and assert the summary equals the cold `passed/failed`. FAIL otherwise.
4. **Dimension A fidelity:** also build L by increment — cold-build the PRIOR
   layer, `extend_snapshot.wls <L's file>` onto it, then compare the HOL name
   count and the `run_fast` summary to the cold-L values. Assert equal. (If L is
   the first layer with no prior, skip fidelity for it and note `n/a`.)

Emit one `summary.tsv` row per layer:
`layer  cold_bytes  gensym_count  cycle_byte_stable(yes/no)  tests(P/F)  test_stable(yes/no)  fidelity(ok/na/FAIL)`
Exit 0 only if every row passes (gensym_count==0, cycle_byte_stable==yes,
test_stable==yes, fidelity!=FAIL); else exit 1 and print which layer/check failed.

## Expected GREEN baseline (post-021/022 — paste your real run)

With the current tree, expect (approximately): Parser cold snapshot ~280-300KB
(NOT ~11.9MB), `gensym_count == 0`, every cycle byte-identical, `parser` tests
`78/0` stable across cycles, fidelity ok. The other layers similar with their own
test counts. If any layer shows growing cycle bytes or non-zero gensyms, the
021/022 fixes regressed — STOP and report (do not "fix" it in the harness).

## Scope

- Files you MAY modify/create:
  - `scripts/snapshot-reliability.sh` (new).
  - You MAY reuse/read `scripts/run-bounded.py` and `scripts/mma-hol-bounded-
    smoke.sh` but do not modify them.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone (operate on a scratch copy only; never
    write to `/Users/fft/Developments/mma-hol-acceptance`);
  - any `crates/**` source (script-only brief);
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your
    own report, `.git`, git config, `Cargo.lock`.

## Pitfalls (read twice)

1. **Scratch only.** The build writes `bootstrap.mx` into the mma-hol root —
   confirm with `git -C /Users/fft/Developments/mma-hol-acceptance status
   --short` (must stay empty) after your runs.
2. **Byte-identical, not "small".** Dimension A asserts EQUAL byte counts across
   cycles, not just "not growing". A single `+80`/cycle is a FAIL (that was the
   BUG-RT symptom).
3. **Use the real rum invocation.** Replicate `mma-hol-bounded-smoke.sh`'s
   `rum-cli --eval '$ScriptCommandLine = {...}; Get["<abs>"];'` form; the `.wls`
   tools need `$ScriptCommandLine` and an absolute `Get`.
4. **`extend_snapshot` throws if you reload a file already in the snapshot** —
   for fidelity, cold-build the PRIOR layer (pattern = prior file's stem), then
   extend exactly L's file once.
5. **Keep it fast.** Default to the 5 fast layers and N=3; the whole run should
   be well under a couple of minutes. Do not include `Pair` or later in the
   default set (those are the slow gate; the harness may ALLOW them via
   `--layers` but must not default to them).
6. **Stop-loss:** if the baseline is not green (growing bytes or gensyms), STOP
   and report — that means a real regression, not a harness bug.

## Verification (MANDATORY — run, paste verbatim)

```
bash scripts/snapshot-reliability.sh --mma-hol /Users/fft/Developments/mma-hol-acceptance --output-dir /tmp/cdx023-snaprel
cat /tmp/cdx023-snaprel/summary.tsv
echo "exit=$?"   # capture the script's exit (0 = all green)
git -C /Users/fft/Developments/mma-hol-acceptance status --short   # MUST be empty (pristine untouched)
```

Paste the full per-check stdout, the `summary.tsv`, the exit code, and the
pristine-status check (empty). Also paste a `--list`/`-h` snippet showing the
flags.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone; scratch copy only.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo (except the named
  `/tmp` scratch + output dir), no network. Commands: file edits within Scope,
  `bash`/`rsync`/`grep`/`ls`, the release `rum-cli`, `cargo build -p rum-cli`
  if needed, `python3`.
- Minimal, self-contained script; match the existing harness style. No drive-by
  edits to other scripts.
- You MUST paste real verbatim output; if not green, say so explicitly — never
  fake it.

## Report format (final message)

1. What changed — the new script, its checks, and the flag surface.
2. Verification — the full run output, `summary.tsv`, exit code, pristine-status
   check (empty).
3. The verdict — confirm all default layers are green (byte-identical cycles,
   gensym_count 0, stable tests, fidelity ok) on the current tree.
4. Stop-loss / open questions — anything unsure (empty if none).
