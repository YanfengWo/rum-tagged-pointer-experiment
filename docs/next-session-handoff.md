# Next Session Handoff: Meson Triage, Pair Multiplier Hunt, Representation Work

Date: 2026-06-12. State as of commit `f06c23c`. This note hands the current
`mma-hol` frontier to the next development session. Read it together with
the two PROGRESS entries "Representation Optimization Pass (External)" and
"Block Scope Invalidation Fix (External)", and with
`docs/proptaut-performance-audit.md` for the diagnosis method.

## Where things stand

| Gate | Status |
| --- | --- |
| PropTaut distribution probe | `13.2-13.4s` release internal vs WolframKernel `3.80s` (~3.4x) |
| Bool-stack load | `0.78s` vs `0.19s` (~4.1x) |
| `proptaut_tests.wl` | green, ~`238s` |
| `meson_tests.wl` | **completes** (~`92s`) but reports `List[34, 9]` — 9 semantic failures, all pre-existing |
| `stdlib/Pair.wl` load | still exceeds `600s` budget |
| `cargo test --workspace`, `rum-difftest check` | pass |

The probe-visible cache-invalidation lane is closed: Do/Table iterator
Blocks no longer wipe the stability caches and matcher revisions per
iteration (`6df7bfe`), and the re-access probe is flat in expression size.
Constant-factor tuning of the current representation has reached
diminishing returns; the items below are ordered by expected value.

## Task 1 (correctness, do first): `Catch` does not evaluate its tag form

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

Acceptance: a runtime regression test for the symbol-valued tag repro
(both the catching and the deliberately non-matching case), the bounded
`test/meson_tests` smoke reporting `List[43, 0]`, and
`test/meson_proveprop_tests` re-probed afterwards (its current status is
polluted by this bug).

## Task 2 (diagnosis before optimization): Pair/Simp multiplier hunt

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

Acceptance for this task is a written verdict (PROGRESS entry): either
"Pair is algorithmically divergent at <specific construct>" with a repro,
or "Pair is constant-bound; needs Task 3", with the counter numbers.

## Task 3 (large, only after 1-2): representation rework

Unchanged from the prior plan, in order: single-allocation call node
(head+args in one `Arc`), which halves per-node allocation/refcount
traffic and enables an inline per-node evaluated stamp replacing the
pointer-keyed call-stability side cache; then borrowed argument
preparation so shared argument vectors are not cloned per evaluation
pass. Multi-day, cross-cutting; re-estimate after Task 2 because a Pair
multiplier fix may change the payoff calculus.

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
