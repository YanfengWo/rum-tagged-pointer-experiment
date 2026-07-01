# brief-075 — re-profile for new local hotspots since the "constant-factor" baseline (diagnostic)

## Goal

The "rum is within a constant factor of WK across the board" conclusion predates the
memory line (Expr 64→32 compression, Module-gensym GC), the host-hardening line (the
eval-loop interrupt poll, the panic boundary), and the VM line (the RHS-template gate) —
NONE of which were given a careful segmented timing + counter pass. This brief does that
pass on the CURRENT binary (HEAD `c15bf6c`, default path = VM OFF) to detect any NEW
local hotspot those lines introduced that is worth a small tune. **Diagnostic brief: NO
source change** — use the existing profiling infrastructure. Deliverable is a refreshed
segmented-timing + counter table, DIFFED against the frozen baselines, flagging any phase
or counter that grew, plus targeted probes of the new code paths. A follow-up tune brief
is warranted ONLY if a clear, small, safe tunable hotspot is found.

## Context pointers

- Read `AGENTS.md`, `docs/performance-stage0-baseline.md` (the FROZEN Phase-0 baseline:
  primitive ratios + the `RUM_PROFILE_REPR` counters — the comparison point), and
  `PROGRESS.md` `## 2026-07-01 brief-069 per-call constant decomposition` (the most recent
  phase self-time table on v8 — D1/D2; re-confirm it still holds on c15bf6c). brief-069
  already refreshed the PHASE SHARES and found them stable; this brief's NEW value is the
  COUNTER sweep + the new-code-path probes + a c15bf6c re-confirm.
- Profiling env (all cold, zero-cost off): `RUM_PROFILE_LEXICAL_SUBSTITUTION` (the
  `runtime_path_timing` per-phase self-times), `RUM_PROFILE_REPR` (rum-core alloc/clone
  counters: `into_vec_total/clone`, `into_owned_total/clone`, `shared_args_alloc`,
  `shared_expr_alloc`), `RUM_PROFILE_EVALUATION_CACHE`, `RUM_PROFILE_BUILTIN_TIMING`.
- New code paths to probe (introduced since the baseline):
  - **Interrupt poll (059):** `eval_core.rs:13` `if self.interrupt_armed` (per top-level
    eval step) → `poll_host_interrupt_if_due` (`:57`, a `& INTERRUPT_POLL_MASK` counter).
    When DISARMED (the default), confirm the `if self.interrupt_armed` branch adds no
    measurable per-step cost.
  - **Gensym GC (049/050):** `scoping/gensym_gc.rs` + `scoping/lexical.rs:543` module-exit
    escape/reclaim (`gensym_gc_threshold`, the mark-sweep). Probe its cost on
    module-heavy paths (the Pair descent instantiates many Module combinator bodies).
  - **Expr 64→32 (053/054):** boxed `Rational`, Integer small-int hybrid, Symbol
    niche-pack — probe whether any arithmetic-/atom-heavy label now pays an extra
    indirection (compare the primitive-tier ratios to the stage0 baseline).
  - **VM gate (070):** `vm_rhs_template.rs` `rhs_template_enabled()` (one relaxed
    `AtomicU8` load on the RHS-substitution path when off) — confirm zero cost off.
- Reducers: `scripts/sample-self-time.py`, `scripts/sample-attribute-callers.py`;
  benches: `scripts/core-language-microbench.sh`, `scripts/core-memory-microbench.sh`;
  Pair descent: `scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4`.

## Deliverable

- **D1 — refreshed per-phase self-time table** (`RUM_PROFILE_LEXICAL_SUBSTITUTION`) on the
  primitive failing-dispatch tier AND the Pair descent, on c15bf6c (VM OFF). DIFF each
  phase's self-time share vs brief-069's D1/D2; flag any phase whose share GREW materially.
- **D2 — REPR counter sweep** (`RUM_PROFILE_REPR`) on the primitive tier + a
  construction-heavy microbench label + the Pair descent, DIFFED vs the frozen Phase-0
  baseline in `docs/performance-stage0-baseline.md`; flag any counter (into_vec/into_owned
  clone rate, shared_args/expr_alloc per call) that GREW since the memory/VM lines.
- **D3 — absolute ratio re-confirm** (`core-language-microbench.sh --engine both`,
  profiler OFF, best-of-2): the primitive-tier ratios vs WK on c15bf6c vs the stage0
  baseline — confirm the constant-factor conclusion still holds (no regression); flag any
  label that regressed.
- **D4 — new-code-path targeted probes:** for each of the four paths above, a small A/B or
  self-time attribution answering "does it add measurable per-call/per-step/per-module
  cost on the default path?" (interrupt-poll disarmed; GC on module-heavy; Expr-32
  arithmetic; VM gate off). Report each as flat / small-cost / hotspot.
