# TODO

This file is for active or near-term work only.

Completed history, archived planning detail, and operational lessons live in
[PROGRESS.md](./PROGRESS.md). Current support status belongs in
[docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md).

## Current Cut

- [ ] Treat the next phase as `v0.1` kernel stabilization: correctness,
  contract boundaries, harnesses, and docs come before new same-name breadth.
  Use
  [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
  as the concrete acceptance boundary.
- [ ] Promote new Wolfram-style functions only when the support matrix,
  `mma-hol`, or another concrete downstream blocker justifies them.
- [ ] Keep project-owned Python adapter work frozen until a real downstream
  need justifies reopening API, packaging, and binding decisions.
- [ ] Keep Jupyter scoped as a `rum-host`-backed basic Notebook tech preview:
  useful for contract validation, not a full notebook product.

## Immediate Correctness And Debt

- [ ] Audit `Catch` tag-form side-effect scoping beyond the Meson fix.
  WolframKernel evaluates nontrivial tag forms for matching and preserves
  visible effects such as `Print`, but assignment effects inside that tag-form
  evaluation do not leak after the `Catch`. rum currently uses ordinary
  matcher-pattern resolution for this path, which fixes symbol-valued exact
  tags but does not model that rollback boundary.
- [ ] Fix lexer unexpected-character spans so structured diagnostics report
  the actual offending byte range instead of zero-width placeholders.
- [ ] Fix REPL history persistence so multiline inputs reload as single logical
  entries instead of fake per-line history entries.
- [ ] Split the regrown
  `crates/rum-runtime/src/runtime/tests/names_and_syntax_information.rs`
  coverage along real behavior boundaries.
- [ ] Continue auditing remaining non-`Expr` helper paths that need the same
  suspension-safe treatment as the main evaluator flows.
- [ ] Continue moving host-facing inspection/debug helper payloads onto typed
  continuation values when those flows become suspendable.

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
  summary. The next downstream gate is `stdlib/Pair.wl`:
  its load still exceeds a `600s` budget (the earlier `335s` observation was
  an interrupted run, not a completion), with the `PropTaut`-backed `Simp`
  schema path as the suspected dominant cost. The old Pair hard failure
  (`mkPairInjThm` consumed as a non-theorem) was traced to rum's too-low
  default recursion limit and the default has been raised to the local
  Wolfram baseline of `1024`. After `Pair` loads cleanly, rerun the later
  `Sum`, `Option`, `Set`, `Num`, and Real package probes because their
  current statuses are polluted by the failed Pair dependency.
  The ordered next steps for this lane are the Pair/Simp logical-work
  multiplier hunt before any further constant tuning, and only then the
  single-allocation call-node / evaluated-stamp rework if the counter evidence
  justifies it; the plan lives in
  [docs/next-session-handoff.md](./docs/next-session-handoff.md).
  Performance status, retained work, and rejected probes are recorded in
  `PROGRESS.md` under "2026-06-12 Representation Optimization Pass",
  "2026-06-12 Same-Bit Stability Invalidation Follow-Up", and
  "2026-06-12 Block Scope Invalidation Fix", and in
  `docs/proptaut-performance-audit.md`; resume from those, not from this
  bullet.
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
  `snapshot/extend-proptaut` re-dumps quietly on the fixed probe clone after
  the `$Packages` restore cleanup. Remaining work is to carry the
  `extend_snapshot.wls` `$Packages` cleanup into the live/upstream `mma-hol`
  tree before refreshing the clone, then full validation once the `Pair.wl` /
  `Simp` performance gate is cleared. Cold `run_all.wls` remains the semantic
  baseline; Strict mode is not a snapshot restore target; `rum` `DumpSave` is a
  textual snapshot format rather than Wolfram `.mx` compatibility. The current
  gap audit lives in
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
- [ ] Provide one documented deploy path for a real Notebook frontend; other
  frontends and installation styles are user-managed unless explicitly
  promoted.
- [ ] Validate the minimal Notebook workflow against a real Notebook frontend:
  code cell execution, `text/plain` results, messages/prints, failures as
  Jupyter errors, inspect, complete, is-complete, stdin resume, `kernel_info`,
  and shutdown.
- [ ] Handle common unsupported Jupyter messages gracefully rather than
  terminating the kernel process.
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
