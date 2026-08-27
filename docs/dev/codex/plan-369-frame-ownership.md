# plan-369 — how a rooted frame resumes without deep-copying

Codex's design-only report for [brief-369](./brief-369-frame-ownership-plan.md),
produced 2026-08-27 against tree `843d84d3`, after brief-368's cutover stopped
on the ownership tension. Raw material for the director's implementation briefs.

Director's gate: working tree clean, no filter hit. Rulings at the bottom.

---

Design only, no source change.

`git status --short` and `git diff --stat` both returned no output. All generated artifacts are under `/tmp`; Cargo was offline, every `rum-cli` invocation used `< /dev/null`, and no network or git write occurred.

## 2. Fact 2 measurement

The release build was bounded to 60 seconds, derived from the prior same-shape 45.29-second build recorded by plan-364:

```sh
CARGO_TARGET_DIR=/tmp/rum-brief369-target \
scripts/run-bounded.py --timeout 60 -- \
cargo build --release --offline -p rum-cli --features alloc-count < /dev/null
```

```text
Finished `release` profile [optimized] target(s) in 57.57s
```

The CLI probes used a 30-second bound, derived from prior subsecond probes of the same `--eval` form.

Exact commands:

```sh
probe_source=$(python3 -c 'print("Switch[1,1,7,2,0]")')
RUM_PROFILE_REPR=1 RUM_PROFILE_ALLOC=1 \
scripts/run-bounded.py --timeout 30 -- \
/tmp/rum-brief369-target/release/rum-cli --eval "$probe_source" < /dev/null
```

```text
7
rum-profile repr into_vec_total=0 into_vec_clone=0 into_owned_total=0 into_owned_clone=0 shared_args_alloc=7 shared_expr_alloc=7
rum-profile alloc count=1551 bytes_total=436600 peak_live_bytes=259416
```

```sh
probe_source=$(python3 -c 'pairs=1000; print("Switch[1,1,7," + ",".join(["2","0"] * pairs) + "]")')
RUM_PROFILE_REPR=1 RUM_PROFILE_ALLOC=1 \
scripts/run-bounded.py --timeout 30 -- \
/tmp/rum-brief369-target/release/rum-cli --eval "$probe_source" < /dev/null
```

```text
7
rum-profile repr into_vec_total=0 into_vec_clone=0 into_owned_total=0 into_owned_clone=0 shared_args_alloc=7 shared_expr_alloc=7
rum-profile alloc count=13562 bytes_total=1743326 peak_live_bytes=908091
```

The matched control uses `CompoundExpression`: it makes the one required working `Vec`, but its suspension closure clones only inside the lazy factory.

```sh
probe_source=$(python3 -c 'print("CompoundExpression[Return[7],2,0,2,0]")')
RUM_PROFILE_REPR=1 RUM_PROFILE_ALLOC=1 \
scripts/run-bounded.py --timeout 30 -- \
/tmp/rum-brief369-target/release/rum-cli --eval "$probe_source" < /dev/null
```

```text
rum-profile repr into_vec_total=0 into_vec_clone=0 into_owned_total=0 into_owned_clone=0 shared_args_alloc=8 shared_expr_alloc=8
rum-profile alloc count=1567 bytes_total=439310 peak_live_bytes=260366
7
```

```sh
probe_source=$(python3 -c 'pairs=1001; print("CompoundExpression[Return[7]," + ",".join(["2","0"] * pairs) + "]")')
RUM_PROFILE_REPR=1 RUM_PROFILE_ALLOC=1 \
scripts/run-bounded.py --timeout 30 -- \
/tmp/rum-brief369-target/release/rum-cli --eval "$probe_source" < /dev/null
```

```text
rum-profile repr into_vec_total=0 into_vec_clone=0 into_owned_total=0 into_owned_clone=0 shared_args_alloc=8 shared_expr_alloc=8
rum-profile alloc count=13579 bytes_total=1682228 peak_live_bytes=908703
7
```

All four liveness assertions evaluated to exactly `7`.

The `Expr` size probe was also bounded to 30 seconds:

```sh
rum_core_rlib=$(find /tmp/rum-brief369-target/release/deps -name 'librum_core-*.rlib' -print -quit)
scripts/run-bounded.py --timeout 30 -- \
rustc --edition=2024 /tmp/rum-brief369-probes/expr_size.rs \
  --extern "rum_core=$rum_core_rlib" \
  -L dependency=/tmp/rum-brief369-target/release/deps \
  -o /tmp/rum-brief369-probes/expr_size
scripts/run-bounded.py --timeout 30 -- \
/tmp/rum-brief369-probes/expr_size < /dev/null
```

```text
expr_size=32
```

Results:

| Probe | Wide − narrow `bytes_total` |
|---|---:|
| Eager `Switch` | 1,306,726 |
| Lazy control | 1,242,918 |
| Differential | **63,808** |

There are 1,998 additional outer arguments. One additional `Vec<Expr>` payload predicts `1,998 × 32 = 63,936` bytes; the measured differential is only 128 bytes lower. That isolates one eager bulk copy, approximately 31.94 bytes per added argument.

Verdict: Fact 2 holds and is non-negligible. A width-2,003 ordinary `Switch` call pays about 63.8 KiB for a suspension capture that is never used.

One correction to the brief is important: `INTO_VEC_CLONE` does not observe `args.to_vec()` because the builtin boundary has already degraded `SharedArgs` to a slice. The call bypasses `SharedArgs::into_vec`, so `into_vec_total=0`, `into_vec_clone=0`, and the shared-allocation counters remain flat. The existing allocation counter is the applicable deterministic counter for this particular copy. `INTO_VEC_CLONE` remains essential later for detecting accidental `SharedArgs` COW/unwrap fallback.

The source mechanism is direct:

```text
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs
77        let subject_result = self.evaluate(args[0].clone());
78        let subject = self.suspend_with_frame(subject_result, {
79            let args = args.to_vec();
80            move |session, subject| session.resume_switch_after_subject(args, subject)
81        })?;
82
83        self.resume_switch_after_subject(args.to_vec(), subject);
```

The lazy control instead performs its frame copy only in the `InputRequired` arm:

```text
333        let runtime_profile_suspend = runtime_path_timing_start("compound.item.suspend");
334        let outcome = self.suspend_with_result_frame_lazy(current_result, || {
335            let args = args.clone();
336            move |session, outcome| {
337                session.resume_compound_expression_after_item(args, index, outcome)
338            }
339        });
```

## 3. Ownership resolution

### Existing mechanisms

`Expr::Call` already carries `SharedArgs`, but the builtin ABI discards that handle:

```text
crates/rum-core/src/expr.rs
637 #[derive(Clone, Debug, Eq, Hash, Serialize, Deserialize)]
638 pub struct SharedArgs(Arc<Vec<Expr>>);
647     pub fn into_vec(self) -> Vec<Expr> {
648         match Arc::try_unwrap(self.0) {
649             Ok(args) => {
650                 record_into_vec(false);
651                 args
652             }
653             Err(args) => {
654                 record_into_vec(true);
655                 args.as_ref().clone()
656             }
657         }
717 impl AsRef<[Expr]> for SharedArgs {
718     fn as_ref(&self) -> &[Expr] {
719         self.0.as_slice()
749 #[derive(Clone, Debug, Hash, Serialize, Deserialize)]
750 pub enum Expr {
751     Atom(Atom),
752     Call { head: SharedExpr, args: SharedArgs },
```

```text
crates/rum-runtime/src/runtime/metadata.rs
289 pub type BuiltinFn = fn(&mut Session, &Expr, &[Expr]) -> Result<Expr, EvalError>;
```

Current frames are consumed and removed before resume:

```text
crates/rum-runtime/src/runtime/suspension/core.rs
5 pub(in crate::runtime) struct SuspendedEvaluation {
6     pub(in crate::runtime) active_stack: Vec<EvaluationStackEntry>,
7     pub(in crate::runtime) frames: VecDeque<Box<dyn SuspensionFrame + Send>>,
8 }
10 pub(in crate::runtime) trait SuspensionFrame {
11     fn resume(
12         self: Box<Self>,
13         session: &mut Session,
14         outcome: Result<Expr, EvalError>,
15     ) -> Result<Expr, EvalError>;
```

```text
228        let mut suspension = self
229            .pending_suspension
230            .take()
231            .expect("resume requires a pending suspended evaluation");
...
242            while let Some(frame) = suspension.frames.pop_front() {
243                outcome = frame.resume(self, outcome);
```

That is why the currently executing frame is absent from `Session` during a sweep.

### Candidate assessment

