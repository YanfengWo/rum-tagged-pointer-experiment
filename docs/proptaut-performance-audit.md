# PropTaut Performance Audit And Hotspot-Diagnosis Method

Date: 2026-06-11. Audited tree: rum `6d817e1` plus the uncommitted evaluator
fast-path diff present that evening. Workload: clean `mma-hol` HEAD `698e3d0`
clone at `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`. Machine: the same
x86-64 macOS box used for the Bool lane numbers in `PROGRESS.md`.

Maintainer note, 2026-06-12: follow-up retained optimizations after this audit
reduced the same PropTaut distribution probe to about `27.40-27.64s` in
release, but the local WolframKernel internal timing remains about `3.80s`.

Second maintainer note, 2026-06-12 (external representation pass): symbol
interning, dependency-aware cache invalidation, mimalloc, and the related
structural changes recorded in `PROGRESS.md` under "Representation
Optimization Pass" brought the probe to about `13.1s` (3.4x) and the
Bool-stack load to `0.78s` (4.1x). `proptaut_tests.wl` is green for the
first time. The remaining gap is flat across the evaluator; section 4's
item 5 (per-node evaluated bits / single-allocation call nodes) is now the
frontier. Use this report as diagnosis history.

Third maintainer note, 2026-06-12: the same-bit stability-invalidation
follow-up retained a conservative rewritten-dirty-bloom fix and moved the
distribution probe to about `12.73-12.88s`; `proptaut_tests.wl` remains green
at about `239s`. Exact per-symbol/per-bloom-bit version snapshots and
dirty-entry cache skipping were measured and rejected because they regressed
the real PropTaut workload. The next frontier remains the larger call-node /
evaluated-stamp representation work.

This audit answers two questions: where the `PropTaut` time actually goes,
and why a night of sample-guided micro-probes produced mostly neutral or
negative results. The short version: the profile was being read in a form
that cannot localize cost in a deeply recursive evaluator, and the
cross-engine baseline used for the Bool lane was startup-contaminated.

## 1. Corrected baselines

All numbers are internal `AbsoluteTiming` unless stated otherwise.

| Stage | rum (release) | WolframKernel | Ratio |
| --- | --- | --- | --- |
| Load `ErrorUtil` … `Bool.wl` | 2.18s | 0.190s | 11.5x |
| `propTaut` distribution theorem | 48.7-50.1s | 3.80s | ~13x |

Two corrections to the working assumptions in `PROGRESS.md` / `TODO.md`:

- The "local WolframKernel reference of roughly 2.4s" for the Bool stack is
  startup-dominated wall time. `wolframscript` startup alone is ~3.2s on this
  machine; the kernel-side load of the full Bool stack measures **0.19s**.
  The Bool lane therefore never reached parity — it reached rum-wall ≈
  WK-startup. The honest evaluator gap is ~11-12x there too.
- Consequently `PropTaut` is not a new pathological cliff. The ~13x on the
  distribution theorem is the same systemic constant factor, surfacing on a
  workload large enough to hurt. Fixes should target the systemic factor,
  not a `PropTaut`-shaped special case.

## 2. Where the time goes

Evidence: a 30s/1ms `sample` window in the middle of the distribution probe
(15,160 samples), reduced to a self-time flat profile and leaf→caller
attribution with `scripts/sample-self-time.py` and
`scripts/sample-attribute-callers.py`; plus temporary atomic counters in
`evaluation_cache.rs`; plus one single-variable A/B probe.

Self-time flat profile (top groups):

| Group | Share | Members |
| --- | --- | --- |
| Allocator + byte compare | ~40% | `memcmp` 12.2%, `malloc` 11.2%, `free` 10.4%, `memmove` 3.5%, misc allocator helpers |
| Stability scanning | 5.2% self, 18.4% inclusive | `cached_expr_is_evaluation_stable` |
| Expr clone/drop | ~3.5% | `Expr::clone`, `drop_in_place<Expr>` |
| Expression rendering | ~5% inclusive | `core::fmt::write`, `<Expr as Display>::fmt` |

Leaf→caller attribution (who actually pays for the leaves):

