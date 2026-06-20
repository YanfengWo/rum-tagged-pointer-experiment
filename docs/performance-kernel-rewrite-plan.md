# Performance Kernel Rewrite Plan

This document is the mainline plan for bringing the `mma-hol` target slice of
`rum` to WolframKernel parity or better. The premise is deliberate: `rum` does
not implement the full WolframKernel feature surface, so the supported kernel
slice should eventually be able to match or beat WolframKernel on the same
downstream workload.

The plan replaces ad hoc performance work with staged runtime engineering. The
current `Pair.wl` / `Simp` wall is not expected to fall to isolated micro
patches. The work must move down to representation, dispatch, matching,
control flow, and allocation constants, with root-linked benchmarks proving
each stage.

## Current Evidence

The retained `mma-hol` pressure test is the unmodified clean checkout. Do not
patch downstream files to pass a rum benchmark.

Current Pair/Simp evidence shows:

- `pair4-stock-repeat8` remains roughly `6x` slower in rum than
  WolframKernel on the same machine.
- WolframKernel does not avoid the visible WL-level conversion failure chain:
  the depth-19 base-conversion probe still runs about `19` failed
  `REWRCONV` / `simpRuleConv` calls per attempt.
- WolframKernel wins by making the ordinary failure path cheap. Its visible
  failures are tens to low hundreds of microseconds under intrusive WL
  wrappers, while rum pays larger constants in dispatch, matching, structural
  operations, call application, and nonlocal control.
- Broad generic call-result caching and held-`Catch` failure-result caching
  were measured and rejected. A failed-conversion/no-change cache may still be
  useful as a rum-specific optimization, but it should not be treated as the
  inferred WolframKernel mechanism.

## Non-Negotiable Method

Every performance stage must keep three benchmark layers:

- microbenchmarks for the primitive under work, such as symbol lookup,
  downvalue miss, `Part`, `SameQ`, `MatchQ` failure, `Catch`/`Throw`,
  `Module`, curried call application, and structural hashing/equality.
- mid-level Pair/Simp probes, especially `basic-chain-orelsec-split`,
  `repeatc-tryconv-control-split`, and `pair4-stock-repeat8`.
- root benchmarks: the first `mkPairInj` `CONVRULE` root,
  dependency-preloaded `Pair.wl`, and eventually full `Pair.wl` load.

For each stage, record:

- rum release timing without profiling.
- WolframKernel timing where the comparison is meaningful.
- rum/WolframKernel ratio.
- allocation, clone, dispatch, and matcher counters when relevant.
- profile timing separately from no-profile timing.

Do not accept a stage on an isolated microprobe if it is neutral or negative
on the root-linked Pair/Simp targets. Do not compare profiled rum timing with
unprofiled WolframKernel timing as if they were equivalent.

## Stage 0: Baseline Freeze

Goal: make performance changes auditable.

Tasks:

- Create or refresh a machine-readable performance baseline for the micro,
  mid, and root benchmark layers.
- Keep WolframKernel baselines alongside rum baselines for comparable WL
  workloads.
- Preserve known negative experiments in `PROGRESS.md` and the performance
  audit so later agents do not repeat them.
- Ensure all long-running probes use `scripts/run-bounded.py` or a wrapper
  that kills the whole process group.

Acceptance:

- A baseline file or documented table names every benchmark used by the stage.
- Each benchmark has a no-profile rum timing.
- The root Pair/Simp target is represented, not only tiny reproducers.

## Stage 1: Symbol And Identity Hot Paths

Goal: no hot evaluator path should compare or clone symbol strings.

Tasks:

- Make dense `SymbolId` the primary key for definitions, attributes, builtin
  dispatch, contexts, messages, and binding maps.
- Keep context/name strings for parsing, display, and diagnostics only.
- Audit hot maps so they are id-keyed rather than string-keyed.
- Add counters or profiles that make symbol string traffic visible.

Acceptance:

- Symbol lookup, attribute lookup, and builtin dispatch microbenchmarks are
  close to WolframKernel scale.
- Pair/Simp profiles no longer show meaningful `String::clone`, string hash,
  or string compare cost in hot rows.
- `pair4-stock-repeat8` does not regress.

## Stage 2: Expression Representation

Goal: make expression traversal and equality cheap enough for proof workloads.

Tasks:

- Move toward compact atom storage and single-allocation call nodes.
- Inline or small-vector common arities, especially unary, binary, and ternary
  calls.
- Avoid repeated `Arc<Vec<Expr>>` decomposition and reconstruction on hot
  paths.
- Add structural hash/fingerprint or stable stamps where they can be read
  without recursive rescans.
- Keep `Head`, `Length`, `Part`, `SameQ`, and no-match structural traversal
  read-only and allocation-free where possible.

Acceptance:

- `tryTermMatch` failure, `REWRCONV` failure, `SameQ`, and `Part`
  microbenchmarks move toward a `1-2x` WolframKernel ratio.
- Allocation count for `pair4-stock-repeat8` falls by at least an order of
  magnitude before accepting deep representation churn.
- Parser/printer, snapshot, and runtime semantic tests remain green.

## Stage 3: Borrowed Evaluator Frames And Call Application

Goal: avoid rebuilding expressions merely to evaluate ordinary calls.

