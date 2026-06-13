# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

## 2026-06-13 DumpSave Package Registration Restore

Closed the stable-snapshot `$Packages` workaround as a rum runtime conformance
bug. `mma-hol` maintainers verified that Wolfram Language 14.3 restores package
registration when `Get` reads a `DumpSave` `.mx`: public package contexts from
the snapshot reappear in both `$Packages` and `$ContextPath`, while
`` `Private` `` contexts do not. The earlier proposal to patch
`tests/extend_snapshot.wls` therefore masked a rum restore divergence and
should not be carried into upstream `mma-hol`.

rum's textual `DumpSave` now writes restore metadata for package contexts that
were present in the dumping session's `$Packages` and covered by the saved
symbol set. Reading the snapshot through ordinary `Get` evaluates that metadata
before the saved definitions, restoring the public package contexts to
`$Packages` and `$ContextPath` while excluding `` `Private` `` contexts.

Regression coverage updates
`dump_save_context_strings_roundtrip_public_and_private_contexts`: after
`Get[bootstrap.mx]`, `Select[$Packages, StringMatchQ[#, "Snap`*"] &]` and the
matching `$ContextPath` query both return only `"Snap`"`, short-name
`public[9]` resolves, and `Needs["Snap`"]` is a no-op.

Validation:

- `cargo test -p rum-runtime dump_save_context_strings_roundtrip_public_and_private_contexts`
- `cargo test -p rum-runtime package_tools`
- `scripts/mma-hol-bounded-smoke.sh --mode snapshot` with release `rum-cli` on
  a refreshed unmodified clone at
  `/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` (HEAD `51bbe9e`):
  `snapshot/build-parser`, `snapshot/run-fast-parser`, and
  `snapshot/extend-proptaut` all exited `0`; `snapshot_extend-proptaut.stderr`
  was empty.

## 2026-06-13 Pair/Simp Stability-Check Amplifier Diagnosis

Completed Task 2b from
[docs/next-session-handoff.md](./docs/next-session-handoff.md). The unexplained
warm public mkPair signal is not call-stability cache size-cap thrash, and a
wider dirty-bloom saturation threshold is a negative optimization. The dominant
mechanism is fresh pointer-keyed call trees from the Simp/Drule conversion
chain, with dirty-overlap cache skips as a secondary cost.

The opt-in `RUM_PROFILE_EVALUATION_CACHE=1` counters now also report
call-stability cache size-cap clears, dirty-bloom saturations, stable insertions,
rewritten-dirty skips, and miss reasons (`absent`, `generation`, `dirty`,
`pointer`). Two diagnostic A/B knobs were added for bounded probes only:
`RUM_CALL_STABILITY_CACHE_MAX_ENTRIES` and
`RUM_STABILITY_DIRTY_BLOOM_MAX_BITS`.

Counter split on the clean-clone warm public mkPair shape
(`basicSimpset[]`, `impCongLemma[]`, `andCongLemma[]`, `orCongLemma[]` preheated,
then `CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]`):

- default 180s bounded window: timeout, `stability_checks=268M+`,
  `evaluate_entries=67M+`, `down_value_applications=8M+`,
  `call_generation_bumps=16k+`, `dirty_bloom_saturations=16k+`,
  `call_stability_size_cap_clears=1024+`, `call_stability_hits=67M+`,
  `call_stability_misses=134M+`, `call_stability_insertions=16M+`, and
  `call_stability_rewritten_dirty_skips=33M+`.
- 16x cache-cap A/B (`RUM_CALL_STABILITY_CACHE_MAX_ENTRIES=262144`) also timed
  out at 180s with the same major lower bounds; only
  `call_stability_size_cap_clears` fell to `32+`. This rejects size-cap thrash
  as the amplifier.
- dirty-bloom threshold A/B
  (`RUM_STABILITY_DIRTY_BLOOM_MAX_BITS=128`) also timed out and was worse:
  `call_generation_bumps`/`dirty_bloom_saturations` disappeared, but
  `stability_checks=4B+`, `call_stability_misses=1B+`,
  `call_stability_rewritten_dirty_skips=1B+`, `call_stability_hits=32k+`, and
  `call_stability_insertions=2k+`. The generation bump is therefore a
  protective reset against rewritten-dirty accumulation, not the primary
  amplifier.
- default 120s miss-reason window: timeout, `stability_checks=268M+`,
  `evaluate_entries=67M+`, `down_value_applications=8M+`,
  `call_stability_miss_absent=67M+`, `call_stability_miss_dirty=33M+`,
  `call_stability_miss_generation=4M+`, and no visible pointer-mismatch lower
  bound. The main misses are absent pointer keys, not stale generations.

Verdict: no cheap cache-cap or bloom-threshold fix closes the gate. The warm
public mkPair / `Pair.wl` gate should move either to a downstream `Simp`
algorithm experiment that avoids rebuilding the nested `ORELSEC` conversion
chain, or to the larger runtime representation lane (single-allocation call
nodes with inline stable/evaluated stamps and borrowed argument preparation).
Do not spend more time on call-cache knob tuning without a new probe signal.

## 2026-06-12 `Catch` Tag-Form Evaluation Fix

Closed the `mma-hol` Meson semantic bug identified in
[docs/next-session-handoff.md](./docs/next-session-handoff.md) Task 1.
`Catch` now resolves its optional tag form through the same matcher-pattern
normalization path used by `MatchQ` / `MemberQ`: explicit pattern syntax stays
held, while ordinary expressions are evaluated before compiling the exact
matcher. The implementation preserves body-first `Catch` behavior; the tag
form is resolved only when a `Throw` reaches the `Catch` frame.

The regression covers both sides of the symbol-valued tag case:
`tag = "$t$"; Catch[Throw[42, tag], tag]` now returns `42`, while
`tag = "$t$"; Catch[Throw[42, "$u$"], tag]` still escapes as a non-matching
throw with tag `"$u$"`.

Validation:

- `cargo test -p rum-runtime catch_evaluates_symbol_valued_tag_form_before_matching`
- `cargo test -p rum-runtime holding_and_control_part6::cleanup_throw`
- `cargo test -p rum-runtime`
- bounded clean-clone `test/meson_tests` smoke with the release CLI:
  `List[44, 0]` in `79s` on `mma-hol` HEAD `698e3d0`. The handoff expected
  `List[43, 0]`, but the current test tree reports 44 cases; the semantic gate
  is zero failures.
- bounded clean-clone `test/meson_proveprop_tests` was re-probed after the fix
  and timed out at `300s` without a stdout summary. The bounded runner killed
  the process group cleanly, so the remaining status is a performance/scale
  gate rather than the now-fixed `Catch` tag-form bug.

Known remaining boundary: WolframKernel has a subtler side-effect scope for
nontrivial `Catch` tag-form evaluation. A tag form like
`(Print["SIDE"]; x = 2; "$t$")` prints while matching, but the `x = 2`
assignment does not leak after the `Catch`; rum currently uses ordinary
matcher-pattern resolution here, so that assignment would persist. This is
tracked as separate semantic debt because the Meson regression only needs the
symbol-valued exact-tag case.

## 2026-06-12 Pair/Simp Multiplier Diagnosis

