# plan-364 — gensym roots hidden inside suspension frames

Codex's design-only report for [brief-364](./brief-364-suspension-frame-roots-plan.md),
produced 2026-08-27 against tree `5599893a`. Raw material for the director's
implementation briefs, not a dispatched artifact.

Director's gate: working tree clean, no filter hit, all four defect probes
reproduced independently. Rulings on the open questions are recorded at the
bottom of this file.

---

Design only, no source change.

Current tree: `5599893a`; `git status --short` produced no output. All scratch files and generated MIR stayed under `/tmp`.

## Verification

### Bounds

The first cold release build took 45.29 seconds. That established:

- 60 seconds for subsequent release builds and scratch probes.
- 120 seconds for the two-crate MIR emission.
- 30 seconds for CLI and source-inventory commands, whose observed runs were under one second.

Every `rum-cli` invocation used `stdin < /dev/null`. Cargo ran `--offline`; no network or external engine was used.

```text
$ scripts/run-bounded.py --timeout 60 -- cargo build --release -p rum-cli < /dev/null
    Finished `release` profile [optimized] target(s) in 0.05s
```

Release-CLI controls prove the forced threshold both preserves a real root and reclaims an unrooted candidate:

```text
$ RUM_GENSYM_GC_THRESHOLD=1 scripts/run-bounded.py --timeout 30 -- target/release/rum-cli --eval 'Module[{gcControl}, gcControl = 705; gcControlRoot = Hold[gcControl]]; Module[{gcControlTrigger}, gcControlTrigger = 1; Hold[gcControlTrigger]]; gcControl$1' < /dev/null
705

$ RUM_GENSYM_GC_THRESHOLD=1 scripts/run-bounded.py --timeout 30 -- target/release/rum-cli --eval 'Module[{gcGone}, gcGone = 706; Hold[gcGone]]; Module[{gcGoneTrigger}, gcGoneTrigger = 1; Hold[gcGoneTrigger]]; gcGone$1' < /dev/null
gcGone$1
```

### Defect probes

The release scratch crate used public `Session`/`rum_stdlib::install_core` APIs. Its exact command and output were:

```text
$ RUM_GENSYM_GC_THRESHOLD=1 CARGO_TARGET_DIR=/tmp/rum-brief364-probes/target scripts/run-bounded.py --timeout 60 -- cargo test --release --offline --manifest-path /tmp/rum-brief364-probes/Cargo.toml -- --nocapture --test-threads=1 < /dev/null
   Compiling rum-brief364-probes v0.0.0 (/tmp/rum-brief364-probes)
    Finished `release` profile [optimized] target(s) in 0.08s
     Running unittests src/lib.rs (/tmp/rum-brief364-probes/target/release/deps/rum_brief364_probes-7db8340e2d49fc32)

running 4 tests
test tests::block_frame_bare_symbol_loses_candidate_tracking_during_suspension ... symbol allocated: Hold[gcS$1]
symbol before suspension: gcS$1 -> 704
symbol suspended: InputRequired prompt="S: "
symbol first sweep trigger while suspended: Hold[gcSTrigger$2]
symbol resumed value: "ready"
symbol second sweep trigger after resume: Hold[gcSTrigger2$3]
symbol after post-resume sweep: gcS$1 -> 704
ok
test tests::path_a_second_request_sweeps_frame_only_candidate ... path A allocated: Hold[gcA$1]
path A before suspension: gcA$1 -> 701
path A suspended: InputRequired prompt="A: "
path A sweep trigger: Hold[gcATrigger$2]
path A after second-request sweep: gcA$1 -> gcA$1
ok
test tests::path_b_resume_sweeps_candidate_in_remaining_frame ... path B allocated: Hold[gcB$1]
path B before suspension: gcB$1 -> 702
path B suspended: InputRequired prompt="B: "
path B resumed value: List[Hold[gcBTrigger$2], gcB$1]
path B after resume-time sweep: gcB$1 -> gcB$1
ok
test tests::rhs_guard_binding_swept_while_input_string_is_suspended ... matcher allocated: Hold[gcM$1]
matcher before suspension: gcM$1 -> 703
matcher suspended: InputRequired prompt="M: "
matcher sweep trigger while suspended: Hold[gcMTrigger$2]
matcher resumed value: gcM$1
matcher after sweep: gcM$1 -> gcM$1
ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests rum_brief364_probes

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The green tests assert the buggy values deliberately:

- Path A: `gcA$1` was `701`, then became the bare symbol after a second request triggered a sweep.
- Path B: resumption evaluated the trigger Module, but the remaining frame then produced bare `gcB$1`.
- Matcher/RHS guard: the binding initially resolved to `703`; after a sweep during `InputString`, the resumed guard/body produced bare `gcM$1`.
- Bare `Symbol`: a suspended `Block` restored the definition after its candidate-tracking entry had been swept, so a later sweep leaked `gcS$1 -> 704`.

### Call-site inventory

The brief’s exact grep still reports 398:

```text
$ scripts/run-bounded.py --timeout 30 -- sh -c 'grep -rn "suspend_with_frame\|suspend_with_result_frame\|drive_evaluated_value\|drive_internal_value\|push_suspension_frame" --include="*.rs" ./crates | grep -v "/tests/\|suspension/core.rs" | wc -l'
     398
```

The exact exclusion gives 122 files, not 123:

```text
$ ... | grep -v "/tests/\|suspension/core.rs" | sort -u | wc -l
     122

$ ... | grep -v "/tests/\|suspension/core.rs" | sed ... | sort | uniq -c
  79 rum-runtime
  43 rum-stdlib
```

Including `suspension/core.rs` explains the director’s 123 / 80+43 file figure:

```text
$ ... | grep -v "/tests/" | wc -l
     123

$ ... | grep -v "/tests/" | sed ... | sort | uniq -c
  80 rum-runtime
  43 rum-stdlib