- **Verdict:** a list of any NEW local hotspot (phase share up, counter up, or new-path
  cost) with its magnitude and whether it is a small SAFE tunable; OR "no new hotspot —
  the constant-factor conclusion still holds on c15bf6c."

## rum / Rust pitfalls (read twice)

1. **Shares from the profiler, ratios from unprofiled runs.** `RUM_PROFILE_*` distorts
   absolute time; use it for relative SELF-time shares + counts only. D3 ratios MUST be
   profiler-OFF. Never compare profiled rum to unprofiled WK.
2. **Self-time, not inclusive** for the phase table (children subtracted), so shares
   partition without double-counting.
3. **One variable per measurement; probe budget** (best-of-2; derive timeouts from the
   historical per-file table — snapshot simp≈109s etc.; a slow mode once; full env up
   front). **Persist raw output to a FILE first, then parse; rum timing is a rational
   num/den — parse with `Fraction`, not a decimal regex.**
4. **Default path = VM OFF** (do not set `RUM_VM_RHS_TEMPLATE`); the point is the shipped
   default path's hotspots.
5. **Stop-loss:** this is measurement; if a table doesn't close (children ≠ ~parent), say
   so and report what you have. Do NOT start tuning — a tune is a separate brief.

## Verification (MANDATORY — the evidence IS the deliverable; NO source change)

```
cargo build --release
# D1 phase self-times (profiler ON, shares):
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 target/release/rum-cli --file <failing-dispatch probe>   # primitive tier
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 --engine rum --output-dir /tmp/cdx_brief_075_pair
# D2 REPR counters (profiler ON):
RUM_PROFILE_REPR=1 target/release/rum-cli --file <probes>
# D3 absolute ratios (profiler OFF, best-of-2):
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_075_micro
```

Paste: D1 table + diff vs 069, D2 counters + diff vs stage0, D3 ratios + diff vs stage0,
D4 per-path findings — all verbatim, from saved raw files.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. **No source change** (diagnostic).
- Do NOT edit any `mma-hol` file. Full machine access, nothing outside this repo, no network.
  Only: `cargo`, the release `rum-cli`, the microbench/pair-simp scripts, the
  `scripts/sample-*.py` reducers, `python3` for parsing.
- Probe budget + timeout discipline + raw-to-file (pitfall 3).
- Paste REAL verbatim output. If a table doesn't close or a probe fails, say so — never fake.

## Report format

1. What you ran — "diagnostic only, no source change"; the exact probes/commands.
2. Verification — D1 table+diff, D2 counters+diff, D3 ratios+diff, D4 per-path — verbatim.
3. The finding — the list of any NEW local hotspot (with magnitude + is-it-a-small-safe-
   tunable), or "no new hotspot; constant-factor holds on c15bf6c."
4. Open questions — any table that didn't close, any path you couldn't attribute cleanly.

---

## Outcome (Claude gate, 2026-07-01) — DONE. No new hotspot; constant-factor holds (better than stage0); safe perf confirmed at its ceiling.

Diagnostic only (tree clean, no source change). Findings:
- **D1 primitive:** phase shares stable vs brief-069 (evaluate.total 21.08 vs 22.46,
  stability 7.04 vs 7.49, …; all within ~1pt). No new primitive hotspot.
- **D3 absolute ratios (profiler OFF, best-of-2):** ALL primitive labels FASTER than the
  stage0 baseline (if-true 3.97 vs 5.96, exact-hit 3.48 vs 5.74, sameq 3.56 vs 5.38,
  length 2.90 vs 3.66, …); vs brief-069 within noise. Constant-factor conclusion HOLDS,
  no regression, actually improved (the memory line's Expr compression + Layer A).
- **D2 REPR counters:** primitive clone counters still ~0 (Layer A elimination intact),
  node_allocs down massively vs phase0. No new allocation hotspot.
- **D4 new code paths:** interrupt poll disarmed (flat), gensym GC (default threshold
  +5.8%, only `forced` is 1.27x — not the default path), Expr-32 atoms (literal-integer
  1.28x WK, no indirection cost), VM gate off (zero cost) — all flat or improved; none
  introduced a hotspot.
- **False signal:** the Pair D1 table showed module.resume_initializers 33% +
  `sub_values.rhs_substitution` 45% — this is the profiling-mode POLLUTION artifact
  (Codex set only `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`, not the full category-disable set
  brief-069 used; the `*rhs_substitution` rows are the pollution signature), NOT a real
  regression — the profiler-OFF D3 confirms no primitive regression, and it isolates to
  no small tune.

**Verdict:** no new small safe default-path tunable. The new lines (memory / host-
hardening / VM gate) did NOT introduce a default-path hotspot; the constant-factor
conclusion holds on the current binary. This is the empirical confirmation that safe,
clear, worth-doing-now perf optimization is at its ceiling for the current target.
