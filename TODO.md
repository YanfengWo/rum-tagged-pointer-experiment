# TODO

This file is for active or near-term work only.

Completed history, archived planning detail, and operational lessons live in
[PROGRESS.md](./PROGRESS.md). Current support status belongs in
[docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md).

## Current Cut

- [ ] **ACTIVE LINE — memory management: get rum within a constant factor of WK
  memory across the supported `mma-hol` slice** (the analog of the time line).
  Plan + research + calibration in
  [docs/memory-management-plan.md](./docs/memory-management-plan.md). Start at
  Phase 0 (implement `MaxMemoryUsed[]` + a cross-engine memory comparator); the
  big lever is Phase 1, the Module-gensym definition GC (also the prerequisite for
  the authoritative cold `run_all` 3126/0 gate, which OOMs at 50 GB today). Gate
  every phase on the comparator + Lane D `--suite memory` no-regression +
  correctness (`cargo test --workspace` + the 3126/0 corpus + `difftest check`).
- [ ] **Adopt the four-C memory-evaluation panel (external review 2026-06-30, the
  most useful single idea from the cross-model consult).** Upgrade the Phase-0
  comparator (`scripts/core-memory-microbench.sh`) from reporting only
  {`MemoryInUse` delta, `MaxMemoryUsed`, peak-RSS delta} to the FOUR constants
  vs WK, each as `C·M_ref + B`: (1) **peak RSS** (OOM risk), (2) **retained**
  (final-representation fatness: live bytes after the result settles), (3)
  **allocation traffic** (total allocated bytes — churn / temporary-object storms),
  (4) **post-cleanup delta** (leaks / cache / intern-table growth after the result
  is dropped + caches cleared). Plus the key derived metric
  **`TransientOverhead = peakRSS − retained`** (mid-evaluation bloat, distinct from
  representation overhead). Report `C_max`/`C_p95`/`C_geomean` + an outlier list,
  and check each `C` does NOT drift with input size N (drift ⇒ a complexity/order
  gap, not a constant). We already have the instruments to build on
  (`MaxMemoryUsed`, the `MemoryInUse` proxy, `guarded-test-run` RSS, the
  `alloc-count` GlobalAlloc) — this is mostly wiring + a richer report. This is the
  yardstick the rest of the memory line (Phase 2 interner, Phase 3 representation)
  gates against; the gensym leak = the post-cleanup `C`, the interner leak = the
  retained/RSS gap the proxy is blind to.
