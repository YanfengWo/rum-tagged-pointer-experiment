# Codex Task Brief Template (rum)

Claude owns direction, diagnosis, and target selection; Codex executes one
brief exactly and self-verifies; Claude runs the authoritative gate afterward
and commits. Copy this file to `docs/dev/codex/brief-NNN-<slug>.md`, fill every
section, then dispatch:

```
cdx exec -C "$(git rev-parse --show-toplevel)" -s danger-full-access \
  "Read docs/dev/codex/brief-NNN-<slug>.md and execute it exactly. Follow its Hard rules." \
  > /tmp/cdx_brief_NNN.log 2>&1
```

`-s danger-full-access` lets Codex run `cargo`, the release `rum-cli`, and the
macOS `sample` profiler on the real machine (a sandbox blocks process
attach / profiling). The brief's **Scope** and **Hard rules** are the real
fence: full machine access plus "touch nothing outside this repo, no network,
only the listed commands."

Two brief shapes use this template:

- **Diagnostic brief** — profile/measure and report; no source change. The
  first brief in any performance lane is almost always this: Claude cannot
  prescribe a precise change before the profile exists.
- **Implementation brief** — a precise, pre-diagnosed mechanical change with a
  test + measured-ratio gate.

---

## Goal

One paragraph. What must be true when you are done. For a diagnostic brief,
the deliverable is evidence, not a code change.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` hard line) and
  the relevant `PROGRESS.md` / `docs/` entries named below before touching
  anything.
- Key files for this task: <list with one-line roles>.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. Acceptance
  is mechanical: the named tests and the named benchmark ratio decide, not
  prose.

## Scope

- Files you MAY modify: <explicit list, or "NONE — diagnostic only">.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` standard — NEVER edit
    the acceptance standard to pass it (see `AGENTS.md`); diagnostic edits go
    only in throwaway scratch clones, never the canonical clone, and are never
    left behind;
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation and any public
    type/serde shape, unless this brief's Goal is exactly that;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, anything under
    `docs/dev/codex/` except appending to your own report;
  - `.git`, git config, `Cargo.lock` (unless the Goal is a dependency change).
- Do NOT add unconditional cost to the default evaluation path. Any profiling
  counter or instrumentation MUST be behind an env-gated cold path (the
  existing `RUM_PROFILE_*` `AtomicU8` pattern in `evaluation_cache.rs` /
  `profiling.rs`); never a `Mutex`/allocation/branch paid on every call when
  the flag is off. A default-path regression voids the change.

## Deliverable

Bullet list of concrete artifacts (for diagnostic: the tables/numbers to
produce; for implementation: code + test + the before/after ratio).

## rum / Rust pitfalls (read twice)

1. **Measure internal `AbsoluteTiming`, never wall time.** `rum-cli` / engine
   startup is large and contaminates wall comparisons. Compare rum internal
   timing to WolframKernel internal timing only.
2. **Never compare profiled rum to unprofiled WolframKernel.** Profiling env
   vars must be OFF for any timing number; they are for counts/attribution
   only.
2b. **A probe must be the label's EXACT statement — nothing added.** Do not
   wrap the timed expression in an assignment, a sink `Set`, a `Print`, or any
   extra builtin to "use" the result. The microbench label is `Do[expr, {n}]`;
   a probe of `Do[sink = expr, {n}]` measures a different thing (the `Set`
   alone injects one ownvalue write + cache invalidation per iteration, which
   dominated counters and inflated per-iteration time ~70% in brief-001). If
   you need the loop not to be elided, raise the iteration count; do not add a
   sink. Replicate the generated `core-language-microbench.wls` shape verbatim.
3. **Default-path cost is unconditional cost.** A counter that does an atomic
   load per call when disabled still costs; keep the disabled path a single
   relaxed load that returns early (see existing pattern). No `Mutex`, no
   allocation, no per-call heap touch when the feature is off.
4. **Evaluator flows are suspension-safe.** If you touch an `eval_*` /
   `resume_*` path, preserve the suspend/resume continuation shape; do not
   collapse a `suspend_with_frame*` into an eager call.