```

A `syn` AST inventory, pinned to the versions already in `Cargo.lock`, confirmed that all 398 grep lines are actual method-call expressions:

```text
AST call count: 398
method	18	drive_evaluated_value
method	148	drive_internal_value
method	27	push_suspension_frame
method	166	suspend_with_frame
method	2	suspend_with_frame_lazy
method	28	suspend_with_result_frame
method	9	suspend_with_result_frame_lazy
arg-kind	81	block-closure / move
arg-kind	267	closure / move
arg-kind	30	closure / non-move
arg-kind	18	identity-closure / non-move
arg-kind	2	path / named-value
```

The per-family breakdown is:

```text
1 runtime/attributes.rs
1 runtime/builtin_adapters.rs
3 runtime/builtins_input.rs
20 runtime/debug_tools
28 runtime/definitions
2 runtime/eval_call_prepare.rs
3 runtime/eval_core
1 runtime/eval_dispatch
1 runtime/hold_thread.rs
119 runtime/kernel_data
2 runtime/messages
3 runtime/options
1 runtime/package_tools.rs
66 runtime/pattern_matching
7 runtime/pattern_matching_interpreted.rs
2 runtime/resolution.rs
10 runtime/scoping
1 runtime/session
16 runtime/tree_walkers
1 stdlib/arithmetic
1 stdlib/arithmetic.rs
36 stdlib/associations
27 stdlib/functional_ops
4 stdlib/logic
11 stdlib/predicates
2 stdlib/schema.rs
16 stdlib/strings
1 stdlib/strings.rs
12 stdlib/structure_ops
```

Release MIR supplied compiler-resolved capture types. The classification is:

```text
190 direct Expr/container
82 matcher/rewrite/binding
7 matcher test-only
59 named aggregate/scalar
47 stateless
13 generic/opaque callback
```

Exact `(capture-type signature, normalized resume body)` grouping gives the required named-frame count:

```text
calls=398
distinct_shapes=367
multiplicity_18=1
multiplicity_1=353
multiplicity_2=12
multiplicity_3=1
```

The 31 shared instances comprise one 18-site identity frame, one three-site `SetSameTestState` frame, and twelve exact pairs. Therefore the planned conversion needs exactly **367 named frame structs**, plus a small number of named continuation enums/specs.

### Current mechanism and tests

The threshold and only production sweep trigger are:

```rust
// crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:4
const GENSYM_GC_THRESHOLD_ENV: &str = "RUM_GENSYM_GC_THRESHOLD";
// crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:5
const DEFAULT_GENSYM_GC_THRESHOLD: usize = 200_000;

// crates/rum-runtime/src/runtime/scoping/lexical/module.rs:240
if self.gc_candidate_gensyms.len() >= self.gensym_gc_watermark {
    let extra_roots = gensym_gc_extra_roots_for_outcome(&outcome);
    self.collect_and_sweep_dead_gensyms(&extra_roots);
}
```

The root walk stops before the frames:

```rust
// crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:187
if let Some(suspension) = &self.pending_suspension {
    for entry in &suspension.active_stack {
        visit(&entry.expr);
    }
}
```

The opaque representation and constructor are:

```rust
// crates/rum-runtime/src/runtime/suspension/core.rs:5
pub(in crate::runtime) struct SuspendedEvaluation {
    pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
    pub(in crate::runtime) frames: VecDeque<Box<dyn SuspensionFrame + Send>>,
}

// crates/rum-runtime/src/runtime/suspension/core.rs:24
struct ClosureFrame<F> {
    resume: Option<F>,
}
```

Path B moves the root container local before evaluating resumed work:

```rust
// crates/rum-runtime/src/runtime/suspension/core.rs:228
let mut suspension = self
    .pending_suspension
    .take()
    .expect("resume requires a pending suspended evaluation");

