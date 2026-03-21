# TODO

This roadmap intentionally uses two levels of detail:

- compact phase summaries for capability blocks that are already largely done
- detailed checklists only for work that is still open or likely to move soon

## Phase Summary

### Phase 0: Repository Foundation

- [x] Set up the Git-managed Rust workspace, baseline docs, and exact integer support
- [x] Keep the workspace compiling and passing tests as the project evolves
- [x] Establish source-organization guidelines and convert major `include!`-style roots into real module boundaries
- [ ] Keep source organization aligned with the strengthened code-organization rules: human-readable modules, maintainable file sizes, feature-oriented subfolders, incremental-compilation-friendly boundaries, and unit-test-friendly function design

### Phase 1: Syntax Frontend

- [x] Build a stable `rum-syntax` foundation with AST, lexer, parser, spans, source-aware diagnostics, preserved trivia, multiline-aware parsing, and a document-IR formatter
- [ ] Continue expanding operator coverage and keep splitting formatter internals as the syntax surface grows

### Phase 2: Core Lowering and IR

- [x] Lower the currently supported syntax into working core `Expr` forms used by the runtime
- [x] Formalize the lowering pipeline, canonical core-head inventory, validation boundary, and golden test coverage
- [ ] Revisit integer representation later if performance demands it; evaluate whether `rum-core::Atom::Integer` should stay pure `BigInt` or grow a `small-int + bigint` hybrid representation

### Phase 3: Definitions and Evaluation

- [x] Implement sessions, contexts, package loading, ownvalues/upvalues/downvalues/subvalues, hold-aware evaluation, shared normalization families, and core assignment semantics
- [x] Add first-class definition inspection plus shared raw-form normalization across message, definition, scoping, and loader families
- [ ] Tighten the remaining evaluator semantics where behavior is still somewhat ad hoc

### Phase 4: Rules and Pattern Matching

- [x] Implement matcher-IR-backed matching for the current early subset, including conditions, sequence blanks, `PatternTest`, `Alternatives`, `Optional`, `Except`, `Repeated`, `MatchQ`, `Cases`, `ReplaceAll`, and `ReplaceRepeated`
- [x] Extend matcher and rewrite flows so suspended input can resume without replaying whole outer rewrites
- [ ] Extend the matcher language and traversal semantics beyond the current early subset

### Phase 5: Scoping and Runtime Control

- [x] Implement `Hold*`, `Evaluate`, `Sequence`, `If`, `Which`, `Function`, `With`, `Module`, `Block`, tracing, resource limits, and the current attribute/threading subset
- [x] Add host-driven input through `InputString` plus real suspension/resume for request-oriented execution
- [ ] Continue auditing remaining non-`Expr` helper paths that still need the same suspension-safe treatment

### Phase 6: REPL and Developer Tooling

- [x] Ship a usable REPL baseline with multiline input, history, completion, `:parse`, `:format`, `:lower`, `:inspect`, `:messages`, and shared terminal rendering
- [x] Add CLI parse/format entrypoints and formatter width overrides shared with REPL behavior
- [ ] Continue tightening regression coverage and decide whether trace viewing belongs in the REPL surface

### Phase 7: Embedding and Jupyter Preparation

- [x] Stabilize a host-facing request/config API around `Session` and keep terminal rendering out of it
- [x] Create an initial `rum-jupyter` crate with execute/inspect/complete/is-complete mapping, structured outputs, stdin handling, wire codecs, connection-file/bootstrap support, local socket backends, and native ZeroMQ-backed transport coverage
- [ ] Continue hardening `rum-jupyter` from an early adapter into a fuller Jupyter-facing kernel surface with more lifecycle/frontend compatibility polish

### Phase 8: String Language and Text Processing

- [ ] Build toward full Wolfram core-language string support for business, industrial, and DLS-oriented workloads
- [ ] Decide when the string subsystem is large and stable enough to justify a dedicated crate boundary

### Phase 9: Verification, Coverage, and Conformance

- [ ] Build a deliberate, cross-cutting test strategy instead of relying only on feature-local regressions
- [ ] Expand the project toward a fuller test suite covering unit behavior, boundary conditions, semantic regressions, and host-facing conformance
- [ ] Add optional differential checking against Wolfram behavior where local `wolframscript` is available

## Active Workstreams

### Code Organization and Testability

- [ ] Keep turning large branch-heavy functions into smaller helpers when that makes important policy or normalization behavior reachable from direct unit tests
- [ ] Keep separating pure planning/normalization logic from state mutation and rendering paths when that improves testability without creating micro-modules
- [ ] Split active files that have already grown well past the `10k` warning threshold, starting with:
  - [ ] `crates/rum-jupyter/src/transport.rs`
  - [ ] `crates/rum-jupyter/src/tests.rs`
  - [ ] `crates/rum-jupyter/src/zeromq.rs`
  - [ ] `crates/rum-jupyter/src/adapter.rs`
  - [ ] `crates/rum-syntax/src/formatter.rs`
  - [ ] `crates/rum-runtime/src/runtime/pattern_matching.rs`
  - [ ] `crates/rum-runtime/src/runtime/tests/session_requests.rs`