| Candidate | Ordinary path | Rooted-resume result |
|---|---|---|
| `SharedArgs`, `Arc<[T]>`, `Arc<Map>` with borrowed downstream APIs | Zero allocation/refcount bump when the existing handle is borrowed; freeze or clone only in `InputRequired` | Correct for immutable/read-only state |
| Downstream functions borrow `&[T]`/`&Map` | Best ordinary-path result | Correct only while work remains synchronous; a borrow cannot become a later `'static` frame and cannot represent mutation |
| `self: Arc<Self>` plus `Arc::try_unwrap` | No ordinary cost | Fails categorically while rooted: the queue owns one strong reference and the driver must clone another to invoke `resume`, so the count is at least two |
| Pop the queue Arc, then `try_unwrap` | No clone | Violates the primary invariant because the executing frame is no longer Session-visible |
| Store a `Weak` queue reference | Cheap | Not a root; current tracing explicitly treats `WeakSharedArgs` as rootless |
| `Arc::make_mut`/COW | Cheap only until mutation | The rooted frame and working handle make the allocation shared, forcing an O(N) clone on mutation |
| `Option::take`, `RefCell::take`, or similar | No ordinary cost | The root disappears from the frame during the resume call; unsafe under a sweep |
| `Mutex`-protected mutable state | Lock/branch and possible allocation | Rejected by the brief and standing default-path rule; taking state while locked also removes roots |
| Deferred sweep, pinning, snapshot roots | Various | Explicitly rejected; each preserves the defect rather than rooting frame state |

The relevant trace behavior is verbatim:

```text
crates/rum-runtime/src/runtime/suspension/trace.rs
41 impl FrameRootTrace for SharedArgs {
42     fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
43         for expr in self {
44             visitor.visit_expr(expr);
45         }
46     }
47 }
49 impl FrameRootTrace for WeakSharedArgs {
50     fn trace_roots(&self, _visitor: &mut dyn FrameRootVisitor) {}
...
106 impl<T: FrameRootTrace> FrameRootTrace for Arc<T> {
107     fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
108         self.as_ref().trace_roots(visitor);
109     }
110 }
112 impl<T: FrameRootTrace> FrameRootTrace for Arc<[T]> {
113     fn trace_roots(&self, visitor: &mut dyn FrameRootVisitor) {
114         for value in self.iter() {
115             value.trace_roots(visitor);
116         }
117     }
118 }
```

### Recommendation

Use an immutable/persistent hybrid:

1. Preserve the upstream handle by changing `BuiltinFn` globally to receive `&SharedArgs`. Helpers that only need a slice continue receiving `args.as_ref()`.

2. Keep the ordinary path in borrowed or uniquely owned form. Construct no frame, `Arc`, frozen carrier, or persistent node unless the evaluated result is `InputRequired`.

3. Frame fields use:

   - `SharedArgs` or `Arc<[T]>` plus a cursor for fixed sequences;
   - immutable queue slices plus scalar cursors instead of consuming iterators;
   - persistent chunk chains for append-only accumulators;
   - an immutable base plus delta/tombstone overlays for matcher bindings and maps;
   - family-specific immutable state structs for compound cases.

4. A rooted frame never moves bulk state directly into a continuation. It reads immutable state and returns a driver action. Any evaluation that could sweep is represented as `Evaluate { expr, next_frame }`; the driver installs `next_frame` in the Session-rooted queue before evaluating `expr`.

5. Where final output needs ownership, return a named pure finalizer. The driver first removes and drops the completed frame and its dispatch Arc, then invokes the finalizer. The finalizer has no `&mut Session`, cannot evaluate or sweep, and can safely `Arc::try_unwrap` uniquely held persistent chunks without cloning.

This has zero ordinary-path cost by construction. Costs occur only after a real suspension: small Arc bumps, cursor updates, and O(1) persistent nodes. There is one final linear traversal, not an O(N) clone at every continuation step.

The inventory proves pure borrowing is insufficient: at least 94 downstream functions have a syntactically mutable bulk parameter, covering 105 parameters. That is a lower bound, not merely an estimate.

## 4. Frame construction only in the suspending arm

Use one declarative macro, with no stored or runtime closure factory:

```rust
resume_or_suspend! {
    session,
    result,
    ok(value) => continue_owned(session, owned_state, value),
    suspend => SwitchAfterSubjectFrame {
        args: shared_args.clone(),
    },
}
```

Conceptual expansion:

```rust
match result {
    Ok(value) => continue_owned(session, owned_state, value),
    Err(EvalError::InputRequired { request }) => {
        session.insert_named_frame(SwitchAfterSubjectFrame {
            args: shared_args.clone(),
        });
        Err(EvalError::InputRequired { request })
    }
    Err(error) => Err(error),
}
```

Rust evaluates the frame expression only in the `InputRequired` arm. The `Ok` arm contains no frame, no frame Arc, no allocation, and no frame-state conversion.

Proposed internal protocol:

```rust
trait SuspensionFrame: FrameRootTrace + Send + Sync + 'static {
    fn resume(
        self: Arc<Self>,
        session: &mut Session,
        outcome: Result<Expr, EvalError>,
    ) -> FrameAction;
}

enum FrameAction {
    Complete(Result<Expr, EvalError>),
    Evaluate {
        expr: Expr,
        next_frame: Arc<dyn SuspensionFrame + Send + Sync>,
    },
    Finalize(Box<dyn PureFrameFinalizer + Send>),
}

trait PureFrameFinalizer {
    fn finish(self: Box<Self>) -> Result<Expr, EvalError>;
}
```

Operational rules:

- `pending_suspension` moves into `resuming_suspension`.
- The original frame Arc remains at the front of `resuming_suspension` while a cloned Arc invokes `resume`.
- `Complete` pops the completed frame, then passes its outcome to the suffix.
- `Evaluate` replaces the current frame with `next_frame` before evaluation. If evaluation suspends, newly produced inner frames precede `next_frame` and the old suffix, preserving current nested ordering.
- `Finalize` pops and drops the rooted frame and dispatch clone before invoking a named, Session-free finalizer.
- GC and memory traverse both suspension slots.
- There remains one queue, one suspension representation, one nested append rule, and one traversal. `FrameAction` is an internal transition protocol, not a second suspension mechanism.
- `Arc::try_unwrap` is forbidden inside `SuspensionFrame::resume`; it is permitted only in the Session-free finalization phase after the rooted references have been dropped.

The macro is preferable to 399 explicit matches. The matches have identical runtime behavior, but repeating queue insertion and error propagation at every boundary makes the invariant harder to audit and exposes more direct queue-mutation sites. The macro provides one readable three-arm idiom, keeps insertion private, and makes “frame expression appears only in the suspending arm” mechanically searchable.

Current eager versus lazy behavior confirms the construction point:

```text
crates/rum-runtime/src/runtime/suspension/core.rs
142 pub(in crate::runtime) fn suspend_with_result_frame<F>(
...
152     match result {
153         Ok(value) => Ok(value),
154         Err(EvalError::InputRequired { request }) => {
155             self.push_suspension_frame(frame);
...
162 pub(in crate::runtime) fn suspend_with_result_frame_lazy<M, F>(
...
173     match result {
174         Ok(value) => Ok(value),
175         Err(EvalError::InputRequired { request }) => {
176             self.push_suspension_frame(make_frame());
```

## 5. Downstream inventory, landability, and file size

The grep seed was:

```sh
rg -n \
  "suspend_with_frame|suspend_with_result_frame|drive_evaluated_value|drive_internal_value|push_suspension_frame" \
  crates/rum-runtime/src crates/rum-stdlib/src -g '*.rs' |
rg -v '/tests/|suspension/core.rs' |
wc -l
```

```text
399
```

A throwaway syntax-aware resolver under `/tmp/rum-brief369-inventory` parsed those sites, resolved direct terminal calls and recursively classified custom nominal types. Its liveness output was:

```text
rust_files 462
suspension_sites 399
captured_terminal_calls 479
owned_bulk_type_names 244
bulk_terminal_functions 232
captured_bulk_parameters 462
ambiguous_bulk_resolutions 2
```

Breakdown:

- 399 current boundaries in 122 caller files: 79 runtime and 43 stdlib.
- 270 boundaries reach at least one owned-bulk continuation.
- Runtime: 145 functions, 296 parameters, 55 files.
- Stdlib: 87 functions, 166 parameters, 36 files.
- 94 functions/105 parameters are syntactically mutable.
- `Vec<Expr>` alone accounts for 176 parameters, 43 mutable.
- The two qualified-constructor ambiguities resolve to `CasesCollectionOutcome::new` and `PositionCollectionOutcome::new` in `model/traversal.rs:119,136`.

<details>
<summary>Complete 232-function downstream signature inventory</summary>