// crates/rum-runtime/src/runtime/suspension/core.rs:242
while let Some(frame) = suspension.frames.pop_front() {
    outcome = frame.resume(self, outcome);
```

Nested ordering currently appends the unprocessed suffix:

```rust
// crates/rum-runtime/src/runtime/suspension/core.rs:246
let mut nested = self
    .pending_suspension
    .take()
    .expect("nested input request should establish a fresh suspension");
nested.frames.append(&mut suspension.frames);
```

Path A is reachable because neither entry point checks suspension state:

```rust
// crates/rum-runtime/src/runtime/session/session_requests.rs:108
pub fn eval_request(&mut self, input: &str) -> Result<SessionEvalResponse, SessionRequestFailure>

// crates/rum-runtime/src/runtime/session/session_requests.rs:126
pub fn eval_expr_request(
    &mut self,
    expr: Expr,
) -> Result<SessionEvalResponse, SessionRequestFailure>
```

The matcher/RHS-guard capture is direct:

```rust
// crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:150
fn evaluate_owned_guards(
    &mut self,
    guards: OwnedGuards,
    bindings: SymbolExprMap,
    false_bindings: SymbolExprMap,
    option_scope: Option<OptionValueScope>,
    index: usize,
) -> Result<CompiledMatchOutcome, EvalError> {

// crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:164
self.drive_evaluated_value::<Expr, CompiledMatchOutcome, _>(
    guard_result,
    move |session, result| {
```

The recursive matcher types are at:

```text
crates/rum-runtime/src/runtime/session/session_state.rs:239 MatcherNode
crates/rum-runtime/src/runtime/session/session_state.rs:270 DefinitionConditionMatcher
crates/rum-runtime/src/runtime/session/session_state.rs:277 MatcherArg
crates/rum-runtime/src/runtime/session/session_state.rs:286 CompiledSequencePattern
crates/rum-runtime/src/runtime/session/session_state.rs:296 SequencePatternCheck
crates/rum-runtime/src/runtime/session/session_state.rs:308 CompiledOptionalPattern
crates/rum-runtime/src/runtime/session/session_state.rs:314 CompiledOptionalDefault
crates/rum-runtime/src/runtime/model/cache.rs:13 SymbolExprMap = HashMap<Symbol, Expr, ...>
```

The representative association capture is unchanged:

```rust
// crates/rum-stdlib/src/associations/mutation.rs:182
let target_result = session.evaluate(target_expr.clone());
let expr = session.suspend_with_frame(target_result, {
    let call_args = args.to_vec();
    let target_expr = target_expr.clone();
    let spec = spec.clone();
    move |session, expr| finish_associate_to_value(session, call_args, target_expr, expr, spec)
})?;
```

The bare-symbol capture is also real:

```rust
// crates/rum-runtime/src/runtime/scoping/lexical/block.rs:368
let outcome = self.suspend_with_result_frame(body_result, {
    let saved_state = saved_state.clone();
    move |session, outcome| {
        session.finish_single_symbol_block_scope(
            symbol,
            saved_state,
            dynamic_depth,
            outcome,
        )
    }
});
```

Memory accounting has the same blind spot:

```rust
// crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:572
fn render_suspended_evaluation_memory_proxy_bytes(suspended: &SuspendedEvaluation) -> usize {
    std::mem::size_of::<SuspendedEvaluation>()
        // active_stack ...
        .saturating_add(suspended.frames.len().saturating_mul(16))
}
```

The existing GC test explicitly verifies only `active_stack`:

```rust
// crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:492
let suspension = session.pending_suspension.as_ref().unwrap();
// ...
for entry in &suspension.active_stack {
    seed_live_gensyms_from_expr(&entry.expr, &candidates, &mut live, &mut worklist);
}
assert!(
    live.contains(&candidate),
    "the candidate must be held by the suspension's traversed active stack"
);
```

It then drops the resumed value rather than asserting it:

```rust
// crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:509
let response = session
    .resume_eval_request_with_input("ready")
    .expect("the suspended candidate-holding call should resume");
drop(response);
```

Existing matcher tests compare queued and resumed evaluation but never sweep between them:

```rust
// crates/rum-runtime/src/runtime/tests/session_requests/matcher_resume.rs:46
fn assert_suspend_resume_matches_queued(source: &str, inputs: &[&str], expected: &str) {
    let queued = eval_with_queued_inputs(source, inputs);
    let resumed = eval_by_suspending_and_resuming(source, inputs);

    assert_eq!(queued, expected);
    assert_eq!(resumed, queued);
}
```

Existing memory-limit ceilings are unsuspended controls, such as:

```rust
// crates/rum-runtime/src/runtime/tests/session_requests/api.rs:181
fn eval_request_memory_ceiling_returns_classified_termination_and_session_recovers() {
    // ...
    limits.eval_memory_limit = Some(100_000);
    // ...
    .eval_request("Table[k, {k, 10000}]")
```

### Dependency evidence

The workspace currently has nine members and MSRV 1.85:

```text
Cargo.toml:3-12   nine workspace members
Cargo.toml:19     rust-version = "1.85"
Cargo.toml:34     serde = { version = "1", features = ["derive", "rc"] }
```

The relevant lock entries are already present:

```text
Cargo.lock:518 name = "proc-macro2"
Cargo.lock:519 version = "1.0.106"

Cargo.lock:527 name = "quote"
Cargo.lock:528 version = "1.0.45"

Cargo.lock:763 name = "serde_derive"
Cargo.lock:767 dependencies = [
Cargo.lock:768  "proc-macro2",
Cargo.lock:769  "quote",
Cargo.lock:770  "syn",

Cargo.lock:853 name = "syn"
Cargo.lock:854 version = "2.0.117"
```

Thus `rum-derive` adds a workspace package and direct dependency edges, but no new fundamental transitive dependency family.

## Re-probe verdict

All requested paths still reproduce on `5599893a`.

| Shape | Verdict | Exact liveness evidence |
|---|---|---|
| Path A, second request | Reproduced | `gcA$1 -> 701`, then `gcA$1 -> gcA$1` |
| Path B, sweep during resume | Reproduced | resumed `List[Hold[gcBTrigger$2], gcB$1]` instead of using `702` |
| Matcher binding/RHS guard | Reproduced | binding resolved to `703` before suspension; resumed value was bare `gcM$1` |
| Bare `Symbol` in `Block` frame | Additional exposure | candidate tracking was lost; after restoration and another sweep `gcS$1 -> 704` leaked |

Exposure is broad:

- 190 boundaries directly capture `Expr`, `SharedExpr`, `SharedArgs`, or their containers.
- 89 boundaries are matcher/definition/rewrite frames, including recursive matchers and `SymbolExprMap`.
- 59 capture named aggregate/control state; many are transitively expression-bearing.
- Four production boundaries capture a bare `Symbol` without a directly captured `Expr`: [block.rs:368](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/scoping/lexical/block.rs:368), [block.rs:477](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/scoping/lexical/block.rs:477), and [driver.rs:260](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:260), [driver.rs:279](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:279).
- 47 boundaries are genuinely stateless. A derived traversal for them emits no visits.
- The direct/mechanical families include associations, functional folds/arrays/iteration, strings, structure operations, debug/test-report state, control/scoping, and ordinary rewrite walkers.
- The matcher group is the hard family because bindings, recursive compiled patterns, guards, and option scopes all compose transitively.

The root visitor therefore must cover both `Expr` and bare `Symbol`. Treating `Symbol` as an automatic no-op would fix the three requested probes while preserving the `Block` leak.

## Plan

### 3. Frame-shape inventory

Use **367 named frame structs** as the migration number.

The equivalence rule is mechanical: share a type only when the compiler-resolved captured type signature and normalized resume body are identical. It saves 31 definitions without merging semantically different continuations.

The work classes are:

- 190 direct state frames: mechanical named fields plus a resume method.
- 89 matcher/rewrite/binding frames: mechanical construction, but recursive traversal dependencies.
- 59 aggregate/control frames: derive their named state recursively.
- 47 stateless frames: named unit frames; the 18 exact identity sites share one `IdentityExprFrame`.
- 13 opaque callback boundaries: redesign before mass conversion.

Frame definitions should live beside their owning resume behavior. Files entering the 20–35k split range get a responsibility-based sibling `resume_frames.rs`; no central 367-type file.

### 4. Capture-type closure

The conservative compiler/source closure is 187 custom nominal types:

- 116 currently contain `Expr` directly or transitively.
- 71 are current leaf/control types. They still receive a derived no-op traversal so a future field addition becomes visible automatically.
- `Run<T>` is not semantically empty: it needs a derived `T: FrameRootTrace` bound because its concrete `T` can contain expressions.
- Type aliases such as `SymbolExprMap` are covered by the generic `HashMap` implementation rather than a derive.

All custom structs/enums below can use the field-recursive derive; none needs a handwritten custom traversal.

<details>
<summary>Expression-bearing custom closure, with source lines</summary>

```text
crates/rum-runtime/src/runtime/debug_tools/reap.rs:3 ReapScopeRenderState, :9 ReapBucketRenderState
crates/rum-runtime/src/runtime/debug_tools/test_report/model.rs:4 TestCaseSpec, :13 TestReportOptions, :28 EvalCapturedOutcome
crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:6 TestReportEvalState, :15 TestReportCaseEvalState, :26 TestReportCaseOutputState
crates/rum-runtime/src/runtime/debug_tools/timing.rs:6 RepeatedTimingEvalState
crates/rum-runtime/src/runtime/debug_tools/trace.rs:8 TraceEvalState
crates/rum-runtime/src/runtime/definitions/application/replace_all/plan.rs:7 RewriteRulePlan, :10 RewriteRulePlanData, :16 RewriteRuleExactIndex
crates/rum-runtime/src/runtime/definitions/definition_updates.rs:4 HeldUpdateReturn, :10 HeldUpdateTarget
crates/rum-runtime/src/runtime/definitions/part_assignment.rs:8 PositionalPartAssignmentTarget
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:183 OwnedGuards
crates/rum-runtime/src/runtime/eval_call_prepare.rs:22 UnevaluatedRestorePlan
crates/rum-runtime/src/runtime/kernel_data/association_data.rs:7 AssociationEntryForm, :565 AssociationSetLValueTarget
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/mod.rs:35 DoSpec
crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:4 ExceptionHandler
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace.rs:11 ReplacePartRule, :20 ReplacePartPositionSpec, :44 ReplacePartTarget, :51 GuardedReplacePartTarget
crates/rum-runtime/src/runtime/kernel_data/structure_data/selectors.rs:48 PartSelector
crates/rum-runtime/src/runtime/model/cache.rs:13 SymbolExprMap
crates/rum-runtime/src/runtime/model/definitions.rs:52 SymbolDefinition, :63 UpValue, :74 DownValue, :86 SubValue, :140 DownValueShallowSignature, :174 DownValueShallowArg, :276 RewriteRule
crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:33 DownValueDispatchMetadata
crates/rum-runtime/src/runtime/model/messages.rs:102 MessageDefinition
crates/rum-runtime/src/runtime/model/traversal.rs:4 CasesSpec
crates/rum-runtime/src/runtime/options.rs:13 SymbolOptionEntry, :20 SymbolOptions, :131 OptionValueScope
crates/rum-runtime/src/runtime/options/builtins.rs:4 OptionValueQuery
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/candidates.rs:4 OrderlessCandidateChoice
crates/rum-runtime/src/runtime/scoping/packages.rs:33 SourceUnit, :43 PackageInstallPlan
crates/rum-runtime/src/runtime/session/session_state.rs:88 CallDispatchState, :124 BlockSymbolState, :171 ReapBucket, :177 ReapBucketGroup, :239 MatcherNode, :270 DefinitionConditionMatcher, :277 MatcherArg, :286 CompiledSequencePattern, :296 SequencePatternCheck, :308 CompiledOptionalPattern, :314 CompiledOptionalDefault, :348 EvalError

crates/rum-stdlib/src/associations/aggregate.rs:6 CatenateStep
crates/rum-stdlib/src/associations/basic.rs:477 PreparedLookup, :483 LookupOutcome
crates/rum-stdlib/src/associations/grouping.rs:12 IndexedGroups, :108 GroupBySpec
crates/rum-stdlib/src/associations/resumable/key_sort.rs:11 KeySortByEntry, :28 KeySortByOrderingState
crates/rum-stdlib/src/associations/transforms/key_collections.rs:35 KeyUnionPlan
crates/rum-stdlib/src/functional_ops/arrays/generators.rs:4 ArrayEvalState, :19 OuterEvalState
crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:3 TensorCallbackEvalState, :10 InnerEvalState
crates/rum-stdlib/src/functional_ops/iteration.rs:9 NestEvalState, :15 FixedPointEvalState, :22 NestWhileEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/pair.rs:3 FoldPairEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/plain.rs:3 FoldEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs:3 SequenceFoldEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:3 FoldWhileEvalState
crates/rum-stdlib/src/functional_ops/sequence/select.rs:10 SelectSource
crates/rum-stdlib/src/functional_ops/sequence/window.rs:4 SequenceLikeExpr, :16 BlockMapEvalState
crates/rum-stdlib/src/functional_ops/targeted_map.rs:13 MapAtEvalState, :19 SubsetMapEvalState
crates/rum-stdlib/src/predicates/arrays.rs:5 ArrayLeafTestState
crates/rum-stdlib/src/predicates/collections.rs:17 DuplicateFreeSameTestState, :24 SameTestMatrixState
crates/rum-stdlib/src/predicates/ordering.rs:5 OrderedQComparatorState, :11 LexicographicListComparatorState, :18 LexicographicPairComparatorState
crates/rum-stdlib/src/strings.rs:140 StringApplyEvalState
crates/rum-stdlib/src/strings/patterns.rs:117 ExactStringReplacementRule, :123 GuardedExactStringReplacementRule, :135 ExactStringReplacementValue, :141 ExactStringCaseRule, :147 GuardedExactStringCaseRule, :153 StringRuleGuard, :159 ExactStringCaseOutput, :165 ExactStringCaseSpec
crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:5 GuardedStringCasesRuleEvalState, :12 GuardedStringCasesValuesEvalState
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:5 GuardedStringReplaceRuleEvalState, :12 GuardedStringReplaceValuesEvalState
crates/rum-stdlib/src/strings/patterns/search/rules.rs:7 StringReplaceRuleEvalState, :16 StringReplaceValuesEvalState, :23 StringCasesRuleEvalState, :32 StringCasesValuesEvalState
crates/rum-stdlib/src/structure_ops/collection_stats.rs:9 KeyedEntry, :22 DecorateByState, :29 TallyTestState, :37 GatherByFunctionListState
crates/rum-stdlib/src/structure_ops/dedup.rs:6 AdaptiveExprSet, :35 DeleteDuplicatesTestState
crates/rum-stdlib/src/structure_ops/list_lookup.rs:13 SelectPositionsState
crates/rum-stdlib/src/structure_ops/ordering_sets.rs:10 SortByEntry, :64 SetOperationKind, :85 SetSameTestState
crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:67 MergeSortState
crates/rum-stdlib/src/structure_ops/selectors.rs:32 PartSelector
```

</details>

The handwritten implementations are limited and mechanically allowlisted:

- Root-bearing foreign/core types: `Expr`, `SharedExpr`, `SharedArgs`, and `Symbol`.
- Generic containers: `Option`, `Result`, `Vec`, `VecDeque`, slices/arrays, tuples, `Box`, `Arc`, `HashMap`, `HashSet`, `BTreeMap`, `BTreeSet`, `IntoIter`, references, and `OnceLock`.
- Exact no-root leaves: primitive scalars, `String`/`str`, `Instant`, `Duration`, the two audited function-pointer signatures, and other compiler-proven scalar external types.
- `WeakSharedArgs`, if encountered later, must be explicitly no-op because a weak reference must not become a liveness root.

The visitor should be a single field-recursive contract:

```rust
trait FrameRootTrace {
    fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor);
}

trait FrameRootVisitor {
    fn visit_expr(&mut self, expr: &Expr);
    fn visit_symbol(&mut self, symbol: &Symbol);
}
```

GC and memory accounting use different visitors over this same traversal. No second registry is introduced.

### 5. Escape hatches

There are exactly 13 opaque callback boundaries:

| Count | Source | Current escape | Required redesign |
|---:|---|---|---|
| 2 | `tree_walkers/count_like.rs:210,225` | generic `F: FnOnce(usize) -> Expr` | `CountLikeFinish` enum: Count, MemberQ, FreeQ |
| 2 | `definitions/definition_catalog/rewrite_rules.rs:57,72` | forwarded `cont: F` | named continuation-state enums for the six rule-expression and two compiled-rule callers |
| 7 | `replace/once.rs:86,97,173,231,273,379,478` | returned `impl Fn` and generic `F` | carry `RewriteRulePlan` and source explicitly; call the named rule-application method |
| 1 | `replace_at.rs:119` | generic rule-applier `F` | same concrete `RewriteRulePlan` representation |
| 1 | `resumable_merge_sort.rs:144` | `MergeSortState<T,C>` with `C: FnOnce` | named `MergeCompletion<T>` implementations; no blanket closure implementation |

The generic evidence is verbatim at:

```text
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:207
F: FnOnce(usize) -> Expr + Send + 'static

crates/rum-runtime/src/runtime/definitions/definition_catalog/rewrite_rules.rs:54
F: FnOnce(&mut Session, Option<Vec<Expr>>) -> Result<U, EvalError> + Send + 'static

crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:161
F: Clone + Fn(&mut Session, Expr, RuleApplicationSource) -> Result<Option<Expr>, EvalError> + Send + 'static

crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:86
C: FnOnce(Vec<T>) -> Expr + Send + 'static
```

Two function-pointer leaves exist and cannot capture an environment:

```text
crates/rum-runtime/src/runtime/metadata.rs:288
pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>;

crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:9
Custom(fn(&Expr) -> bool),
```

They get exact no-op implementations; no general `Fn`/closure implementation is permitted.

The direct-capture scan found no dynamic or interior-mutable escape:

```text
$ ... DETAIL=1 ... | rg 'Box<dyn|dyn Any|dyn Fn|RefCell|Cell<|Mutex|RwLock|FileIoProvider|EntropyProvider|fn\('
exit=1
```

The only transitive synchronization cell found was the derived, expression-free dispatch cache:

```rust
// crates/rum-runtime/src/runtime/model/definitions/down_value_dispatch.rs:40
pub(super) struct DownValueDispatchIndexCache {
    index: OnceLock<Box<DownValueDispatchIndex>>,
}
```

It is safely traversable by borrowing `OnceLock::get`; its current payload has no expression roots.

### 6. Enforcement

After the planned cutover, no existing insertion route can skip traversal:

1. Make `SuspendedEvaluation::frames` private to `suspension::core`, not `pub(in crate::runtime)`.
2. Keep `SuspendedEvaluation::new`, queue mutation, pop, and append private.
3. Define `SuspensionFrame: FrameRootTrace + Send + Sync + 'static`.
4. Expose only a doc-hidden generic `Session` insertion method whose parameter is `F: SuspensionFrame`; it creates the `Arc`.
5. Delete `ClosureFrame`, `drive_evaluated_value`, `drive_internal_value`, all `suspend_with_frame*`, and closure-taking `push_suspension_frame`.
6. Add a repository invariant test that:
   - bans those old API names;
   - bans direct frame-queue mutation outside `suspension/core.rs`;
   - bans handwritten custom `FrameRootTrace` implementations outside the exact sorted manual-implementation allowlist.

Current direct frame access is only:

```text
crates/rum-runtime/src/runtime/suspension/core.rs:7
crates/rum-runtime/src/runtime/suspension/core.rs:49
crates/rum-runtime/src/runtime/suspension/core.rs:212
crates/rum-runtime/src/runtime/suspension/core.rs:242
crates/rum-runtime/src/runtime/suspension/core.rs:250
crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs:581
```

Honest limitation: Rust cannot distinguish a proc-macro-generated impl from a handwritten impl when the trait must be implementable by sibling crate `rum-stdlib`. Therefore:

- The compiler guarantees that every inserted frame has a traversal.
- A frame lacking the derive fails compilation unless someone writes an explicit traversal.
- The sorted source allowlist makes such an explicit implementation a named, mechanically reviewed exception.
- Absolute “the compiler knows this syntax came from derive” sealing is impossible without moving all frame implementations into `rum-runtime` or abandoning the decided per-frame implementation shape.

### 7. Derive host

Recommendation: add `rum-derive`.

| Option | Merits | Residual risk |
|---|---|---|
| Proc macro | Handles structs/enums, generics, tuple/unit variants, and field additions automatically; generated bounds are compiler-checked; familiar serde-style model | Adds a tenth published workspace crate and macro expansion work |
| `macro_rules!` | No proc-macro package; can define a frame and traversal together | Cannot derive traversal for the 187 existing nominal types without redeclaring them; poor enum/generic ergonomics |
| Handwritten impls + source test | No macro dependency | Every field addition remains a manual obligation; the enforcement parser becomes a fragile home-grown derive |

Cost of `rum-derive`:

- `syn 2.0.117`, `quote 1.0.45`, and `proc-macro2 1.0.106` are already locked through `serde_derive`.
- It adds one small proc-macro compilation unit and roughly 187 state derives plus 367 frame derives.
- It adds compile/expansion time but zero runtime work.
- MSRV remains 1.85; implementation must use APIs supported by the pinned versions.
- Because `rum-runtime` and `rum-stdlib` are publishable, `rum-derive` must also be publishable and referenced with path plus workspace version, then published before dependent crates.

### 8. Path B representation

Use two Session-rooted suspension slots and immutable shared frame objects:

```rust
pending_suspension: Option<SuspendedEvaluation>,
resuming_suspension: Option<SuspendedEvaluation>,

frames: VecDeque<Arc<dyn SuspensionFrame + Send + Sync>>,
```

`SuspensionFrame::resume` takes `self: Arc<Self>` or `&self`; it does not consume the rooted object.

Resume sequence:

1. Move `pending_suspension` into `Session::resuming_suspension`.
2. Move only `active_stack` into `evaluation_stack`; leave all frames in `Session`.
3. Clone the front `Arc` and invoke it while the original remains at the front of `resuming_suspension`.
4. Pop the original only after the frame returns.
5. On nested input, pop the completed current frame, take the new `pending_suspension`, and append the still-rooted suffix from `resuming_suspension`, preserving current ordering.
6. On success or terminal error, clear `resuming_suspension` explicitly.

Both GC and memory walks visit `pending_suspension` and `resuming_suspension`.

Per popped frame cost is suspension-only:

- One `Arc` increment/decrement pair.
- One `VecDeque` front/pop.
- Shallow state-handle cloning needed by an immutable resume implementation.

Bulk captured state should become `SharedArgs`, `Arc<[T]>`, or another named Arc-backed state so resume does not clone expression trees or perform an accidental O(N) container clone. No unsafe, raw pointers, pinning, deferred sweep, or parallel root snapshot is needed.

`clear_suspension_state` must be split: queued input and best-effort monitor callbacks may clear a fresh pending suspension while an outer resumption remains active; they must not erase `resuming_suspension`.

### 9. Memory proxy

The same traversal feeds both consumers:

- GC visitor: `visit_expr` calls `seed_live_gensyms_from_expr`; `visit_symbol` seeds the candidate directly.
- Memory visitor: `visit_expr` calls `expr_memory_proxy_bytes`; `visit_symbol` charges the same symbol context/name proxy used for an atomic symbol expression.

For each suspension, account for:

- `size_of::<SuspendedEvaluation>()`;
- active-stack entries as today;
- `frames.capacity() * size_of::<Arc<dyn SuspensionFrame>>()`;
- each Arc allocation header and concrete `size_of_val(frame)`;
- every traced expression/symbol using the shared visitor.

Count both pending and resuming suspensions. Shared allocations may remain conservatively double-counted, consistent with the existing proxy’s treatment of shared expressions.

The existing `Table` and print-flood memory tests do not suspend, so their exact ceilings should remain unchanged. Add a suspension-specific memory test with a large frame state and a limit between the old flat-16 estimate and the corrected proxy. It must assert classified `MemoryLimit` termination and subsequent session recovery.

### 10. Non-suspending cost

| Step | Ordinary-path cost | Gate |
|---|---|---|
| Derive crate and model derives | None; compile-time only | Build/test |
| Opaque-callback elimination | Enum/spec dispatch may affect count/rewrite/sort paths | Full relevant microbench A/B |
| Named-frame conversion | Frame expression and Arc allocation exist only in `InputRequired` arm | Full hot-path A/B |
| Root/memory traversal | One additional `Option::is_some` on cold sweep/memory checks; no frame walk when unsuspended | Memory tests plus hot A/B |
| Arc resume representation | None before suspension; atomic pair per resumed frame | Suspension tests |

Use explicit matches or a declarative token macro such as `resume_or_suspend!` that expands to a match. It must not accept or store a Rust closure. The generated frame expression appears only in the `InputRequired` arm; the `Ok` arm calls the same resume helper directly, preserving [core.rs:67](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/suspension/core.rs:67).

Matcher conversion touches a hard hot path even with zero intended work. Required A/B labels include matcher single/tiny/wide, guarded rewrite, `replaceall-simple`, wide Count/Replace/ReplacePart, and their same-tier neighbors. Merge completion conversion additionally gates the callback Sort/SortBy/ReverseSort/KeySort/KeySortBy rows.

### 11. Test design

Use the existing subprocess helper at [scoping_gensym_gc.rs:890](/Users/fft/Developments/rum/crates/rum-runtime/src/runtime/tests/scoping_gensym_gc.rs:890) with threshold `1`, so the `OnceLock` threshold is initialized in a fresh process.

Every liveness regression must:

1. Assert the candidate definition/value before suspension.
2. Suspend.
3. Cross the watermark through a real `Module` exit.
4. Assert `gensym_gc_sweep_count` increased.
5. Resume.
6. Assert the exact resumed value.

Tests:

- Retain the existing active-stack test, rename it to make that scope explicit, and replace `drop(response)` with exact `List[7, List["ready"]]`.
- Path A: suspend `List[InputString["A: "], gcA$1]`, run a second request containing `gcATrigger$2`, then use the crate-private resume function and assert `List["ready", 701]`.
- Path B: suspend `List[Input["B: "], gcB$1]`, resume with the trigger Module, and assert `List[Hold[gcBTrigger$2], 702]`.
- Matcher/RHS guard: use the probe’s `gcMThunk` setup, sweep while the binding table is frame-only, resume `"yes"`, and assert `703`.
- Bare Symbol: suspend `Block[{gcS$1}, InputString["S: "]]`, sweep, resume to `"ready"`, trigger a second sweep, and assert `gcS$1` is reclaimed rather than leaked.
- Derive traversal: nested struct/enum tests covering `MatcherArg`, `CompiledOptionalPattern`, `SymbolExprMap`, `AssociationEntryForm`, `SharedArgs`, and a bare `Symbol`.
- Path B current-frame test: the currently executing frame and a later queued frame each hold a different candidate; a Module sweep during current-frame resume must preserve both.
- Nested suspension ordering: A → nested B → old suffix C, with exact side-effect order and values.
- Memory proxy: large frame state, classified termination, and recovery.
- Compile/source invariants: missing derive cannot satisfy `SuspensionFrame`; old closure API names and direct queue writes are absent.

Existing matcher-resume tests remain useful continuation-equivalence tests, but do not replace the forced-sweep regressions.

### 12. Smallest safe implementation sequence

1. **Derive substrate.** Add `rum-derive`, the root visitor/trace traits, base implementations, and derive tests. No frame behavior changes.
2. **Capture closure.** Apply derives to the 187 custom nominal types and validate matcher/association/Symbol traversal. Still no frame behavior changes.
3. **Eliminate 13 opaque callbacks.** Replace generic continuation/applier/completion state with named enums/specs. Preserve current suspension API temporarily as migration input; do not claim the defect fixed. Gate rewrite/matcher/sort performance.
4. **Atomic frame cutover.** Convert all 398 boundaries into 367 named shapes, introduce Session-rooted pending/resuming Arc queues, wire GC and memory traversal, delete every closure-taking API, add enforcement and regressions, and update the TODO/audit documentation. This step is not complete until runtime, stdlib, host, workspace, and microbench gates pass together.

The mass conversion is deliberately atomic at the acceptance boundary. Splitting runtime and stdlib into separately landed partial fixes would require a legacy opaque-frame escape and would violate the no-stopgap ruling.

## Draft follow-up briefs

These are raw drafts in `TEMPLATE.md` shape for director review and renumbering.

<details>
<summary>Draft 365 — field-derived frame-root traversal substrate</summary>

### Goal

Add a publishable `rum-derive` proc-macro crate and a runtime-owned, borrowed root-traversal contract that derives recursively across named fields and supports both `Expr` and bare `Symbol` roots. Do not alter suspension representation or behavior.

### Context pointers

- `AGENTS.md`, architecture and testing policy.
- `Cargo.toml:1-35`, `Cargo.lock` entries for `syn`/`quote`/`proc-macro2`.
- `runtime/scoping/gensym_gc.rs:236-259` for expression/symbol marking.
- Brief 364’s 187-type closure and manual-type allowlist.

### Scope

May modify:

- root `Cargo.toml`, `Cargo.lock`;
- new `crates/rum-derive/**`;
- `crates/rum-runtime/Cargo.toml`;
- `crates/rum-runtime/src/runtime/suspension.rs`;
- new `crates/rum-runtime/src/runtime/suspension/trace.rs`;
- `crates/rum-runtime/src/runtime/kernel_api.rs`;
- focused runtime tests for derive expansion/traversal.

Must not modify suspension queues, closure APIs, evaluator behavior, stdlib behavior, TODO, or unrelated docs.

### Deliverable

- `#[derive(FrameRootTrace)]` for structs/enums/generics.
- Visitor methods for `Expr` and `Symbol`.
- Borrowed implementations for core and standard containers.
- Exact manual no-op function-pointer/scalar implementations.
- Sorted manual-implementation allowlist and tests.
- Publish metadata/path+version dependency suitable for workspace publication.

### rum / Rust pitfalls

- No deep clone; traversal only borrows.
- `Expr` is visited once as a root; its recursive symbol scan remains in GC.
- `Symbol` is a root, not a no-op.
- Generic derives add only necessary bounds.
- Reject unions.
- Do not provide blanket `Fn`, `Any`, or opaque-object implementations.

### Verification

Run offline and bounded:

```text
cargo fmt --check
cargo test -p rum-derive
cargo test -p rum-runtime
cargo test --workspace
```

Also compare a cold base/candidate build in separate `/tmp` target directories and report elapsed build rows; this is informational, not a runtime performance gate.

### Hard rules

No git writes, network, external-engine output, unsafe, unrelated manifest changes, or default-path instrumentation. Use `apply_patch`; preserve user changes. Leave the tree uncommitted.

### Report format

Per-file changes; verbatim tests/build comparison; derive-contract findings; open questions/stop-loss.

</details>

<details>
<summary>Draft 366 — close the suspension capture-type traversal graph</summary>

### Goal

Apply `FrameRootTrace` derives to the complete 187-type closure established by brief 364, including matcher, bindings, associations, control state, stdlib state machines, and no-root leaf types. Prove nested field traversal without changing runtime behavior.

### Context pointers

- Brief 364 capture-closure list.
- `session/session_state.rs:239-318` matcher recursion.
- `model/cache.rs:13` `SymbolExprMap`.
- `kernel_data/association_data.rs:7` `AssociationEntryForm`.
- `scoping/lexical/block.rs:368` bare-symbol capture.

### Scope

May modify only files containing the 187 types listed in brief 364, plus focused traversal tests and module imports. No suspension callsite or API changes.

### Deliverable

- Derives on all custom structs/enums.
- Generic bounds for `Run<T>`, `IndexedGroups<T>`, and other generic carriers.
- Alias coverage through generic container implementations.
- Tests that record exact visited expression and symbol roots for matcher, association, rewrite-plan, Block state, and stdlib state.
- Zero handwritten custom traversal impls outside the approved base list.

### rum / Rust pitfalls

- Leaf/no-op means the current fields contain no root; derive it so future fields remain automatic.
- Function pointers are exact no-op leaves only.
- Do not initialize or mutate `OnceLock` while traversing.
- Do not traverse weak references as strong roots.

### Verification

```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Run the source invariant test showing no non-allowlisted handwritten traversal implementation.

### Hard rules

No git writes, network, unsafe, behavior changes, suspension API changes, or drive-by organization edits. No performance claim is required because derives are not invoked on evaluation paths in this brief.

### Report format

Files/types changed; verbatim tests; final derived/manual counts; any type that could not be derived and why.

</details>

<details>
<summary>Draft 367 — eliminate opaque continuation state</summary>

### Goal

Remove all 13 generic/opaque callback captures so every eventual suspension frame state is expressible as named, derivable data. Preserve evaluation and suspend/resume semantics exactly.

### Context pointers

- `tree_walkers/count_like.rs:196-225`.
- `definitions/definition_catalog/rewrite_rules.rs:47-83` and its eight callers.
- `definitions/application/replace_all.rs:61-77`.
- `rewrite_data/replace/once.rs:81-110,152-198,200-501`.
- `structure_data/replace/replace_at.rs:97-126`.
- `rum-stdlib/structure_ops/resumable_merge_sort.rs:67-150` and five callers.

### Scope

May modify the files above and their direct caller files:

- runtime `session/session_eval.rs`, `definitions/definitions.rs`;
- runtime Replace/ReplaceAll/ReplaceList/ReplaceRepeated/ReplaceAt modules;
- stdlib merge sort, ordering sets, collection stats, and association key-sort modules;
- their focused tests.

Do not yet change `SuspendedEvaluation`, frame storage, root walking, memory accounting, or closure API definitions.

### Deliverable

- `CountLikeFinish` enum.
- Named rewrite-resolution continuation state for six rule-expression and two compiled-rule callers.
- Concrete `RewriteRulePlan` traversal in Replace/ReplaceAt; no returned `impl Fn` stored across suspension.
- Named `MergeCompletion<T>` implementations with no blanket closure implementation.
- A compiler-assisted capture scan returning zero `F`, `impl Fn`, or `FnOnce` frame captures.
- Identical ordinary and suspended results.

### rum / Rust pitfalls

- Preserve continuation order and one-shot behavior.
- Do not turn suspending evaluation eager.
- Avoid an enum branch inside tight loops when static dispatch is possible.
- No deep expression clones.

### Verification

Before editing, run the base microbench into `/tmp/brief367-base`; after editing run `/tmp/brief367-new`. Paste all relevant rows:

- matcher tiny/wide;
- `count-wide-no-match`;
- `replaceall-simple`, guarded rewrite, wide Replace/ReplacePart;
- callback Sort/SortBy/ReverseSort/KeySort/KeySortBy and same-tier neighbors.

Also run:

```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

The capture grep/MIR inventory must report zero opaque callback boundaries.

### Hard rules

No git writes, network, external-engine output, unsafe, dependency changes, or default-path regression. A regression on a hot label or neighbor is a defect, not noise.

### Report format

Per-family redesign; verbatim tests; base/new timing rows; final escape-hatch scan; open questions/stop-loss.

</details>

<details>
<summary>Draft 368 — atomic named-frame cutover and complete suspension rooting</summary>

### Goal

Replace all closure-based suspension boundaries with the 367 named derived frame shapes, keep current and remaining frames Session-rooted during resume, feed the single traversal into gensym GC and memory accounting, delete every closure-taking API, and close the documented defect.

### Context pointers

- Brief 364’s callsite, type, duplicate-shape, escape, and test inventories.
- `suspension/core.rs`.
- `model/session.rs:77-118`.
- `scoping/gensym_gc.rs:8-67,112-195`.
- `debug_tools/resources/memory.rs:230-308,572-586`.
- `session/session_helpers.rs` snapshot/reset behavior.
- TODO “Opened by batch 7”.

### Scope

May modify:

- the 79 runtime and 43 stdlib files from the exact brief-364 inventory;
- suspension core/module files;
- Session model/constructor/helper/reset files;
- gensym GC and memory proxy;
- focused runtime, stdlib, host tests;
- `TODO.md` and the runtime evaluation audit only to record the completed invariant.

No unrelated APIs, language semantics, manifests, or generated artifacts.

### Deliverable

- 367 named derived frame structs.
- Private `VecDeque<Arc<dyn SuspensionFrame + Send + Sync>>`.
- `pending_suspension` plus `resuming_suspension`.
- Current frame retained through its complete resume call.
- Exact nested append ordering.
- One trace over pending/resuming frames for both GC and memory.
- Closure APIs and `ClosureFrame` deleted.
- Source/compile enforcement tests.
- Forced-sweep Path A, Path B, matcher, current-frame, nested-order, and bare-Symbol regressions.
- Corrected suspension memory-limit coverage.
- Responsibility-based module splits for files crossing size thresholds.

### rum / Rust pitfalls

- Frame construction/Arc allocation only in `InputRequired`.
- No runtime closure factories.
- No deep clones; bulk state is Arc-backed or borrowed.
- Do not clear `resuming_suspension` when discarding a fresh nested/monitor suspension.
- Every liveness assertion occurs after a real Module-triggered sweep.
- Every new regression asserts the exact resumed value.
- No unsafe or deferred-sweep mechanism.

### Verification

Run, bounded:

```text
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test -p rum-host
cargo test --workspace
```

Required invariant greps:

```text
# old API names: no output
rg "ClosureFrame|suspend_with_frame|suspend_with_result_frame|drive_evaluated_value|drive_internal_value" crates

# direct queue writes: only suspension/core.rs
rg "\.frames\.(push|insert|extend|append)|frames:" crates/rum-runtime/src crates/rum-stdlib/src

# opaque captures: no output
<brief-364 MIR capture command>
```

Run base and candidate `scripts/core-language-microbench.sh` before/after and paste every hot label plus same-tier neighbors. No hot-label regression is accepted.

Paste the forced-threshold regression outputs verbatim, including expected fixed values:

```text
Path A: List["ready", 701]
Path B: List[Hold[gcBTrigger$2], 702]
matcher: 703
bare Symbol after post-resume sweep: gcS$1
```

### Hard rules

No git writes, network, external-engine output, unsafe, stopgap opaque API, alternate root registry, sweep deferral, eager-resume simplification, or unmeasured hot-path claim. Leave all work uncommitted.

### Report format

Per-family/file changes; verbatim tests/greps/timings; fixed probe values; frame/type counts; stop-loss/open questions.

</details>

## Open questions and recommendations

1. **Should traversal include bare `Symbol`?**  
   Recommendation: yes. The `Block` probe demonstrates lost candidate tracking and a leaked restored definition. Use one visitor with `visit_expr` and `visit_symbol`.

2. **Accept immutable Arc-rooted frames for Path B?**  
   Recommendation: yes. A local `Box` or popped Arc cannot keep the current frame rooted in safe Rust. The retained original Arc is structurally necessary; refactor bulk state to shared containers.

3. **Is compiler-plus-source-gate enforcement sufficient?**  
   Recommendation: yes. Cross-crate Rust cannot prove an impl originated from a derive. The supertrait guarantees traversal, while the exact sorted manual-impl allowlist makes non-derived custom traversal a failing repository gate.

4. **Should the 367-shape equivalence be the naming rule?**  
   Recommendation: yes. Share only exact capture-signature/resume-body duplicates; do not centralize unrelated stateless behavior merely to reduce type count.

5. **What should happen to a second public request while another request is suspended?**  
   Recommendation: keep this root repair independent. The current second request can clear `pending_eval_request`, making public resumption awkward even though `pending_suspension` remains. Specify request re-entry in a separate host-contract brief; do not reject it here as a substitute for rooting Path A.

6. **How exact should frame memory accounting be?**  
   Recommendation: preserve the existing conservative proxy model: dynamic frame payload plus traced roots and container capacity, without introducing global Arc deduplication.

7. **Publication of `rum-derive`?**  
   Recommendation: publish it as a normal 0.1 workspace crate and use path+version dependency metadata; do not hide an unpublished path dependency beneath publishable runtime crates.

Stop-loss was not triggered. The raw grep/file discrepancy resolved through AST inspection, the capture shapes resolved through release MIR, and the enforcement limitation has a concrete compiler-plus-allowlist design.

---

## Director's rulings on the open questions (2026-08-27)

1. **Bare `Symbol` in the traversal** — ACCEPTED. The `Block` probe is decisive:
   covering only `Expr` would fix the three requested paths and leave the
   `Block` leak in place.
2. **Arc-rooted frames for Path B** — ACCEPTED. Nothing else keeps the
   currently executing frame rooted in safe Rust.
3. **Compiler-plus-allowlist enforcement** — ACCEPTED (user, 2026-08-27). Full
   sealing would require moving every stdlib frame implementation into
   `rum-runtime`, which undoes the crate-rebalance line's separation. That is
   architectural regression, not a more correct fix, and the best-over-cost
   standard retires cost arguments, not design restraint. The allowlist is
   mechanically gated, in the same shape as the message registry's
   declared-or-exempt rule.
4. **367-shape equivalence as the naming rule** — ACCEPTED. Merge only exact
   duplicates.
5. **A second request during a suspension** — ACCEPTED as stated: out of scope
   here, and a separate host-contract brief. It must not be offered as a
   substitute for rooting Path A.
6. **Conservative memory accounting** — ACCEPTED. No global Arc dedup.
7. **Publishing `rum-derive`** — REJECTED, on a false premise. rum's release
   form is source-available GitHub only, with no crates.io publication
   (`CLAUDE.md`, publication-prep ruling); no crate in the workspace carries a
   `publish` field. `rum-derive` is a plain path dependency: no version
   metadata, no publication ordering.
