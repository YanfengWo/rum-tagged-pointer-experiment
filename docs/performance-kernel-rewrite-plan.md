# Performance Kernel Rewrite Plan

This document is the current mainline plan for making the supported `rum`
kernel slice reach WolframKernel parity or better. The target remains
deliberately ambitious: `rum` implements a much smaller Wolfram-style language
surface, so its supported path should not permanently run several times slower
than WolframKernel on the same workload.

## Course Correction

The previous rewrite plan was too tightly anchored to `Pair.wl` / `Simp`
phase trees. Those probes are real downstream evidence, but they are too
compound to be the optimization entry point. Repeatedly drilling inside
`Pair.wl` made it too easy to explain a complex symptom without eliminating the
small runtime constants that create the symptom.

The new rule is bottom-up:

- Start with the smallest Wolfram-language statements that both engines can
  run.
- Compare rum and WolframKernel on the same snippet.
- If a primitive statement or one-step construct is `4-5x` slower, stop there
  and optimize that primitive before moving upward.
- Use `mma-hol` entry files statement-by-statement only after the core
  primitive ladder is understood.
- Use `Pair.wl` only as a downstream acceptance target, not as the first place
  to search for the next optimization.

## Non-Negotiable Method

Every performance claim must name the benchmark label, the rum timing, the
WolframKernel timing when applicable, and the ratio. A change without a repeat
measurement on the same label is not accepted.

The optimization loop is:

1. Measure a minimal statement or construct in both engines.
2. Identify the first label whose ratio is clearly above the target band.
3. Reduce that label to the smallest reproducer if needed.
4. Optimize only the runtime mechanism needed by that label.
5. Rerun the same label, the neighboring labels in the same tier, and focused
   semantic tests.
6. Move to the next label only after the current primitive is understood or a
   documented reason shows it is not the current blocker.

Avoid these failure modes:

- descending into a larger `Pair.wl` child before proving the relevant
  primitive constants are already competitive;
- chasing sub-second side probes while the parent workload is hundreds of
  seconds;
- changing downstream `mma-hol` as the acceptance path;
- comparing profiled rum timings with unprofiled WolframKernel timings;
- accepting an optimization because it helps one isolated row while regressing
  the same-tier baseline.

## Benchmark Ladder

The canonical Stage 0 runner and label set live in
[Performance Stage 0 Baseline](./performance-stage0-baseline.md).

### Stage 0: Core Microbench Baseline

Goal: establish a machine-readable bottom-up timing table before changing more
runtime code.

Tasks:

- Use `scripts/core-language-microbench.sh` to run the same minimal snippets in
  rum and WolframKernel.
- Include an empty-loop baseline so harness overhead is visible.
- Record `measurements.tsv` and `ratios.tsv` for the core label set.
- Treat any `4-5x` primitive ratio as the next candidate for runtime work.

Acceptance:

- The runner completes for both engines on the local machine.
- Labels include atom/symbol lookup, list/call basics, control forms,
  definition dispatch, matching/rewrite, and small string/list builtins.
- The next optimization target is selected from this table, not from `Pair.wl`.

### Stage 1: Atom, Symbol, And Structural Basics

Goal: ordinary atoms and small expression structures should not pay avoidable
interpreter overhead.

Representative labels:

- integer, string, and symbol evaluation;
- OwnValue read and write;
- list and call construction;
- `Head`, `Length`, `Part`;
- `SameQ` on atoms and small expressions.

Likely runtime areas: symbol ids, atom storage, call-node representation,
read-only structural traversal, and equality/fingerprint paths.

Acceptance:

- The slowest retained label in this tier is explained and either improved or
  explicitly deferred with evidence.
- Adjacent labels do not regress after each cut.

### Stage 2: Control Forms And Frames

Goal: common evaluator control constructs should be cheap enough to use as
building blocks inside packages.

Representative labels:

- `If`;
- `Do` and `Table`;
- `Module`, `Block`, and `With`;
- pure function application and curried calls;
- `Map` / `Apply` basics when they are promoted into the benchmark set.

Likely runtime areas: argument preparation, held/non-held call plans, lexical
frame representation, substitution planning, and continuation overhead.

Acceptance:

- Empty-loop and loop-body overhead are separated before optimizing loop-like
  labels.
- Frame-heavy labels narrow toward the same ratio band as atom/structure
  labels.

### Stage 3: Definition Dispatch

Goal: value and rule lookup should fail or succeed quickly.

Representative labels:

- OwnValue read and miss;
- DownValue exact hit;
- DownValue pattern hit;
- DownValue miss;
- SubValue / curried-call lookup when isolated;
- attribute lookup.

Likely runtime areas: id-keyed definition maps, head/arity prefilters,
definition-version checks, and compiled candidate metadata.

Acceptance:

- Hit and miss paths are measured separately.
- A dispatch optimization must improve the minimal dispatch label before it is
  credited to larger proof workloads.

### Stage 4: Matching And Rewrite Basics

Goal: the ordinary rule failure path should be cheap before rule-chain plans or
conversion-specific caches are considered.

Representative labels:

- `MatchQ` success and failure;
- blank and typed blank matches;
- simple `ReplaceAll`;
- no-change rewrite failure;
- `Cases` and `Select` basics;
- `RuleDelayed` only after same-name semantic coverage is confirmed.