```text
crates/rum-runtime/src/runtime/attributes.rs:104 finish_update_attributes_after_resolved_attributes — original_args: Vec<Expr>; targets: Vec<Symbol>
crates/rum-runtime/src/runtime/debug_tools/reap.rs:163 resume_reap_scope_group_with_value — state: ReapScopeRenderState; mut rendered_groups: Vec<Expr>
crates/rum-runtime/src/runtime/debug_tools/reap.rs:217 resume_reap_bucket_with_value — state: ReapBucketRenderState; mut rendered: Vec<Expr>
crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:39 resume_time_constrained_with_limit — raw_args: Vec<Expr>
crates/rum-runtime/src/runtime/debug_tools/resources/constraints.rs:100 resume_memory_constrained_with_limit — raw_args: Vec<Expr>
crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:124 resume_test_report_after_evaluation — case_state: TestReportCaseEvalState
crates/rum-runtime/src/runtime/debug_tools/test_report/report.rs:161 resume_test_report_after_output_match — output_state: TestReportCaseOutputState
crates/rum-runtime/src/runtime/definitions/application/default_values.rs:99 evaluate_first_exact_default_value_from_cursor — down_values: Vec<DownValue>; lookup_order: Vec<Expr>; mut pending: Option<(usize, CompiledMatchOutcome)>
crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:112 apply_replace_all_after_miss — rules: RewriteRulePlan
crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:173 apply_replace_all_args — mut rewritten_args: Vec<Expr>; remaining_args: SharedArgs; rules: RewriteRulePlan
crates/rum-runtime/src/runtime/definitions/application/replace_all.rs:261 apply_rules_once_from_list — rules: RewriteRulePlan
crates/rum-runtime/src/runtime/definitions/application/sub_values.rs:51 try_apply_sub_values_from_symbol — mut pending: Option<CompiledMatchOutcome>
crates/rum-runtime/src/runtime/definitions/application/up_values.rs:46 try_apply_up_values_from_list — up_values: Vec<(Symbol, UpValue)>; mut pending: Option<CompiledMatchOutcome>
crates/rum-runtime/src/runtime/definitions/definition_catalog/arguments.rs:20 resume_resolve_argument_exprs_after_first — rest: Vec<Expr>
crates/rum-runtime/src/runtime/definitions/definition_updates.rs:411 store_held_update — target: HeldUpdateTarget
crates/rum-runtime/src/runtime/definitions/part_assignment.rs:54 resume_positional_part_selector_evaluation — target: PositionalPartAssignmentTarget
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:119 evaluate_guard_owned — option_scope: Option<OptionValueScope>
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:135 evaluate_guard_sequence_owned — guards: Box<[Expr]>; option_scope: Option<OptionValueScope>
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:151 evaluate_owned_guards — guards: OwnedGuards; bindings: SymbolExprMap; false_bindings: SymbolExprMap; option_scope: Option<OptionValueScope>
crates/rum-runtime/src/runtime/definitions/rhs_guards.rs:199 guard_outcome — bindings: SymbolExprMap
crates/rum-runtime/src/runtime/eval_call_prepare.rs:472 resume_call_after_argument_with_unevaluated_restore — mut evaluated_args: Vec<Expr>; remaining_args: Vec<Expr>; unevaluated_restores: Option<Vec<(usize, Expr)>>
crates/rum-runtime/src/runtime/eval_core/call.rs:65 resume_call_after_head — args: SharedArgs
crates/rum-runtime/src/runtime/eval_core/call.rs:89 resume_call_with_dispatch_plan — plan: CallDispatchPlan; state: CallDispatchState; unevaluated_restore: Option<Box<UnevaluatedRestorePlan>>
crates/rum-runtime/src/runtime/kernel_data/association_data.rs:678 resume_association_set_lvalue_selector_evaluation — target: AssociationSetLValueTarget
crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:135 resume_check_abort_with_propagation — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/abort_cleanup.rs:197 resume_check_abort_after_primary — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:20 resume_assert_after_test — assert_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/assertions.rs:47 resume_assert_after_function — assert_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:110 resume_while_loop — args: Vec<Expr>; mut pending: Option<Result<Expr, EvalError>>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:191 resume_until_loop — args: Vec<Expr>; mut pending: Option<Result<Expr, EvalError>>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:238 resume_for_loop — args: Vec<Expr>; mut pending: Option<Result<Expr, EvalError>>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:295 resume_if_after_condition — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:351 resume_compound_expression_after_item — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:382 resume_which_after_condition — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:399 resume_switch_after_subject — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops.rs:407 resume_switch_from_index — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:48 finish_do_spec_eval — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:110 resume_do_after_iteration — args: Vec<Expr>; spec: DoSpec
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:141 finish_table_spec_eval — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:228 resume_table_after_iteration — args: Vec<Expr>; spec: DoSpec; mut collected: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration/driver.rs:348 finish_loop_iterator_scope — saved_state: BlockSymbolState
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:78 finish_iteration_aggregate_spec_eval — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/branching_loops/iteration_aggregate.rs:152 resume_iteration_aggregate_after_iteration — args: Vec<Expr>; spec: DoSpec; mut collected: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:94 resume_check_after_primary — selectors: Option<Vec<MessageSelector>>
crates/rum-runtime/src/runtime/kernel_data/control_data/check_quiet.rs:135 resume_quiet_after_primary — selectors: Option<Vec<MessageSelector>>
crates/rum-runtime/src/runtime/kernel_data/control_data/cleanup.rs:87 resume_with_cleanup_after_cleanup — primary_outcome: Result<Expr, EvalError>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:222 resume_confirm_after_value — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:239 finish_confirm_failure — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:266 resume_confirm_by_after_value — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:282 resume_confirm_by_after_test — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:317 resume_confirm_match_after_value — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:357 resume_confirm_quiet_after_body — args: Vec<Expr>; selectors: ConfirmQuietSelectors
crates/rum-runtime/src/runtime/kernel_data/control_data/confirmation/builtins.rs:397 resume_confirm_assert_after_test — args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:175 resume_catch_exceptions_after_body — handlers: Option<Vec<ExceptionHandler>>
crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:216 dispatch_exception_handlers — type_chain: Vec<String>; handlers: Vec<ExceptionHandler>
crates/rum-runtime/src/runtime/kernel_data/control_data/exceptions.rs:282 evaluate_exception_handler_rhs — handler: ExceptionHandler
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:26 resume_cases_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:51 resume_cases_after_resolved_spec — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:72 finish_cases_after_resolved_traversal — spec: CasesSpec; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/collect.rs:229 collect_cases_children — children: Vec<(Expr, bool)>; spec: CasesSpec; mut matches: Vec<Expr>; mut pending: Option<CasesCollectionOutcome>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:39 resume_delete_cases_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:59 resume_delete_cases_after_pattern — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:81 finish_delete_cases_after_resolved_traversal — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:235 delete_cases_call_children_from_queue — children: Vec<(bool, Expr, bool)>; mut rebuilt_args: Vec<Expr>; mut pending: Option<DeleteCasesOutcome>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/cases/delete.rs:425 delete_cases_association_children_from_queue — children: Vec<(bool, Option<Expr>, Expr, bool)>; mut rebuilt_entries: Vec<Expr>; mut pending: Option<DeleteCasesOutcome>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:37 resume_replace_list_after_target — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:62 resume_replace_list_after_limit — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list.rs:137 collect_replace_list_results_with_pending — rules: Vec<RewriteRule>; collected: Vec<Expr>; mut pending: Option<Vec<SymbolExprMap>>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:110 filter_replace_list_guard_bindings_owned — guards: Box<[Expr]>; bindings: Vec<SymbolExprMap>; option_scope: Option<OptionValueScope>; mut accepted: Vec<SymbolExprMap>; mut pending: Option<CompiledMatchOutcome>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/list/matching.rs:161 collect_replace_list_rule_results_owned — rules: Vec<RewriteRule>; rule: RewriteRule; bindings: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:43 resume_replace_after_target — heads_option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:198 replace_children_owned — rule_plan: RewriteRulePlan
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:304 replace_call_children_owned — args: Vec<Expr>; rule_plan: RewriteRulePlan; mut rebuilt_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/once.rs:388 replace_association_children_owned — args: Vec<Expr>; rule_plan: RewriteRulePlan; mut rebuilt_entries: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:43 resume_replace_repeated_after_current — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:116 resume_replace_repeated_loop — rules: Vec<RewriteRule>
crates/rum-runtime/src/runtime/kernel_data/rewrite_data/replace/repeated.rs:152 resume_replace_repeated_after_next — rules: Vec<RewriteRule>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:73 plan_guarded_replace_part_pattern_targets_owned — rules: Vec<ReplacePartRule>; mut targets: Vec<GuardedReplacePartTarget>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:106 match_guarded_replace_part_pattern_targets_owned — rules: Vec<ReplacePartRule>; mut targets: Vec<GuardedReplacePartTarget>; mut pending: Option<CompiledMatchOutcome>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/guarded.rs:172 select_guarded_replace_part_targets_owned — rules: Vec<ReplacePartRule>; mut targets: Vec<GuardedReplacePartTarget>; mut pending: Option<CompiledMatchOutcome>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:41 resume_replace_at_after_target — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:62 resume_replace_at_after_positions — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_at.rs:102 replace_at_paths_owned — paths: Vec<Vec<isize>>; rule_plan: RewriteRulePlan
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:55 apply_replace_part_rules_owned — rules: Vec<ReplacePartRule>; mut pending_targets: Option<Vec<ReplacePartTarget>>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:129 collect_rule_targets_owned — position_specs: Vec<ReplacePartPositionSpec>; mut collected: Vec<ReplacePartTarget>
crates/rum-runtime/src/runtime/kernel_data/structure_data/replace/replace_part.rs:213 collect_pattern_position_targets_from_candidates_owned — candidates: Vec<Vec<isize>>; mut collected: Vec<ReplacePartTarget>
crates/rum-runtime/src/runtime/model/traversal.rs:119 new — matches: Vec<Expr>
crates/rum-runtime/src/runtime/model/traversal.rs:136 new — positions: Vec<Expr>
crates/rum-runtime/src/runtime/options/builtins.rs:154 finish_set_options_from_resolved — targets: Vec<Symbol>; original_args: Vec<Expr>
crates/rum-runtime/src/runtime/options/builtins.rs:225 finish_option_value_from_explicit — query: OptionValueQuery; original_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/core.rs:63 match_compiled_alternatives_owned — alternatives: Vec<MatcherNode>
crates/rum-runtime/src/runtime/pattern_matching/core.rs:264 match_compiled_promoted_singleton_owned — args: Vec<MatcherArg>
crates/rum-runtime/src/runtime/pattern_matching/core.rs:303 match_compiled_key_value_pattern_entries_owned — entries: Vec<(MatcherNode, MatcherNode, bool)>
crates/rum-runtime/src/runtime/pattern_matching/core.rs:324 match_compiled_key_value_pattern_choice_owned — key_pattern: MatcherNode; value_pattern: MatcherNode; remaining_entries: Vec<(MatcherNode, MatcherNode, bool)>; candidates: Vec<AssociationEntryForm>
crates/rum-runtime/src/runtime/pattern_matching/core.rs:404 evaluate_sequence_pattern_checks_owned — checks: Vec<SequencePatternCheck>; bindings: SymbolExprMap; snapshot: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/outcome.rs:82 match_outcome — bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:129 match_compiled_flat_single_choices_owned — pattern: MatcherNode; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:213 match_compiled_flat_wrapped_single_choice_owned — pattern: MatcherNode; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:401 collect_compiled_flat_single_choice_matches_owned — pattern: MatcherNode; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence.rs:488 collect_compiled_flat_wrapped_single_choice_matches_owned — pattern: MatcherNode; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:4 match_compiled_args_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:31 match_compiled_args_with_flat_head_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:57 match_compiled_orderless_args_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:76 match_compiled_orderless_args_with_flat_head_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:102 collect_compiled_args_matches_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:133 collect_compiled_args_matches_with_flat_head_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:163 collect_compiled_orderless_arg_matches_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/entry.rs:191 collect_compiled_orderless_arg_matches_with_flat_head_owned — pattern_args: Vec<MatcherArg>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:4 match_compiled_sequence_choices — spec: CompiledSequencePattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/choices.rs:87 match_compiled_flat_sequence_choices — spec: CompiledSequencePattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:4 collect_compiled_sequence_choice_matches_owned — spec: CompiledSequencePattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap; consumes: Vec<usize>; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/collect.rs:59 collect_compiled_flat_sequence_choice_matches_owned — spec: CompiledSequencePattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap; consumes: Vec<usize>; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:152 match_compiled_optional_omission_owned — spec: CompiledOptionalPattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/optional.rs:195 match_compiled_flat_optional_omission_owned — spec: CompiledOptionalPattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/optional_sequence/pattern.rs:4 match_compiled_sequence_items_owned — matcher: MatcherNode; values: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:9 match_compiled_ordered_args_iter_owned — mut pattern_args: std::vec::IntoIter<MatcherArg>; mut value_args: std::vec::IntoIter<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/ordered.rs:71 collect_compiled_ordered_args_iter_owned — mut pattern_args: std::vec::IntoIter<MatcherArg>; mut value_args: std::vec::IntoIter<Expr>
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:5 collect_compiled_orderless_choices_matches_owned — pattern: MatcherArg; rest_patterns: Vec<MatcherArg>; choices: Vec<OrderlessCandidateChoice>; bindings: SymbolExprMap; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/collect.rs:59 collect_compiled_orderless_flat_choices_matches_owned — pattern: MatcherArg; rest_patterns: Vec<MatcherArg>; choices: Vec<OrderlessCandidateChoice>; bindings: SymbolExprMap; collected: Vec<SymbolExprMap>
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:23 match_compiled_orderless_choices_owned — pattern: MatcherArg; rest_patterns: Vec<MatcherArg>; choices: Vec<OrderlessCandidateChoice>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/match_choices.rs:88 match_compiled_orderless_flat_choices_owned — pattern: MatcherArg; rest_patterns: Vec<MatcherArg>; choices: Vec<OrderlessCandidateChoice>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:157 match_compiled_orderless_optional_omission_owned — spec: CompiledOptionalPattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching/sequence/orderless/optional.rs:259 match_compiled_orderless_flat_optional_omission_owned — spec: CompiledOptionalPattern; rest_patterns: Vec<MatcherArg>; value_args: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:3 interpreted_match_outcome — bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:141 match_alternatives_interpreted_owned — alternatives: Vec<Expr>; bindings: SymbolExprMap
crates/rum-runtime/src/runtime/pattern_matching_interpreted.rs:299 match_call_args_interpreted_owned — pattern_args: Vec<Expr>; value_args: Vec<Expr>
crates/rum-runtime/src/runtime/resolution.rs:386 resume_fixed_point_from — fixed_point_seen: Vec<Expr>
crates/rum-runtime/src/runtime/scoping/lexical/block.rs:312 resume_block_after_initializer — items: Vec<(Symbol, Option<Expr>)>; saved_state: BTreeMap<Symbol, BlockSymbolState>
crates/rum-runtime/src/runtime/scoping/lexical/block.rs:337 resume_single_symbol_block_after_initializer — saved_state: BlockSymbolState
crates/rum-runtime/src/runtime/scoping/lexical/block.rs:388 finish_single_symbol_block_scope — saved_state: BlockSymbolState
crates/rum-runtime/src/runtime/scoping/lexical/block.rs:488 finish_block_scope — saved_state: BTreeMap<Symbol, BlockSymbolState>
crates/rum-runtime/src/runtime/scoping/lexical/module.rs:73 resume_module_initializers — items: Vec<(Symbol, Option<Expr>)>; renames: SymbolExprMap; mut pending: Option<Result<Expr, EvalError>>
crates/rum-runtime/src/runtime/scoping/lexical/with.rs:39 resume_with_bindings — items: Vec<(Symbol, Expr)>
crates/rum-runtime/src/runtime/scoping/packages.rs:274 resume_package_after_form — plan: PackageInstallPlan
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:88 resume_count_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:108 resume_count_after_pattern — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:130 resume_member_q_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:150 resume_member_q_after_pattern — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:172 resume_free_q_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:192 resume_free_q_after_pattern — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:379 count_pattern_children — children: Vec<(Expr, bool)>
crates/rum-runtime/src/runtime/tree_walkers/position.rs:26 resume_position_after_target — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/position.rs:46 resume_position_after_pattern — option_exprs: Vec<Expr>; call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/position.rs:67 finish_position_after_resolved_traversal — call_args: Vec<Expr>
crates/rum-runtime/src/runtime/tree_walkers/position.rs:250 collect_position_children — children: Vec<(Expr, Vec<Expr>, bool)>; path: Vec<Expr>; mut positions: Vec<Expr>; mut pending: Option<PositionCollectionOutcome>
crates/rum-stdlib/src/associations/aggregate.rs:191 resume_counts_after_association_value — entries: Vec<AssociationEntryForm>; mut values: Vec<Expr>; spec_keys: Option<Vec<Expr>>
crates/rum-stdlib/src/associations/aggregate.rs:239 resume_catenate_direct_association_value — call_args: Vec<Expr>; entries: Vec<AssociationEntryForm>; mut values: Vec<Expr>
crates/rum-stdlib/src/associations/aggregate.rs:341 resume_catenate_after_association_value — call_args: Vec<Expr>; steps: Vec<CatenateStep>; mut flattened: Vec<Expr>
crates/rum-stdlib/src/associations/basic.rs:71 resume_association_key_evaluation_with_key — entries: Vec<AssociationEntryForm>; mut evaluated_entries: Vec<AssociationEntryForm>
crates/rum-stdlib/src/associations/basic.rs:136 eval_lookup_with_evaluated_data — rest: Vec<Expr>
crates/rum-stdlib/src/associations/basic.rs:154 finish_eval_lookup — rest: Vec<Expr>
crates/rum-stdlib/src/associations/basic.rs:198 finish_prepared_lookup — lookup: PreparedLookup
crates/rum-stdlib/src/associations/grouping/association.rs:111 resume_association_group_by_with_key — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; entries: Vec<AssociationEntryForm>; grouped: IndexedGroups<AssociationEntryForm>
crates/rum-stdlib/src/associations/grouping/association.rs:175 resume_association_group_by_with_entry — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; entries: Vec<AssociationEntryForm>; mut grouped: IndexedGroups<AssociationEntryForm>
crates/rum-stdlib/src/associations/grouping/association.rs:308 resume_association_group_by_nested_with_value — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; grouped: Vec<(Expr, Vec<AssociationEntryForm>)>; mut nested: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/grouping/association.rs:372 resume_association_group_by_reduce_with_value — call_args: Vec<Expr>; grouped: Vec<(Expr, Vec<AssociationEntryForm>)>; mut reduced: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/grouping/list.rs:97 resume_group_by_with_key — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; items: Vec<Expr>; grouped: IndexedGroups<Expr>
crates/rum-stdlib/src/associations/grouping/list.rs:139 resume_group_by_with_item_value — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; items: Vec<Expr>; mut grouped: IndexedGroups<Expr>
crates/rum-stdlib/src/associations/grouping/list.rs:277 resume_group_by_nested_with_value — call_args: Vec<Expr>; specs: Vec<GroupBySpec>; grouped: Vec<(Expr, Vec<Expr>)>; mut nested: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/grouping/list.rs:343 resume_group_by_reduce_with_value — call_args: Vec<Expr>; grouped: Vec<(Expr, Vec<Expr>)>; mut reduced: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/key_reshape.rs:173 resume_key_group_by_with_key — entries: Vec<AssociationEntryForm>; mut grouped: IndexedGroups<AssociationEntryForm>
crates/rum-stdlib/src/associations/key_reshape.rs:233 resume_key_group_by_reduce_with_value — grouped: Vec<(Expr, Vec<AssociationEntryForm>)>; mut reduced: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/missing.rs:280 resume_delete_missing_list_after_child — items: Vec<Expr>; mut kept: Vec<Expr>
crates/rum-stdlib/src/associations/missing.rs:437 resume_delete_missing_association_after_predicate — entries: Vec<AssociationEntryForm>; mut kept: Vec<AssociationEntryForm>
crates/rum-stdlib/src/associations/missing.rs:462 resume_delete_missing_association_after_child — entries: Vec<AssociationEntryForm>; mut kept: Vec<AssociationEntryForm>
crates/rum-stdlib/src/associations/mutation.rs:199 finish_associate_to_value — call_args: Vec<Expr>
crates/rum-stdlib/src/associations/mutation.rs:331 finish_key_drop_from_value — call_args: Vec<Expr>
crates/rum-stdlib/src/associations/resumable/aggregate.rs:149 resume_merge_nested_with_value — grouped: Vec<(Expr, Vec<Expr>)>; mut merged: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/resumable/aggregate.rs:234 resume_merge_with_value — grouped: Vec<(Expr, Vec<Expr>)>; mut merged: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/resumable/aggregate.rs:288 resume_counts_by_with_value — call_args: Vec<Expr>; items: Vec<Expr>; mut counts: IndexedGroups<()>
crates/rum-stdlib/src/associations/resumable/key_select.rs:69 resume_key_select_with_value — entries: Vec<AssociationEntryForm>; mut selected: Vec<AssociationEntryForm>
crates/rum-stdlib/src/associations/resumable/key_sort.rs:162 resume_key_sort_by_with_value — entries: Vec<AssociationEntryForm>; mut decorated: Vec<(AssociationEntryForm, Expr)>
crates/rum-stdlib/src/associations/resumable/key_sort.rs:175 resume_key_sort_by_ordering — mut state: KeySortByOrderingState
crates/rum-stdlib/src/associations/resumable/map.rs:80 resume_association_map_with_value — keys: Vec<Expr>; mut entries: Vec<(Expr, Expr)>
crates/rum-stdlib/src/associations/resumable/map.rs:132 resume_association_map_association_with_value — items: Vec<Expr>; mut mapped_items: Vec<Expr>
crates/rum-stdlib/src/associations/transforms/key_collections.rs:334 resume_key_union_missing_value — plan: KeyUnionPlan; missing_keys: Vec<Expr>; mut fill_values: Vec<(Expr, Expr)>
crates/rum-stdlib/src/functional_ops/arrays/generators.rs:207 resume_array_with_value — state: ArrayEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/functional_ops/arrays/generators.rs:252 resume_outer_with_value — state: OuterEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:250 resume_tensor_callback_with_value — state: TensorCallbackEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:372 resume_inner_term_with_value — state: InnerEvalState; mut terms: Vec<Expr>; output_scalars: Vec<Expr>
crates/rum-stdlib/src/functional_ops/arrays/tensor.rs:392 resume_inner_reduced_with_value — state: InnerEvalState; mut output_scalars: Vec<Expr>
crates/rum-stdlib/src/functional_ops/iteration.rs:268 resume_nest_with_value — mut items: Vec<Expr>
crates/rum-stdlib/src/functional_ops/iteration.rs:305 resume_fixed_point_with_value — mut seen: Vec<Expr>; mut items: Vec<Expr>
crates/rum-stdlib/src/functional_ops/iteration.rs:357 resume_nest_while_with_test_value — history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/iteration.rs:389 resume_nest_while_with_step_value — mut history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/fold/pair.rs:76 resume_fold_pair_with_pair_result — state: FoldPairEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/pair.rs:113 resume_fold_pair_with_output — state: FoldPairEvalState
crates/rum-stdlib/src/functional_ops/sequence/fold/plain.rs:48 resume_fold_with_value — state: FoldEvalState; mut history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/fold/sequence_fold.rs:63 resume_sequence_fold_with_value — state: SequenceFoldEvalState; mut history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:74 resume_fold_while_with_step_value — state: FoldWhileEvalState; mut history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:105 resume_fold_while_with_test_value — state: FoldWhileEvalState; history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/fold/while_loop.rs:187 resume_fold_while_tail_with_value — state: FoldWhileEvalState; mut history: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/select.rs:147 resume_select_with_value — source: SelectSource; mut selected: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/select.rs:222 resume_select_first_with_value — source: SelectSource
crates/rum-stdlib/src/functional_ops/sequence/window.rs:196 resume_while_sequence_with_value — sequence: SequenceLikeExpr
crates/rum-stdlib/src/functional_ops/sequence/window.rs:245 resume_accumulate_with_value — sequence: SequenceLikeExpr; mut accumulated: Vec<Expr>
crates/rum-stdlib/src/functional_ops/sequence/window.rs:289 resume_block_map_with_value — state: BlockMapEvalState; mut results: Vec<Expr>
crates/rum-stdlib/src/functional_ops/targeted_map.rs:307 resume_map_at_with_value — state: MapAtEvalState
crates/rum-stdlib/src/functional_ops/targeted_map.rs:322 finish_subset_map_from_mapped — state: SubsetMapEvalState
crates/rum-stdlib/src/logic/truth.rs:192 resume_and_with_value — args: Vec<Expr>; mut residual: Vec<Expr>
crates/rum-stdlib/src/logic/truth.rs:245 resume_or_with_value — args: Vec<Expr>; mut residual: Vec<Expr>
crates/rum-stdlib/src/logic/truth.rs:298 resume_nand_with_value — args: Vec<Expr>; mut residual: Vec<Expr>
crates/rum-stdlib/src/logic/truth.rs:351 resume_nor_with_value — args: Vec<Expr>; mut residual: Vec<Expr>
crates/rum-stdlib/src/predicates/arrays.rs:117 finish_array_q_with_rank_value — leaf_test: Option<ArrayLeafTestState>
crates/rum-stdlib/src/predicates/arrays.rs:164 resume_array_leaf_test_with_value — mut state: ArrayLeafTestState
crates/rum-stdlib/src/predicates/collections.rs:391 resume_duplicate_free_same_test_with_value — mut state: DuplicateFreeSameTestState
crates/rum-stdlib/src/predicates/collections.rs:469 resume_same_test_matrix_with_value — mut state: SameTestMatrixState
crates/rum-stdlib/src/predicates/ordering.rs:142 resume_ordered_q_comparator_with_value — mut state: OrderedQComparatorState
crates/rum-stdlib/src/predicates/ordering.rs:259 resume_lexicographic_list_comparator_with_forward_value — state: LexicographicListComparatorState
crates/rum-stdlib/src/predicates/ordering.rs:289 resume_lexicographic_list_comparator_with_reverse_value — mut state: LexicographicListComparatorState
crates/rum-stdlib/src/predicates/truth.rs:173 resume_truth_aggregate_with_value — call_args: Vec<Expr>; parts: Vec<Expr>
crates/rum-stdlib/src/strings.rs:237 resume_string_apply_with_value — state: StringApplyEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:129 resume_collect_string_cases_after_guard — state: GuardedStringCasesRuleEvalState; matches: Vec<Expr>; bindings: SymbolExprMap
crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:182 resume_collect_string_cases_with_output — state: GuardedStringCasesRuleEvalState; mut matches: Vec<Expr>
crates/rum-stdlib/src/strings/patterns/search/guarded/cases.rs:238 resume_collect_string_cases_values_with_value — state: GuardedStringCasesValuesEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:145 resume_replace_string_value_after_guard — state: GuardedStringReplaceRuleEvalState; bindings: SymbolExprMap
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:224 resume_replace_string_value_with_output — state: GuardedStringReplaceRuleEvalState
crates/rum-stdlib/src/strings/patterns/search/guarded/replace.rs:296 resume_replace_string_values_with_value — state: GuardedStringReplaceValuesEvalState; mut rendered: Vec<String>
crates/rum-stdlib/src/strings/patterns/search/rules.rs:301 resume_replace_string_value_with_output — state: StringReplaceRuleEvalState
crates/rum-stdlib/src/strings/patterns/search/rules.rs:370 resume_replace_string_values_with_value — state: StringReplaceValuesEvalState; mut rendered: Vec<String>
crates/rum-stdlib/src/strings/patterns/search/rules.rs:514 resume_collect_string_cases_with_output — state: StringCasesRuleEvalState; mut matches: Vec<Expr>
crates/rum-stdlib/src/strings/patterns/search/rules.rs:574 resume_collect_string_cases_values_with_value — state: StringCasesValuesEvalState; mut outputs: Vec<Expr>
crates/rum-stdlib/src/structure_ops/collection_stats.rs:153 resume_tally_test_with_value — mut state: TallyTestState
crates/rum-stdlib/src/structure_ops/collection_stats.rs:417 resume_gather_by_function_list_with_key — mut state: GatherByFunctionListState
crates/rum-stdlib/src/structure_ops/collection_stats.rs:600 resume_decorate_by_with_key — state: DecorateByState; mut decorated: Vec<KeyedEntry>
crates/rum-stdlib/src/structure_ops/dedup.rs:184 resume_delete_duplicates_by_with_key — items: Vec<Expr>; mut keys: AdaptiveExprSet; mut kept: Vec<Expr>
crates/rum-stdlib/src/structure_ops/dedup.rs:254 resume_delete_duplicates_test_with_value — mut state: DeleteDuplicatesTestState
crates/rum-stdlib/src/structure_ops/list_lookup.rs:178 resume_select_positions_with_value — mut state: SelectPositionsState
crates/rum-stdlib/src/structure_ops/ordering_sets.rs:604 resume_set_same_test_with_value — mut state: SetSameTestState
crates/rum-stdlib/src/structure_ops/ordering_sets.rs:721 resume_sort_by_with_key — items: Vec<Expr>; mut decorated: Vec<SortByEntry>
crates/rum-stdlib/src/structure_ops/ordering_sets.rs:778 resume_sort_by_ordering_with_key — items: Vec<Expr>; mut decorated: Vec<SortByEntry>
crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:109 resume_merge_sort — mut state: MergeSortState<T, C>
```

