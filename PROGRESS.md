# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

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