- `memcmp`: 26% under `cached_expr_is_evaluation_stable`, 15% under
  `try_apply_down_values_from_symbol`, 11% under `attribute_set_for_head`,
  11% under string-keyed `BTreeMap` get/entry.
- `malloc`: 18% under `cached_expr_is_evaluation_stable`, 14% under
  `Expr::call`, 11% under `SharedExpr::into_owned` + `SharedArgs::into_vec`.
- `free`: 19% under `store_definition_target`, 14% under
  `try_apply_down_values_from_symbol`, 12% under stability scanning.
- `<Expr as Display>::fmt`: **92.3% under `store_definition_target`**.
- `String::clone`: 32% under `cached_expr_is_evaluation_stable`, 11% under
  association key evaluation, 11% under `SharedExpr::into_owned`.

Counter probe over one full load + distribution proof run:

- `bump_evaluation_epoch`: ~328,000 calls. A load-only run stays below the
  65,536 reporting threshold, so **>80% of all epoch bumps come from the
  single `propTaut` call**.
- `cached_expr_is_evaluation_stable`: ~79M node checks, of which ~21M were
  uncached full recomputes (27% miss rate). That averages ~240 rescanned
  nodes and ~64 recomputes per epoch bump, forever, because the next
  assignment wipes the caches again.

## 3. Root causes, ranked

1. **Per-assignment global cache wipe × always-fresh cache keys.**
   `store_definition_target` ends with `bump_evaluation_epoch()`, which
   clears `stable_symbol_cache`, `symbol_call_effect_cache`, and
   `call_stability_cache`. Module-heavy prover code performs an assignment
   every few evaluator steps (Module initializers, body `Set`s, Module
   cleanup, association mutation all bump). Meanwhile
   `CallStabilityCacheKey` is `(head ptr, args ptr)`, so freshly built trees
   (every rule-RHS instantiation) can never hit anyway. Net effect: the
   stability machinery degenerates into repeated full-tree walks plus
   constant cache rebuild churn — the single largest identified region
   (≈18% inclusive plus large shares of `memcmp`/`malloc`/`String::clone`).
2. **`Symbol` is two heap `String`s with no interning.** Every symbol clone
   is two allocations; every comparison is string `memcmp`; `definitions`
   lookups, `attribute_set_for_head`, `builtin_dispatch_spec_for_symbol`,
   and `context() == SYSTEM_CONTEXT` checks all pay it on every evaluation
   step. This is the broadest contributor to the ~40% allocator+memcmp
   floor and is why the flat profile looks "smeared everywhere".
3. **Unconditional trace-string rendering on every assignment.**
   All four `store_definition_target` arms construct
   `TraceEvent::DefinitionStored { symbol.to_string(), rhs.to_string() / format!(...) }`
   eagerly even when tracing is disabled and no monitors exist;
   `push_trace_with_depth` then drops the strings. 92% of all expression
   `Display` time sits here. A/B probe (same pattern as the existing
   `push_rule_applied_trace` gating): 50s → 48.35s, ~3-5%.
4. **Owned-tree churn in the apply path.** `Expr::call` rebuilds,
   `into_owned`/`into_vec` Arc-unwrap clones, `down_value.clone()` per
   candidate, and Module init/cleanup account for most of the remaining
   allocator traffic.

## 4. Suggested order of attack

1. Gate the `DefinitionStored` trace-event construction behind
   `tracing_enabled` / active monitors, mirroring `push_rule_applied_trace`.
   Trivial, measured ~3-5%.
2. Make cache invalidation proportional to the mutation. Candidates, in
   increasing ambition: skip the epoch bump for assignments to fresh
   Module-local symbols (they cannot appear in any cached expression);
   per-symbol version stamps for the two symbol-keyed caches so an
   assignment invalidates one symbol instead of everything; only then
   rethink the pointer-keyed call cache. Measure after each step — the
   counters above make the win directly observable (target: epoch-bump
   count per `propTaut` call drops by orders of magnitude).