</details>

### Landability and exact seam

Ownership-first work is independently landable. It retains the existing closure queue as the sole suspension mechanism and uses the current lazy factory where needed. Runtime and stdlib ownership conversions may therefore land separately without creating a named-frame/opaque-frame dual path.

The seam before atomic cutover is mechanical:

- All 462 bulk parameters have been reclassified as borrowed, `SharedArgs`/immutable Arc-backed, or an approved persistent carrier.
- Every initial freeze/Arc construction occurs only in the current lazy `InputRequired` factory.
- No continuation requires moving a raw `Vec`, map, consuming iterator, or mutable custom state out of a future rooted frame.
- The syntax-aware inventory and compiler trait bounds report zero unapproved owned-bulk captures.
- The legacy closure APIs remain the only suspension mechanism until the final atomic cutover.
- The final cutover converts all current boundaries together and deletes the legacy APIs.

The global `BuiltinFn` change is a Rust API break and must compile across runtime and stdlib in one ownership step. It does not create a second handler lane; a permanent `&[Expr]`/`&SharedArgs` dual ABI is specifically rejected.

### File-size plan

Historical failed-run sizes:

| File | Before | Failed run | Added | Planned sibling |
|---|---:|---:|---:|---|
| `pattern_matching/sequence.rs` | 21,841 | 53,626 | 31,785 | `sequence/resume_frames/{mod.rs,matching.rs,collect.rs}` |
| `pattern_matching/core.rs` | 22,105 | 50,626 | 28,521 | `core/resume_frames/{mod.rs,matching.rs,guards.rs}` |
| `rewrite_data/replace/once.rs` | 20,716 | 43,299 | 22,583 | `once/resume_frames/{mod.rs,rules.rs,children.rs}` |
| `sequence/orderless/optional.rs` | 10,251 | 43,222 | 32,971 | `optional/resume_frames/{mod.rs,matching.rs,collect.rs}` |
| `rewrite_data/cases/delete.rs` | 21,890 | 39,974 | 18,084 | `delete/resume_frames.rs` |
| `confirmation/builtins.rs` | 13,898 | 37,723 | 23,825 | `builtins/resume_frames/{mod.rs,confirm.rs,quiet_assert.rs}` |
| `stdlib/structure_ops/ordering_sets.rs` | 28,749 | 36,485 | 7,736 | `ordering_sets/resume_frames.rs` |

