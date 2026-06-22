# Next Session Handoff: Simp-Descent Loop Diagnosis (Task 2c), then Representation Work

> **2026-06-22 — CURRENT STATUS (supersedes the Task-2c/2d framing below).**
> The full running log is `PROGRESS.md` (read the `2026-06-21`/`2026-06-22`
> entries); the active plan is `docs/representation-rewrite-plan.md` (Layer A/B),
> NOT the older `performance-kernel-rewrite-plan.md`. Everything is committed
> (clean tree). Brief packages 009-014 are in `docs/dev/codex/`.
>
> **Done this session (all gated + committed):** Phase 0 repr-cost counters
> (`RUM_PROFILE_REPR`, zero-cost when off) + frozen baseline; **Layer A complete
> (A1+A2+A3+A4)** — A1 thread `SharedArgs` through dispatch, A2 copy-on-write
> `prepare_arguments`, A3 reuse `SharedArgs` in the down/up/sub-value candidate
> build, A4 thread the head as `SharedExpr`. Cumulative vs the frozen Phase-0
> baseline: worst microbench labels down ~13-17% (`downvalue-exact-hit 5.74 ->
> ~4.76x`, `if-true 5.96 -> ~5.15`, `pattern-hit 5.47 -> ~4.56`); A4 is a
> SYSTEMIC win (every call has a head).
>
> **The Task-2d framing below is SUPERSEDED.** The `~34x` was NOT a separate
> "failed-REWRCONV dispatch defect" distinct from the ~3x constant. Re-diagnosed
> this session (bounded two-engine `basic-chain-orelsec-split` + a faithful
> fresh-term failing-dispatch profile): the Pair `34x` is the SAME distributed
> per-call dispatch/eval constant the primitive microbench measures, compounded
> over the deep descent and amplified ~1.7x by the WL control combinators
> (`ORELSEC`/`REPEATC`/`TRYCONV`, curried down/sub-value functions). The failing
> MATCH itself is only ~5% of the failing dispatch; the cost is the per-call
> machinery (fixed-point evaluate loop ~2x/call, ~4 stability checks/call,
> prepare, dispatch glue). **Proof it is one lever, not two:** A4 (a per-call
> head-clone elimination, measured on the fast microbench) translated to the
> bounded Pair raw failing-dispatch `-5.8 to -7.9%` in a clean back-to-back A/B.
>
> **Dead-ends eliminated this session (do not re-attempt as the Pair lever):**
> (1) the inline normal-form stamp — already prototyped Task-3b, hits 33M+ but
> does not move Pair (refuted; `docs/representation-rewrite-plan.md` B3 marked
> refuted); (2) `SharedArgs = Arc<[Expr]>` node compaction — hit ~70 hidden
> in-place mutators, deferred (moderate flaw-2 prize); (3) Module frame setup —
> `module-local` is only `1.58x`, not the lever; (4) A3-style pure candidate
> allocation cuts — real on the microbench but did NOT translate to Pair (only
> WORK reductions like A4's head clone translate).
>
> **Next target:** keep reducing the distributed per-call dispatch/eval constant
> (the real `34x` lever via compounding) on the fast microbench proxy
> (`downvalue-exact-hit`/`pattern-hit`), validating each cut with a clean
> back-to-back A/B AND a bounded Pair re-measure. Candidates not yet cut: the
> dispatch plan build + stage iteration, redundant stability checks, the
> `prepare_arguments` loop, the no-change rebuild on non-symbol heads. Prefer
> cuts that remove real per-call WORK (clones/walks), not just allocations.
>
> **Measurement discipline (learned the hard way this session):** cross-run
> microbench/Pair ratios are unreliable when anything else shares the CPU (a
> concurrent probe, or MacBook nighttime maintenance `mds`/`backupd`). Gate with
> the machine-independent `RUM_PROFILE_REPR` counters PLUS a clean same-session
> back-to-back A/B (nothing else running). For an uncommitted change the A/B can
> stash it; **once committed, A/B against the parent commit via `git checkout
> <commit>^ -- <file>` (a `git stash` of a committed file is a no-op).** Verify
> the machine is idle first (maintenance daemons at 0% CPU); `if-true` /
> `block-local` stay intrinsically jittery (~6-9%) even idle, so use the A/B,
> not a single run, for them.

Process correction, 2026-06-17: recent probes drifted away from this handoff's
root target. The added `PropTaut` / one-case
``HOL`Bool`EXCLUDEDMIDDLE[p]`` probes are valid side evidence, but they were
not proven to dominate the current `Pair.wl` / warm public mkPair gate. Do not
continue drilling them as the main line. Resume from the root performance
question below: build a no-profile closed phase tree for unmodified `Pair.wl`
or the warm public mkPair reproducer, verify child phases sum close to the
parent wall time, then descend only into the largest child. Record root target,
parent segment, child percentage, profile overhead ratio, and logs at every
descent. This rule exists because the investigation lost the current-root
invariant and followed readable local profile output instead of the dominant
hundreds-of-seconds gate.

Current focus: **Task 2d** — fix rum's failed-rule-dispatch cost in the kernel,
not in `Simp.wl`. The diagnosis below (Task 2c + secondary splits) is correct
and excellent; the prototyped fix is at the wrong layer. See the review block
immediately below, then Task 2d.

Task-2c/patch review, 2026-06-13 (external): the root cause is correctly found
and the loop-count diagnosis is clean (rum and WK take identical
`simpFixpointConv` counts → not a fixpoint divergence; the earlier
origin-tag/`===` hypothesis is correctly ruled out). The multiplier is
**basic-rule failed dispatch**: `17240` `REWRCONV` calls, all match failures,
over the 19 basic rules on a no-op descent. That much is solid.

The problem with the patch is the **layer**, and it shows up as the symptoms
you noticed (snapshot blowup, `simp_tests` still `208s`, can't run stock
mma-hol):

- The prototyped fix (head-indexed dispatch + `buildBaseConv` cache) edits the
  downstream `auto/Simp.wl`, i.e. it **forks mma-hol**. The entire point of rum
  is to run the *same* HOL source WolframKernel runs, faster. A Simp.wl fork
  concedes that rum cannot, and rum still can't run an unmodified mma-hol.
- It **masks rather than fixes** the real defect. Use the developer's own
  numbers on the identical-entry-count three-lambda RHS (`431` entries in every
  rule set): adding the basic rules costs rum `+91.2s` (`22.5s → 113.7s`) but
  costs WolframKernel only `+2.7s` (`7.8s → 10.5s`). Same `17240` failed
  dispatches in both engines, but rum pays ~`5.3ms` per failed `REWRCONV`
  versus WK's ~`0.16ms` — a **~34x** gap on this path, against the ~3x general
  constant. WK proves the algorithm (try 19 rules per subterm, fail fast) is
  cheap; rum makes it catastrophic. Head-indexing "works" only by cutting the
  dispatch *count* so rum's ~34x per-dispatch slowness is hit fewer times. The
  defect is in rum's execution of a failed conversion dispatch, and it stays
  unfixed — every other dispatch-heavy workload still pays it.
- The snapshot breakage (`402MB` private-context dumps, the generated-symbol
  cleanup dance) is a *direct consequence* of fixing this in `Simp.wl`: the WL
  cache and its `Module` locals land in the symbol table. A rum-kernel fix
  touches no HOL symbol and the snapshot lane is never perturbed.

Keep the WL experiments as diagnosis — they prove that reducing dispatch count
closes the gate, which confirms the dispatch path is the cost. Do not commit or
upstream them as the fix. The fix is Task 2d.

Earlier tasks (Catch fix, Pair/Simp multiplier hunt, stability-check amplifier,
substitution-sharing audit and stamp prototype) are done and kept below as
history. The Task-3b carrier was correctly reverted.

Post-RHS-lambda split update, 2026-06-13: the single-term split has reduced
the hot shape to no-op descent over the rewritten `mkPair` RHS lambda, and the
RHS lambda has now been split. Each remaining binder roughly doubles
`simpFixpointConv` entries: full opened body `29.616s` / `53` entries, one
lambda `40.590s` / `107`, two lambdas `63.673s` / `215`, three lambdas
`113.656s` / `431` in rum; WolframKernel takes `7.640s` on the body and
`10.535s` on the three-lambda shape with identical entry counts. Rule-set
splits on the three-lambda shape are `user` only `22.492s`, `basic` only on
the user-generated RHS `94.288s`, and `userBasic` `113.656s`, so the public
multiplier is basic-rule failed dispatch layered over the no-op descent.
Scratch `Drule.wl` counters show `17240` `REWRCONV` calls on the three-lambda
shape, all match failures. Failure payload removal, sentinel throw, no-throw
failure, and fast const-head rejection inside `REWRCONV` did not materially
move the body time. A scratch indexed `buildBaseConv` reduced body
`REWRCONV` calls from `2120` to `122` but regressed without caching because
`100` indexed builds cost `27.877s`; adding exact `buildBaseConv` cache moved
the body to `21.293s` (`51` hits / `2` misses) and the three-lambda RHS to
`24.452s` (`429` hits / `2` misses, `976` failed `REWRCONV` calls). Next
evidence-backed optimization: design a real cached/compiled base-conversion
dispatch keyed by rules/context/depth and head-indexed rule candidates. Do not
ship the scratch WL patches directly; they are diagnostic.

Post-clean-baseConv experiment update, 2026-06-13: a fresh scratch clone at
`/private/tmp/rum-mma-hol-baseconv-exp.HtBwlQ/mma-hol` tested only the minimal
`auto/Simp.wl` optimization direction: head-indexed rule dispatch plus exact
`buildBaseConv` cache. This crosses the previous Pair gate. Key rum release
results: RHS three-lambda `userBasic` `27.618s` (`430` hits / `2` misses),
public `pair2Same` `34.932s` (`1509` / `2`), `pair3Same` `34.394s`,
`pair4Same` `36.681s`, `CONVRULE[simpConv[{mkPairDefThm}], REFL[pair2Same]]`
`53.047s`, cold load through `stdlib/Pair.wl` `85.537s` with `pairInjThm`
valid, `pair_tests.wl` green `42/0`, and `simp_tests.wl` green `46/0`.
WolframKernel also runs the patched `pair_tests.wl` green. A cache-only clone
was negative: RHS three-lambda still `115.585s` and public `pair2Same` timed
out at `180s`, so head-indexed candidate pruning is required. Snapshot caveat
is now diagnosed: full private-context `DumpSave` saved `Module` temporary
symbols such as `baseConv$...` / `xEq$...` and inflated
`HOL`Auto`Simp`Private`` to `402MB`. Explicitly removing
`generated = Names["HOL`*$*"]; Scan[Remove, generated];` before `DumpSave`
and restoring intentional Simp globals makes a filtered through-Pair snapshot
complete in `114.3s`, size `500KB`; restoring it takes `0.1s` and
`pair_tests.wl` remains green `42/0`. Real snapshot tooling should include a
careful generated-symbol cleanup phase before dumping.

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

Post-Task-3a update, 2026-06-13: the substitution-sharing audit is recorded
in [PROGRESS.md](../PROGRESS.md). Rule RHS instantiation does preserve shared
call subtrees for repeated bound values, unchanged sibling arguments, and
`Sequence` splices; `Expr` clones keep the shared head/argument-vector arcs.
Parent RHS calls are still rebuilt with fresh top-level pointers, so the Task
3 design constraint stands: an evaluated/stable stamp must travel on reused
shared subtrees, and the next prototype should prove that it reduces absent
call-stability misses on a workload with real shared subterms before broad
representation rollout.

Post-Task-3b update, 2026-06-13: an opt-in shared-subtree stamp prototype is
now available behind `RUM_CALL_STABILITY_INLINE_STAMP=1` and recorded in
[PROGRESS.md](../PROGRESS.md). It stores call-stability verdicts on the
existing shared argument-vector object, preserves the textual expression shape,
and uses the existing generation/dirty-bloom invalidation logic. Unit litmus
tests prove that a shared subtree can hit after the side cache is cleared, and
that a later write to the subtree head invalidates the stamp. A small rule-RHS
probe shows `call_stability_stamp_hits`.

Post-Task-3b A/B update, 2026-06-13: do not promote the stamp path to the
default runtime. The release warm public mkPair probe was run with and without
`RUM_CALL_STABILITY_INLINE_STAMP=1`. Profiled `240s` runs showed real stamp
activity (`call_stability_stamp_hits=33M+`,
`call_stability_stamp_stores=16M+`) but both variants timed out and reached the
same `268M+` stability-check scale. No-profile `360s` runs also timed out in
both variants. A follow-up split corrected a measurement trap: timeout logs can
lose earlier stdout `Print` output, so absence of `RUM_PROBE preheat_basic=` in
the full timeout log was not proof that preheat failed. Dedicated probes show
public `basicSimpset[]` still completes normally (`24.40s` standalone,
`22.68s` after mkPair setup) and the congruence preheat is `0.0018s`. The next
performance step is therefore to split the final public
`CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]` conversion itself.

Post-final-conversion split, 2026-06-13: the first split is done. A false-start
probe applied conversions to the theorem object; ignore that. The corrected
probes use `targetTm = concl[lhsAppXY]`. `buildBaseConv` takes `0.0048s` for 20
rules, and `TRYCONV[REPEATC[baseConv]][targetTm]` takes `0.079s` and leaves the
term unchanged. `descendOnce[allRules, {}, depth][targetTm]` times out at
`240s`; splitting the ordinary `comb` branch shows both
`simpFixpointConv[allRules, {}, depth][targetTm[[1]]]` and the same call on
`targetTm[[2]]` time out at `240s`. Next split: one `mkPair` application term,
comparing user rule only, basic rules only, and smaller partial applications.

Task-3a/3b review, 2026-06-13 (external): the audit and prototype were done in
the right order and the prototype is a genuinely useful **negative result**,
but it has a regression that must be corrected before any of this is committed,
and it changes how Task 3 should be scoped. Two action items, then the next
diagnosis below.

1. **Correct the default-path regression before committing (do not ship the
   carrier as-is).** The stamp prototype puts a `Mutex<Option<CallStabilityStamp>>`
   on every `SharedArgs`, and that slot is allocated and initialized on every
   call-node construction **unconditionally** — the `RUM_CALL_STABILITY_INLINE_STAMP`
   flag only gates whether the stamp is *consulted*, not whether the carrier is
   *paid for*. A clean same-session A/B (stash the uncommitted `crates/` changes,
   rebuild, measure; restore, rebuild, measure) puts the default-path PropTaut
   distribution probe at `~11.6s` with the changes versus `~10.2s` without —
   a ~12-14% regression that every workload pays, for a feature that is off by
   default and does not close the gate. The PROGRESS note "do not promote the
   stamp path to the default runtime" addresses the flag, not this structural
   cost. Fix options: revert the carrier entirely and keep only the diagnostic
   findings plus the opt-in counters (those are correctly cold-gated and cheap),
   or make the slot truly zero-cost when off (compile-time `cfg`/feature gate, or
   drop the `Mutex` — the per-session evaluator is single-threaded, so the real
   cost is the larger allocation, not the lock). Recommendation: revert the
   carrier; its value is already fully captured by the negative result below.
2. **Re-scope Task 3: its Pair justification is now refuted.** The prototype is
   exactly the cheap probe of Task 3's core mechanism, and it came back negative
   for the gate: the stamp carries the verdict correctly (litmus passes; warm
   mkPair shows `call_stability_stamp_hits=33M+`) yet the warm probe still times
   out at `240s` at the same `268M+` stability-check scale. This **refutes the
   earlier "redundant re-evaluation of shared subterms" hypothesis** — if the
   cost were re-checking shared stable subterms, the stamp would have moved it.
   It did not, so the Pair cost is not shareable redundancy; it is genuine work
   in the conversion descent. The single-allocation-node + inline-stamp rework
   may still be worth doing for the general ~2.7x constant, but it should no
   longer be greenlit on the expectation of closing Pair. Pursue the descent
   diagnosis below first.

## Where things stand

| Gate | Status |
| --- | --- |
| PropTaut distribution probe | committed HEAD `~10.2s` vs WolframKernel `3.80s` (~2.7x). The Task-3b carrier that regressed this to `~11.6s` has been reverted |
| Bool-stack load | `0.54s` vs `0.19s` (~2.8x) at committed HEAD |
| `proptaut_tests.wl` | green, ~`238s` |
| `meson_tests.wl` | green after Task 1: `List[44, 0]` in `79s` on clean `mma-hol` HEAD `698e3d0` |
| `meson_proveprop_tests.wl` | re-probed after Task 1; timeout at `300s` without a stdout summary |
| `stdlib/Pair.wl` load (stock mma-hol) | still exceeds bounded budgets; root cause is Task 2d (rum's ~34x failed-rule-dispatch cost), not a representation gap |
| `stdlib/Pair.wl` load (forked Simp.wl) | crosses the gate in a scratch clone (`85.5s` cold, `pair_tests 42/0`) but the fix is a downstream fork that masks the rum hotspot and breaks snapshots — not a shippable fix; see Task 2d |
| warm public mkPair conversion (stock) | times out; loop counts identical to WK (not looping), per-step cost is the gate |
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

## Task 2c (completed diagnosis): is the Simp descent looping? — No; it is per-step cost

This is the focus for the next session. The Task 3b stamp result rules out
"shareable redundant re-evaluation" as the Pair cost, and the conversion split
narrowed the timeout to recursive descent over `mkPair` application terms
(`descendOnce` → `simpFixpointConv`, with both `comb` halves independently
timing out). The decisive next question is whether `simpFixpointConv`'s
fixpoint loop **runs the same number of steps in rum and WolframKernel** (so the
gap is the ~3-4x per-step constant) or **runs far more steps in rum** (a
non-converging fixpoint — a different, usually cheaper, class of bug).

The enabling fact: `simpFixpointConv`, `descendOnce`, and `buildBaseConv` are
**WL-level functions in `auto/Simp.wl`, not kernel builtins**, so they can be
instrumented identically in both engines. This finally makes the
multiplier-vs-constant question at the descent level *directly measurable*
rather than inferred from rum-side counters. Read `simpFixpointConv` (around
`Simp.wl:446`): its `While[True, ...]` loop breaks on `newRhs === rhs`
(fixpoint reached) or `KeyExistsQ[seen, newRhs]` (cycle), and it rebuilds
`buildBaseConv` once per entry.

Completion, 2026-06-13: the two-engine loop-count probe was run on a scratch
instrumented `Simp.wl` copy at
`/private/tmp/rum-mma-hol-simp-loop.sSsJv1/mma-hol`. The completing
user-rule-only probe
`simpFixpointConv[{mkPairDefThm}, {}, $simpDepthLimit][concl[lhsAppXY]]`
reported identical counts in rum release and WolframKernel: `3565` entries,
`3573` while iterations, max `seen` size `3`, `3565` fixpoint breaks, and `0`
cycle breaks. Timings were `40.977489205s` in rum release and `12.837901s` in
WolframKernel, about `3.2x`. This completing shape is therefore not looping in
rum and does not support the `newRhs === rhs` / origin-annotation
non-convergence hypothesis. The next split should focus on rule-set and term
shape for one `mkPair` application; if the full public ruleset is suspected of
a separate loop, use explicit in-run snapshots rather than killed-process
stdout.

Plan:

1. In a scratch copy of `Simp.wl`, add global counters: total `simpFixpointConv`
   entries, total `While` iterations across all entries, and the max `seen`-set
   size. Print them after a probe. Apply the identical edit to the WolframKernel
   run.
2. Run a **completing** probe in both engines — user-rule-only mkPair finishes
   (~`12.8s` WK, ~`40-52s` rum), so it gives comparable counts — and compare the
   three numbers.
   - **Counts roughly equal between engines** → rum's cost is the per-step
     constant; the full-ruleset timeout is just more of the same. The lane then
     legitimately belongs to the general evaluator constant (Task 3) or a
     downstream `Simp` change, and Task 3's payoff estimate applies.
   - **rum's counts much higher (10x+)** → the fixpoint is not converging in
     rum: `newRhs === rhs` keeps returning False where WK returns True, so the
     loop spins on logically-equivalent but structurally-different terms. This
     is the high-value, usually-cheap case.
3. If the loop is spinning, find the spurious structural difference: dump the
   first few rum `newRhs`/`rhs` pairs where `newRhs =!= rhs` but the terms are
   alpha/semantically equal, and diff their `FullForm`. **Prime suspect: HOL
   origin annotations.** `stripOrigin` was already a hot per-symbol downvalue in
   the Task 2b counters; if rum's rewrite output carries origin tags that WK
   strips (or normalizes differently), every `===` comparison in the fixpoint
   loop fails and the descent never settles. Other candidates to rule out:
   `Plus`/`And` argument ordering, association internal form, and beta/`betaSubst`
   output shape (`betaSubst`/`vsubstImpl` were also hot). The fix for any of
   these is a local normalization so `===` matches WK, not a representation
   rework.
4. Independently, confirm the breadth vs depth shape: the developer's observation
   that splitting `targetTm` in half leaves **both** halves at the full `240s`
   timeout is itself evidence against a simple linear descent (a linear descent
   would roughly halve). Either each half independently spins (supports step 2's
   non-convergence case) or the descent is super-linear in term size (supports a
   beta-expansion term-size blowup — measure intermediate term size across the
   descent to tell these apart).

Acceptance: a PROGRESS entry stating, with the two-engine loop counts, whether
the Simp descent is constant-bound or looping in rum, and — if looping — the
named structural difference that breaks `===`, with a minimal repro. Only after
this verdict should the lane commit to either a local normalization fix, a
downstream `Simp` change, or the Task 3 representation rework.

The developer's own proposed split (one `mkPair` application term: user rule
only vs basic rules only vs smaller partial applications) is a reasonable
secondary cut, but the two-engine loop count above is more decisive and should
come first — it distinguishes "looping" from "slow" in one measurement instead
of bisecting rule sets.

## Task 2d (current focus): fix rum's per-failed-dispatch cost in the kernel

Task 2c is answered (not looping; per-step constant) and the secondary split
localized the multiplier to **basic-rule failed dispatch** — but at a ~34x
rum-vs-WK ratio on that specific path, not the ~3x general constant (see the
review block at the top for the arithmetic). That ~34x is a rum kernel hotspot
in how a *failed* conversion dispatch is executed. Fix it there so stock
mma-hol runs fast and the snapshot lane is untouched. Do not fork `Simp.wl`.

The unit to profile is **one failed `REWRCONV` dispatch**. Its HOL body
(`Drule.wl:292`) is small: `concl`, a `MatchQ` shape check, two `Part`s,
`tryTermMatch[lhs, t, ...]`, then on `$matchFail` a `convFail[...]`. The
developer already ruled out the obvious suspects at the WL layer: removing the
`Failure` payload, a no-throw sentinel, and a fast const-head pre-check inside
`REWRCONV` each failed to move wall time, and `tryTermMatch` depth is not it
(the fast head check caught `1998/2120` failures with no wall-time change). So
the ~5.3ms is *not* in the parts that a WL edit can see — which is exactly why
this must be profiled in rum, not in `Simp.wl`.

Concrete steps:

1. Build the smallest reproducing probe that does many failed dispatches and
   completes: the `basic`-only descent on `pair4Same` is ideal — `8.739s`,
   `61` entries, `Changed=False`, all dispatch failures. It is a few seconds,
   so it samples cleanly.
2. Sample it and reduce with the existing tools:
   `scripts/sample-self-time.py` then
   `scripts/sample-attribute-callers.py <sample> <hot leaf>`. The question is
   what rum executes per failed `REWRCONV` that WK does not. Likely suspects,
   in order of how cheaply each is confirmed:
   - **Per-attempt conversion-combinator construction.** `buildBaseConv`
     builds `combineSimpConvs[Join[ctxConvs, ruleConvs]]` and each
     `simpRuleConv[#, prover]` closes over `prover = makeProver[...]`. If rum
     re-evaluates/reconstructs these `Module`/`Function` closures per subterm
     (the caching experiment helping is circumstantial evidence it does), the
     cost is closure/`Module` setup, not matching. Check whether `eval_module`
     / closure construction dominates the failed-dispatch self time.
   - **`prover` recursion on conditional rules.** If any basic rule is
     conditional, trying it invokes `prover`, which may recursively simplify
     hypotheses even when the rule ultimately fails. Check the call tree under
     a failed `REWRCONV` for re-entrant `simpFixpointConv` / prover calls.
   - **Owned-tree churn per attempt.** `concl`, `Part`, and `tryTermMatch`
     each clone HOL subterms; on a ~34x path even modest per-call cloning
     compounds. This overlaps the general representation lane (Task 3) but may
     have a cheap local win (borrow instead of clone in the match-and-fail
     path).
3. Counter cross-check: add an opt-in counter for `eval_module` entries (or
   closure constructions) per failed `REWRCONV`, run the `basic`-only
   `pair4Same` probe, and compare against the `61` entries. If closures are
   rebuilt thousands of times for `61` entries, that is the multiplier and the
   fix is to build the conversion once per `simpFixpointConv` entry (or per
   rule set) inside the kernel's handling, not per subterm.

Acceptance: a PROGRESS entry naming what rum spends the ~5.3ms-per-failed-
`REWRCONV` on, with the self-time/attribution evidence, and a rum-side change
that brings the `basic`-only `pair4Same` descent and the `userBasic` mkPair
shapes down without any edit to `auto/Simp.wl`. Target: stock (unmodified)
mma-hol `pair_tests.wl` green and `stdlib/Pair.wl` load inside a bounded budget.

If the profile shows the ~5.3ms is irreducible without the call-node
representation change (e.g. it is dominated by per-attempt owned-tree cloning
that only borrowed traversal fixes), then and only then does this merge into
Task 3 — and a Simp.wl improvement *upstreamed to mma-hol* (not a private rum
fork) becomes the pragmatic fallback. Decide that from the profile, not before.

## Task 2e (snapshot lane): make rum preserve `$Packages` across DumpSave restore

A snapshot of the Pair lane failed under rum, and the scratch experiment
"fixed" it by editing the standard's harness — `tests/extend_snapshot.wls` in
the canonical clone was modified to add
`$Packages = Union[$Packages, holPublic]` after restoring a `DumpSave`. That
edit has been reverted; the canonical clone is pristine again. The harness edit
is not the fix (it changes the acceptance standard — see `AGENTS.md`).

The real gap is rum-side: `DumpSave` does not persist `$Packages`, so after
`Get`-restoring a snapshot, `BeginPackage` dependency checks in the unmodified
harness do not know the restored packages are already loaded. WolframKernel
runs the unmodified `extend_snapshot.wls` correctly, so this is a rum
`DumpSave`/`Get` round-trip gap. Fix: have rum's snapshot save/restore preserve
(or reconstruct on restore) `$Packages` for the saved public contexts, so the
unmodified harness resolves short names and dependency state without any edit.
Note the separate, legitimate observation already in PROGRESS: rum's full
private-context `DumpSave` also serializes generated `Module` locals
(`baseConv$…`, `xEq$…`) and inflates the dump (~`402MB`); that is a real rum
snapshot-hygiene bug to fix too (exclude generated/temporary symbols), but it
is independent of the `$Packages` restore gap and must also be fixed in rum,
not by pre-cleaning the standard's scripts.

Acceptance: an unmodified-`mma-hol` snapshot build/extend through `Pair`
completes and restores correctly under rum (`pair_tests.wl` green after
restore), with no edits to any file under the `mma-hol` tree.

## Task 3 (large, gated behind Task 2d): representation rework

> Re-scoping note (Task 3b review): the inline-stamp prototype already probed
> this lane's core mechanism and did **not** close the Pair gate, so do not start
> Task 3 expecting to close Pair. It may still be worth doing for the general
> ~2.7x constant. Run Task 2c first; if that verdict is "looping," a local
> normalization fix likely closes Pair far more cheaply than this rework.

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
hash-consing of subterms beyond the sharing already confirmed in Task 3a and
the existing-`SharedArgs` stamp prototype from Task 3b, that is the real shape
of the work, and it is larger than "add a stamp field". The warm mkPair A/B
did come back weak: inline stamps hit on small shared-subtree litmus tests and
in the profiled warm run, but they do not move the public gate. Before deeper
representation work, split the final public `simpConv[{mkPairDefThm}]`
conversion and check whether a downstream `Simp` algorithm change is the
cheaper route. The first split now points at recursive descent over single
`mkPair` application terms, not at `buildBaseConv` or top-level rewriting.

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
- The acceptance `mma-hol` clone is now the persistent
  `/Users/fft/Developments/mma-hol-acceptance` (latest published HEAD
  `200f13a`, the completed project — a large advance past `698e3d0`; older
  PROGRESS/audit entries that cite the decayed
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` are historical records of
  what was measured then, not live paths). `wolframscript` is at
  `/usr/local/bin/wolframscript` (~3.2s startup —
  never compare wall times).
