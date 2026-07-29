# Brief 299 — add positional Part assignment microbench labels (baseline only)

Step 2 of `docs/dev/codex/plan-297-part-assignment.md`. **Measurement only: no
Rust changes, no speedup expected.**

## Goal

`lst[[i]] = v` started working in `88625df` (it previously failed silently).
The suite has no label for it, so the planned fast path has no A/B baseline.
Add four labels and record the working baseline.

## THE CALIBRATION REQUIREMENT — read this first

Every performance number ever recorded for this workload before `88625df`
measured a **failing** path: the assignment did nothing and the cost was a
rebuild-then-reject. Those numbers are void. The probe that produced them
contained a calibration point (`Print[qL[[200]]]`, which printed `0` instead
of `200`) that was never read.

Worse, the scaling check this project relies on did **not** catch it: the
failing path is genuinely O(m), so the timing scaled correctly with the
parameter while measuring entirely the wrong thing.

Therefore this brief requires: **prove that each new label measures a
successful assignment**, not a failure. A label whose statement silently does
nothing would look perfectly plausible in `timings.tsv`. Show, outside the
suite, that the exact statement in each label leaves the expected value in the
list, and that no message is emitted. State this per label in your report.

## Context pointers

- Read `AGENTS.md`.
- `docs/dev/codex/plan-297-part-assignment.md` §6 Step 2 has the design.
  Follow **this** brief where they differ.
- `brief-292` (`df9985d`) is the precedent for adding labels to this suite;
  its heredoc constraints apply here too.
- The suite times with `First[AbsoluteTiming[...]]` (monotonic). **Do not
  introduce `AbsoluteTime[]` differencing** — removed in `3e42c09`.

## Scope

- File you MAY modify: **`scripts/core-language-microbench.sh`** — only the
  generated benchmark source (the `cat > "${bench_wls}"` heredoc) and its
  `ClearAll[...]` list. Do not change the shell driver, TSV emission, awk
  parsing, or the profiling-env list.
- Files you MUST NOT touch: **every `.rs` file**, every other script,
  `Cargo.*`, `docs/**`, `AGENTS.md`, `CLAUDE.md`, `.git`.
- The heredoc is **unquoted** (`<<EOF`), so the shell expands `$` and
  backticks inside it. Any statement you add MUST NOT contain `$` or a
  backtick. Verify with `bash -n` and by inspecting the generated `.wls`.

## Deliverable

Four `rumMicroOnce` labels, added at the **very end** of the generated source,
immediately before `Print["RUM_CORE_MICRO_END"]` (same placement rationale as
`df9985d`: heavy allocation last, so it cannot perturb the cheap labels).

| Label | Shape |
|---|---|
| `set-part-fixed100-len100` | 100 assignments to position 1 of a length-100 List |
| `set-part-fixed100-len800` | 100 assignments to position 1 of a length-800 List |
| `set-part-fill-small` | fill positions 1..200 of a length-200 List |
| `set-part-fill-large` | fill positions 1..400 of a length-400 List |

Two structural requirements:

1. **Initialize each dedicated top-level List in its own statement *before*
   the label, outside the timed region**, so the fixed-work pair varies only
   in List length and never times `Table`. Positional assignment does not
   change the list's length, so a single initialization is correct across all
   8 evaluations (1 warm + 7 timed) — each repetition performs identical work.
   Give each label its own symbol (prefix `gMicro`) and add them to
   `ClearAll[...]`.
2. **The pairs must differ only in the varied parameter** — length for the
   fixed-work pair, n for the fill pair. Their quotients are what the fast
   path will be judged by, so any other difference destroys the measurement.

## Verification (MANDATORY — you run it; iterate to evidence)

1. **Calibration, per label** (see above). For each of the four statements,
   run it standalone and assert the resulting list holds the expected values
   and that no message was emitted. This is the most important check in this
   brief.
2. **Script integrity.** `bash -n`, then run the suite and inspect the
   generated `.wls`: no unintended `$`/backtick expansion.
3. **Complete table.** Two runs into separate directories; `timings.tsv` must
   have **49 label rows** (45 existing + 4 new) plus header; `runs.tsv` shows
   `status=ok`, `exit_code=0`.
4. **Existing 45 labels undisturbed.** Compare against a run of the
   *unmodified* script. Report median change and direction split. **A lopsided
   direction split is NOT evidence of a regression** — this suite has a
   per-run whole-run offset that moves all labels together (two runs of the
   identical script measured -4.07% with 7/38 up; see `df9985d`). Judge by
   within-run quotients or against specific untouched labels; **never
   normalize against the suite median**, which drifts cumulatively in an
   optimization lane (`cc86cdd`, `a904935`).
5. **Cross-check against standalone probes.** Claude measured, min-of-5,
   `AbsoluteTiming`, on an idle machine:

   ```
   fixed 100 ops, len 100   0.289 ms
   fixed 100 ops, len 800   0.830 ms      (len800/len100 = 2.87x)
   fill n=200               0.746 ms
   fill n=400               1.811 ms      (large/small = 2.43x)
   ```

   Your suite values should land in the same ballpark. Report both quotients —
   they are the two signals the fast path will be judged by, and both are
   within-run quotients, hence immune to the per-run offset.

   Note: **do not expect the fill quotient to be near 4.** The plan text says
   "about 4", but that came from the *failing* path. The real working baseline
   is ~2.43x, because a fixed per-operation evaluator cost dominates the O(m)
   copy at these sizes.

6. **Runtime budget.** Report suite wall time before and after. Expected
   addition is roughly 26 ms of timed work at 7 reps; the timeout is 180 s.
7. All `RUM_PROFILE_*` off for timing; the driver already forces this.
8. `cargo fmt --check` and `cargo test` are **not** gates here — you change no
   Rust. Say so explicitly rather than running them.

Persist raw output to files before parsing; report raw rationals plus decimals.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- **Change no `.rs` file.**
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Do not add a second timing harness to the repo; standalone probes live in
  `/tmp` and stay there.
- Do not "improve" existing labels, the min-of-reps logic, warm-up, or the
  emission format while you are in the file.
- Touch nothing outside this repo except `/tmp`. No network.
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (your final message)

1. **The calibration result per label** — the exact statement, the resulting
   list state, and confirmation no message was emitted. Put this first.
2. The four new rows from `timings.tsv` (raw rational + decimal).
3. Both quotients (len800/len100, fill large/small), with your reading.
4. The 45-label comparison against the unmodified script: median, direction
   split, and whether anything looks systematic rather than per-run offset.
5. Suite wall time before/after and headroom.
6. Cross-check against the standalone figures above.
7. Confirmation that `git status` shows exactly one modified file.