Frames stay beside their owning behavior; there is no central frame registry. The two 28–33 KiB generated groups are responsibility-split below 20 KiB each. The final gate reports every grown file and splits any projection that would exceed 35 KiB before compilation.

The failed run also left `count_like.rs` at 33,180, optional-sequence `optional.rs` at 31,966, `associations/basic.rs` at 31,965, and `eval_call_prepare.rs` at 31,622. These require explicit final-size checks even though they did not cross 35 KiB.

Current GC and memory omissions that the atomic step must replace are:

```text
crates/rum-runtime/src/runtime/scoping/gensym_gc.rs
187        if let Some(suspension) = &self.pending_suspension {
188            for entry in &suspension.active_stack {
189                visit(&entry.expr);
190            }
191        }
```

```text
crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs
572 fn render_suspended_evaluation_memory_proxy_bytes(suspended: &SuspendedEvaluation) -> usize {
573     std::mem::size_of::<SuspendedEvaluation>()
...
581         .saturating_add(suspended.frames.len().saturating_mul(16))
582 }
```

## 6. Plan and drafted implementation briefs

### Step-by-step plan

| Step | Scope and named areas | Preserved invariant | Gate and ripple |
|---|---|---|---|
| 370. Preserve call-argument sharing | `metadata.rs:289`, `eval_dispatch/mod.rs:81-83,115-126,169-175`, all registered runtime/stdlib handlers; convert eager read-only `args.to_vec()` captures | Still one closure suspension mechanism; handlers borrow `SharedArgs`; no frame/Arc on ordinary path | fmt, runtime/stdlib/host/workspace tests; exact Switch/control counters; full hot A/B because dispatch changes |
| 371. Runtime non-matcher ownership | The runtime entries in the inventory outside matcher/rewrite: control, scoping, debug tools, options, evaluation preparation, tree walkers | Legacy lazy closures remain sole mechanism; owned-fast state freezes only after `InputRequired`; no Session publication | Runtime/workspace tests; `do-count-null`, `count-wide-no-match`, control and iteration counter probes; relevant A/B |
| 372. Matcher/rewrite ownership | `definitions/application`, `rhs_guards`, `pattern_matching/**`, `rewrite_data/**`, structure replacement paths | Immutable matcher plans, Arc slices/cursors, binding overlays and persistent result chunks; no COW mutation while rooted | Runtime/workspace tests; matcher/rewrite labels; exact suspend/resume value probes and allocation counters |
| 373. Stdlib ownership | All 87 stdlib functions/166 parameters across associations, functional operations, predicates, strings and structure operations | Crate-local immutable/persistent carriers; no second suspension path and no new public stable API | stdlib/host/workspace tests; callback sort/array/fold/string probes; sort controls and A/B |
| 374. Atomic frame/action cutover | All 399 current boundaries; suspension core, Session/reset paths, GC, memory, kernel bridge, focused runtime/stdlib/host tests and sibling frame modules | One named-frame queue; pending and resuming slots Session-rooted; frame expression only in macro’s suspend arm; one trace; old APIs deleted | Full brief-368 correctness gate, forced sweeps, memory recovery, source invariants, counter comparison, three-run hot A/B and file-size table |