- [ ] Treat the next phase as `v0.1` kernel stabilization: correctness,
  contract boundaries, harnesses, and docs come before new same-name breadth.
  Use
  [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
  as the concrete acceptance boundary.
- [ ] Treat
  [docs/representation-rewrite-plan.md](./docs/representation-rewrite-plan.md)
  as the active performance mainline (supersedes
  `docs/performance-kernel-rewrite-plan.md`). The goal is WolframKernel parity or
  better on the supported `mma-hol` slice by lowering fundamental runtime
  constants, not by patching downstream `mma-hol`. **Phase 0 + Layer A (A1-A4)
  are done (2026-06-22).** Next: keep reducing the distributed per-call
  dispatch/eval constant (the unified `~34x` lever) on the
  `scripts/core-language-microbench.sh` proxy (`downvalue-exact-hit`/
  `pattern-hit`), gating each cut with the `RUM_PROFILE_REPR` counters + a clean
  back-to-back A/B + a bounded Pair re-measure. Prefer cuts that remove real
  per-call WORK (clones/walks), not just allocations (A3's pure alloc cut did not
  translate to Pair; A4's head-clone cut did). See the
  `docs/next-session-handoff.md` `2026-06-22` block.
- [ ] Promote new Wolfram-style functions only when the support matrix,
  `mma-hol`, or another concrete downstream blocker justifies them.
- [ ] Keep project-owned Python adapter work frozen until a real downstream
  need justifies reopening API, packaging, and binding decisions. **When the
  embedding line is reopened (after `mma-hol`), start from
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md)**
  — the surveyed capability boundary, the tiered host-hardening gaps (interrupt/
  cancel, session time/memory budget, panic-safety + the deep-nesting crash,
  structured `HostValue`, reset, session-image-from-bytes, concurrency contract),
  and the two durable showcase enablers (sandbox-by-design; WASM-friendliness). The
  specific showcase project is deliberately undecided.
- [ ] Keep Jupyter scoped as a `rum-host`-backed basic Notebook tech preview:
  useful for contract validation, not a full notebook product.

## Immediate Correctness And Debt

- [ ] **Deep-nesting stack-overflow — CONFIRMED real (probed 2026-06-30; Gemini's
  catch). FIX needed.** A deeply-nested literal `f[f[…f[x]…]]` of depth **~5000
  already crashes `rum-cli` with SIGABRT (exit 134, native stack overflow)** — far
  below any pathological depth. Confirmed it is NOT (only) deep EVAL: the `Hold[…]`
  path (eval blocked) crashes too, so the unguarded recursers are the
  **recursive-descent PARSER** and/or the **default recursive `Drop`** of the
  `Arc<Expr>` tree (rum-core has NO manual/iterative `Drop` impl for `Expr`). EVAL
  is already guarded (`recursion_limit` 1024, `evaluation_pass_limit` 256 → graceful
  `TerminatedEvaluation["IterationLimit"]`, WK-conformant) — but PARSE / CONSTRUCT /
  DROP have NO depth guard. WK gives a graceful `$RecursionLimit`/`$IterationLimit`
  here; rum aborts the whole process. NOT currently blocking (the HOL tower loads;
  real terms aren't ~5000 deep; eval-built structures are bounded at 1024) — a
  robustness gap, not a live blocker. FIX is bigger than "iterative Drop" alone:
  (a) a PARSE-depth guard that errors gracefully well below the native-stack limit
  (mirror the eval `recursion_limit` pattern); and/or (b) an iterative/manual `Drop`
  for `Expr`/`SharedExpr` (explicit work-stack teardown, respecting `Arc` refcounts
  — only unique nodes free). Gate any fix on no default-path cost. Side quest off
  the memory line; brief separately.
- [ ] Keep the suspension-safe helper boundary from the 2026-06-13 audit from
  regressing. The public callback-loop tranches are now closed for
  fold/window, functional iteration, targeted mapping (`MapAt`, `SubsetMap`),
  tensor/string callbacks (`ArrayReduce`, `Inner`, `ArrayFilter`,
  `StringApply`, delayed `StringReplace` / `StringCases` rules), and selected
  debug/tooling flows (`RepeatedTiming`, `Reap` handler rendering, `Trace`,
  `TestReport`, and resource-constraint limit/fallback expressions).
- [ ] Continue moving those helper payloads onto typed continuation values
  when the flow cannot be represented as `Result<Expr, EvalError>` plus a
  single ordinary suspension frame. Keep Monitor display/callback refresh and
  `ValueQ` rollback behavior intentionally best-effort/non-resumable unless
  their public contract is promoted. Review `Scan` remaining-part state and
  exotic `Sow` pattern-test suspension only if a concrete downstream workflow
  promotes those edges.

## Architecture And Contracts

- [ ] Keep the concrete `v0.1` public-contract boundary in
  [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
  aligned as host-facing types, downstream gates, or adapter commitments
  change.
- [ ] Keep the release/check gate contract in
  [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
  aligned with default local checks, optional Wolfram-oracle lanes, optional
  performance lanes, and stabilization-cut requirements.
- [ ] Keep `rum-host` as the Rust embedding boundary for sessions, requests,
  configuration, structured outcomes, failures, and suspend/resume.
- [ ] Keep `rum-jupyter` request handling on `rum-host` for eval, resume,
  inspect, complete, and is-complete; do not reintroduce direct broad
  `rum-runtime` coupling in the adapter surface.
- [ ] Treat any future adapters as peers to `rum-jupyter`, layered on the same
  host-facing runtime API rather than special cases inside `rum-runtime`.
- [ ] Keep source organization aligned with
  [docs/code-organization-guidelines.md](./docs/code-organization-guidelines.md);
  revisit warning-range files opportunistically when touching nearby code.
- [ ] Decide later whether the string subsystem is large and stable enough to
  justify a dedicated crate boundary; do not create a string crate by default.

## Core Runtime Semantics

- [ ] Tighten remaining evaluator behavior that is still ad hoc, especially
  held/spec-driven target families outside the message/definition/attribute
  normalization cluster.
- [ ] Extend matcher language, traversal, and tree-walker semantics only when
  a supported feature or downstream package needs the added breadth.
- [ ] Continue structure access/update work beyond the current integer,
  `All`, `Span`, keyed, and pattern-position subset when concrete parity gaps
  matter.
- [ ] Broaden source-language parsing for real Wolfram scripts, including
  Unicode identifier characters used as local names and operator glyphs that
  appear in package-level source or string data.
- [ ] Keep current string semantics documented and covered; defer deeper
  unicode-heavy, language-data, crypto-envelope, and nearest-neighbor parity
  until user-visible demand appears.
- [ ] Decide the practical mathematical surface after stabilization, favoring
  exact arithmetic, symbolic manipulation, and migration-relevant same-name
  behavior over broad CAS expansion.
- [ ] Keep the math IR layered so later work can distinguish polynomial
  normalization from rational-function normalization.
- [ ] Treat pattern-bearing indeterminates and full multivariate factorization
  over `Q` as later math work, not current kernel blockers.

## `mma-hol` Pressure Test

- [ ] Use `mma-hol` as the preferred near-term showcase and pressure test:
  audit the Wolfram-language surface it needs, then promote only concrete
  blockers into focused kernel tasks.
- [ ] Frame the long-running downstream target as an `mma-hol`-to-`rum-hol`
  migration path: cold proof-script correctness, stable snapshot development
  loop, and a basic notebook theorem demo are the three acceptance surfaces.
- [ ] Keep `mma-hol/tests/run_all.wls` as the first full real-project smoke
  lane. The clean-HEAD cold source smoke now gets through the core package and
  harness load path. The first bounded probe has identified two immediate
  gates before this can become routine: `PropTaut` proof-construction
  performance, and `Pair.wl` proof-load performance after the low-limit
  recursion failure fix. Use
  [`scripts/mma-hol-bounded-smoke.sh`](./scripts/mma-hol-bounded-smoke.sh) for
  bounded diagnostic load/test probes; it is a triage harness, not a timing
  benchmark. The driver now supports `--only LABEL` and preloads the package
  prefix needed by each selected test stage, so `test/proptaut_tests` reports
  the intended performance timeout instead of `Needs[...]` package-lookup
  noise.
- [ ] Use the updated `CL-Y` audit in
  [docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md)
  as the source of truth for `mma-hol` blockers. The current latest-HEAD cold
  probe now gets through the core, Bool, Drule, Tactics, Printer, and Parser
  test layers on a clean clone without source normalizations. The promoted
  parser/evaluator compatibility set exposed by `mma-hol` HEAD `698e3d0` has
  been folded back into `rum`: pattern `|` and `_?` syntax sugar, assignment
  RHS `& /@` precedence, `Table` substitution into held `Rule` lhs positions,
  definition lhs own-value evaluation for non-pattern constants, the
  state-progressing recursive parser cycle-guard case, and ordinary-expression
  pattern evaluation for `MemberQ` / `MatchQ` style calls. `Catch` tag-form
  evaluation has also been fixed: symbol-valued tag forms now evaluate before
  matcher compilation, closing the 9 `meson_tests` semantic failures recorded
  in the 2026-06-12 handoff.
  After the 2026-06-12 representation optimization pass (see
  `PROGRESS.md`), `proptaut_tests.wl` is green for the first time (`238s`
  full suite) and the PropTaut distribution probe is about `13.0s` release
  versus the local WolframKernel internal `3.80s` (~3.4x; Bool-stack load
  `0.78s` vs `0.19s`, ~4.1x). After the `Catch` fix, bounded clean-clone
  `test/meson_tests` reports `List[44, 0]`; the current
  `test/meson_proveprop_tests` re-probe times out at `300s` without a stdout
  summary. The old Pair hard failure (`mkPairInjThm` consumed as a
  non-theorem) was traced to rum's too-low default recursion limit and the
  default has been raised to the local Wolfram baseline of `1024`.
  The remaining Pair performance gate must stay rum-side. The old scratch
  ``HOL`Auto`Simp`` head-indexed `buildBaseConv` experiment crossed the gate,
  but it changed the downstream standard and is not an acceptance path. The
  2026-06-15 probe-first diagnosis found and fixed a rum hygienic RHS
  substitution bug: pattern-bound values inserted into delayed RHS expressions
  were capturable by inner `Module` locals, unlike WolframKernel behavior for
  downvalues, `HoldAll` / `HoldFirst`, `RuleDelayed`, `With`, and explicit
  `Function`. The semantic regressions are now covered in `rum-runtime`, and
  the same stock Pair/Simp probe still proves the theorem without changing the
  target term. The performance wall remains rum-side. The 2026-06-19 runtime
  sharing cut retained an exact atomic `Catch` tag fast path and changed
  `normalize_match_expr` to preserve shared call trees when normalization is a
  no-op. On the unmodified clean clone, `basic-only-pair4` improved from about
  `5.68-6.36s` to `2.23s`, `pair4-stock-repeat8` from `60.76s` to `18.30s`,
  and the first `mkPairInj` `CONVRULE` root from about `564s` to `307s`.
  A follow-up 2026-06-19 lexical presence-cache cut added a per-substitution
  shared-call presence cache for lexical `arg_prefilter` absence scans. On the
  same clean clone, `pair4-stock-repeat8` is now `15.2-15.6s`, pair-depth
  `pair4` is about `1.93s`, profiled lexical substitution total time moved
  from `10.01s` to `6.77s`, and presence-scan node volume fell from
  `606.8M -> 75.1M` for single-binding scans and `178.3M -> 13.5M` for
  multi-binding scans. The root retest after this cut moved the first
  `mkPairInj` `CONVRULE` root to a `193.83s` closed Pair-body phase sum,
  dominated by `mkPairInj-first-simp-apply-CONVRULE` at `188.07s`
  (~97%); WolframKernel spends `37.24s` in the same child. The
  dependency-preloaded `Pair.wl` root now reports `root-get-stdlib-pair`
  `188.27s` in rum versus `37.66s` in WolframKernel, with
  `mkPairInjThm=True` and `pairInjThm=True`. Pair/Simp is still open at about
  a `5x` full-root gap, but these results validate conservative rum-side
  representation/sharing work as the current path. A 2026-06-20 follow-up
  kept the same small-cut discipline: a rule-RHS presence prefilter was
  measured and rejected as a negative optimization, while a resolved
  `MatchQ` `Blank`/`Blank[head]` fast path retained a small positive
  failure-tail cut (`pair4-stock-repeat8` `16.02s -> 15.69s`). Do not
  reintroduce downstream `Simp.wl` patches or unsafe text-fingerprint semantic
  caches. A 2026-06-20 runtime-path follow-up then isolated the
  `REPEATC`/`TRYCONV` control-stack probe without preheat noise: definition
  dispatch selection is only about `0.06s` on `~1.0-1.5s` labels, and
  nonlocal `Return` propagation is not the owner. The remaining rum-side
  runtime self rows are ordinary frame/preparation costs: `Module` body
  substitution, `call.stage.down_values`, and `call.stage.prepare_arguments`
  around thousands of failing ``HOL`Drule`ORELSEC`` / `tryConv` /
  `simpRuleConv` / `REWRCONV` calls. The first `prepare_arguments` split shows
  that cheap `Flat` / `Orderless` normalization is not the owner: normalization
  self is only milliseconds in the `N=80` control-stack probe, while recursive
  argument evaluation plus high-frequency preparation loop/control overhead
  owns the visible self rows. Follow-up evaluator splits show that
  `prepare_arguments.evaluate_arg` itself is no longer a hot leaf. It mostly
  enters `evaluate_atom_fast_path.call`, then the ordinary call path. Stable
  symbol, non-symbol atom, dispatch-plan construction, and fixed-point equality
  are small, so that branch is now retained as side evidence rather than the
  Pair root frontier. The current root-linked Pair/Simp frontier is repeated
  no-change conversion failure over stable atomic leaves inside the first
  `mkPairInj` `CONVRULE` root. Current-worktree sibling probes show the
  context-length `1` comb `7361...0bfc` at about `64-67s`, with sibling const
  and var leaves at about `25-30s` each; direct `topRepeatc` samples on those
  leaves are about `21-24ms` per full failure scan through one context theorem,
  twenty basic rules, and beta. A temporary generated-copy
  `--prototype failure-cache` diagnostic cut the same first `CONVRULE` root
  `simpFixpointConv` from about `226-231s` to about `137.65s`, using `21`
  entries and `5283` hits. A broad opt-in generic call-result cache
  prototype was measured and rejected: it made
  `runtime-profile-var-basic-baseconv-chain` time out at `240s` versus the
  default-off `54s` baseline because full evaluated-call key construction is
  the wrong granularity during load/preheat. A narrower held-`Catch`
  failure-cache prototype was also measured and rejected before commit: the
  safe version had `0/40` hits on the tiny base-conversion chain because dirty
  invalidation rejected every possible reuse, and even the unsafe ignore-dirty
  upper bound slowed `pair4-stock-repeat8` (`16.94s` default-off versus
  `18.17s`). The temporary `auto/Simp.wl`
  prototype is only a diagnostic ceiling and must not become the acceptance
  path. Continue from the repo-level probe assets under
  `scripts/perf-probes/pair-simp/` and the root repetition probe; split only
  closed parent phases or implement small sharing cuts whose before/after can
  be checked on `basic-only-pair4`, `pair4-stock-repeat8`, and the first
  `CONVRULE` root. A post-revert runtime-path probe again excludes evaluator
  glue as the target: `prepare_arguments.evaluate_arg`, dispatch-plan
  construction, and normalization are millisecond-scale self rows inside a
  101s profiled conversion-control probe. A WolframKernel-side wrapper probe
  then changed the immediate inference: WK does not skip the failed rewrite
  chain at the WL level. It still performs about `19` failed `REWRCONV` /
  `simpRuleConv` calls per depth-19 base-conversion attempt, and
  `pair4-stock-repeat8` still shows thousands of failed conversions. Its
  advantage is a much lower per-failure constant. Therefore first copy the WK
  shape by making rum's existing failure path cheap: downvalue/definition
  dispatch, pattern dispatch, structural term operations, `Catch`/`Throw`,
  curried call application, and rule-chain evaluation. A narrow
  failed-conversion/no-change cache may still be tested as a rum-specific
  shortcut, but do not treat it as the inferred WK mechanism. The current
  closed `CONVRULE` descent names two retained
  hot leaves rather than one opaque primitive: `x/f/f-descendOnce` under the
  outer `x` sibling, and nested `abs/abs-descendOnce` under the
  `f/x/f/f/f/f/a/b` left-chain path. Follow-up root-linked splits keep the
  main multiplier on repeated conversion failure rather than Abs opening:
  `x/f/f/b` names `x/f/f/f-descendOnce` at `35.71s` plus a `16.60s` local
  loop; `x/f/f/f/b` reaches an `Abs` at `x/f/f/f/f` with `16.39s`
  `descendOnce` and an `18.08s` loop; `x/f/f/f/f/a/b` shows
  `freesIn`/fresh-name/`BETA-open`/`ABS` are sub-millisecond to
  millisecond-scale while the opened nested body still has
  `abs/abs-descendOnce` about `3.99s` and enclosing loops of about `4.03s`,
  `8.10s`, `16.13s`, and `17.93s`. Stage profiles at both the larger
  `x/f/f/f-descendOnce` leaf and the smaller
  `x/f/f/f/f/a/b` `abs/abs-descendOnce` leaf confirm the same rum-side
  amplifier: thousands of failing `ORELSEC` / `REWRCONV` /
  `simpRuleConv` conversion calls, millions of call-stability misses dominated
  by dirty/generation invalidation with no pointer misses, and rule-RHS
  call-argument rebuild traffic concentrated in Drule conversion combinators.
  A full shape/rebuild profile for the larger leaf timed out at `900s`, so
  broad shape accounting is too disturbing there; use narrower probes or split
  smaller first. The earlier runtime baseline showed the finishing
  opened-body target already performs about `973k` evaluator entries, `70k`
  down-value applications, `20.6k` `Module` entries, and `8.8M`
  call-stability checks with no rewrite. Source-level timing put the
  largest exclusive chunks in `Module` wrapper/body-substitution work under
  Drule conversion combinators such as ``HOL`Drule`ORELSEC``,
  ``HOL`Drule`Private`tryConv``, ``HOL`Drule`REPEATC``, and
  ``HOL`Drule`TRYCONV``. The follow-up source/context probe now shows those
  module body substitutions are essentially lexical `arg_prefilter` absent
  scans over large Drule conversion trees, especially
  ``HOL`Drule`ORELSEC`` and ``HOL`Drule`Private`tryConv``. The latest
  source-target/fingerprint split names the top owner as
  ``ORELSEC``/``tryConv`` scanning large text-equivalent conversion trees for
  ``HOL`Drule`Private`res``; exact fingerprints churn with fresh identities
  (`max exact scans` about `2`). The follow-up split explains both open
  questions: generation bumps are not source-less after target-family
  aggregation, but come from concrete fresh-local dirty-bloom saturation sites
  such as ``HOL`Kernel`Private`tyMatch`` / `k`,
  ``HOL`Drule`ORELSEC`` / `res`, ``HOL`Drule`Private`tryConv`` / `res`,
  ``HOL`Drule`REWRCONV`` / `c`, and
  ``HOL`Auto`Simp`Private`simpRuleConv`` / `c`; Drule conversion trees get
  fresh exact identities because rule RHS application alpha-renames scoped
  locals with `Symbol::with_identity` before rebuilding large
  `Module`/`CompoundExpression`/`If`/`Catch`/`ORELSEC` closures. The
  call-stability dirty/generation miss stream is therefore a separate
  invalidation amplifier: dirty writes are led by
  ``HOL`Drule`Private`collectAppSpine``, ``HOL`Drule`REWRCONV``,
  ``HOL`Drule`Private`tryTermMatch``, and
  ``HOL`Auto`Simp`Private`productiveSimpRule``. Treat these as retained
  diagnostic constraints for future rum-side optimization. The WL-hygiene
  boundary is now covered by
  `rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values` and
  `delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals`.
  The provenance split is complete for the finishing opened-body target: the
  dominant ``ORELSEC``/``tryConv`` `res` absence-scan node volume is over
  rule-RHS rebuilt parents whose RHS alpha ids are older than the exact fresh
  target local. But timed presence scanning itself is under a second in that
  probe, so the next Pair/Simp work stayed diagnostic. Rule-RHS
  substitution/rebuild timing has now also been measured and rejected as the
  100s-class owner on the already-preheated opened-body target: the largest
  single RHS timing bucket was only about `0.42s`, while the final post-preheat
  target was about `7.7s`. For that opened-body side target, the relevant
  upstream wall was Simp preheat: `basicSimpset[]` and the three lazy
  congruence lemmas run through
  `PropTaut` theorem construction. The new preheat profile probes show
  `preheat-basic` around `40-47s` under profile and congruence preheat around
  `118-133s` under profile, with each congruence lemma about `39-40s`.
  DownValue-index timing names the hot branches as
  ``HOL`Kernel`Private`tyMatch`` `tyVar`/`tyApp` branches and abs-recursive
  ``HOL`Terms`stripOrigin`` / `typeOf` / `vsubstImpl`; builtin self timing then
  puts the next runtime layer in `Fold` and `Module` under `tyMatch`, plus
  `Set`/`CompoundExpression` costs in theorem plumbing. `Part` and
  `KeyExistsQ` are not major self-time owners in the latest basic-preheat run.
  Do not let these side probes replace the current Pair root. The smaller
  single-lemma and single-function probes under `scripts/perf-probes/pair-simp/`
  are retained side evidence only: `imp`/`and`/`or` congruence lemmas are same-shaped
  `PropTaut` runs; no-profile `impCongLemma[]` is `22.54s` in rum versus
  `7.08s` in WolframKernel; bare ``HOL`Bool`EXCLUDEDMIDDLE[p]`` is `0.969s`
  in rum versus `0.322s` in WolframKernel. These numbers do not explain the
  hundreds-of-seconds Pair gate by themselves. The next Pair/Simp task is to
  rebuild a closed no-profile phase tree rooted at the unmodified `Pair.wl`
  load or the warm public mkPair probe: dependency preload, preheat, public
  `simpConv`/`CONVRULE` construction and application, theorem plumbing, and
  post-conversion package work must sum close to the parent wall time. Only the
  largest closed child segment may be split further. Record the root target,
  parent segment, child percentage, profiling overhead ratio, and log paths for
  every descent.
  The stable snapshot lane also
  has an experimental closure path:
  loading through Pair in Stable mode is bounded, but raw full-context
  `DumpSave` captures generated `Module` locals in private contexts
  (`baseConv$...`, `xEq$...`) and can inflate `HOL`Auto`Simp`Private`` to
  hundreds of MB. Explicitly enumerating and removing `Names["HOL`*$*"]`
  before dump, then restoring intentional globals/lazy-cache state, produced
  a through-Pair snapshot in `114.3s` (`500KB`) and restored `pair_tests.wl`
  green (`42/0`). Turn this into a careful snapshot-tooling patch before
  declaring the Pair snapshot lane closed.
  After the Pair optimization boundary is decided, rerun the later `Sum`,
  `Option`, `Set`, `Num`, and Real package probes because their current
  statuses are polluted by the failed Pair dependency.
  The targeted `Simp` base-conversion memo experiment did not close the gate,
  and the retained runtime stability cuts now put `basicSimpset[]` at about
  `20.27s` and user-rule-only mkPair simplification at about `40.61s`.
  Warm public mkPair simplification still times out at `240s` after those
  cuts.
  The follow-up stability-check amplifier diagnosis is now complete (see
  `PROGRESS.md`): 16x call-cache capacity does not move the warm probe, and
  disabling dirty-bloom saturation bumps is much worse. Miss-reason counters
  point mostly at absent pointer-keyed call entries, so the remaining Pair work
  should be treated as either a real `Simp` algorithmic change in the
  downstream package or the larger runtime representation / argument-preparation
  lane. Avoid more call-cache knob tuning unless a probe identifies a new hot
  construct.
  Performance status, retained work, and rejected probes are recorded in
  `PROGRESS.md` under "2026-06-13 Pair/Simp Stability-Check Amplifier
  Diagnosis", "2026-06-12 Representation Optimization Pass",
  "2026-06-12 Same-Bit Stability Invalidation Follow-Up", and
  "2026-06-12 Block Scope Invalidation Fix", and in
  `docs/proptaut-performance-audit.md`; resume from those, not from this
  bullet. The first Task 3a substitution-sharing audit confirms that rule RHS
  instantiation already preserves shared call subtrees for repeated bound
  values, unchanged sibling arguments, and `Sequence` splices, but rebuilt RHS
  parents still have fresh top-level pointers. The Task 3b opt-in
  shared-subtree stamp prototype was built and measured, then reverted before
  commit after external review showed a default-path PropTaut regression
  (`~10.2s` committed HEAD versus `~11.6s` with the uncommitted carrier). Its
  negative result is retained: profiled `240s` warm public mkPair runs showed
  real `call_stability_stamp_*` activity but no gate movement, and no-profile
  `360s` runs with and without the env var both timed out. Follow-up stage
  probes show public `basicSimpset[]` still completes in the expected range
  (`24.40s` standalone, `22.68s` after mkPair setup; congruence preheat
  `0.0018s`). The first final-conversion split shows `buildBaseConv`
  (`0.0048s`) and top
  `REPEATC[baseConv]` (`0.079s`) are tiny, but corrected `descendOnce` on
  `concl[lhsAppXY]` times out at `240s`, and each recursive subcall on
  `targetTm[[1]]` and `targetTm[[2]]` also times out at `240s`. Task 2c's
  two-engine WL-level loop-count probe is complete for the user-rule-only
  mkPair shape: rum release and WolframKernel both report `3565`
  `simpFixpointConv` entries, `3573` while iterations, max `seen` size `3`,
  `3565` fixpoint breaks, and `0` cycle breaks. That shape is not looping in
  rum. The secondary split has now reduced the long-tail shape further:
  `userBasic` on `mkPair[x, y]` times out in rum at `180s` and `360s`, while
  WolframKernel completes the same shape in `16.522s` with the same `1511`
  entries / `1513` while iterations as user-only. `preheat` and
  `buildBaseConv` are small, top rewrite is `0.068s`, and the timeout is in
  `descendOnce`. The smallest named hot subshape is the `mkPair` constant after
  top rewrite: descending the rewritten RHS lambda is a no-op but costs
  `112.766s` in rum with `userBasic`, versus `23.027s` with user rule only and
  `10.687s` in WolframKernel with `userBasic`. The next performance step is to
  split the rewritten RHS lambda/body descent itself and identify whether a
  particular body subterm or uniform basic-rule failed dispatch owns the cost;
  after the 2026-06-15 hygienic substitution finding, do not continue this
  lower-level split until the minimal Module/RHS semantic mismatch is fixed and
  the stock probe is remeasured.
- [ ] Treat `mma-hol` stable-mode snapshot helpers as a required
  developer-workflow gate for `rum-hol` migration. First-pass `Contexts[]`,
  file timestamp/size/existence helpers, `AbsoluteTime`, `Round`, simple
  decimal timing literals, and recursive `FileNames[..., ..., Infinity]` are in
  place. `SelectFirst` now covers partial layer selection, `Get` propagates
  loaded-file `Print` output, and a tiny `DumpSave` / `Get` public/private
  context round-trip regression plus `$ContextPath` / `$Packages`
  session-field assignment are in place. Partial validation now works:
  `tests/build_snapshot.wls Parser` writes a Stable snapshot,
  `tests/run_fast.wls parser` exits `0` with visible parser logs, and
  `snapshot/extend-proptaut` re-dumps quietly through rum's current textual
  `DumpSave` restore path, which auto-registers restored public package
  contexts in `$Packages` and `$ContextPath`; this has been re-validated on a
  refreshed unmodified `mma-hol` clone at HEAD `51bbe9e`. Remaining work is
  full validation once the `Pair.wl` / `Simp` performance gate is cleared. Cold
  `run_all.wls` remains the semantic baseline; Strict mode is not a snapshot
  restore target; `rum` `DumpSave` is a textual snapshot format rather than
  Wolfram `.mx` compatibility. The current gap audit lives in
  [docs/mma-hol-smoke-and-snapshot.md](./docs/mma-hol-smoke-and-snapshot.md).
- [ ] Preserve the already-demonstrated `Module` closure pattern where
  exported definitions retain access to fresh private mutable state after the
  enclosing `Module` returns.
- [ ] Track the already-matrix-covered `Throw` / `Catch` / `Failure` /
  `FailureQ` / `Stack[]` path as part of the `mma-hol` smoke lane.
- [ ] Keep the private-context lint script below the cold and snapshot lanes.
  Text-only `Row`, script-time helpers, `Import[..., "Text"]`, `Reap`, and
  `Sow` now have first-pass support, but the lint script itself still needs a
  focused validation pass before it becomes an acceptance gate.

## Basic Notebook Tech Preview

- [ ] Keep the lightweight `rum-jupyter` dependency guard aligned so the
  adapter does not silently reintroduce direct `rum-runtime` / `rum-stdlib`
  request-handling dependencies.
- [ ] Preserve Jupyter-owned concerns in `rum-jupyter`: kernelspec/bootstrap,
  ZeroMQ/native transport, signing, wire codecs, message envelopes, IOPub
  lifecycle, and MIME bundles.
- [ ] Maintain the isolated `jupyter_home/` deploy path as the documented
  Notebook frontend route; other frontends and installation styles are
  user-managed unless explicitly promoted.
- [ ] Keep the minimal Notebook workflow covered by the real harness:
  `kernel-smoke` for `kernel_info`, execute, shutdown/restart, stdin resume,
  messages/prints, failures as Jupyter errors, inspect, complete, is-complete,
  history, and unsupported-message keepalive probes; `browser-smoke` for
  JupyterLab/headless-Chromium code-cell execution with `text/plain` results.
- [ ] Maintain graceful handling for common unsupported Jupyter messages:
  empty `comm_info_reply`, ignored comm open/msg/close, explicit unsupported
  `debug_reply`, and non-fatal ignoring for unknown frontend probes.
- [ ] Keep robust interrupt/cancel, widgets/comm, debug protocol, rich
  display/update/clear-output behavior, and broad frontend compatibility
  outside `v0.1` unless deliberately promoted.

## Harness And Conformance

- [ ] Maintain the page-driven core-language support matrix as the public
  definition of the kernel feature target.
- [ ] For every matrix function or mechanism, keep status conservative:
  implemented, partial, planned, excluded, extension, or deliberately
  divergent.
- [ ] Strengthen boundary and edge-case coverage for parser, lowering,
  evaluator, matcher, scoping, threading, loader, and message semantics.
- [ ] Add broader regression corpora and golden fixtures for syntax
  formatting, lowering, evaluation, runtime messages, and transport-facing
  APIs.
- [ ] Add differential-conformance fixtures for selected supported same-name
  behavior while keeping local Wolfram-kernel execution environment-gated.
- [ ] Keep `rum-difftest` split into explicit lanes instead of one monolithic
  diff bucket.
- [ ] Add optional long-lived Lane B oracle process reuse only after measured
  throughput shows the current single-process multi-batch transport is not
  enough.
- [ ] Capture checked Lane D `ci-debug` / `ci-release` baseline artifacts on
  intended runners before tightening elapsed thresholds beyond broad smoke
  guardrails.
- [ ] Expand `rum-stdlib` tests from installation smoke coverage toward
  builtin-registration, package-loading, and context-management behavior.
- [ ] Expand `rum-terminal` tests beyond the current small baseline so
  terminal rendering changes are protected by direct unit coverage.

## Syntax, CLI, And Developer UX

- [ ] Continue expanding syntax operator coverage only where the evaluator
  roadmap or real packages need it.
- [ ] Continue splitting formatter internals into smaller
  planner/builder/render/test modules as the syntax surface grows.
- [ ] Decide whether trace viewing belongs in the REPL and, if so, add a
  minimal command flow for it.
- [ ] Add snapshot-style regression coverage for help output, inspection
  flows, and runtime-message rendering.
- [ ] Add CLI regression coverage for future machine-readable output modes if
  and when they are introduced.
- [ ] Realign runtime test file names and contents so review-critical tests
  live in modules whose names match their behavior.

## Performance And Representation

- [ ] **Pattern-matching as TWO under-evaluated axes (external review 2026-06-30;
  measure before investing).** rum has optimized the PER-MATCH cost (compiled
  `MatcherNode`, `fast_match_compiled`, the shallow prefilter) but never evaluated
  the matching ALGORITHM on two distinct axes: (a) **dispatch strategy / average
  case** — `try_apply_down_values_from_symbol` is a LINEAR scan over a symbol's
  downvalues with a cheap per-rule prefilter, i.e. O(total rules), NOT a
  discrimination net / term index (O(plausible rules)); a win only where a symbol
  carries MANY downvalues and rum's NATIVE dispatch does the trying — NOT the HOL
  Simp `~34x` (that rule-loop is WL-orchestrated `ORELSEC` in `Simp.wl`, the
  rejected-fork wall). (b) **worst-case complexity / adversarial** — `Orderless` +
  `BlankSequence` matching is AC-matching, NP-hard in general; the axis that
  distinguishes implementations is the DEGRADATION CURVE on adversarial patterns
  (graceful pruning vs exponential blowup), not the common-case constant. HOL
  likely triggers neither, but both are real gaps for general WL conformance.
  MEASURE FIRST: (a) the per-symbol downvalue-count distribution across the corpus
  (long tail ⇒ indexing pays); (b) rum's backtracking behavior on a crafted
  `f[a___, b___]`-style adversarial `Orderless`+`BlankSequence` pattern (is there
  any pruning / backtrack bound, or naive enumeration?). Decide investment from the
  measurement, per the project's measure-before-invest discipline.
- [ ] Revisit `rum-core::Atom::Integer` only if performance measurements show
  that pure `BigInt` should become a small-int plus bigint hybrid.
- [ ] Revisit matcher compilation/storage strategy after the attribute-revision
  cached-definition matcher path has more downstream mileage, especially
  whether replacement rules and sequence/orderless backtracking need similar
  caching or representation support.
- [ ] Continue probe-first evaluator optimization around large HOL proof
  terms. Already in place after the 2026-06-12 pass: interned 32-byte
  symbols with dense pair ids, id-hashed definition/attribute/binding maps,
  dependency-aware stability-cache invalidation (per-symbol removal plus a
  128-bit dirty bloom; repeated same-bit rewrites no longer bump the whole
  call-stability generation; only stable verdicts cached), mimalloc as the
  `rum-cli` global allocator, a message-symbol existence index, and the
  earlier shared call-tree representation. A same-symbol rewrite follow-up
  retained the rewritten-dirty-bloom fix and rejected exact per-symbol and
  per-bloom-bit version snapshots because they regressed the real PropTaut
  probe to about `24-25s`. The measured next cuts are now the
  single-allocation call node (head+args in one `Arc`) enabling an inline
  per-node evaluated stamp, borrowed argument preparation, and only after
  new profiler evidence, a non-allocating exact dependency representation.
  Whole-expression exact caching was measured and rejected because owned-tree
  hashing/equality became a new cost; depth-gated cycle tracking was rejected
  because the evaluation stack also feeds trace depths and `Stack[]`.
- [ ] Revisit large request/error payload ergonomics only after measurement
  shows that current shapes are a real cost.
- [ ] **Binary snapshot serialization (deferred, LOW priority — worthwhile, not
  urgent).** Today rum's `DumpSave` is implemented as TEXT (shares
  `eval_persist_package_state` with `Save`; the real binary `.mx` is NOT
  implemented, `.mx` extension is misleading). The snapshot is verbose UTF-8
  WL-source (`Set[...]`), ~3.3x WK's binary (mma-hol: 5M vs 1.5M), gzips 17.6x.
  At HOL scale this is NEGLIGIBLE — measured `Get` restore ~0.21 s/MB, ~1.0s for
  the full 4.8M tower (it stores pre-computed defs, re-assign not re-prove), ~1%
  of a cold rebuild. So NOT worth doing for rum's current target. BUT it IS worth
  doing eventually for (a) WK fidelity (`DumpSave` should be binary) and (b)
  potential larger-scale applications: linear extrapolation to a 150G-class cache
  is ~9h text restore (super-linear / OOM in reality), where a binary
  deserialize-direct path (skips parse + eval + matcher recompile; lower peak
  memory) would be ~30-100x faster and a feasibility flip ("can't finish" ->
  "minutes"). Scope when picked up: a binary serialization for `Expr` + the
  definition/attribute tables + interning, and binary `DumpSave`/`Get` paths
  (distinct from `Save`/text). Estimate-only until a prototype is measured.
- [ ] **Symbolic `Plus`/`Times` like-term detection cost (deferred, LOW
  priority — from brief-043/044).** brief-043 added like-term/like-base
  combination to `combine_additive_args`/`combine_multiplicative_args` (WK-aligned
  `x^2 x^3 -> x^5`, `x-x -> 0`); brief-044 took the cheap win (no-merge fast path
  + direct `Times[coeff,...]` build instead of recursive `combine_multiplicative_args`),
  cutting symbolic `Plus` 2.7x->1.96x, `Times` 2.0x->1.75x vs pre-043. The
  REMAINING regression is the O(n^2) like-term DETECTION itself (per-term
  `split_*` + linear `Expr ==` compare on every symbolic `Plus`/`Times`): measured
  no-merge 6-term `Plus` `0.295s` (≈unchanged by 044 — detection is intrinsic to
  merging), 3-term `0.074s` (the 4x for 2x terms = the O(n^2)). Lowering it needs
  hash-bucketed grouping (`Expr`'s derived `Hash` is consistent with its manual
  structural `PartialEq`, so a HashMap is sound) or a cheap pre-filter — both
  algorithm-level with BOUNDED upside at small term counts. The real-number tower
  has small term counts and shows NO macro regression (extend time even slightly
  faster: combination cuts downstream term counts), so the micro cost is an
  accepted tradeoff, not a net loss. A separate, even smaller lever: thread
  `SharedArgs` (`Arc<Vec<Expr>>`) into the two `combine_*` (Layer A did this for
  dispatch, not the arithmetic handlers) so the no-merge/no-fold case reuses the
  input `Arc` (`Expr::call_shared`, zero Vec alloc) — but `Expr` clone is already
  a SHALLOW `Arc` clone (no deep copy), so this only saves a Vec alloc + a few
  atomic incs; pure in-place mutation is impossible (the `Arc<Vec>` is shared
  through substitution). Pick up only with fresh profiler evidence that symbolic
  algebra (not the HOL slice) is a real workload.

## Samples And Application Directions

- [ ] Prepare a small, polished sample set only after the core kernel and
  host-adapter surfaces are stable enough to demonstrate end-to-end value.
- [ ] Avoid project-owned speculative showcase packages for now; survey
  object/class packages only if `mma-hol` or another concrete downstream
  package makes that abstraction relevant.
- [ ] Keep the former Python AI policy and Rust Wasm prototype ideas as brief
  application directions only, not active project-owned examples.
- [ ] Defer reviewing any restored or regenerated `WLBuiltin/` inventory until
  a deliberate post-mainline builtin survey is useful; see
  [docs/wolfram-builtin-inventory.md](./docs/wolfram-builtin-inventory.md).