Tasks:

- Teach argument preparation to work over borrowed argument slices where the
  call does not need ownership.
- Compile held/non-held argument behavior into compact call plans.
- Reduce temporary call construction for `Function`, curried application, and
  pure function application.
- Represent `Module` locals and substitution plans without repeatedly walking
  and cloning unchanged subtrees.
- Keep internal `Return`, `Throw`, and `Catch` control flow lightweight; only
  convert to public failure/error shapes at the boundary.

Acceptance:

- `repeatc-tryconv-control-split` shows materially lower `TRYCONV` /
  `REPEATC` wrapper overhead.
- `Catch`/`Throw` microbenchmarks narrow against WolframKernel.
- Suspend/resume, host request, and nonlocal control tests remain green.

## Stage 4: Definition Dispatch And Matcher Plans

Goal: make failed rules fail quickly.

Tasks:

- Compile downvalues and subvalues into dispatch entries with head, arity, and
  constant-shape prefilters.
- Compile rule patterns into reusable matcher plans.
- Keep simple `Blank`, `Blank[head]`, named pattern, condition,
  `PatternTest`, and alternatives paths split by cost.
- Give `REWRCONV` a dedicated LHS matcher plan instead of routing every
  failure through the most general matcher path.
- Keep matcher plans invalidated by definition and attribute versions, not by
  broad global guesses.

Acceptance:

- Failed `tryTermMatch` and failed `REWRCONV` approach WolframKernel scale.
- The depth-19 base-conversion probe is no longer a `5-6x` rum/WolframKernel
  gap.
- `pair4-stock-repeat8` reaches the low single-digit-second range on the
  current machine.

## Stage 5: Conversion And Rule-Chain Plans

Goal: execute conversion chains as plans without changing `mma-hol` source.

Tasks:

- Recognize stable WL-level shapes such as right-associated `ORELSEC` chains
  and `TRYCONV[REPEATC[baseConv]]` as internal execution plans.
- Preserve exact WL semantics: this is an implementation plan, not a
  downstream `Simp.wl` rewrite.
- Cache plan construction by function definition versions, rule theorem
  identity, context theorem identity, and relevant attributes.
- Consider a sound failed-conversion/no-change key only after the ordinary
  failure path is cheap and the key can be proven root-linked.

Acceptance:

- The number of WL-level failed rules may remain the same as WolframKernel,
  but per-failure cost should be close to WolframKernel.
- If a rum-specific no-change cache is retained, it must improve
  `pair4-stock-repeat8` and the first `CONVRULE` root without changing
  downstream files.
- The first `mkPairInj` `CONVRULE` root should approach the current
  WolframKernel timing, with a stretch goal of beating it.

## Stage 6: Memory System

Goal: remove allocator traffic as a first-order runtime cost.

Tasks:

- Introduce session-local arenas or generational arenas for short-lived
  evaluator temporaries.
- Keep long-lived definitions and snapshots in immutable, shareable nodes.
- Use hash-consing only where benchmarked stable-expression reuse justifies
  it; avoid global blind hash-consing.
- Add clone/allocation accounting to the root Pair/Simp and PropTaut lanes.

Acceptance:

- Allocator self-time is no longer a dominant sample group.
- `Pair.wl` full load clears the bounded budget on the unmodified clone.
- PropTaut and Pair/Simp both improve; an optimization that helps one while
  regressing the other needs explicit justification.

## Stage 7: Optional Bytecode Or JIT

Goal: compile stable plans only after representation and dispatch are ready.

This is intentionally late. A JIT that compiles the current abstractions would
only make the wrong shape faster while preserving excess allocation and
dispatch. Before considering Cranelift, LLVM, or another machine-code path,
prefer bytecode or compact plans for:

- evaluator forms.
- matcher plans.
- downvalue dispatch.
- conversion chains.

Acceptance:

- Stages 1-6 have already made the ordinary interpreter path competitive.
- Bytecode or JIT work has a root-linked target that still cannot be solved by
  representation, dispatch, matcher, or memory work.
- The compiled path is optional, testable, and does not become the semantic
  source of truth.

## Milestones

Near term:

- Keep `pair4-stock-repeat8` and the depth-19 base-conversion probes as the
  active mid-level acceptance targets.
- Reduce `pair4-stock-repeat8` from the current `15-17s` rum range toward
  `5s` without downstream patches.

Mid term:

- Bring the first `mkPairInj` `CONVRULE` root from the current hundreds of
  seconds down toward `50s`.
- Make dependency-preloaded `Pair.wl` routinely finish under the bounded
  smoke budget.

`v0.1` performance target:

- Dependency-preloaded `Pair.wl` should be within `1.5x` of WolframKernel on
  the same machine, with theorem markers true and no downstream edits.

Stretch target:

- Beat WolframKernel on the supported `mma-hol` target slice by taking
  advantage of rum's smaller feature surface and domain-specific runtime
  plans, while preserving Wolfram-compatible semantics for implemented
  same-name behavior.

## Execution Rule

Each stage should change one main variable at a time. When a change touches
representation or evaluator control flow, run focused semantic tests first,
then the mid-level Pair/Simp probes, then the root target. A change that lacks
a root-linked measurement is a side experiment, not a mainline performance
result.
