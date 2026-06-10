# TODO

This file is for active or near-term work only.

Completed history, archived planning detail, and operational lessons live in
[PROGRESS.md](./PROGRESS.md). Current support status belongs in
[docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md).

## Current Cut

- [ ] Treat the next phase as `v0.1` kernel stabilization: correctness,
  contract boundaries, harnesses, and docs come before new same-name breadth.
- [ ] Promote new Wolfram-style functions only when the support matrix,
  `mma-hol`, or another concrete downstream blocker justifies them.
- [ ] Keep project-owned Python adapter work frozen until a real downstream
  need justifies reopening API, packaging, and binding decisions.
- [ ] Keep Jupyter scoped as a `rum-host`-backed basic Notebook tech preview:
  useful for contract validation, not a full notebook product.

## Immediate Correctness And Debt

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

- [ ] Write down the concrete `v0.1` public-contract boundary: stable
  host-facing types and flows versus evaluator internals that remain unstable.
- [ ] Define the release/check gate contract: default local checks, optional
  Wolfram-oracle lanes, optional performance lanes, and what must pass before
  a stabilization cut.
- [ ] Keep `rum-host` as the Rust embedding boundary for sessions, requests,
  configuration, structured outcomes, failures, and suspend/resume.
- [ ] Migrate `rum-jupyter` to consume `rum-host` for eval, resume, inspect,
  complete, and is-complete before treating basic Notebook behavior as a
  `v0.1` delivery surface.
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
  harness load path, but the full script still needs a bounded driver and
  failure triage before it can become a routine check.
- [ ] Use the updated `CL-Y` audit in
  [docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md)
  as the source of truth for `mma-hol` blockers. The next cold-lane work is to
  push past the now-green `types_tests.wl`, `terms_tests.wl`,
  `kernel_tests.wl`, `bootstrap_tests.wl`, and `equal_tests.wl` core layer.
  The current concrete blocker is Bool top-level proof performance:
  dependency-preloaded `Bool.wl` now completes in about 2.45-2.51s warmed
  external default-release wall time, with internal
  `AbsoluteTiming[Get["Bool.wl"]]` around 2.30-2.36s. A local
  `target-cpu=native` release build measured about 2.29-2.35s warmed external
  wall time, but is not a portable default profile. The first evaluator audit
  quick wins, symbol-level evaluation epoch cache, shared call-tree
  representation, call-stability cache, upvalue fast path,
  stable-expression early return, rule-RHS binding-substitution copy-on-write,
  and lexical-substitution copy-on-write are in place. Low-yield probes already
  rejected include symbol fingerprint hashing, shallow definition matcher
  metadata, codegen-units=1, and thin LTO; continue with fresh profiling before
  deeper low-level work.
- [ ] Treat `mma-hol` stable-mode snapshot helpers (`DumpSave`, `Contexts[]`,
  `$ContextPath` / `$Packages`, file timestamps, and timing reports) as a
  required developer-workflow gate for `rum-hol` migration. Cold `run_all.wls`
  remains the semantic baseline; Strict mode is not a snapshot restore target;
  `rum` `DumpSave` is a textual snapshot format rather than Wolfram `.mx`
  compatibility.
- [ ] Preserve the already-demonstrated `Module` closure pattern where
  exported definitions retain access to fresh private mutable state after the
  enclosing `Module` returns.
- [ ] Track the already-matrix-covered `Throw` / `Catch` / `Failure` /
  `FailureQ` / `Stack[]` path as part of the `mma-hol` smoke lane.
- [ ] Keep small glue helpers such as text-only `Row`, script-time helpers,
  and lint-only `Import[..., "Text"]` on the compatibility radar, but prefer
  sample-code cleanup when a dependency is accidental rather than important.

## Basic Notebook Tech Preview

- [ ] Make `rum-jupyter` depend on `rum-host` instead of broad `rum-runtime`
  for request handling and public result, failure, input, inspect, completion,
  and completeness shapes.
- [ ] Preserve Jupyter-owned concerns in `rum-jupyter`: kernelspec/bootstrap,
  ZeroMQ/native transport, signing, wire codecs, message envelopes, IOPub
  lifecycle, and MIME bundles.
- [ ] Provide one documented deploy path for a real Notebook frontend; other
  frontends and installation styles are user-managed unless explicitly
  promoted.
- [ ] Support the minimal Notebook workflow: code cell execution,
  `text/plain` results, messages/prints, failures as Jupyter errors, inspect,
  complete, is-complete, stdin resume, `kernel_info`, and shutdown.
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
- [ ] Continue probe-first evaluator optimization around large HOL proof terms:
  symbol interning, node-level evaluated state, remaining clone accounting,
  repeated derived-rule workloads, and debug-vs-release smoke policy before
  adding broader low-level performance work. A symbol-level evaluation epoch
  cache and shared call-tree representation are already in place;
  whole-expression exact caching was measured and
  rejected because owned-tree hashing/equality became a new cost.
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
