# Brief 292 — add incremental container-update microbench labels (baseline only)

Step 1 of the plan in `docs/dev/codex/plan-291-assoc-append-ownership.md`.
**Measurement only: no Rust changes, no runtime win expected.**

## Goal

`scripts/core-language-microbench.sh` currently has **no label** covering
incremental container update, so the planned work on `AppendTo` accumulation
and Association key-by-key construction has no A/B baseline at all. Add seven
labels that cover those workloads, then record the baseline numbers. When you
are done the suite still passes, the pre-existing 38 labels are unchanged in
behavior, and we have a committed baseline to compare future work against.

This brief deliberately delivers **no speedup**. Do not attempt any
optimization; a change to any `.rs` file voids the work.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- `docs/dev/codex/plan-291-assoc-append-ownership.md` §6 has the label
  proposals this brief is derived from. Follow **this** brief where the two
  differ; the differences are deliberate and explained inline below.
- The suite times with `First[AbsoluteTiming[...]]`, which reads the kernel's
  monotonic clock. **Do not introduce `AbsoluteTime[]` differencing** — that
  was removed in `3e42c09` and the comment above `rumMicroBench` says why.
- `rumMicroBench` runs the bare expression n times inside one timed region;
  `rumMicroOnce` times the expression once per rep. Both take min-of-7 after
  one untimed warm eval.

## Scope

- File you MAY modify: **`scripts/core-language-microbench.sh`** — and within
  it, only the generated benchmark source (the `cat > "${bench_wls}"`
  heredoc). Do not change the shell driver, the TSV emission, the awk
  parsing, or the profiling-env list.
- Files you MUST NOT touch: **every `.rs` file in the repo**, every other
  script, `Cargo.*`, `AGENTS.md`, `CLAUDE.md`, `TODO.md`, `PROGRESS.md`,
  `docs/**` (including the plan document), `.git`.
- The heredoc is **unquoted** (`<<EOF`), so the shell expands `$` and
  backticks inside it. The existing body relies on this for `${scale}` and
  `${reps}`. Any statement you add MUST NOT contain `$` or a backtick unless
  you intend shell expansion. Verify with `bash -n` and by inspecting the
  generated `.wls` file.

## Deliverable

Seven new labels, added **at the very end of the generated benchmark source**
— immediately before `Print["RUM_CORE_MICRO_END"]`, after
`character-range-small`.

*(This placement differs from plan §6, which suggested grouping them with the
basic list/Part labels. Put them last: these labels build 500-element lists
and a 1000-element outer table, and placing heavyweight allocation ahead of
the existing cheap labels risks perturbing them. Last means there is no
"after" to perturb.)*

All seven use `rumMicroOnce` (each is one stateful loop, timed once per rep).

| Label | n | Workload |
|---|---|---|
| `appendto-list-incremental-small` | 250 | `AppendTo` accumulation onto a flat list |
| `appendto-list-incremental-large` | 500 | same, 2x |
| `appendto-part-outer1000-small` | 100 | `AppendTo[outer[[1]], i]` with outer size 1000 |
| `appendto-part-outer1000-large` | 200 | same, 2x |
| `association-set-key-incremental-small` | 100 | `assoc[i] = i` accumulation |
| `association-set-key-incremental-large` | 200 | same, 2x |
| `association-small-literal-repeat` | 500 | small-Association evaluation sentinel |

Two requirements on how the workloads are written:

1. **Use a top-level symbol reset in the timed expression, not `Module`
   locals**, e.g. `(gAppendList = {}; Do[AppendTo[gAppendList, i], {i, 1, 250
   * rumMicroScale}])`. Plan §6 used `Module[{lst = {}}, ...]`. Measured
   difference between the two shapes is only 1–11% and shrinks as n grows
   (Module's fixed scope-setup cost), so this costs nothing in comparability —
   but a `Module` local keeps a module activation live for the whole loop,
   which adds a variable to exactly the predicate space the later steps will
   work in. Top-level symbols are also what real user code uses. The `sym = {}`
   reset is O(1) and does not dilute the signal. Give each label its own
   symbol name (prefix `gMicro`) and add them to the `ClearAll[...]` list at
   the top of the generated source.
2. **The small/large pairs must differ only in n.** Their ratio is the
   complexity indicator we will read later, so any other difference between
   the two arms destroys the measurement.

For the sentinel (`association-small-literal-repeat`), keep plan §6's intent:
a repeated small-Association evaluation whose key varies per iteration
(`<|a -> i, b -> 2|>`) so it does not degenerate into a pointer-identical
cache hit. It exists to catch a future optimization that speeds up large
Associations by making small ones slower.

## Verification (MANDATORY — you run it; iterate to evidence)

1. **Script integrity.** `bash -n scripts/core-language-microbench.sh`, then
   run the suite and inspect the generated `.wls` in the output directory:
   confirm no `$`/backtick was expanded unintentionally and the new statements
   read as intended.
2. **The suite still produces a complete table.** Run it twice into two
   different output directories. `timings.tsv` must have **45 label rows**
   (38 existing + 7 new) plus the header, and `runs.tsv` must show
   `status=ok`, `exit_code=0`.
3. **The pre-existing 38 labels must not be disturbed.** Compare the 38 old
   labels between your two runs AND against a run of the *unmodified* script
   (stash your change, run, restore). Report the per-label percentage change
   as a table. Movements are expected — this suite's noise floor is large and
   bidirectional — but you must state the median change and the direction
   split, and flag anything that looks systematic (e.g. all 38 moving the same
   way by a similar amount) rather than like noise.
4. **Cross-check the new labels against independent probes.** Write standalone
   probes (under `/tmp`) that measure the same seven workloads directly with
   `AbsoluteTiming`, min-of-5, and confirm the suite's numbers land in the same
   ballpark. A label whose suite value disagrees with a direct probe by more
   than the noise floor means the label is measuring something else.
5. **Verify the timings scale with n.** For each small/large pair, report the
   large/small ratio. `AppendTo` accumulation and Association key insertion are
   expected to be super-linear, so a ratio at or below 2.0 for a 2x n increase
   is a red flag that the label is not measuring the intended work — **this
   project has twice been burned by a probe that measured process startup or
   an already-evaluated value, and the tell both times was a number that did
   not scale with n.** Investigate rather than reporting such a ratio as fact.
6. **Runtime budget.** Report the wall time of one full suite run before and
   after your change (`elapsed_seconds` in `runs.tsv`). The suite's timeout is
   180 s; state the new total and confirm the headroom.
7. All `RUM_PROFILE_*` variables must be OFF for every timing number. The
   suite's driver already forces them off — do not weaken that list.
8. `cargo fmt --check` and `cargo test` are **not** required gates here: you
   change no Rust. Say so explicitly in the report instead of running them.

Persist raw output to files before parsing. Report raw rationals alongside
decimal conversions.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate and commits.
- **Change no `.rs` file.** Not one. If you believe a label cannot be
  expressed without a runtime change, STOP and report that instead.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no comparison against other engines, no oracle harnesses, no
  third-party engine output committed.
- Do not add a second ad-hoc timing harness to the repo. Standalone probes for
  cross-checking live in `/tmp` and stay there.
- Do not "improve" the existing labels, the min-of-reps logic, the warm-up, or
  the emission format while you are in the file.

## Report format (your final message)

1. The seven new label rows from `timings.tsv` (raw rational + decimal).
2. The large/small ratio for each of the three pairs, with your reading of
   whether each is consistent with the expected complexity.
3. The 38-label comparison table against the unmodified script: median change,
   direction split, and whether anything looks systematic rather than noisy.
4. Suite wall time before and after, and remaining headroom against the 180 s
   timeout.
5. The cross-check results (suite value vs standalone probe, per new label).
6. Confirmation that `git status` shows exactly one modified file
   (`scripts/core-language-microbench.sh`) and no other changes.