Counter-first acceptance throughout:

- `RUM_PROFILE_REPR`: `into_vec_total`, `into_vec_clone`, `shared_args_alloc`, `shared_expr_alloc`.
- `RUM_PROFILE_ALLOC`: `count`, `bytes_total`, `peak_live_bytes`, specifically for slice `to_vec()` copies.
- The measured Switch/control differential must lose its current approximately 63.8 KiB width slope.
- Timing remains the required hot-path regression gate, but is not cited as the mechanism when deterministic counters are unchanged.
- Affected labels: `matcher-single-tiny`, `matcher-single-wide-small`, `matcher-single-wide-large`, `replaceall-simple`, `replace-wide-no-match`, `replacepart-wide-rule-list`, `replacepart-guarded-hit`, `replacepart-guarded-decline-fallback`, `count-wide-no-match`, `do-count-null`, and the callback-sort family.
- Controls: `sort-default-wide`, `sortby-default-wide`, `reversesort-default-wide`, `keysort-default-wide`, `keysortby-default-wide`, `sort-symbol-default-width4000`, `sort-symbol-default-width16000`.

Every implementation run must use `scripts/run-bounded.py`; its report must name the prior same-shape duration used to derive each timeout. Cargo stays `--offline`, raw A/B artifacts stay under `/tmp`, and profiling variables are off for timings.

