# brief-069 — per-call constant decomposition on v8 (bytecode-VM go/no-go sizing)

## Goal

Produce the addressable-prize split for the bytecode-VM line (task-2 of the
reopened perf line) by RE-MEASURING the distributed per-call dispatch/eval constant
on the CURRENT v8 binary (post Layer-A + the 32-byte `Expr`), then classifying each
phase into three buckets so we can size what a VM buys OVER cheaper eval-strategy
fixes. The 2026-06-21 decomposition (PROGRESS `## 2026-06-21 Pair 34x ROOT CAUSE`)
was taken PRE-Layer-A and PRE-32-byte-`Expr`; Layer A (A1-A4) already cut the worst
labels ~13-17%, so those phase shares are stale. This is a **diagnostic brief: NO
source change** — the phase-timing infrastructure already exists
(`runtime_path_timing_start` / `record_profile_runtime_path_timing`, gated by
`RUM_PROFILE_LEXICAL_SUBSTITUTION`). The deliverable is a table + a classification,
with verbatim numbers. Nothing is built until this data is in.

## Context pointers

- Read `AGENTS.md`, `docs/bytecode-vm-plan.md` (the plan this brief serves — the
  three-bucket split and the feasibility crux are defined there), and PROGRESS
  `## 2026-06-21 Pair 34x ROOT CAUSE: distributed eval-loop constant (match is 5%)`
  (the old table you are refreshing) + `## 2026-06-21 Downstream re-measure: A3 did
  NOT move Pair` (why microbench ≠ Pair — you must report BOTH).
- Key files (NO edits — for locating the phase labels):
  - `crates/rum-runtime/src/runtime/profiling.rs:994` `runtime_path_timing_start`
    (gated by `lexical_profile_enabled()` = env `RUM_PROFILE_LEXICAL_SUBSTITUTION`);
    `record_profile_runtime_path_timing` accumulates child self-time. The phase
    labels are the `runtime_path_timing_start("…")` string literals across
    `crates/rum-runtime/src/runtime/` (e.g. `evaluate.total`,
    `evaluate.stability_check`, `evaluate.once_unstable`, `prepare_arguments.total`,
    `evaluate_call.dispatch`, `down_values.*`, `evaluate_inner_unstable.*`).
  - `scripts/pair-simp-hotspot-probes.sh` + probe `runtime-profile-basic-only-pair4`
    — the bounded Pair descent (the downstream denominator).
  - `scripts/core-language-microbench.sh` — the primitive tier for absolute ratios.
  - The 2026-06-21 numbers were taken on a "faithful failing-pattern dispatch" — a
    pattern downvalue applied to FRESH non-matching args each iteration (the
    stability cache cannot short-circuit; mirrors the Pair descent's fresh-subterm
    failing conversions). Reproduce that shape as the primitive profile subject.
    If you cannot find the exact prior probe, construct a minimal equivalent (a
    `SetDelayed` pattern rule that never matches, applied in a `Do` loop over fresh
    args) and say exactly what you used.
- Acceptance/source clone (READ-ONLY): `/Users/fft/Developments/mma-hol-acceptance`
  (HEAD `200f13a`) for the Pair probe. Never edit it.

## Scope

- Files you MAY modify: **NONE — diagnostic only.** The phase-timing infrastructure
  exists; you only set env vars and run. (If you find you must add a probe `.wls`
  under `scripts/perf-probes/`, that is allowed as a throwaway measurement input,
  but prefer the existing probes; say so if you add one.)
- MUST NOT touch: any `mma-hol` clone/standard; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except your own report; `.git`; any source file (no code change).

## Deliverable

- **D1 — per-phase self-time table on v8, primitive failing-dispatch tier.** With
  `RUM_PROFILE_LEXICAL_SUBSTITUTION` on, run the faithful failing-dispatch profile
  and report each phase's SELF-time share of the per-call total + its multiplicity
  per call (the columns of the 2026-06-21 table), refreshed on v8. Rank descending.
