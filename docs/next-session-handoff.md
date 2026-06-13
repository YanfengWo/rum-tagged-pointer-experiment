# Next Session Handoff: Meson Triage, Pair Multiplier Hunt, Representation Work

Date: 2026-06-12. Initial diagnosis was written against commit `f06c23c`;
Task 1 has since been completed and the current status is recorded below.
This note hands the current `mma-hol` frontier to the next development
session. Read it together with the two PROGRESS entries "Representation
Optimization Pass (External)" and "Block Scope Invalidation Fix (External)",
and with `docs/proptaut-performance-audit.md` for the diagnosis method.

Post-Task-1 update, 2026-06-12: the `Catch` tag-form evaluation defect below
has been fixed in `rum-runtime`. The bounded clean-clone `test/meson_tests`
smoke now reports `List[44, 0]` in `79s` on `mma-hol` HEAD `698e3d0` (the
handoff's `43` expected count was stale for the current test tree).
`test/meson_proveprop_tests` was re-probed afterward and timed out at `300s`
without a stdout summary. Continue from Task 2 unless Task 1 regresses.

Post-Task-2 update, 2026-06-12: the Pair/Simp multiplier hunt has a written
verdict in [PROGRESS.md](../PROGRESS.md). It pointed first at a targeted
`HOL`Auto`Simp`` base-conversion experiment before any broad representation
work: warm public `simpConv[{mkPairDefThm}]` timed out in rum at `360s`
versus WolframKernel `23.10866s`, while user-rule-only mkPair simplification
was about `4.1x` and therefore not the multiplier.

Post-runtime-stability update, 2026-06-12: the targeted base-conversion memo
experiment did not close the gate, but conservative runtime stability cuts
reduced user-rule-only mkPair simplification from about `52.14s` to `40.61s`
and `basicSimpset[]` from `27.07s` to `20.27s`. The public warm mkPair /
`Pair.wl` gate remains open: the current warm public mkPair probe still times
out at `240s`. Do not keep iterating narrow stability tweaks without a new
probe.

Review update, 2026-06-12 (external): Tasks 1 and 2 are verified complete
(independent re-runs of the Catch repro, `meson_tests` `List[44, 0]`, the
full workspace suite, and `rum-difftest check`). The stability cuts also
moved the general lanes — PropTaut distribution `13.2s -> 10.0-10.2s`
(~2.7x), Bool-stack load `0.78s -> 0.54s` (~2.8x).

Post-Task-2b update, 2026-06-13: the stability-check amplifier diagnosis is
complete and recorded in [PROGRESS.md](../PROGRESS.md). Call-cache size-cap
thrash was rejected by a 16x cap A/B, and widening the dirty-bloom saturation
threshold to 128 was much worse. Miss-reason counters point mostly at absent
pointer-keyed call entries, so the remaining gate is fresh call-tree churn from
the Simp/Drule conversion chain. Task 3 is no longer blocked by an unnamed
cache mechanism, but a downstream `Simp` algorithm experiment remains a
reasonable alternative before large representation work.

Task-2b review, 2026-06-13 (external): verdict confirmed, with one gap and
one sharpening that change how Task 3 must be scoped — read Task 3 below
before starting it.

- Gap (does not change the verdict): the `1<<7`-bit dirty bloom is one bit
  per symbol, so in a write-heavy Simp phase ~96/128 bits are dirty and
  almost any cached entry is rejected by false bloom overlap. The
  `33M+ call_stability_rewritten_dirty_skips` are mostly this false overlap,
  and exact per-symbol write versions (which the original Task 2b note asked
  to re-measure on this shape) were never tried here — only the coarser
  wider-bloom knob was. It does not matter: `absent` misses (`67M+`, fresh
  trees with no entry at all) dominate the `dirty` misses (`33M+`), and
  fresh trees are immune to *any* dirty-tracking quality, so exact versions
  would be a modest win, never a gate-closer. The verdict stands.
- Sharpening (verified first-hand on a `fib[27]` baseline:
  `call_stability_miss_absent == evaluate_entries`, call-stability hit rate
  `0%`): the dominant cost is not Simp-specific. rum rebuilds a new-pointer
  tree on every rewrite, so the pointer-keyed call cache structurally cannot
  hit on freshly built trees; WolframKernel avoids the equivalent
  re-evaluation through expression sharing. The ~15x on warm mkPair versus
  the ~3x systemic constant is this redundant re-evaluation of logically
  equivalent subterms, which no cache *tuning* can remove.

## Where things stand

| Gate | Status |
| --- | --- |
| PropTaut distribution probe | `10.0-10.2s` release internal vs WolframKernel `3.80s` (~2.7x); improved as a side effect of the Pair/Simp runtime stability cuts |
| Bool-stack load | `0.54s` vs `0.19s` (~2.8x); same side effect |
| `proptaut_tests.wl` | green, ~`238s` |
| `meson_tests.wl` | green after Task 1: `List[44, 0]` in `79s` on clean `mma-hol` HEAD `698e3d0` |
| `meson_proveprop_tests.wl` | re-probed after Task 1; timeout at `300s` without a stdout summary |
| `stdlib/Pair.wl` load | still exceeds bounded budgets after runtime stability cuts |
| `cargo test --workspace`, `rum-difftest check` | pass |

The probe-visible cache-invalidation lane is closed: Do/Table iterator
Blocks no longer wipe the stability caches and matcher revisions per
iteration (`6df7bfe`), and the re-access probe is flat in expression size.
Constant-factor tuning of the current representation has reached
diminishing returns; the items below are ordered by expected value.

## Task 1 (completed): `Catch` does not evaluate its tag form

All 9 `meson_tests.wl` failures are the same defect. Minimal repro:

```wolfram
tag = "$t$"; Catch[Throw[42, tag], tag]
```

WolframKernel returns `42` (the second argument of `Catch` is evaluated,
then used as the pattern; `tag` evaluates to the string, which matches the
thrown tag). rum reports an uncaught throw because the tag form stays held
as the symbol `tag` and is matched as a pattern against the thrown string.
Literal-string forms (`Catch[Throw[42, "$t$"], "$t$"]`) and genuine
patterns (`Catch[..., _String]`) already behave correctly.

This is the same normalization issue fixed earlier for `MatchQ` /
`MemberQ` style calls (see PROGRESS "CL-Y Promoted Compatibility Fixes":
explicit pattern syntax stays held, non-pattern expressions are evaluated
before compiling an exact matcher). Apply the same treatment to the
`Catch` tag position, mirroring whatever `normalize_match_expr` path that
fix used. `Meson.wl` hits it through `mesonAttemptTag = "$mesonAttempt$"`
used as `Catch[..., mesonAttemptTag]` (Meson.wl lines 544-651).

Completion: a runtime regression test now covers the symbol-valued tag repro
(both the catching and the deliberately non-matching case), bounded
`test/meson_tests` reports `List[44, 0]` on the current clean clone, and
`test/meson_proveprop_tests` has been re-probed. Its current timeout is no
longer polluted by this bug.

Follow-up oracle check: WolframKernel does not sandbox nontrivial tag-form
side effects in the tested exact-tag path. `Print` is visible and assignments
inside the tag form remain visible after both matching and nonmatching nested
`Catch` cases, so this is no longer tracked as separate rollback debt.

## Task 2 (completed diagnosis): Pair/Simp multiplier hunt

`stdlib/Pair.wl` still exceeds `600s` with every retained optimization in
place, while the overall evaluator constant is only ~3.4x. If the load
were constant-bound it would finish in roughly `3.4x` of WolframKernel's
time; verify what WolframKernel actually needs for the same
dependency-preloaded `Get["stdlib/Pair.wl"]` (internal `AbsoluteTiming`,
not wall time). If Wolfram needs only seconds, rum is doing work Wolfram
does not do — an algorithmic multiplier, not a constant — and no amount of
representation tuning will close it.

Method (count, do not sample — see the audit doc):

1. Get the Wolfram reference timing for the same preloaded `Pair.wl` load.
2. Bisect the load cost by phase: `basicSimpset[]`, the first
   `simpConv`/`CONVRULE` use, and the `mkPairInjThm` construction were the
   expensive stages in the 2026-06-11 triage; re-measure each in both
   engines.
3. Add a cheap opt-in counter (pattern: `RUM_PROFILE_EVALUATION_CACHE` in
   `evaluation_cache.rs`) counting definition rule applications and
   `evaluate` entries; compare counts between a fast stage and a slow
   stage, and against a logical-work estimate. A stage whose rum count is
   orders of magnitude above the estimate is the multiplier; find which
   downvalue/loop generates it before changing any representation.
4. Only if counts match Wolfram-plausible work and the stage is still
   slow does the cost go back into the constant-factor bucket (Task 3).

Completion: the PROGRESS entry "Pair/Simp Multiplier Diagnosis" records the
verdict and counter numbers. The concrete construct is the public
`simpConv` / `withBasic` base-conversion path, especially repeated
`buildBaseConv` / nested `ORELSEC` / `TRYCONV` failure dispatch inside
`simpFixpointConv` and `descendOnce` over the mkPair term.

Follow-up runtime work reduced the constant factor for the same probes but
did not remove the multiplier. The retained cuts and rejected
base-conversion memo experiment are recorded in PROGRESS under
"Pair/Simp Runtime Stability Cuts".

## Task 2b (completed diagnosis): explain the stability-check amplifier

The Task 2 diagnosis localized the multiplier but did not explain it, and
the unexplained signal is precise: the warm public mkPair probe reaches
**`536M+` stability checks while `evaluate_entries` and
`down_value_applications` do not advance proportionally past the preheat
background**. Something performs enormous amounts of stability checking
without corresponding evaluation. WolframKernel runs the same logical
algorithm in `23.1s`; the general rum constant elsewhere is now only
~2.7-4x. Until the mechanism is named, neither a downstream `Simp`
algorithm change nor the Task 3 representation rework is clearly the right
spend — a third, much cheaper fix may exist. Do not start Task 3 before
this verdict is written.

Probe: the warm public mkPair shape from the Task 2 entry (preload through
`auto/Simp.wl`, precompute `basicSimpset[]`, `impCongLemma[]`,
`andCongLemma[]`, `orCongLemma[]`, then time
`CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]`), bounded at `120-180s` —
counters and samples from a bounded window are sufficient; the probe does
not need to complete.

Three candidate mechanisms, each cheap to confirm or kill. Instrument
first, conclude second:

1. **Call-stability cache size-cap thrash.** The pointer-keyed cache is
   capped at `CALL_STABILITY_CACHE_MAX_ENTRIES = 1 << 14` and is cleared
   wholesale on overflow (`evaluation_cache.rs`). The runtime stability
   cuts added data constructors, hold forms, and status objects to the
   cacheable set, so the Simp working set (large persistent simpset
   structures plus HOL terms) may now exceed `16k` live stable subtrees.
   If so the cache clears itself in a loop: insert past the cap, clear,
   re-walk everything, repeat — exactly "many checks, few evaluations".
   Confirm: add an opt-in counter that distinguishes size-cap clears from
   generation bumps and report both for the bounded warm window. Cheap
   falsification A/B: raise the cap `16x` and re-time the warm probe; a
   large move confirms, no move kills.
2. **Dirty-bloom saturation generation bumps.** Every `popcount > 96`
   saturation bumps the call-cache generation, killing all entries. A
   write-heavy Simp phase saturates every ~100 distinct written symbols,
   so long-lived stable trees may be re-walked after every ~100 writes
   forever. Confirm from the same run: compare `call_generation_bumps`
   against `symbol_invalidations` for the warm window, and correlate with
   `call_stability_misses`. If this dominates, the fix direction is finer
   invalidation for long-lived entries (exact per-symbol write versions
   were rejected once on PropTaut, but the cost profile here is different
   — re-measure on THIS shape before re-rejecting), a wider bloom, or
   exempting entries whose bloom does not intersect the saturating write
   set.
3. **Genuinely fresh trees from conversion-chain rebuilding.** If the
   checks hit mostly never-seen pointer keys (high
   `call_stability_misses` with low hit rate and no clear/bump
   correlation), then `simpFixpointConv` / `descendOnce` really does
   rebuild the `withBasic` chain per subterm and the cost is real work.
   Then the choice is the downstream algorithmic discussion or Task 3,
   as already recorded.

Attribution to callers complements the counters: sample the warm window
and run
`scripts/sample-attribute-callers.py <sample> cached_expr_stability_with_bloom`
to see which evaluator paths drive the checks (fixed-point pre-checks vs
uncached-walk recursion vs argument preparation).

Completion: the PROGRESS entry "Pair/Simp Stability-Check Amplifier Diagnosis"
names mechanism 3 as dominant for the current warm window. In the default
120s miss-reason probe, `call_stability_miss_absent=67M+` dominates
`call_stability_miss_dirty=33M+` and
`call_stability_miss_generation=4M+`; in the 180s windows, 16x cache capacity
does not move the major counts, while a dirty-bloom threshold of 128 explodes
misses and rewritten-dirty skips to `1B+`. The end-state target for the lane
stays the same: warm public mkPair completes inside a bounded budget and the
`Pair.wl` load enters the constant-bound regime (roughly `4x` of
WolframKernel's `37.7s`, i.e. about `150s`).

## Task 3 (large, now unblocked): representation rework

Plan, in order: single-allocation call node (head+args in one `Arc`),
which halves per-node allocation/refcount traffic and enables an inline
per-node evaluated/stable stamp replacing the pointer-keyed
call-stability side cache; then borrowed argument preparation so shared
argument vectors are not cloned per evaluation pass. Multi-day,
cross-cutting.

**Load-bearing design constraint from the Task 2b review — get this
right or the mkPair gate will not close.** The Pair/Simp gate is
dominated by `absent` call-stability misses: rum rebuilds a new-pointer
tree on every rewrite, so a pointer-keyed cache (the current side cache,
and equally a stamp that lives only on a freshly allocated node) can
never hit on those trees. An inline evaluated/stable stamp therefore
only helps if it **travels on the shared subtree `Arc`s reused through
substitution**, not on the freshly allocated parent node. Concretely:
when a rule RHS is instantiated, the bound subtrees are reused by `Arc`
clone; their stamps must survive into the new tree so that an
already-evaluated subterm is recognized as evaluated even though its new
parent has a fresh pointer. If the rework only stamps whole freshly
built nodes, it will reproduce the `absent`-miss behavior and leave the
~15x mkPair gap intact while still paying for the rewrite.

A litmus test for the design before committing to it: the `fib[27]`
baseline currently shows `call_stability_miss_absent == evaluate_entries`
and a `0%` call-stability hit rate (every rewrite is a fresh tree). After
the stamp change, a memoization-free recursive workload that re-encounters
structurally shared evaluated subterms should show those subterms
short-circuiting — i.e. the absent-miss count should fall well below the
evaluate count on a workload with real subterm sharing. If it does not,
the stamp is not propagating through substitution and the design needs
revisiting before the larger rollout.

If the propagation requirement turns out to need structural sharing /
hash-consing of subterms rather than just a stamp bit (because rum does
not currently reuse subtree `Arc`s aggressively enough through
substitution to make stamps land), that is the real shape of the work,
and it is larger than "add a stamp field". Scope it from a fresh read of
the substitution paths (`binding_substitution.rs`,
`lexical_substitution.rs`, the rule-RHS instantiation in
`definition_application.rs`) to see how often bound subtrees are already
`Arc`-shared versus deep-cloned. The downstream `Simp` algorithm change
remains the cheaper alternative if Task 3's payoff estimate comes back
weak after that read.

## Methodology guardrails (lessons already paid for)

- **Decompose composite probes before attributing their cost.** The
  interleaved-write probe was attributed to the dirty-bloom same-bit
  fallback; one decomposition command showed the cost was entirely the
  Do-iterator Block wrapper (`Do[e2;, {100}]` 2.3us/iter vs
  `Do[e2;, {i, 100}]` 428us/iter with no writes at all). Two rounds of
  invalidation-scheme work optimized a path the probe never exercised.
- **Estimate the expected win from the profile before keeping a change**,
  and reject probes whose target does not appear in attribution output.
- **One variable per A/B**, internal `AbsoluteTiming` against internal
  `AbsoluteTiming`, and never benchmark while another heavy process (or a
  second rum probe) shares the machine.
- Profile reduction tools: `scripts/sample-self-time.py`,
  `scripts/sample-attribute-callers.py`; cache counters:
  `RUM_PROFILE_EVALUATION_CACHE=1`.

## Known environment notes

- `Do[..., {10000}]`-scale loops can overflow the CLI stack and
  `Range[10^6]` exceeds the generated-element budget; scale probes
  accordingly or raise limits explicitly in the probe session.
- The clean `mma-hol` probe clone lives at
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` (HEAD `698e3d0`);
  `wolframscript` is at `/usr/local/bin/wolframscript` (~3.2s startup —
  never compare wall times).