Completed the Task 2 diagnosis from
[docs/next-session-handoff.md](./docs/next-session-handoff.md). The verdict is
that `stdlib/Pair.wl` is not currently blocked by another Pair correctness bug
or by a single cold PropTaut lemma. The multiplier is in the
`HOL`Auto`Simp`` simplifier path triggered by
`HOL`Drule`CONVRULE[HOL`Auto`Simp`simpConv[{mkPairDefThm}], lhsAppXY]`: adding
`withBasic` turns a constant-range user-rule simplification into a
failure-heavy nested `ORELSEC` / `TRYCONV` / `REPEATC` conversion chain over
the `mkPair` application term.

Reference timings, all bounded and using internal `AbsoluteTiming` where
available:

- WolframKernel dependency-preloaded `Get["stdlib/Pair.wl"]`: `37.697875s`.
- rum release dependency-preloaded `load/stdlib/Pair`: timeout at `300s`,
  with empty stdout and only the bounded-runner timeout in stderr.
- WolframKernel phase probe: `basicSimpset[]` `7.327259s`,
  `mkPairDefThm` construction `0.000538s`,
  first public `simpConv[{mkPairDefThm}]` `CONVRULE` `29.306584s`, and the
  later `simpConv[{xEqxTh, yEqyTh}]` collapse `0.76233s`.
- rum release `basicSimpset[]`: `27.070526269s`, about `3.7x` the
  WolframKernel time and in the current systemic constant-factor range.
- `andCongLemma[]` alone: WolframKernel `7.117217s`, rum
  `25.646827478s`, also about `3.6x`.
- Warm public mkPair simplification after precomputing `basicSimpset[]`,
  `impCongLemma[]`, `andCongLemma[]`, and `orCongLemma[]`: WolframKernel
  `23.10866s`, rum timed out at `360s`.
- User-rule-only mkPair simplification using
  `simpFixpointConv[{mkPairDefThm}, {}, $simpDepthLimit]`: WolframKernel
  `12.84882s`, rum `52.137050616s`, about `4.1x`.

The opt-in `RUM_PROFILE_EVALUATION_CACHE=1` counter was extended to report
`evaluate_entries`, `down_value_scans`, and `down_value_applications`, with
per-symbol power-of-two lower bounds. Counter probes show:

- `basicSimpset[]` completes with lower bounds of `134M` stability checks,
  `33M` evaluate entries, and `2M` downvalue applications.
- preheating `basicSimpset[]` plus the three congruence lemmas completes in
  `111.742s` with `268M` stability checks, `134M` evaluate entries, and
  `8M` downvalue applications.
- the warm public mkPair simplification still times out at `180s`; its lower
  bounds reach `536M` stability checks while downvalue/evaluate powers do not
  advance proportionally past the preheat background. The continuing tail is
  in the Simp/Drule conversion dispatcher: `isEquation`, `combineSimpConvs`,
  `tryConv`, `tryTermMatch`, `tryStructComb`, `collectAppSpine`, and
  `convFail`.
- The hottest successful downvalue symbols in the 180s warm run are term/type
  plumbing rather than Pair-specific definitions: `stripOrigin`, `typeOf`,
  `destFunType`, `tyMatch`, `vsubstImpl`, `freesWalk`, and `betaSubst`.

This identifies the next repair target more narrowly than broad evaluator or
representation work: validate and then fix the `Simp` base-conversion
construction/use pattern. The leading hypothesis is that recursive
`simpFixpointConv` / `descendOnce` rebuilds and re-evaluates a large
`withBasic` conversion chain for many subterms, and rum pays for the chain
through repeated stability checks and failure-driven conversion dispatch. A
targeted next experiment should hoist/cache prepared simplification rules or
the base conversion for the duration of a simplification pass, or otherwise
avoid repeatedly evaluating the same nested `ORELSEC` chain. Only if that
does not move the warm public mkPair probe should this return to the larger
single-allocation call-node / evaluated-stamp representation lane.

## 2026-06-12 Pair/Simp Runtime Stability Cuts

Followed up the Pair/Simp diagnosis on the `rum-runtime` side. The temporary
`mma-hol` source-level experiment that memoized `buildBaseConv` moved the
user-rule-only mkPair probe only from about `52.27s` to `51.10s`, and the
warm public mkPair probe still timed out at `240s`; that makes base-conversion
hoisting an insufficient standalone fix.

The retained runtime changes are conservative stability and dispatch cuts for
expressions that evaluate to themselves:

- stable atom evaluation now bypasses the full fixed-point evaluator for
  already-stable symbols and non-symbol atoms in call heads and ordinary
  argument preparation;
- `Association`, `Rule`, and `RuleDelayed` participate in call-stability
  caching when their data shape is already normalized and dependency-safe;
- `Part[expr, i, j, ...]` has a successful exact-integer fast path that
  reuses the existing exact structural selector and falls back to the old path
  for associations, selector errors, and diagnostics;
- `HoldComplete` / `HoldAllComplete` and ordinary hold-literal forms
  (`Hold`, `HoldAll`, `HoldPattern`, `Function`) participate in stability
  caching when no direct `Evaluate[...]` or `Unevaluated[...]` escape would
  change argument preparation;
- normalized status/data objects (`Failure`, `Missing`, `Success`, and
  `TerminatedEvaluation`) participate in stability caching, while
  `Failure["tag", rule-list]` shapes that still need details normalization
  stay on the old evaluator path.

Release timing on the local clean-clone Pair/Simp probes:

- user-rule-only mkPair simplification:
  `52.14s` before these cuts, `41.77s` after data-constructor stability,
  `41.44s` after the `Part` fast path, and `40.61s` after status/data
  stability.
- `basicSimpset[]`: `27.07s` before these cuts, `21.02s` after
  data-constructor stability, `20.80s` after the `Part` fast path, and
  `20.27s` after status/data stability.
- warm public mkPair simplification still timed out at `240s` after the
  retained runtime stability cuts.

The warm public mkPair / `stdlib/Pair.wl` gate is not closed by these cuts.
The remaining profile-visible work is mostly ordinary evaluator/dispatch cost
over HOL term/type plumbing (`stripOrigin`, `typeOf`, `destFunType`,
`tyMatch`, `vsubstImpl`, `freesWalk`, `betaSubst`) plus high-frequency
ordinary builtins such as `Part`, `If`, `Function`, `Module`, and `Length`.
Further improvement now points either to a real `Simp` algorithmic change in
the downstream package, or to the larger call representation / argument
preparation rework rather than another narrow stability tweak.

## 2026-06-12 Block Scope Invalidation Fix (External)

Re-examined the same-bit follow-up below with a decomposed re-access probe
and corrected the earlier diagnosis: the slow interleaved-write shape was
never about the dirty-bloom same-bit fallback. `Do[e2;, {100}]` re-accessed
a cached 100k-node expression at `2.3us` per iteration while
`Do[e2;, {i, 100}]` cost `428us` per iteration with no writes at all — the
cost was the iterator binding, not the write invalidation. Do/Table
iterator bindings wrap each iteration in `Block[{i = value}, body]`, and
Block entry and exit each called `bump_attribute_revision()`
unconditionally, which clears all three stability caches and invalidates
every compiled definition matcher twice per iteration; the scope-exit path
also rebuilt the message-symbol counts once per restored symbol.

The retained fix (`6df7bfe`) grades Block entry/exit invalidation by what
actually changed: definition shadow/restore invalidates just the affected
symbol, the attribute revision bumps only when an attribute set really
changed, and message-symbol counts rebuild at most once per scope exit.
The re-access probe drops from `332us` to `5.8us` per iteration and stays
flat in expression size. The PropTaut distribution probe is unchanged
(`13.2-13.4s`; its hot path holds no Blocks), `bool_tests` stays green, and
`cargo test -p rum-runtime` passes.

Consequences of this correction for the planning notes below: both the
same-bit rewritten-bloom scheme and the rejected exact-version experiments
were optimizing a path that the motivating probe never exercised; their
small effect sizes are explained. The probe-visible invalidation lane is
now genuinely closed. `meson_tests.wl` now completes inside a `300s`
budget and reports `List[34, 9]` — the 9 failures reproduce identically
on the pre-fix commit, so the Meson lane has crossed from a performance
gate to a semantic-triage gate. `stdlib/Pair.wl` still exceeds a `600s`
load budget with the fix in place, so the Pair gate is not a Block-scope
artifact; the next step there is a logical-work comparison of the
`Simp` simpset construction path against WolframKernel (multiplier hunt),
not further constant-factor tuning.

## 2026-06-12 v0.1 Acceptance Boundary

Added [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
as the canonical stabilization boundary for the first usable `v0.1` cut.

The document consolidates decisions that were previously spread across
`ARCHITECTURE.md`, the Jupyter host migration audit, and the `mma-hol` smoke
and snapshot notes:

- `v0.1` is a kernel and host-contract stabilization cut, not a full CAS or
  notebook product;
- `rum-host` is the primary public embedding boundary;
- `mma-hol` cold and stable-snapshot lanes are the main downstream pressure
  tests;
- Jupyter is scoped as a `rum-host`-backed basic Notebook tech preview with
  one documented deploy path;
- speculative showcase packages, Python adapter revival, broad frontend
  compatibility, graphics, and `WLBuiltin/` cleanup remain outside the cut.

## 2026-06-12 `rum-jupyter` Host Migration

Migrated `rum-jupyter` request handling onto `rum-host`.

- `rum-jupyter` now depends on `rum-host` instead of direct `rum-runtime` and
  `rum-stdlib`.
- `KernelAdapter`, bootstrap, socket, ZeroMQ, launcher, and tests now pass
  `HostSession`.
- Public Jupyter model outcomes use host-facing success, input-required,
  failure, and inspect response types instead of runtime response/failure
  types.
- Jupyter rendering now consumes host messages, prints, traces, failures, and
  text results; `Print` output maps to stdout streams and runtime messages map
  to stderr streams.
- The private Jupyter cursor helper was removed because inspect and completion
  cursor handling now belongs to `rum-host`.
- A lightweight manifest guard now fails if `rum-jupyter` reintroduces direct
  `rum-runtime` or `rum-stdlib` dependencies.

Validation: `cargo test -p rum-jupyter` passed locally, including native
ZeroMQ loopback tests.

Remaining Jupyter lane work: document one real Notebook deploy path and run
the host-backed adapter through an actual Notebook frontend.

## 2026-06-12 `mma-hol` Stable Snapshot Validation

Ran the stable snapshot lane against the fixed clean clone at
`/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`, bounded through
`scripts/run-bounded.py`.

Follow-up: `scripts/mma-hol-bounded-smoke.sh --mode snapshot` now owns the
bounded stable snapshot lane. It runs three stages in order:
`snapshot/build-parser`, `snapshot/run-fast-parser`, and
`snapshot/extend-proptaut`. With `RUM_CLI_BIN=target/release/rum-cli`, the lane
is green on the fixed probe clone:

```text
snapshot/build-parser      ok
snapshot/run-fast-parser   ok
snapshot/extend-proptaut   ok
```

This run originally used a fixed probe clone whose `tests/extend_snapshot.wls`
restored `$Packages` alongside `$ContextPath`, making
`auto/PropTaut.wl` extension quiet. That workaround was later superseded by
the 2026-06-13 rum runtime fix: textual `DumpSave` restore now auto-registers
public package contexts, matching the WolframKernel behavior observed by
`mma-hol` maintainers.

The CLI now renders accumulated prints/messages before honoring `Exit[n]`, so
`tests/run_fast.wls parser` produces visible progress and parser test logs
while still exiting with the test status code.

- Added the missing `SelectFirst` sequence subset needed by
  `tests/build_snapshot.wls` partial layer selection. The supported forms now
  cover unary operator form, two-argument search, three-argument default, and
  the snapshot shape
  `SelectFirst[Range[Length[libs]], StringContainsQ[Last[libs[[#]]], pat] &, $Failed]`.
- Fixed `Get` to propagate `Print` output from loaded files, so snapshot
  scripts keep useful progress logs when they are invoked through `Get`.
- `tests/build_snapshot.wls Parser` now succeeds through 11 of 32 libraries in
  Stable mode, dumps 21 `HOL` contexts, and writes
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol/bootstrap.mx` at about
  `11904 KB`.
- `tests/run_fast.wls parser` exits with status `0` against that snapshot and
  prints the parser test log; the parser summary is `passed: 78`, `failed: 0`.
- `tests/extend_snapshot.wls auto/PropTaut.wl` exits with status `0`, restores
  the Parser snapshot, loads `auto/PropTaut.wl`, and re-dumps 23 `HOL`
  contexts at about `11940 KB`.

Remaining snapshot boundaries: full `build_snapshot.wls` still runs into the
known downstream `Pair.wl` / `Simp` performance gate.

## 2026-06-12 Same-Bit Stability Invalidation Follow-Up

Followed the external pass recommendation to test the same-symbol rewrite
hole in the call-stability dirty bloom. The retained change is a conservative
rewritten-dirty-bloom scheme in `evaluation_cache.rs`: repeated writes whose
bloom bit is already dirty no longer bump the whole call-stability generation,
but cache hits still reject entries touched by newly dirty bits or by bits
rewritten after the entry snapshot. New entries touching rewritten dirty bits
are not cached. This preserves the cheap 128-bit dependency filter without
pretending it is an exact symbol dependency set.

Measured and rejected alternatives:

- Exact per-symbol and per-bloom-bit version snapshots fixed the black-box
  re-access shape but regressed the PropTaut distribution probe to about
  `24.9s` and `24.6s`, respectively.
- Skipping call-stability entries related to dirty bits regressed the same
  probe to about `20.5s`.

Retained result: the PropTaut distribution probe now measures about
`12.73-12.88s` release internal timing on the reference machine, compared with
the external pass frontier of about `13.1s` and the local WolframKernel
internal timing of about `3.80s`. The bounded `proptaut_tests.wl` smoke is
green in about `239s`.

The original expert black-box script cannot currently run unchanged through
the default CLI path: `Range[10^6]` exceeds the generated-element limit and
`Do[..., {10000}]` overflows the current CLI/runtime stack. Scaled diagnostic
probes are therefore only partial reproductions: with `Range[10000]` and
`Do[100]`, repeated same-write re-access measured about `37-39us` per access;
with `Range[50000]` and `Do[100]`, about `164us` per access.

Checks after the retained change: `cargo fmt --check`, `git diff --check`,
`cargo test -p rum-runtime`, unsandboxed `cargo test --workspace`, and the
bounded `proptaut_tests.wl` smoke passed. The workspace-wide Rust test needs
unsandboxed execution because the `rum-jupyter` native ZeroMQ tests bind
loopback sockets.

Next evidence-backed representation work is not another exact-version cache
attempt. The remaining plausible cuts are the larger single-allocation call
node / inline evaluated-stamp representation, borrowed argument preparation,
and only if new profiling justifies it, a non-allocating exact dependency
representation for stability invalidation.

## 2026-06-12 Representation Optimization Pass (External)

A second external optimization pass reopened the performance lane after the
pause below and landed five retained structural changes. All numbers are the
PropTaut distribution probe (dependency-preloaded, release, internal
`AbsoluteTiming`) on the reference machine; WolframKernel internal reference
is `3.80s`.

- Symbol interning with dense pair ids (`7c0ff8a`): `Symbol` equality,
  hashing, and hot map lookups became integer operations; `definitions` and
  `attributes` moved from `BTreeMap` to id-hashed `HashMap`s. `28.6s -> 20.4s`.
- Dependency-aware stability-cache invalidation (`f1806fb`): definition
  writes invalidate exactly the written symbol plus a 128-bit dirty bloom
  over call-stability entries instead of clearing all three caches per
  assignment; only stable verdicts are cached. Full clears remain for
  package/context/attribute changes. `20.4s -> 19.9s`, and cache wipes per
  proof dropped from ~65k to ~4-8k generation bumps.
- mimalloc as the `rum-cli` global allocator (`cfd3680`): expression-tree
  churn made small-object allocation the largest single block (~34% self
  time). `19.9s -> 14.1s`.
- Message-symbol index plus pre-sized argument preparation (`9920aff`):
  `symbol_exists` stopped scanning every message definition per name lookup.
  `14.4s -> 13.9s`.
- Id-hashed pattern/lexical binding maps and a 32-byte `Symbol`
  (text-pointer + inline id): `13.9s -> 13.1s`.

End state: distribution probe `13.1s` (3.4x vs WolframKernel, down from
12.8x), Bool-stack load `0.78s` vs `0.19s` (4.1x, down from 11.5x).
Downstream gates moved materially: `proptaut_tests.wl` is green for the
first time (`238s` full suite, previously a hard timeout at `300s`),
`bool_tests` `6s`, `drule/tactics/printer/parser/kernel` tests `~1s` each.
`cargo test --workspace` (24 suites) and `rum-difftest check` pass. The
`stdlib/Pair.wl` load still exceeds a `600s` budget; the earlier `335s`
observation was an interrupted run, not a completion, so Pair remains the
next downstream gate after `proptaut_tests`.

The original `<2x` target was not reached. The remaining ~3.4x is spread
flat across the evaluator (no single item above ~7% self time): stability
scanning ~10%, allocator/drop/memmove ~20%, substitution ~7%, definition
matching ~8%, dispatch skeleton ~9%, long tail ~45%. Closing it requires a
representation rework, in order of expected leverage: a single-allocation
call node (head+args in one `Arc`, halving per-node allocations and
refcount traffic, and enabling an inline per-node evaluated/stable stamp in
place of the pointer-keyed side cache), borrowed argument preparation so
shared arg vectors are not cloned per evaluation pass, and a non-recursive
dispatch loop. Each is multi-day, cross-cutting work; per-probe estimates
suggest together they plausibly reach ~2x but not below without a JIT-class
overhaul. Rejected this pass: depth-gated cycle tracking (broke trace-depth
and `Stack[]` semantics for ~1.5%), caching unstable call verdicts (pins
dead trees, regressed to 25s), text-pointer-only `Symbol` equality without
the inline id (hot-path pointer chase, ~3% regression).

A black-box evaluation-state probe (re-accessing a large inert expression
stored in a symbol, with interleaved definition writes) shows where the
invalidation scheme still trails WolframKernel. Both engines re-access a
100x larger expression at unchanged cost and survive a write to an
unrelated symbol, but with a write to one symbol interleaved between
accesses WolframKernel stays at ~0.35us per access while rum pays ~394us.
The follow-up above tested this same-bit rewrite path. A cheap rewritten dirty
bloom was retained; exact version snapshots were rejected because their entry
snapshot overhead regressed the real PropTaut workload. This means the next
high-leverage frontier has moved back to the larger call-node representation
work rather than more blind stability-cache micro-optimization.

## 2026-06-12 Performance Lane Pause And Cleanup

- Cancelled the long-lived PropTaut optimization goal and stopped automatic
  work toward the earlier `<2x` cross-engine target. That target was not met:
  the retained release frontier is about `27.40-27.64s` for the PropTaut
  distribution probe versus the local WolframKernel internal timing of about
  `3.80s`.
- Verified the repository was clean before this documentation pass and found no
  lingering agent-owned `rum-cli`, `run-bounded.py`, `sample`, or `cargo`
  processes. No generated profiler or bounded-smoke artifacts were present in
  the workspace outside normal `.git/hooks/*.sample` templates.
- Reframed the current `mma-hol` status for future agents: `PropTaut` / `Simp`
  remains the first stdlib/auto gate beyond the green Parser/Printer cold lane,
  but the next action is external review of the accumulated evidence rather
  than continuing speculative evaluator/representation micro-optimization.
- The useful artifacts for that review are
  [docs/evaluator-performance-audit.md](./docs/evaluator-performance-audit.md),
  [docs/proptaut-performance-audit.md](./docs/proptaut-performance-audit.md),
  `scripts/sample-self-time.py`, `scripts/sample-attribute-callers.py`, and
  the retained/rejected probe history below.

## 2026-06-11 PropTaut Performance Attribution

- Added `docs/proptaut-performance-audit.md` and the local macOS `sample`
  reduction helpers `scripts/sample-self-time.py` and
  `scripts/sample-attribute-callers.py`. The audit corrected the Bool-lane
  oracle comparison by separating Wolfram startup from kernel-side
  `AbsoluteTiming`, and reframed the current `PropTaut` cost as the same
  systemic evaluator factor rather than a special proof-script cliff.
- Kept the positive evaluator fast-path follow-up on the `PropTaut`
  distribution theorem: delayed suspension-frame cloning in hot synchronous
  paths and reduced definition/rewrite-rule cloning moved the release timing
  from roughly `48.8s` to roughly `46.5s`.
- Gated `DefinitionStored` trace rendering so ordinary non-tracing execution
  no longer formats stored-definition RHS strings. The same `PropTaut` probe
  then measured about `43.55-43.78s`.
- Added an opt-in cache-invalidation counter probe behind
  `RUM_PROFILE_EVALUATION_CACHE=1`. It reports power-of-two checkpoints for
  `epoch_bumps`, `cache_clears`, `stability_checks`,
  `call_stability_hits`, and `call_stability_misses` to stderr. With the env
  var unset, the same `PropTaut` probe stayed in the `43.7-43.8s` range.
- Made definition writes skip global evaluation-cache invalidation when the
  written symbol has not been observed by the current stability scan. This
  kept the same `PropTaut` result and moved the release probe to about
  `40.07s`; the cache counter run showed `epoch_bumps/cache_clears` reduced
  from the previous `262144+` range to the `65536+` range, with
  `262144+` skipped unobserved-symbol writes. A regression test covers the
  prior-read case (`Module[{x}, x; x = 1; x]`) where invalidation must still
  happen.
- Changed `Symbol` storage from two owned `String`s to shared `Arc<str>`
  context/name fields while preserving the public `context()`/`name()` API,
  ordering/hash semantics, display behavior, and serialized field shape. This
  is a low-risk representation step short of full numeric symbol interning;
  the same release `PropTaut` probe then measured about `32.60-32.82s`.
- Delayed association key-evaluation suspension-frame cloning until an input
  request actually occurs. This removed synchronous-path cloning of the parsed
  entry list and evaluated prefix for ordinary `Association[...]` construction;
  the same release `PropTaut` probe then measured about `30.06s`. A similar
  `Lookup` lazy-frame probe was neutral to slightly worse and was not kept.
- Re-profiled current HEAD `ec372d6`: the remaining self-time is again led by
  `memcmp`, allocator/free traffic, and `cached_expr_is_evaluation_stable`.
  Leaf attribution now points `memcmp` mostly at stability scanning,
  attribute lookup, definition matching, and symbol existence checks; allocator
  traffic is led by `Expr::call`, rule-binding substitution, `Symbol::new`,
  sequence splicing, and `SharedArgs` construction/unwrapping. Association key
  evaluation is no longer the dominant clone caller after the lazy-frame
  change.
- Added session-local caches for hot attribute-set and builtin-dispatch
  resolution, invalidated on attribute revision changes and builtin
  registration. The dispatch planner and stability scanner now use these
  caches while colder inspection paths keep the direct lookup API. The same
  release `PropTaut` probe then measured about `28.10-28.32s`. A hand-written
  `Symbol` comparison probe using `Arc<str>` pointer equality measured worse
  (`31.55-32.51s`) and was reverted.
- Reused shared `Arc<str>` storage for Global/System contexts and
  canonical core System symbol names through the hot `Symbol::from_name` /
  `Symbol::system` constructors, and routed obvious System-symbol runtime
  constructors through that path. The same release `PropTaut` probe then
  measured about `27.40-27.64s`. Two nearby probes were rejected: removing the
  recursive definition shape prefilter regressed to about `29.16s`, and
  skipping unchanged literal-`Sequence` checks during RHS substitution stayed
  neutral to slightly worse (`28.35-28.70s`).
- Re-profiled after the shared System-symbol text change: `Symbol::new`
  allocation is no longer a leading allocator caller, and the remaining
  frontier is `Expr::call`, `SharedArgs::{new,into_vec}`, rule-binding
  substitution, stability scanning, and downvalue matching. Two representation
  probes were rejected: reusing a static empty `SharedArgs` regressed to about
  `28.50s`, and skipping evaluation-cache clearing on epoch bumps regressed to
  about `37.00s` because stale cache-table growth outweighed avoided clears.

## 2026-06-10 Scope Reset

The project is moving from broad core-language expansion toward a `v0.1`
kernel-first stabilization pass.

Current direction:

- stop treating Wolfram same-name surface growth as the default next action
- use concrete downstream pressure, especially `mma-hol`, to promote missing
  semantics back into active work
- keep `rum-host` as the main embedding boundary
- freeze project-owned Python adapter work until a real downstream need returns
- keep Jupyter as an optional capability-validation adapter, not a roadmap
  driver

Detailed future showcase plans for a Python AI policy router and a Rust Wasm
prototype were removed. Those remain possible application directions, but they
are not active project-owned examples.

## 2026-06-10 Correctness Fixes

- Definition insertion now applies the same specificity ordering to
  `UpValues` and `SubValues` that `DownValues` already used, so later typed or
  literal rules can outrank earlier generic patterns while equivalent pattern
  definitions still replace in place.
- Added runtime regressions plus curated Lane A `definitions_precedence`
  coverage for generic-before-typed `UpValue` and `SubValue` definitions.

## 2026-06-10 CL-Y Cold Lane First Pass

- Added first-pass `mma-hol` cold-lane coverage for exact `Total`,
  `Denominator`, `Merge[..., Total]`, `Apply[LCM|GCD, list]`, `/@` map syntax,
  and ordinary Unicode letter identifiers.
- `Total` now reduces list totals through the existing exact `Plus` semantics,
  so grouped association coefficients in `Merge[..., Total]` no longer remain
  inert.
- `Denominator` now covers exact integers and rationals, which is enough for the
  current Farkas-certificate denominator extraction shape.

## 2026-06-10 CL-Y Clean Source Smoke

- Used a clean archive of sibling `mma-hol` HEAD `83d768f` instead of the dirty
  working tree for the cold source smoke.
- Cleared the next loader/source blockers exposed by that smoke: initial
  shebang trivia, multiline comments in the script splitter, logical and
  comparison infix operators, `:` named-pattern syntax, `Function` postfix
  followed by `/@`, string-join `<>`, package usage-message public-symbol
  visibility from private contexts, and string-key `Part` selectors for
  association-backed `Failure` payloads.
- That pass loaded `ErrorUtil.wl`, `Types.wl`, `Terms.wl`, `Kernel.wl`,
  and `tests/harness.wl` in sequence. Narrowed test summaries were:
  `errorutil_tests.wl` `List[10, 0]`, `types_tests.wl` `List[10, 6]`,
  `terms_tests.wl` `List[16, 13]`, and `kernel_tests.wl` `List[8, 15]`.
- At that point, the CL-Y work was no longer parser/loader triage in the
  abstract. The next step was to expose first-failure details from the harness
  and promote only the concrete semantic gaps behind the remaining type, term,
  and kernel failures.

## 2026-06-10 CL-Y Core Assertion Triage

- Cleared the first core semantic failures exposed by the clean `mma-hol`
  snapshot: `Lookup` now holds and evaluates association/default/wrapper
  arguments in the needed order; `Association[...]` evaluates keys before
  canonical entry storage; `BeginPackage[..., deps]` keeps dependency contexts
  available on `$ContextPath`; association key lvalues and list destructuring
  `Set` now cover the current `Types.wl` / `Terms.wl` mutation shapes.
- Definition installation now evaluates call heads in definition lhs positions
  before selecting the definition owner. This matches Wolfram behavior for
  forms such as `h = f; g[h[x_]] := ...` and fixes the strict kernel theorem
  tag patterns generated through `Module` / `Unique`.
- Definition RHS evaluation now consumes `Return[...]` at definition
  application boundaries, matching Wolfram's `g[] := Return[1]; f[] := (g[]; 2)`
  behavior. This removed the immediate `"P"` short-circuit in `HOL`Bool`GEN`.
- Current clean-snapshot summaries after these fixes:
  `errorutil_tests.wl` `List[10, 0]`, `types_tests.wl` `List[36, 0]`,
  `terms_tests.wl` `List[67, 0]`, `kernel_tests.wl` `List[71, 0]`,
  `bootstrap_tests.wl` `List[45, 0]`, and `equal_tests.wl` `List[17, 0]`.
- The next concrete blocker is the Bool layer. Valid probes must preload the
  lower HOL files before `Bool.wl`; a fresh direct `Get["Bool.wl"]` still hits
  package-dependency/autoload mismatches and is not the representative proof
  benchmark. After the first Bool optimization pass, full dependency-preloaded
  `Bool.wl` completes in release, but debug loading is still too slow for a
  routine smoke and `bool_tests.wl` has not yet been promoted into a bounded
  check.

## 2026-06-10 Bool Block Profiling

- Re-ran the Bool blocker against the fixed clean `mma-hol` snapshot. The issue
  is no longer a parser/loader failure or a semantic short-circuit: with the
  definition-boundary `Return` fix in place, the Bool proofs enter their real
  beta/derived-rule path.
- Isolated full `Bool.wl` loading to the first heavyweight top-level theorem:
  a temporary slice through line 580, before `condTThm`, loads; adding
  `condTThm` makes loading exceed an interactive window. A direct `condTThm`
  body probe still timed out under a 60s internal `TimeConstrained` window.
- Ruled out `Part` as the primary hotspot. `rhsOfConcl[APTHM[condDefThm, T]]`
  returns a large redex successfully; the cost is in repeated `BETACONV`,
  `BETA`/`INST`, `TRANS`, `SYM`, `CONJ`, and `DISCH` over large HOL terms.
- `GEN[x, REFL[x]]` is not a dead loop in isolation: under the Bool pre-theorem
  slice it completes, but still costs several seconds beyond package load.
- WolframKernel loads the same full `Bool.wl` in roughly 2.4s on this machine,
  while rum still exceeded 150s for full Bool loading before interruption. This
  makes the remaining blocker a rum evaluator performance problem, not just an
  expensive downstream proof script.
- Kept targeted evaluator optimizations after probe/measure cycles:
  non-held definition matching skips recursive candidate normalization; simple
  fixed-arity patterns use a conservative fast matcher and shape prefilter;
  trace strings are materialized only when tracing is enabled; only small
  expressions keep full fixed-point/call-stack cycle histories; clearly stable
  expression trees can return without another dispatch pass; suspend/resume
  clones are delayed until an input request actually occurs.
- Local debug microbenchmarks improved materially: the large Bool `BETACONV`
  redex dropped from roughly 1.8s to roughly 0.4s, `CONJ[ASSUME[p], ASSUME[q]]`
  from roughly 7.6s to roughly 1.8s, and a `DISCH[T=T, REFL[a]]` subpath from
  roughly 25s to roughly 8s. The dependency-preloaded prefix before `condTThm`
  is now around 4.5s in debug instead of the earlier mid-teens.
- Full dependency-preloaded `Bool.wl` now completes, but is still expensive:
  around 231s in debug and around 60s in release on the measured machine. This
  is progress, not closure. The remaining hot path is dominated by repeated
  derived-rule work in `MP`, `GEN`, `SPEC`, `BETACONV`, and witness/equality
  construction over large HOL terms.
- Dead-end or low-yield probes so far: treating `Part` as the primary hotspot,
  a broad top-level stable-evaluation shortcut, and optimizing direct fresh
  `Get["Bool.wl"]` before resolving the dependency/autoload mismatch.
- The next likely performance cut is deeper than surface dispatch: reduce
  owned-expression cloning in recursive pure-function and derived-rule
  workloads, preferably with borrowed traversal/matcher paths or profiler-guided
  clone accounting before adding more special cases.

## 2026-06-10 Evaluator Performance Audit Follow-up

- Added `docs/evaluator-performance-audit.md` as a historical external review
  of the Bool proof-load bottleneck. The audit was captured for rum commit
  `fddf4fd` and sibling `mma-hol` HEAD `96cf163`, and confirmed that allocator
  traffic, owned expression cloning, fixed-point loop cloning, catalog scans,
  definition dispatch cloning, and matcher binding snapshots dominate the
  release profile.
- Implemented the first quick-win series from that audit: hash-backed builtin
  catalog lookup; delayed fixed-point resume-state cloning; symbol+index
  definition dispatch that avoids cloning whole `SymbolDefinition` values;
  cached stored-definition matchers guarded by an attribute revision; and
  matcher failure rollback using a small binding undo trail instead of cloning
  the entire binding map at every matcher entry.
- Fixed the CLI tooling side issue from the audit: `rum-cli --eval` and
  `--file` now render `Print` output before the final value, so load-time
  bisection markers are visible.
- Re-ran the dependency-preloaded Bool release smoke after these changes:
  `AbsoluteTiming[Get["Bool.wl"]]` was about 46.84s and external wall time was
  48.26s, down from the previous comparable `real 62.82s`. This is a real
  improvement, but still far from WolframKernel's roughly 2.4s reference; the
  structural representation work remains open.

## 2026-06-10 Evaluation Epoch Cache Follow-up

- Rebuilt the post-quick-win Bool baseline before making deeper changes:
  dependency-preloaded `AbsoluteTiming[Get["Bool.wl"]]` reported about 46.21s,
  with external wall time `real 47.64`.
- Collected a fresh macOS `sample` profile from that build. The profile still
  showed allocator traffic, owned `Expr` cloning/dropping, structural compares,
  `run_call_dispatch_stage`, `try_apply_down_values_from_symbol`, and
  `expr_is_evaluation_stable` on the hot path. Parser and loader work remained
  absent from the hot stacks.
- Added a session-level evaluation epoch and two exact symbol-keyed caches for
  stability scanning: atom-symbol evaluation stability and symbol-head
  call-effect decisions. The epoch is bumped and the caches are cleared when
  definitions, attributes, registered builtins, declared package stubs, loaded
  packages, removed symbols, or dynamic-scope definition state changes.
- Tested and rejected a whole-expression exact normal-form cache. With the
  current owned-tree `Expr`, hashing and equality on large expressions became a
  visible new profile cost; keeping only symbol-level caches was faster on the
  Bool smoke.
- Final dependency-preloaded Bool release smoke after this pass:
  `AbsoluteTiming[Get["Bool.wl"]]` about 43.09s and external wall time
  `real 44.49`, with earlier confirmation runs at `real 44.72` and `43.79`.
  This is a modest but repeatable improvement, not a substitute for the shared
  expression / symbol interning work.

## 2026-06-10 Shared Call Tree Follow-up

- Replaced the owned `Expr::Call { head: Box<Expr>, args: Vec<Expr> }`
  representation with shared call nodes:
  `Expr::Call { head: SharedExpr, args: SharedArgs }`, backed by `Arc`.
  `SharedArgs` preserves borrowed iteration and copy-on-write mutation for the
  existing runtime code shape; old owned-Vec boundaries now explicitly call
  `to_vec`/`into_vec`, while pure expression rebuilds can reuse shared args.
- Kept `Expr::call(head, Vec<Expr>)` as the normal source-compatible
  constructor and added shared constructors for paths that can preserve an
  existing argument spine. The expression type remains `Send`/`Sync` friendly by
  using `Arc`, not `Rc`.
- Re-ran the same dependency-preloaded Bool release smoke after this change.
  External wall times were `real 8.09`, `8.07`, and `8.13`; internal
  `AbsoluteTiming[Get["Bool.wl"]]` was about 7.68-7.74s. This is the first
  large structural win on the Bool blocker, down from the prior 43.8-44.7s
  range. Remaining work should now re-profile before choosing between symbol
  interning, node-level evaluated-state bits, and derived-rule workload
  specialization.

## 2026-06-11 Bool Performance Retest

- Re-ran the post-23:30 Bool timing work after a separate system-heavy task
  finished; the late 2026-06-10 timing samples were treated as contaminated
  and not used for decisions.
- Kept the evaluator-side wins that survived clean retesting: call-stability
  caching over shared call trees; the global no-upvalue fast path; planned
  builtin handler dispatch; early stable-expression return from the fixed-point
  evaluator; and copy-on-write binding substitution for rule RHS/template
  substitution.
- Rechecked the late-session decisions that affected the retained diff:
  rule-RHS binding-substitution copy-on-write survived clean A/B timing, while
  skipping call-stability cache entries for leaf calls was effectively neutral
  and was not kept. Other low-yield probes remain reverted: lazy epoch cache
  clearing, fast/default hasher experiments, fixed-array dispatch stages, a
  registered-builtin fallback shortcut, and slot-substitution copy-on-write.
- Final dependency-preloaded Bool release smoke on the clean retest:
  first run `real 3.99`; five warmed runs `real 3.57`, `3.51`, `3.52`,
  `3.53`, and `3.51`. Internal
  `AbsoluteTiming[Get["Bool.wl"]]` for those warmed runs was about
  `3.17-3.22s`, with median about `3.19s`. This is down from the shared
  call-tree baseline of about `8.1s` external wall time, but still does not
  reach the local WolframKernel reference of roughly `2.4s`.
- Clean A/B checks: disabling rule-RHS binding-substitution copy-on-write
  regressed warmed Bool to about `3.59-3.61s` external wall and `3.24-3.26s`
  internal timing; disabling the leaf-call cache skip was effectively neutral,
  so the skip was not kept.

## 2026-06-11 Bool Optimization Direction Sweep

- Re-ran the next five optimization directions in order after the clean
  retest baseline: clean profile, symbol fingerprint/ID, definition matcher
  metadata, expression construction/allocation, and release flags.
- The clean profile confirmed that parser/loader work is no longer the blocker.
  The visible hot costs were allocator/free traffic, string/structural compare,
  SipHash on symbol-keyed maps, stability checks, expression clone/drop/call,
  lexical substitution, rule binding substitution, and definition matcher work.
- Tested a conservative `Symbol` fingerprint field that preserved lexicographic
  `Ord` while hashing only a precomputed deterministic `u64`. It removed the
  visible SipHash string-write hotspot, but warmed Bool wall time stayed in the
  same `3.50-3.57s` range and the larger `Symbol` plus custom serde complexity
  was not kept.
- Tested per-definition matcher metadata for root arity and syntactic fast-path
  eligibility. Runtime tests passed, but warmed Bool stayed around
  `3.53-3.57s`; the additional definition size and branching were not kept.
- Kept the expression-construction win: lexical substitution now uses an
  internal copy-on-write traversal, returning unchanged subtrees instead of
  eagerly rebuilding every visited call. This directly addressed the allocator
  and clone-heavy `Module`/`With` path called out in the external evaluator
  audit.
- Final dependency-preloaded Bool release smoke after lexical-substitution
  COW: first run `real 3.13`; warmed runs `real 2.45`, `2.51`, `2.46`, and
  `2.45`. Internal `AbsoluteTiming[Get["Bool.wl"]]` on those warmed runs was
  about `2.30-2.36s`. This reaches the same order as the local WolframKernel
  reference, previously measured at roughly `2.4s`.
- Release-flag probes: `RUSTFLAGS="-C target-cpu=native"` improved warmed wall
  time further to about `2.29-2.35s` on this machine. `codegen-units=1` and
  thin LTO did not produce a clear additional runtime win over native-only, and
  increased clean build/link time substantially. Treat native CPU builds as an
  optional local benchmark/deployment knob, not the portable default profile.

## 2026-06-11 Bool Evaluator/Representation Follow-Up

- Committed the lexical-substitution COW sweep as `bfadc4b` and continued from
  a clean default-release baseline of about `real 2.45` on the dependency-
  preloaded Bool smoke.
- Kept a lightweight runtime cache hasher for epoch-scoped symbol/effect caches.
  This removed the remaining default SipHash cost from those hot caches without
  changing cache keys or semantics. Warmed default-release Bool smoke moved to
  roughly `real 2.41-2.45`.
- Kept owned call-argument preparation: the evaluator now consumes the owned
  remaining argument vector during normal call preparation instead of cloning
  each argument before planning. Warmed Bool smoke improved to about
  `real 2.38-2.42`.
- Kept an empty-binding rule-RHS fast path. Definition and rewrite-rule storage
  now precomputes whether a RHS may need literal `Sequence` splicing even when
  matcher bindings are empty; rule application can reuse the shared RHS directly
  when it is safe. Warmed Bool smoke improved to about `real 2.36-2.40`.
- Rejected a one-entry call-stability MRU cache after measurement. Runtime tests
  passed, but warmed Bool smoke regressed to about `real 2.41-2.44`, so the
  probe was reverted.
- Deferred deeper performance directions until a future profiling pass:
  borrowed fast paths for compiled matcher application, delayed materialization
  of definition candidates before rule traces/options need an owned expression,
  narrower pointer/shape fast paths for exact expression comparisons, and
  head-specific investigation of repeated `cached_expr_is_evaluation_stable`
  calls. A whole-expression DAG/arena remains a last-resort architecture change
  rather than the next default step.

## 2026-06-11 Latest `mma-hol` Cold Probe

- Cloned sibling `mma-hol` clean HEAD `698e3d0` (`M7-7 (Real)
  Complete.wl: Dedekind completeness + Archimedean + Q-density`) to
  `/private/tmp/rum-mma-hol-latest.FtARYw/mma-hol` and used that fixed clone as
  the probe target so live downstream development can continue independently.
- Baseline low layers still pass on the latest tree: dependency-preloaded
  `kernel_tests.wl` reported `List[71, 0]`; dependency-preloaded
  `bool_tests.wl` reported `List[73, 0]` with external wall time about
  `20.69s` for cold package load plus tests.
- Temporary source normalizations in the `/private/tmp` clone let the probe
  move past syntax sugar without changing `rum`: pattern `|` was rewritten to
  explicit `Alternatives[...]`; several assignment RHS `& /@` forms were
  rewritten to `Map[...]`; `_?pred` was rewritten to
  `PatternTest[_, pred]`; one `Table[heldLhs -> rhs, ...]` mapping was
  rewritten to `MapThread[Rule, ...]`; and a Meson recognizer using
  `$boolEqTy` in a definition lhs was rewritten to compare a captured `ty_` on
  the RHS.
- With those probe-only normalizations, `drule_tests.wl` reported
  `List[43, 0]` and `tactics_tests.wl` reported `List[49, 0]`. This suggests
  the Bool/Drule/Tactics semantic base is now mostly healthy once the promoted
  source/evaluator compatibility gaps below are fixed.
- The promoted compatibility gaps from this latest probe are:
  Wolfram pattern infix `|` lowering to `Alternatives`, `_?` lowering to
  `PatternTest`, correct precedence for assignment RHS forms like
  `ks = #[[1]] & /@ args`, `Table` iterator substitution inside held `Rule`
  lhs positions, evaluation of non-pattern own-value constants in definition
  lhs forms such as `p = a; f[p] := ...`, and a less eager evaluation-cycle
  guard for state-progressing recursive definitions such as
  `parseTypeExpr[]`.
- Additional follow-up observations from this pre-fix probe: `Printer` had a
  low-risk display mismatch for a naked registered infix constant
  (`formatTerm[=]` rendered `"="` instead of `"(=)"`); `Parser` tests stopped
  at the same recursive parser cycle issue; `PropTaut` and `Meson` could load
  and execute small non-parser probes after the syntax/lhs normalizations;
  `Simp` loaded but emitted many recursion-limit messages during tests; and
  the first stdlib type packages (`Pair`, `Sum`, `Option`) then exposed
  derived-theorem failures that should be revisited after the evaluator/source
  fixes, not before.

## 2026-06-11 CL-Y Promoted Compatibility Fixes

- Folded the latest `mma-hol` probe-only normalizations back into `rum`:
  source `|` now lowers to `Alternatives`, `_?pred` lowers to `PatternTest`,
  and pure-function postfix parsing now lets rule bodies such as
  `Map[f[#[[1]]] -> #[[2]] &, pairs]` become `Function[Rule[...]]` while
  preserving assignment RHS map forms such as `ks = #[[1]] & /@ args`.
- Aligned rule/definition evaluation for the observed HOL workloads:
  `Rule` / `RuleDelayed` evaluate their lhs enough for substitution into
  `Table`-generated held rules; pattern names remain protected while blank
  heads and non-pattern constants are normalized; and definition lhs constants
  with ownvalues now store rules under the evaluated form (`p = a; f[p] := ...`
  defines `f[a]`).
- Replaced the same-expression cycle guard with an epoch-aware guard so
  state-progressing recursive definitions can continue after mutations while
  unchanged direct and mutual recursion still fail fast.
- Fixed ordinary-expression pattern resolution for `MatchQ` / `MemberQ` style
  calls: explicit pattern syntax stays held, but non-pattern expressions such
  as `spec["kind"]` are evaluated before compiling an exact pattern. This
  cleared the `Printer` registered-infix-constant failure.
- Current clean-clone `mma-hol` HEAD `698e3d0` smoke results, using
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` without source
  normalizations:
  `bool_tests.wl` reports `List[73, 0]` with cold release wall time about
  `22.95s`; `drule_tests.wl` reports `List[43, 0]` in about `4.18s`;
  `tactics_tests.wl` reports `List[49, 0]` in about `5.18s`; and
  `printer_tests.wl` plus `parser_tests.wl` report `List[126, 0]` in about
  `4.62s`.
- Verification: `cargo fmt --check`, `cargo test -p rum-syntax`,
  `cargo test -p rum-runtime`, and `cargo test --workspace` pass. The next
  CL-Y step is a bounded cold `run_all.wls` driver and triage of the first
  stdlib/auto failure beyond Parser/Printer.

## 2026-06-11 Bounded `mma-hol` `run_all` Probe

- Added a temporary out-of-tree probe script at
  `/private/tmp/rum_mma_hol_bounded_run_all_probe.wl` and then switched to
  one-layer `rum-cli --eval` probes because `--file` did not surface enough
  intermediate progress for long runs.
- First post-Parser/Printer test frontier is performance, not correctness:
  `proptaut_tests.wl` reaches
  `propTaut: distribution ⊢ p ∨ (q ∧ r) = (p ∨ q) ∧ (p ∨ r)` and exceeds a
  90s suite budget. The isolated distribution formula eventually succeeds
  (`{True, True}` for theorem status and conclusion alpha-equivalence) but
  takes about `real 60.68s`. With a 300s suite budget, `proptaut_tests.wl`
  still times out after reaching `nnfThm: literal unchanged`, so `PropTaut`
  is the first new performance gate.
- Package-only loading through `Parser`, `PropTaut`, `Meson`, and `Simp`
  succeeds under the current probe. The first stdlib package gate is
  `stdlib/Pair.wl`: with a 60s per-package load budget it times out; with a
  180s budget it reaches an uncaught HOL error after about `real 119.66s`.
- The `Pair.wl` failure is semantic/correctness-oriented rather than a parser
  failure. Non-quiet loading reports `concl: not a theorem` while constructing
  `pairInjThm`: `HOL`Bool`MP[...]` tries to use `mkPairInjThm`, but
  `mkPairInjThm` is not a theorem at that point. Running `pair_tests.wl` after
  the failed load cascades into missing constants/types (`mkPair`, `prod`,
  `ABS_prod`, `REP_prod`, `FST`, `SND`) because `Get` rolls back the failed
  package load.
- Current triage order after Parser/Printer:
  1. profile/optimize the `PropTaut` proof-construction path enough that the
     propositional suite becomes a useful signal;
  2. debug why `Pair.wl` fails to construct `mkPairInjThm`, likely around the
     `Simp`simpConv` / `CONVRULE` path used to simplify the unfolded
     `mkPair` equality;
  3. only after `Pair` loads cleanly, revisit `Sum`, `Option`, `Set`, `Num`,
     and later Real packages, because their current probe status is polluted by
     the failed Pair dependency.

## 2026-06-11 `Pair.wl` Recursion-Limit Triage

- Isolated the first `Pair.wl` hard failure to the `mkPairInjThm` construction:
  the first simplifier conversion over `mkPairDefThm` exceeded rum's old
  default recursion limit of `256` after about `real 121.13s`, producing
  `TerminatedEvaluation["RecursionLimit"]` instead of a kernel theorem. The
  later `pairInjThm` construction then consumed that value as `mkPairInjThm`,
  leading to the observed `concl: not a theorem` failure.
- Checked the local Wolfram kernel baseline: `$RecursionLimit` is `1024`.
  Raised rum's default recursion limit from `256` to `1024` and added a runtime
  regression test for the default. Explicitly lowered test sessions still
  produce `TerminatedEvaluation["RecursionLimit"]`, so the resource-limit
  behavior remains covered.
- Stage probes with the raised limit show that direct `mkPairDefThm` rewriting
  is not the bottleneck: `REWRCONV` / `baseUser` / top-level `TRYCONV` are
  millisecond-to-0.1s operations. The remaining cost is `PropTaut`-backed
  simplifier schema construction: `basicSimpset[]` takes about `106-110s`, the
  first `andCongLemma[]` use via `stepAndXX` takes about `99s`, and later
  `mkPair`-term fixpoints still take tens of seconds.
- A real `Pair.wl` load with the raised default passed the old `real 119.66s`
  hard-failure point without reporting the old non-theorem error, but was
  intentionally terminated after `real 335.35s`. Therefore the correctness
  blocker is explained and the default-limit fix is in place, but `Pair.wl` is
  not yet a green smoke. This identified `PropTaut` / `Simp` performance, not
  more ad hoc Pair-source debugging, as the remaining Pair lane blocker.

## 2026-06-11 Loader Resource-Failure Contract

- Split top-level `TerminatedEvaluation` handling from file-load handling.
  Interactive `eval_text`, `eval_script`, and ordinary eval-request paths still
  convert recursion, iteration, and current resource-budget errors into
  `TerminatedEvaluation[...]` values with the existing `General::*` message.
- `eval_file` / `Get` now evaluate loaded forms with resource termination
  propagated as a real load error. A resource-limit hit during `Get` therefore
  returns `$Failed`, emits the outer `Get::load` message, hides the internal
  failing-form message just like other failed loads, and rolls back definitions
  and package-loader state.
- Added a runtime regression that lowers `$RecursionLimit`, triggers recursion
  inside a loaded file, and verifies that pre-failure assignments, post-failure
  assignments, and the recursive definition are all absent after the failed
  load.

## 2026-06-11 `mma-hol` Harness And Snapshot Audit

- Added `scripts/mma-hol-bounded-smoke.sh`, a reusable bounded diagnostic
  driver for the `mma-hol` lane. It supports package-load probes, independent
  test-file probes, per-stage wall timeouts, captured stdout/stderr, and a
  tab-separated summary. The driver intentionally replays load prefixes in
  fresh `rum-cli` sessions, so it localizes failures but is not a timing
  benchmark.
- Added `docs/mma-hol-smoke-and-snapshot.md` as the canonical note for the
  driver contract and stable snapshot audit. The snapshot lane remains below
  the cold semantic smoke. At audit time, its concrete missing support was:
  `Contexts[]`, `$ContextPath` / `$Packages` session-field assignment,
  `FileExistsQ`, `FileDate`, `FileByteCount`, `AbsoluteTime`, `Round`,
  `FileNames[..., ..., Infinity]`, and a focused `DumpSave` / `Get` context
  round-trip regression.

## 2026-06-11 Stable Snapshot Helper Builtins

- Added the first small-helper tranche for `mma-hol` stable snapshot scripts:
  `Contexts[]`, `FileExistsQ`, `FileByteCount`, `FileDate[..., "Modification"]`,
  `AbsoluteTime`, `Round`, and `FileNames[pattern, dir, Infinity]`.
- Kept the timing-literal support narrow: simple `digits.digits` source forms
  lower to exact rationals, enough for `Round[..., 0.1]` / `Round[..., 0.01]`
  without introducing Wolfram machine-real or precision semantics.
- Added focused runtime regressions for context enumeration, recursive file
  discovery, file metadata helpers, exact/decimal `Round`, and decimal
  lowering. Remaining stable snapshot work is `$ContextPath` / `$Packages`
  session-field assignment plus a focused `DumpSave` / `Get` context
  round-trip regression.

## 2026-06-11 Stable Snapshot Restore And Lint Helpers

- Added session-field assignment for `$ContextPath` and `$Packages`, so stable
  snapshot restore code can mutate runtime lookup/package state instead of
  creating inert own values.
- Extended textual `DumpSave` object handling so context strings from
  `Evaluate[holContexts]` save all persisted symbols in those contexts. Added a
  tiny public/private context round-trip regression that restores via `Get`,
  updates `$ContextPath` / `$Packages`, and calls the restored public symbol.
- Added text-only `Row`, lint-only `Import[..., "Text"]`, and string-pattern
  wildcards such as `"Snap`*"` for the stable snapshot context filter shape.
  The remaining boundary is full validation of `mma-hol` stable scripts and
  private-context lint, not the small helper availability itself.

## 2026-06-11 `PropTaut` Performance Harness Cleanup

- Re-established the current `PropTaut` performance frontier on clean
  `mma-hol` HEAD `698e3d0`: package loading through `auto/PropTaut.wl` is about
  `2.49s`, while the single distribution theorem
  `(p || (q && r)) = ((p || q) && (p || r))` still takes about `49.7-50.1s`
  internally in release. The full `proptaut_tests.wl` suite remains a
  performance gate, not a package-loading gate.
- Captured a macOS `sample` profile for a repeated distribution probe. The hot
  stacks are still dominated by allocation/free traffic, structural compare,
  `cached_expr_is_evaluation_stable`, evaluator dispatch, binding/lexical
  substitution, and builtin/evaluator lookup. Parser and loader work are not
  the bottleneck.
- Rejected two low-yield or unsafe probes: a broad literal `ReplaceAll`
  shortcut broke `Rule` evaluation boundaries inside `PropTaut`, and a borrowed
  rewrite-matcher fast path was effectively neutral on the distribution probe.
  A lightweight hasher for the builtin-name map also regressed the same probe
  and was reverted.
- Fixed the bounded `mma-hol` smoke driver so test stages preload their needed
  package prefix and added `--only LABEL` for single-stage probes. With
  `RUM_CLI_BIN=target/release/rum-cli`, `test/parser_tests` now runs as a
  single green stage, while `test/proptaut_tests --timeout 20` reports the
  intended timeout after preloading through `auto/PropTaut.wl`.

## Completed Architecture Work

- The workspace was split into responsibility-oriented crates:
  `rum-syntax`, `rum-core`, `rum-runtime`, `rum-stdlib`, `rum-terminal`,
  `rum-repl`, `rum-cli`, `rum-host`, `rum-jupyter`, and `rum-difftest`.
- Major `include!`-style and monolithic runtime, syntax, Jupyter, and
  difftest files were replaced with normal Rust module trees.
- The 2026 source-size cleanup pass removed Rust files over the `35k`
  architecture-debt threshold; the remaining work is mostly warning-range
  opportunistic cleanup.
- `rum-host` now provides the first-class Rust embedding facade over runtime
  sessions, requests, structured outcomes, failures, inspection, completion,
  completeness checks, and input suspension/resume.
- `rum-jupyter` was kept optional and layered as an adapter instead of becoming
  evaluator infrastructure.

## Completed Core-Language Mainlines

The `CL-01` through `CL-09` mainlines are considered closed for the current
kernel-first phase. Remaining gaps are demand-driven parity tails, not default
implementation work.

Broad completed areas include:

- expression model, lowering, parser diagnostics, formatter, exact integers
- definitions, attributes, values, scoping, contexts, package loading
- matcher IR, rules, patterns, replacement, traversal, and tree walkers
- functional/list combinators across the current first-pass supported subset
- associations, keyed data, missing values, keyed mutation, and grouping
- procedural control, nonlocal flow, cleanup, status objects, host input
- string/text surface, string patterns, formatting, templates, byte arrays
- package/options/protection/name-inspection tooling
- tracing, messages, resource limits, timing, memory proxies, test reports

The useful lesson from this phase is that every new Wolfram same-name builtin
is expensive: it tends to touch evaluator semantics, messages, resource
budgets, suspension/resume, host surfaces, docs, and conformance fixtures.
Future additions should therefore be promoted by a concrete blocker rather
than by inventory coverage alone.

## Archived Roadmap Detail

The old `TODO.md` carried long completed checklists for individual CL waves,
external-review fixes, difftest lane construction, string-language expansion,
and math-surface footholds. Those details were compressed on 2026-06-10 so the
roadmap can show active decisions instead of completed work.

The archived completed waves include:

- `CL-08` package, namespace, options, protection, persistence, and name
  tooling
- `CL-09` tracing, timing, resource-limit proxies, monitors, messages,
  evaluation data, test reports, and dialog support
- post-mainline closure passes for `CL-01` through `CL-09`
- first practical math footholds for exact arithmetic, polynomial expansion,
  coefficient extraction, factorization, `Together`, and symbolic `D`
- the first large string-language surface, including string construction,
  exact search, string patterns, templates, conversions, byte arrays,
  hashing/compression/encryption, nearest-neighbor helpers, and pluralization
- resolved external-review P0/P1 items for definition replacement, Jupyter
  signing/codec/frame hardening, conformance fixtures, and package/context
  coverage

Canonical current status should be read from the support matrix and tests, not
from the archived checklist wording.

## Verification And Harness Work

The `rum-difftest` workstream now has:

- Lane A for stateless Wolfram-oracle comparisons
- Lane B for stateful session comparisons
- Lane C for checked known-difference ledger validation
- Lane D for local performance smoke/baseline runs
- non-oracle inventory and aggregate `check` gates
- artifact-hygiene checks to keep generated oracle material out of the repo
- summary JSON output with stable top-level schema versions

The practical default gate remains:

```bash
cargo fmt --check
cargo test --workspace
cargo run -p rum-difftest -- check
```

Wolfram oracle runs are intentionally environment-gated and should remain
optional for normal development.

The Jupyter harness now has an isolated `jupyter_home/` deploy path with real
`jupyter_client` and JupyterLab smoke commands. `kernel-smoke` covers kernel
start/ready/execute/restart/shutdown, stdin suspend/resume, completion,
inspection, is-complete, history, stdout/stderr stream mapping, runtime
messages, failure-to-error mapping, and unsupported-message keepalive over real
ZMQ channels. Unsupported-message handling now returns an empty
`comm_info_reply`, ignores comm open/msg/close, returns an explicit unsupported
`debug_reply`, and ignores unknown frontend probes so the kernel stays alive.
`browser-smoke` covers notebook execution through JupyterLab and headless
Chromium. On 2026-06-13, `cargo test -p rum-host`, `cargo test -p
rum-jupyter`, `scripts/rum-jupyter-home.sh kernel-smoke`, and
`scripts/rum-jupyter-home.sh browser-smoke` all passed locally. The Codex
sandbox blocks loopback-port binding, so the native ZeroMQ and real Jupyter
smokes need unsandboxed execution, but their filesystem state stays under
`jupyter_home/`. Robust interrupt/cancel, widgets/comm state, and the debug
protocol remain outside the current contract.

## Operational Lessons

- Do not use the full Wolfram builtin inventory as a commitment list.
- Prefer local WolframKernel spot checks for same-name semantics, but do not
  commit generated oracle transcripts.
- Keep resource limits cooperative and explicit. Generated-output functions
  need budget checks before allocation.
- Keep support-matrix statuses conservative. `partial` means only the
  documented subset is compatibility-targeted.
- Prefer one real downstream driver over multiple speculative examples.
- Move completed roadmap detail here once it no longer changes decisions.

## Frozen Or Archived Directions

### Python Adapter

Python remains a plausible future host, especially for AI/data/tooling
workflows, but project-owned Python binding work is frozen for now.

If revived later, the rules remain:

- build on `rum-host`, not broad `rum-runtime`
- keep `pyo3` and packaging concerns outside kernel crates
- start with eval/resume/inspect/complete/is-complete only
- add a real downstream use case before committing to public Python API shape

### Standalone Showcase Plans

The detailed Python AI policy and Rust Wasm prototype plans were removed to
avoid turning speculative examples into roadmap obligations.

The underlying application directions remain valid as short examples of where a
symbolic rule-engine kernel could be useful:

- policy / validation / rewrite layers in AI or data tooling
- semantics and validation prototypes for compilers, runtimes, or DSLs
