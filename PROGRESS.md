# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

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
  not yet a green smoke. The next work is `PropTaut` / `Simp` performance, not
  more ad hoc Pair-source debugging.

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