3. Intern symbols (small integer id + global table; keep display names out
   of the hot path). This was deferred as "symbol fingerprint" earlier; the
   attribution data now justifies the bigger version. Expect it to attack
   the `memcmp`/`malloc` floor across all lanes, including Bool load.
4. Clone accounting in definition application and Module enter/exit.
5. Re-profile. Only consider per-node evaluated bits or arena/DAG
   representation if the gap is still large after 1-4.

Items 1-4 plausibly compound to 3-5x on this workload. Reaching the
remaining 2-4x is representation-level work and should be re-planned from a
fresh profile, not extrapolated.

`PropTaut` is the first gate of the current `mma-hol` frontier; the same
constant factor sits under `basicSimpset[]` (~106s), `andCongLemma[]`
(~99s), and the `Pair.wl` load (>335s), so wins here propagate down the
whole stdlib lane.

## 5. How to find hot regions in this codebase (method)

The night's probes failed not for lack of effort but because the profile
was read in the wrong form. Concretely:

- **Do not read top-down call trees for a recursive evaluator.** The 30s
  `sample` file here is 3.8GB and its tree shows only the dispatch skeleton
  (`resume_call_with_dispatch_plan` → … → `evaluate`) repeated at every
  recursion depth. Nothing actionable lives there.
- **Do not read `sample`'s "Total number in stack" list as a ranking.** It
  counts recursive frames once per stack level (395,964 "samples" for
  `resume_call_with_dispatch_plan` out of 10,005 real ones). It exaggerates
  skeleton frames and buries leaves.
- **Reduce to self time first.** `scripts/sample-self-time.py <file>` turns
  a raw `sample` call graph into a flat self-time profile. Equivalent views:
  `samply record` (Firefox Profiler: "invert call stack" + self time),
  Instruments Time Profiler ("Invert Call Tree", "Top Functions"), or
  `cargo flamegraph --inverted` (icicle view).
- **Attribute runtime leaves to domain callers.** `malloc`, `memcmp`,
  `free`, `fmt` in the top self-time slots are symptoms, not findings. Ask
  "who calls them": `scripts/sample-attribute-callers.py <file> memcmp`
  walks each sampled stack from the leaf upward to the nearest
  non-skeleton, non-runtime frame and aggregates. The tables in section 2
  are its direct output; `Display→store_definition_target 92%` fell out
  immediately and had been invisible all night.
- **Count when sampling smears.** When the flat profile says "everything is
  a bit slow", the cause is usually a multiplier (an operation executed
  vastly more often than the workload logically needs), and counters find
  multipliers faster than samplers. Temporary `AtomicU64` counters on
  suspected operations, printed every 2^k events, cost minutes to add. The
  decisive numbers here — 328k cache wipes and 79M stability node checks
  for one three-variable tautology — cannot be seen in any sample profile,
  but instantly fail the "is this proportional to the logical work?" test.
- **Estimate the expected win before keeping a probe.** Gating trace
  strings could save at most the ~5% that `Display` showed; it saved ~3%.
  A probe whose target does not appear in the attribution tables (the
  builtin-name hasher, the borrowed matcher path) cannot win more than
  noise, and several such probes were measured at length last night.
- **Split startup from work on both sides of any cross-engine comparison.**
  Compare `AbsoluteTiming` to `AbsoluteTiming`, never rum wall time to
  `wolframscript` wall time. This is what made the Bool lane look finished
  when it was ~11x off.
- **Use workload shape as a coarse profiler.** Define-then-evaluate lanes
  (package load) versus assign-evaluate-interleaved lanes (`propTaut`,
  `Simp`) stress different state-invalidation paths; a lane-by-lane ratio
  table against the oracle localizes a regression before any profiler runs.

## 6. Note on the prior Bool optimization arc

The discipline in that arc — clean baselines, one variable per probe,
reverting neutral/negative changes, recording rejected probes — is good and
should continue. The two process gaps this audit closes are the missing
self-time/attribution reduction step (the hot-stack summaries in
`PROGRESS.md` repeatedly name "allocator traffic, structural compare,
stability checks" without ever attributing them to callers, which is why
the same list kept reappearing after each optimization round) and the
startup-contaminated Wolfram reference.