5. **Hot paths avoid owned-tree clones.** `Expr`/`SharedExpr`/`SharedArgs` are
   `Arc`-shared; prefer borrowed traversal and `Arc` clone over deep clone on
   any path that runs per evaluation step.
6. **One variable per measurement.** Change one thing, rerun the SAME label
   plus its same-tier neighbors. An optimization that helps one row and
   regresses a neighbor in the same tier is rejected.
7. **Stop-loss is real.** If the same step fails twice for the same reason, or
   the measured ratio does not move, STOP and report — do not keep tuning.

## Verification (MANDATORY — you run it; iterate to green / to evidence)

Diagnostic brief: produce the named artifacts and paste the key rows VERBATIM
(e.g. the top self-time lines, the caller-attribution table, the counter
totals). No source change means no test gate; the evidence IS the deliverable.

Implementation brief, run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test -p rum-runtime            # or the narrower module the brief names
```

Then the performance gate the brief names, e.g.:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_NNN_micro
# paste the ratios.tsv rows for the target label AND its same-tier neighbors
```

Loop: edit → `cargo test` → fix → rerun, until tests are green; then rerun the
benchmark and paste the before/after ratio for the target label. Never claim a
result without the verbatim count / ratio line.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Full machine access, but touch NOTHING outside this repo, no network. The
  only commands you run are file edits within Scope, `cargo`, the release
  `rum-cli`, `sample` / the `scripts/sample-*.py` reducers, the microbench
  script, and `python3` for parsing.
- Never edit any `mma-hol` file as the fix or the acceptance path.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe` today (the
  only `unsafe` in the repo is the off-by-default alloc-count `GlobalAlloc` in
  `rum-cli`). Do NOT introduce `unsafe` (incl. `ManuallyDrop::take`/`drop`,
  `ptr::*`, transmute, raw-pointer deref) into these crates. There is almost
  always a safe equivalent at the same cost (e.g. `Option<Arc<…>>` niche-packs to
  the same size as `Arc<…>` and gives a safe `.take()` for manual `Drop`). If you
  are convinced `unsafe` is unavoidable, STOP and report — do not ship it.
- **Hot-path changes require a microbench A/B, not just the "no default-path cost"
  rule.** If the change touches an evaluation hot path (eval/dispatch/matcher,
  `Expr`/`SharedExpr`/`SharedArgs` construction or `Drop`, the down/up/sub-value
  candidate path), you MUST run `scripts/core-language-microbench.sh` base-vs-new
  and paste the per-label ratios — an O(N)/per-node cost hidden in a "fast path"
  is exactly what a verbal rule misses. A regression on any hot label is a defect
  to fix or report, not to ship.
- **Probe budget — keep gate runs bounded (added 2026-07-01 after briefs 067-069
  each ballooned to 35-70 min of probe time).** The Pair-descent probe
  (`runtime-profile-basic-only-pair4`) is ~90-270s per run (2.5-2.7x-slow
  measurement modes push single runs to 250-270s); best-of-N × several modes
  silently balloons. So: use **best-of-2, not N≥3**, for a wall A/B; run any
  2x+-slow mode **ONCE** (it is defining, not a best-of candidate); **cap the total
  probe-run count and state the cap** in your report; set the **FULL env** (every
  `RUM_PROFILE_*` category you need on/off) up front so you never re-run on polluted
  data; and if a number looks off, **REPORT it — do NOT silently re-measure every
  mode again** (that is what turned brief-067 into 27 runs). For a quick RELATIVE
  A/B prefer a cheaper bounded probe (e.g. `runtime-profile-loop-null-repeat`) and
  reserve the full descent for the final confirm. Pure share/decomposition
  diagnostics need ONE clean profiled run (shares are stable within a run), not a
  best-of-N.
- Stop-loss: same failure twice, or no ratio movement → STOP and report.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside
  the task.
- You MUST paste the real verbatim verification output. If a run did not reach
  green / did not produce the evidence, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why (or "diagnostic
   only, no source change").
2. Verification — each command run, with the exact verbatim output lines
   (test counts, ratios, or attribution tables).
3. The finding / verdict — for diagnostic, the mechanism the numbers point to;
   for implementation, the before/after ratio and any neighbor checked.
4. Stop-loss / open questions — where you stopped and why, anything unsure
   (empty if none).