- **D2 — the same table on the Pair descent** (`runtime-profile-basic-only-pair4`),
  so the shares are validated on the real downstream workload, not just the
  primitive tier (A3 lesson: they can differ). Note any phase whose share differs
  materially between D1 and D2.
- **D3 — absolute anchor (profiler OFF).** The primitive-tier ratio vs WK
  (`core-language-microbench.sh`, both engines, profiler OFF) for 2-3 representative
  labels, so the shares in D1/D2 attach to a real absolute gap (profiled runs
  distort absolute time; shares are relative, ratios must come from unprofiled runs
  — see the measurement rule below).
- **D4 — the three-bucket classification** (map every D1/D2 phase onto the
  `docs/bytecode-vm-plan.md` buckets and sum each):
  - (i) eval-STRATEGY (no VM): `evaluate.total` fixed-point loop, `stability_check`,
    `once_unstable`, and any redundant re-evaluate/re-check multiplicity.
  - (ii) VM-SPECIFIC: `prepare_arguments.*` + `evaluate_call.dispatch` glue; PLUS a
    separate measurement of the WL-combinator re-interpretation amplification on
    Pair (the ~1.7x from `ORELSEC`/`REPEATC`/curried down-value bodies — quantify
    how much of the Pair descent is re-walking/re-dispatching rule RHS/combinator
    bodies vs primitive dispatch; this is the VM's real prize).
  - (iii) IRREDUCIBLE: `down_values`/match + the necessary arg evaluation.
  Report each bucket as a % of the per-call constant, on BOTH D1 and D2.

## rum / Rust pitfalls (read twice)

1. **Shares from the profiler, ratios from unprofiled runs.** The
   `RUM_PROFILE_LEXICAL_SUBSTITUTION` instrumentation distorts ABSOLUTE time;
   use it only for relative SELF-time shares (D1/D2). Any absolute ratio vs WK
   (D3) MUST be a profiler-OFF run. Never compare profiled rum to unprofiled WK.
2. **Self-time, not inclusive.** The bucket sums must use per-phase SELF-time
   (child time subtracted) so they partition the total without double-counting —
   `record_profile_runtime_path_timing` already tracks `child_nanos`; report the
   self-time column, not inclusive.
3. **Microbench ≠ Pair.** Report D1 and D2 separately and flag divergence; the
   go/no-go is driven by D2 (the real workload) where they differ.
4. **Measure internal `AbsoluteTiming`, never wall** for D3.
5. **Stop-loss.** This is a measurement; if a phase table does not close (children
   don't sum to ~parent), say so and report what you have — do not fabricate a
   clean partition.

## Verification (MANDATORY — the evidence IS the deliverable)

No source change → no test gate. Build release, then produce D1-D4. Paste every
command and its verbatim key rows (the phase self-time tables, the ratio lines, the
bucket sums).

```
cargo build --release
# D1 primitive failing-dispatch, profiler ON (shares):
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 target/release/rum-cli --file <your failing-dispatch probe>
# D2 Pair descent, profiler ON (shares):
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 --engine rum --output-dir /tmp/cdx_brief_069_pair
# D3 absolute ratio, profiler OFF:
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_069_micro
```

Paste: D1 table, D2 table (with divergence notes), D3 ratios, D4 three-bucket sums.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify any source file (diagnostic only). Do NOT edit any `mma-hol` file.
- Full machine access, nothing outside this repo, no network. Only commands:
  `cargo`, the release `rum-cli`, the microbench / pair-simp scripts, the
  `scripts/sample-*.py` reducers, `python3` for parsing.
- Paste REAL verbatim output. If a table does not close or a probe fails, say so —
  never fabricate a clean partition.

## Report format (your final message)

1. What you ran — "diagnostic only, no source change"; the exact probe/commands and
   the failing-dispatch subject you used.
2. Verification — D1 table, D2 table (+ divergence notes), D3 ratios, all verbatim.
3. The finding — the D4 three-bucket split (% of the per-call constant, on D1 and
   D2), and which bucket dominates → whether a VM's residual prize (bucket ii,
   after the strategy fixes in bucket i) justifies the multi-week build, per
   `docs/bytecode-vm-plan.md`.
4. Open questions — any phase that didn't close, or D1/D2 divergence you couldn't
   explain.

---

## Outcome (Claude gate, 2026-07-01) — DONE. Per-call constant confirmed stable; on Pair ~58% is VM-addressable, with a concrete 22% Module-closure hotspot. Do the cheap targeted levers before a full VM.

Diagnostic only, tree clean (no source change; Codex added only a throwaway
`/tmp` probe input). Gated by inspection; D1 matches history (the constant is
stable post-Layer-A), D3 is profiler-OFF, the combinator amplification is
unprofiled (both trustworthy).

**D1 (primitive failing-dispatch `fMiss[x_String]` on ints):** `evaluate.total`
22.46% (2/call), `prepare_arguments` 7.57%, `stability_check` 7.49% (4/call),
`once_unstable` 6.90%, `dispatch` 5.60%, `down_values` 5.31% — matches the
2026-06-21 table almost exactly. The per-call constant is DISTRIBUTED and stable.

**D3 (profiler OFF, vs WK):** if-true 3.81x, downvalue-exact-hit 3.77x,
downvalue-pattern-hit 3.79x, downvalue-pattern-**miss** 2.16x, miss-repeat 1.56x.
(The failing-dispatch path — the Pair descent's dominant op — is already only
1.56-2.16x WK; successful dispatch is ~3.8x.)

**Combinator amplification (unprofiled, `basic-chain-orelsec-split`):** rum
full/manual = **1.467x** (31.84% of the full path is WL control-combinator
overhead). Cross-engine manual 4.54x, full 6.13x. This is a solid VM-specific
prize.

**D2 (Pair descent, phase-only, closed to ~parent):** biggest single row is
`module.resume_initializers` **22.14%** (Module closures rebuilt/re-evaluated per
subterm), then `evaluate.total` 8.97%, `down_values.normalize_candidate` 7.76%,
`prepare_arguments` 5.02%, `stability_check` 4.93%, `dispatch` 3.22%.

**D4 bucket split:**

| bucket | D1 primitive | D2 Pair |
| --- | ---: | ---: |
| eval-strategy (no VM) | 49.31% | 21.12% |
| VM dispatch glue | 34.71% | 21.89% |
| VM WL body/combinator | 0.00% | 35.87% |
| irreducible | 15.97% | 21.12% |

**Verdict:** the A3 lesson holds — D1 (primitive) says the cheap eval-strategy
chunk dominates, but D2 (the REAL Pair workload) is ~**57.8% VM-addressable**,
dominated by WL-body re-interpretation (Module/function/rule bodies, incl. the
22% Module-closure row) + dispatch glue, corroborated by the 1.467x combinator
amplification. So a VM prize is real on Pair — but it must target **compiled WL
bodies/combinators**, NOT a generic expression-tree VM (which would miss it).

**Recommendation (per the plan's incremental principle — do NOT big-bang a VM):**
the decomposition surfaces concrete, likely-cheaper-than-a-full-VM levers to take
FIRST, each Pair-validated: (1) the **22% Module-closure resume** on Pair (avoid
rebuilding/re-evaluating the Module closure per subterm — a targeted fix, not a
VM); (2) the **eval-strategy fixed-point/stability multiplicity** (2x evaluate +
4x stability per call, ~21% Pair / ~49% primitive — no VM). Bank these, re-measure
the Pair descent, THEN size the residual compiled-body VM prize. A full
compiled-body VM is the multi-week step 3 in `docs/bytecode-vm-plan.md`, justified
only by the residual after (1)+(2). Next brief candidate: the Module-closure
resume lever (biggest single Pair row, self-contained).