### Draft brief 370 — preserve SharedArgs at the builtin boundary

**Goal**

Change the single builtin handler ABI to borrow `SharedArgs`, remove ordinary-path argument degradation, and eliminate eager read-only argument captures without changing language behavior or suspension representation.

**Context pointers**

- `rum-core/src/expr.rs:637-729,749-752`
- `runtime/metadata.rs:289-305`
- `runtime/eval_dispatch/mod.rs:81-126,169-175`
- Fact 2’s exact Switch/control probes above
- The full inventory in this report

**Scope**

- May modify `crates/rum-runtime/src/**`, `crates/rum-stdlib/src/**`, and focused tests required by the global handler signature.
- Must not modify `Expr`/`Symbol` representation, suspension queue representation, docs, manifests, lockfile, derive crate, or git state.

**Deliverable**

- `BuiltinFn = fn(&mut Session, &Expr, &SharedArgs)`.
- Special and ordinary dispatch pass their existing `SharedArgs` handles directly.
- Handler helpers remain slice-based where appropriate via `args.as_ref()`.
- No dual handler ABI.
- Read-only argument continuations borrow or clone `SharedArgs` only inside a lazy `InputRequired` factory.
- Updated source inventory and exact counter rows.

**rum / Rust pitfalls**

- Do not create a new `SharedArgs` from a slice.
- Do not clone a handle unconditionally just to satisfy an owned downstream signature.
- Preserve special-form raw-argument semantics and ordinary evaluated-argument semantics.
- No new unsafe, dependency, suspension API, or default-path instrumentation.

**Verification**

- Bounded `cargo fmt --check`.
- Bounded offline runtime, stdlib, host, and workspace tests.
- Repeat the four exact Fact 2 probes with numeric result `7`.
- Require no increase in `shared_args_alloc`/`shared_expr_alloc`; explain any nonzero `into_vec_clone`.
- Require the differential width slope no longer resembles one `Vec<Expr>` payload.
- Full microbench A/B, counters reported before timings.

**Hard rules**

No git writes, network, dependency, unsafe, docs change, vendor oracle, or out-of-scope edit. Preserve one suspension path. Stop after two failures of the same mechanism.

**Report format**

Per-file ABI changes; exact test outputs; counter table and verdict; affected/control A/B rows; ripple inventory; stop-loss/open questions.

### Draft brief 371 — runtime non-matcher continuation ownership

**Goal**

Convert the 145-function runtime inventory outside matcher/rewrite into owned-fast ordinary state plus immutable/persistent suspension state, while legacy lazy closures remain the only suspension mechanism.

**Context pointers**

- Runtime portion of the inventory above
- `suspension/core.rs:124-180`
- `suspension/trace.rs`
- Control, scoping, debug, option, eval-preparation and tree-walker files listed above

**Scope**

- May modify the listed runtime ownership families, family-local state modules, focused runtime tests, and doc-hidden internal helpers.
- Must not change frame storage, Session pending/resuming fields, GC/memory walks, public `Expr` shape, manifests, docs, or stdlib.

**Deliverable**

- Read-only sequences represented by `SharedArgs`/Arc slices plus cursor.
- Mutable work represented by purpose-specific immutable/persistent state.
- Ordinary functions keep owned/borrowed fast forms.
- Conversion to frozen state occurs only in lazy `InputRequired` factories.
- Compiler/source inventory reports no unapproved raw owned-bulk runtime captures in this partition.
- Unit tests cover freeze timing, cheap cloning, cursor progression, persistent accumulation and consuming finalization.

**rum / Rust pitfalls**

- Do not put an `Owned(Vec<_>)` variant inside a future frame type.
- Do not use `Arc::make_mut`, `Mutex`, `RefCell::take`, deferred sweep, pinning or snapshots as the ownership answer.
- Do not add `FrameAction` as a partially used second mechanism.
- No central carrier module that crosses file-size limits; keep family-specific logic beside owners.

**Verification**

- Bounded fmt, offline runtime and workspace tests.
- Numeric suspend/resume liveness probes for each converted family.
- Allocation/repr counters for wide controls, `do-count-null`, and `count-wide-no-match`.
- Relevant microbench A/B with same-tier neighbors.

**Hard rules / report**

Same hard rules as brief 370. Report each converted inventory row, carrier classification, tests/counters/A/B, final file sizes, and unresolved types.

### Draft brief 372 — matcher and rewrite continuation ownership

**Goal**

Close the runtime matcher/rewrite ownership partition without deep-copying `MatcherArg`, `MatcherNode`, rule plans, bindings, candidate queues, rebuilt outputs or target lists.

**Context pointers**

- Inventory rows under `definitions/application`, `rhs_guards`, `pattern_matching`, `rewrite_data`, and structure replacement
- `SharedArgs::into_vec` and trace implementations
- Pattern/rule semantics in rum’s own guide
- Existing matcher and replacement microbench labels

**Scope**

- May modify the named matcher/rewrite/replace files, family-local immutable state modules, and focused runtime tests.
- No suspension storage/driver cutover, stdlib change, dependency, unsafe, docs or public expression-shape change.

**Deliverable**

- Fixed matcher/rule data uses immutable Arc-backed sequences.
- Candidate queues use immutable storage plus cursors.
- `SymbolExprMap` mutation paths use a measured persistent overlay/delta design.
- Accumulated matches/rebuilt arguments use persistent chunks or equivalent O(1) resumed updates.
- No COW deep clone while a root is retained.
- Ownership inventory reports zero unapproved matcher/rewrite captures.

**rum / Rust pitfalls**

- Preserve ordering, backtracking, guard and option-scope semantics exactly.
- Do not weaken tests or use another engine as an oracle.
- A map overlay must trace base, delta and tombstones and must be measured against current matcher labels.
- Stop if the same overlay mechanism fails correctness or performance twice.

**Verification**

- Bounded fmt, offline runtime and workspace tests.
- Numeric suspend/resume probes for matcher bindings, guarded replacement and nested rule application.
- `into_vec_clone`, shared allocation and alloc-byte counters.
- A/B for all named matcher/rewrite/replace labels and neighbors.

**Hard rules / report**

Same common hard rules. Report the inventory mapping, overlay/persistent-state design as implemented, exact tests and counters, A/B rows and stop-loss.

### Draft brief 373 — stdlib continuation ownership

**Goal**

Convert all 87 stdlib functions and 166 bulk parameters to root-safe immutable/persistent suspension state while keeping the legacy closure queue as the sole suspension mechanism.

**Context pointers**

- Complete stdlib inventory above
- Existing `kernel_api` trace bridge
- Association, functional, string, predicate and structure-op owner modules
- Callback-sort control labels

**Scope**

- May modify the listed `rum-stdlib` files, family-local state modules, focused stdlib tests, and only the minimum doc-hidden runtime bridge already anticipated by the final cutover.
- No frame queue/driver cutover, new public stable API, manifest, dependency, unsafe, docs or core representation change.

**Deliverable**

- All call arguments retain `SharedArgs` where available.
- Fixed items/entries/specs use Arc slices and cursors.
- Outputs/history/decorated/grouped state use persistent family-local carriers.
- Freeze occurs only after actual `InputRequired`.
- Inventory reports zero unapproved stdlib owned-bulk captures.

**rum / Rust pitfalls**

- Do not publish generic continuation machinery as a supported embedding API.
- Do not centralize unrelated association, string and sort states.
- Preserve comparator callback order and exact output ordering.
- Avoid COW maps/vectors while a rooted handle exists.

**Verification**

- Bounded fmt, offline stdlib, host and workspace tests.
- Numeric liveness probes for associations, folds/windows, tensors, strings and callback sorts.
- Allocation/repr counters by family.
- Callback-sort affected rows plus all named sort controls.

**Hard rules / report**

Same common hard rules. Report every inventory row’s new ownership category, exact test/counter/A/B output, file sizes and stop-loss.

### Draft brief 374 — atomic named frame/action cutover

**Goal**

Atomically replace all current closure boundaries with named traceable frames; keep pending, executing and remaining frames rooted in `Session`; install future frame state before any nested evaluation; feed one traversal to GC and memory; and delete every closure-taking suspension API.

**Context pointers**

- brief-368 in full
- This report’s ownership resolution and current 399-site inventory
- `suspension/core.rs`, `trace.rs`, `model/session.rs:110`
- `gensym_gc.rs:187-191`
- `memory.rs:572-582`
- `kernel_api.rs:92-99`
- Planned `resume_frames` siblings above

**Scope**

- May modify the 122 caller files, suspension module, Session constructor/reset/clear paths, GC, memory, kernel bridge, focused runtime/stdlib/host tests, and responsibility-based sibling frame modules.
- `rum-derive` is out of scope; stop if the existing derive cannot express a frame.
- No core expression representation, manifest, dependency, docs, unsafe or git changes.