Likely runtime areas: matcher plans, structural traversal, binding maps,
held RHS evaluation, and failure representation.

Acceptance:

- A failed match has a minimal benchmark that compares against WolframKernel.
- Larger `REWRCONV` / `tryTermMatch` probes are not optimized until the basic
  matcher labels are understood.

Bridge label (carry the Task 2d finding forward — do not rediscover it). The
isolated native-dispatch labels do not reproduce the Pair gate: the core
microbench `downvalue-pattern-miss` is ~`3.2x`, but the Pair/Simp failed
`REWRCONV` dispatch was measured at ~`34x` (same `17240` failed dispatches in
both engines; rum ~`5.3ms` vs WK ~`0.16ms` each). That ~10x gap means the Pair
multiplier is *not* the native pattern-miss primitive — it is a WL-level
conversion-combinator path: each failed rewrite goes through
`REWRCONV → tryTermMatch → convFail` wrapped in the `simpRuleConv` / `ORELSEC`
closures that `buildBaseConv` constructs, and the prime suspect is per-attempt
combinator / `Module`-closure construction rebuilt per subterm. Add a core
microbench label that isolates this shape — a held equational rule applied to a
mismatched term through a conversion-combinator wrapper that fails — so the
ladder has a rung for the actual Pair driver. Without it, fixing
`downvalue-pattern-hit`/`-miss` to ~`2x` will look like progress while Pair
stays slow, because that native path is not the one Pair exercises. The Stage 4
matcher work and Stage 5 `Catch`/`Throw` work both feed this; the explicit
profile target is one failed `REWRCONV` in the Simp context (was handoff
Task 2d).

### Stage 5: Nonlocal Control And Runtime Boundaries

Goal: control transfer should be correct and cheap enough for package-level
combinators.

Representative labels:

- `Catch` / `Throw`;
- `Return` from nested frames;
- `Check` / message-control paths when relevant;
- resumable host boundaries only for the v0.1 contract surface.

Likely runtime areas: internal nonlocal representation, boundary conversion,
module-frame unwinding, and suspension-safe continuation state.

Acceptance:

- Semantic tests remain green for nonlocal and suspension behavior.
- Performance work does not weaken the host/Jupyter keepalive and resume
  boundaries already in scope for v0.1.

### Stage 6: `mma-hol` Entry-File Statement Lane

Goal: connect the primitive ladder to real downstream source without returning
to whole-file guesswork.

Method:

- Use the clean unmodified `mma-hol` clone.
- Start from the lower files: `ErrorUtil.wl`, `Types.wl`, `Terms.wl`,
  `Kernel.wl`, `Bootstrap.wl`, `Equal.wl`, `Bool.wl`, and `Drule.wl`.
- Time top-level statements or small consecutive groups in rum and
  WolframKernel.
- When a group shows a large ratio, reduce it back to a primitive label or add
  a new core microbench label.

Acceptance:

- The next runtime cut is justified by a minimal core or statement-level
  benchmark, not by a large opaque file load.
- The unmodified downstream source remains the standard.

### Stage 7: Pair/Simp Acceptance

Goal: verify that primitive and statement-level improvements compound on the
known downstream gate.

Use Pair/Simp probes only after the lower ladder justifies the relevant
runtime work:

- `pair4-stock-repeat8`;
- first `mkPairInj` `CONVRULE` root;
- dependency-preloaded `Pair.wl`;
- eventually full `Pair.wl` load.

Acceptance:

- Pair/Simp improves without downstream patches.
- A Pair/Simp regression blocks a runtime cut only after the same-tier core
  labels and semantic tests have been checked.
- Pair/Simp remains an acceptance gate, not the default place to discover the
  next primitive optimization.

### Stage 8: Optional Bytecode Or JIT

Goal: compile stable plans only after interpreter constants are no longer the
obvious gap.

This remains intentionally late. A bytecode or JIT path that compiles the
current slow abstractions would preserve excess allocation, dispatch, and
matching overhead. Before considering machine-code generation, prefer compact
plans for evaluator forms, matcher plans, definition dispatch, and conversion
chains.

Acceptance:

- Stages 1-7 identify a residual hot path that the interpreter design cannot
  plausibly close.
- The compiled path is optional, testable, and does not become the semantic
  source of truth.

## Milestones

Near term:

- Add and run the core-language microbench runner.
- Select the first high-ratio primitive from `ratios.tsv`.
- Optimize that primitive and prove the result on the same label.

Mid term:

- Work through the promoted core-language labels until common primitives are
  near WolframKernel scale.
- Add `mma-hol` entry-file statement timing only where the core ladder needs
  real source pressure.

`v0.1` performance target:

- The clean `mma-hol` stable snapshot and proof-load path should run without
  source patches and without pathological constant-factor gaps on the supported
  slice.
- Dependency-preloaded `Pair.wl` remains a downstream acceptance target, but it
  is reached through the lower benchmark ladder.

Stretch target:

- Beat WolframKernel on the supported `mma-hol` target slice by using rum's
  smaller feature surface and domain-specific runtime plans while preserving
  Wolfram-compatible semantics for implemented same-name behavior.

## Execution Rule

The next optimization target must come from the lowest benchmark tier that
still shows a clear gap. Run focused semantic tests before and after runtime
changes. After a primitive improves, rerun the same microbench tier; only then
move upward to `mma-hol` statement probes and Pair/Simp acceptance.