### `rum-syntax`

- [ ] Continue splitting formatter internals into smaller planner/builder/render/test modules as the syntax surface grows
- [ ] Audit the next operator set needed by the evaluator roadmap and prioritize it explicitly

### `rum-runtime`

- [ ] Continue tightening evaluator semantics where stage ordering or builtin behavior is still more ad hoc than intended
- [ ] Continue auditing any remaining held/spec-driven target families so raw-form builtins stop hand-rolling target parsing one by one
- [ ] Continue auditing remaining non-`Expr` inspection/debug helper paths that still need typed suspension-driver treatment
- [ ] Continue extending matcher and traversal semantics beyond the currently implemented early subset

### `rum-repl` and `rum-cli`

- [ ] Decide whether trace viewing belongs in the REPL and, if so, add a minimal command flow for it
- [ ] Add snapshot-style regression coverage for help output, inspection flows, and runtime-message rendering
- [ ] Add CLI regression coverage for future machine-readable output modes if and when they are introduced

### `rum-jupyter`

- [ ] Keep the kernel adapter optional and layered on top of host-facing runtime APIs rather than letting protocol details leak back into `rum-runtime`
- [ ] Continue hardening lifecycle and frontend-compatibility details beyond the current local-socket and native-ZeroMQ-backed coverage
- [ ] Continue splitting oversized `rum-jupyter` transport/backend/test modules as part of the code-organization workstream

### Cross-Cutting Test Suites and Conformance

- [ ] Define a project-wide testing pyramid that distinguishes:
  - [ ] direct unit tests for small helpers, planners, normalization logic, and matcher branches
  - [ ] subsystem regression tests for syntax, lowering, runtime, terminal, REPL, CLI, and Jupyter adapters
  - [ ] end-to-end behavior tests for user-facing flows and protocol-level interactions
- [ ] Strengthen boundary and edge-case coverage for parser, lowering, evaluator, matcher, scoping, threading, loader, and message semantics
- [ ] Add broader regression corpora and golden fixtures for syntax formatting, lowering, evaluation, runtime messages, and transport-facing APIs
- [ ] Establish explicit coverage goals for high-risk semantic areas instead of relying only on opportunistic test growth
- [ ] Add differential-conformance fixtures that compare selected `rum` results against Wolfram behavior for the supported subset
- [ ] Keep Wolfram-oracle checks optional and environment-gated:
  - [ ] prefer local `wolframscript` when available
  - [ ] do not make default development or CI flows depend on Wolfram tooling being installed
  - [ ] record any deliberate divergences from Wolfram behavior alongside those fixtures
- [ ] Prioritize oracle-style checks for the most semantics-sensitive areas first:
  - [ ] arithmetic normalization and exact-integer behavior
  - [ ] rules, patterns, replacement, and traversal behavior
  - [ ] scoping and held-evaluation constructs
  - [ ] messages, attributes, and definition precedence
  - [ ] string semantics once the string workstream begins landing

### Post-Core Workstream: Full String Language

- [ ] Treat full string support as a first-class post-foundation roadmap item rather than an incidental builtin grab bag
- [ ] Keep string literal parsing in `rum-syntax`, the canonical string atom in `rum-core`, evaluation semantics in `rum-runtime`, and builtin registration in `rum-stdlib` for the first implementation wave
- [ ] Do not create a dedicated string crate yet; revisit a future `rum-string` extraction only after the builtin surface, shared helpers, and performance constraints are concrete enough to justify a new boundary
- [ ] Audit the Wolfram core-language string surface and split it into explicit implementation tiers with documented priorities
- [ ] Tier 0 correctness: decide the Unicode/codepoint model and lock down escape/unicode regression fixtures
- [ ] Tier 1 basic construction and slicing: `StringJoin`, `StringLength`, `Characters`, `CharacterRange`, `StringRepeat`, `ToString`, `StringTake`, `StringDrop`, and `StringPart`
- [ ] Tier 2 search and transformation: `StringContainsQ`, `StringStartsQ`, `StringEndsQ`, `StringPosition`, `StringSplit`, `StringTrim`, `StringReplace`, and `StringCases`
- [ ] Tier 3 string patterns: `StringExpression`, alternatives, repetition forms, start/end anchors, `Shortest` / `Longest`, and `RegularExpression` interop
- [ ] Tier 4 conversions and interoperability: `CharacterCode`, `FromCharacterCode`, formatting/conversion touchpoints, file/path string conventions, and message-template compatibility
- [ ] Add golden tests that cover evaluator behavior, round-tripping, formatting, matching, and unicode-heavy cases across the implemented string subset
- [ ] Document supported string semantics, temporary limitations, and any deliberate divergence from Wolfram behavior as the subsystem grows