**Deliverable**

- `pending_suspension` and `resuming_suspension`.
- `VecDeque<Arc<dyn SuspensionFrame + Send + Sync>>`.
- `SuspensionFrame: FrameRootTrace`.
- `FrameAction` and Session-free named finalizers.
- `resume_or_suspend!` as the only generic insertion route.
- Frames constructed only inside `InputRequired`.
- Next frame installed before driver-mediated evaluation.
- One root traversal over pending and resuming frames, consumed by GC and conservative memory accounting.
- Old closure structs/APIs deleted; direct queue mutation private.
- Exact-duplicate frames shared only when compiler-resolved field signature and normalized resume body are identical; recompute from current 399 rather than accepting the historical 367 estimate.
- Compiler-plus-sorted-allowlist invariant remains green.

**rum / Rust pitfalls**

- No direct nested evaluation after producing unrooted successor state; return `FrameAction::Evaluate`.
- Never `try_unwrap` the executing frame.
- Pure finalizers receive no Session and cannot suspend.
- Preserve nested ordering: newly suspended inner frames, then installed successor, then old suffix.
- Splitting pending cleanup must never clear an active `resuming_suspension`.
- No deferred sweep, pinning, snapshot, Mutex, runtime closure factory, deep clone, unsafe or second suspension mechanism.

**Verification**

- First, in the same uncommitted atomic worktree, prove the driver/action protocol on representative Switch, current-frame and nested-input paths. If the mechanism fails twice, stop before mass conversion.
- Bounded offline fmt, runtime, stdlib, host and workspace gates.
- Sorted test-name diff against the code baseline with `removed_or_changed=0`.
- Forced-sweep Path A, Path B, matcher-binding and bare-Symbol regressions: pre-suspension value, real `Module` exit, increased sweep counter, exact resumed value.
- Current-frame plus later-suffix root test.
- Nested A → B → C ordering test.
- Large-frame memory-limit classification and session recovery.
- Existing active-stack regression retained with narrower name.
- Source invariants ban old APIs, direct queue mutation, runtime closure factories, unapproved manual trace impls and `try_unwrap` in resume code.
- Counter comparison against the post-ownership base.
- Three-run microbench A/B with profiling off, affected rows, controls and per-label spread.
- Final byte count for every grown file; none above 35 KiB.

**Hard rules**

No git writes, network, dependency, unsafe, docs change, vendor oracle or scope expansion. All commands bounded from recorded same-shape durations; Cargo offline; CLI stdin null. Stop after two failures of the same mechanism.

**Report format**

Frame count and exact-duplicate count; rooting/action design as built; enforcement; defect before/after pairs; exact tests and `removed_or_changed`; counters then A/B; file-size table; stop-loss/open questions.

## 7. Director questions, recommendations, and stop-loss

1. **Accept the public `BuiltinFn` change from `&[Expr]` to `&SharedArgs`?**  
   Recommendation: yes, before v0.1. It restores information already present in `Expr::Call` and avoids a permanent dual handler ABI.

2. **Accept the immutable-carrier plus `FrameAction`/pure-finalizer design?**  
   Recommendation: yes. Arc-only and borrow-only solve read-only state but cannot solve the measured 94-function mutable lower bound while the frame remains rooted.

3. **Use `self: Arc<Self>` or `&self` for `resume`?**  
   Recommendation: start with `self: Arc<Self>` because it composes naturally with immutable parent/state handles. Ban `try_unwrap` there. If the representative prototype proves no Arc self-use is required, prefer `&self` as the narrower receiver.

4. **How generic should persistent carriers be?**  
   Recommendation: one minimal internal sequence/cursor and chunk-chain pattern per owning crate, with matcher binding overlays kept purpose-specific and measured. No dependency and no broadly published abstraction.

5. **Macro or explicit matches?**  
   Recommendation: `resume_or_suspend!`. It provides zero-by-construction frame creation and one enforceable insertion route across 399 sites.

6. **Is a pure finalizer another suspension mechanism?**  
   Recommendation: no, provided it is a named/closed internal type, has no Session argument, cannot evaluate or suspend, and runs only after the rooted frame is dropped. Approve it under those constraints.

7. **Reuse the historical 367 frame-shape count?**  
   Recommendation: no. Recompute exact capture-signature/body equivalence from the current 399 boundaries; merge exact duplicates only.

Stop-loss was not triggered. Two serious simpler candidates—Arc-only ownership and borrow-only continuation signatures—were fully evaluated and their exact limits established. The immutable/persistent action protocol resolves the remaining contradiction without unsafe, locks, deferred sweep, pinning, new dependencies, default-path allocation, or a second suspension path.

If the director rejects the action/pure-finalizer protocol, the task is blocked under the existing constraints: there is no safe lock-free way to move mutable bulk state out of a frame while the same frame remains Session-rooted and traceable.

---

## Director's rulings (2026-08-27)

1. **`BuiltinFn` borrows `&SharedArgs`** — ACCEPTED, with the stated reason
   corrected. The report argues "before v0.1"; rum published `v0.1.0` on
   2026-07-21, so that framing is wrong. The correct reasons are that the
   builtin boundary currently discards information `Expr::Call` already
   carries, that not changing it means maintaining a dual handler ABI
   permanently, and that the measured cost is 63.8 KiB on a width-2003
   ordinary call. Verified separately: `BuiltinFn` is not itself exported from
   `kernel_api`, though `BuiltinSpec` is and its constructors take one, so the
   surface is downstream-visible in form. rum ships source-available with no
   crates.io publication and has no out-of-repo consumers, so this is an
   internal consistency change, not a semver event. 519 stdlib handlers change
   with it; under the 2026-08-27 standard that size is not an argument.

2. **Immutable carriers plus the `FrameAction` / pure-finalizer protocol** —
   ACCEPTED. Arc-only and borrow-only were each given a real hearing and shown
   insufficient against a measured lower bound of 94 functions with mutable
   bulk parameters. The protocol resolves the contradiction structurally
   rather than by convention: the finalizer runs only after the rooted frame
   and its dispatch `Arc` are dropped, so `try_unwrap` succeeds without a
   clone, and it holds no `&mut Session`, so no sweep can occur inside that
   window.

3. **`resume` receiver** — ACCEPTED as recommended: start at
   `self: Arc<Self>` with `try_unwrap` banned there, and narrow to `&self` if
   the prototype shows no Arc-self use is required. This is a measurable
   decision and does not need settling in advance.

4. **Carrier generality** — ACCEPTED. One minimal sequence/cursor and chunk
   chain per owning crate; matcher binding overlays stay purpose-specific. No
   new dependency and no broadly published abstraction: scope is set by the
   problem, and retiring cost arguments did not retire design restraint.

5. **`resume_or_suspend!`** — ACCEPTED, with one requirement made explicit in
   the implementation briefs: the macro must not accept or store a Rust
   closure. Its value is a single enforceable insertion route across 399
   sites, which is exactly the structurally-unavoidable shape this line is
   built on; a macro that can still take a closure would forfeit it.

6. **The pure finalizer is not a second suspension mechanism** — ACCEPTED
   under its four constraints: a named closed internal type, no `Session`
   argument, incapable of evaluating or suspending, and running only after the
   rooted frame is dropped. Those constraints are to be enforced by the type
   system where possible and by the repository invariant test otherwise, not
   left as prose.

7. **The historical 367 frame count is not reused** — ACCEPTED. Recompute from
   the current 399 boundaries; merge exact duplicates only.

**Added by the director:** step 370 must convert **every** eager capture site,
not a sample. `suspend_with_result_frame_lazy` already exists
(`suspension/core.rs:162`) and is used by only 9 sites while 307 still capture
eagerly — the mechanism for the 63.8 KiB waste is already in the tree and
simply was never propagated.

---

## Execution record — step 373 deferred one family (2026-08-28)

**The comparator-driven sort family is deferred into step 374**: `Sort`,
`SortBy`, `ReverseSort`, `KeySort`, `KeySortBy`. `key_sort.rs` and
`resumable_merge_sort.rs` have zero diff at `751d50de`.

Why, and why it is not a gap to paper over: a user comparator observes its
call sequence, so the sort representation cannot differ between the ordinary
and the resumed path in any way a comparator can detect. brief-373's first
attempt tried a suspension-only merge representation and changed exactly that,
which the worker caught and stopped on. Under the two-strike rule no third
merge representation is attempted before the frame lifetime exists.

**The ordering dependency this exposed, stated for step 374's brief.**
plan-369's pure finalizer makes `Arc::try_unwrap` succeed by running after the
rooted frame and its dispatch `Arc` are dropped. That lifetime is built in
step 374 — so steps 371-373 could only ever use the freeze-and-read shape
(step 372's `FrozenMatchBindings`), never a carrier whose completion assumes
uniqueness. Two stop-losses came from reaching for 374's mechanism early:
brief-368's cutover and brief-373's first attempt. Step 374 is where both the
deferred sort family and the finalizer protocol land, and it should say so
explicitly rather than rediscovering it a third time.
