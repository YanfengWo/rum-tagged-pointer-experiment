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

### Phase 7: Embedding and Host Adapters

- [x] Stabilize a host-facing request/config API around `Session` and keep terminal rendering out of it
- [x] Create an initial `rum-jupyter` crate with execute/inspect/complete/is-complete mapping, structured outputs, stdin handling, wire codecs, connection-file/bootstrap support, local socket backends, and native ZeroMQ-backed transport coverage
- [ ] Define and document a minimal stable host-facing public contract around sessions, requests, configuration, structured outcomes, and suspend/resume while keeping evaluator internals and experimental abstractions out of the public API
- [ ] Add a first-class Rust host adapter or embedding crate with a narrow public surface shaped for long-running backend, rule-engine, and DSL-service embedding rather than broad internal exposure
- [ ] Add an optional Python host adapter with a narrow public surface shaped for AI- and data-tooling-oriented integration, likely using `pyo3` behind a thin project-owned boundary instead of leaking Python-binding concerns into `rum-runtime`
- [ ] Continue hardening `rum-jupyter` as one optional capability-validation adapter for the kernel, prioritizing features that prove or exercise kernel maturity over notebook-only polish

### Phase 8: String Language and Text Processing

- [ ] Build toward full Wolfram core-language string support for business, industrial, and DLS-oriented workloads
- [ ] Decide when the string subsystem is large and stable enough to justify a dedicated crate boundary

### Phase 9: Verification, Coverage, and Conformance

- [ ] Build a deliberate, cross-cutting test strategy instead of relying only on feature-local regressions
- [ ] Expand the project toward a fuller test suite covering unit behavior, boundary conditions, semantic regressions, and host-facing conformance
- [ ] Build differential checking against Wolfram behavior into the verification strategy while keeping `wolframscript` execution environment-gated rather than mandatory for every developer or CI run

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

- [x] Tighten `Cases` evaluator semantics to use standard default levels (`{1}`), standard integer/negative/`Infinity` levelspec handling, explicit match limits, and depth-first leaves-before-roots traversal
- [x] Unify symbol/string raw target normalization across message, definition-inspection, attribute, and clear/tag helper families so held builtins stop hand-rolling the same parsing rules separately
- [x] Move `Cases` traversal normalization and collection state through typed continuation payloads instead of repeatedly round-tripping that helper state through ad hoc `Expr` shapes during suspensions
- [x] Extend matcher/traversal coverage beyond the earliest subset by giving `Cases` head-aware traversal control, exact/range/negative level support, and suspension-safe regression coverage for levels, limits, and options
- [x] Widen standard traversal semantics into first-class tree walkers by landing `Level` and `Position` on the shared standard-levelspec / head-traversal model
- [x] Expand that shared traversal foundation into the first follow-on predicate/count walkers by landing `Count`, `MemberQ`, and `FreeQ` on the same standard-levelspec / `Heads` / suspension-safe matcher model
- [x] Start the structure access/update branch on top of the shared traversal groundwork by landing first passes of `Part`, `Extract`, and `ReplacePart`, including exact paths, `All` / `Span` selectors, `[[...]]` / `;;` sugar with omitted-endpoint spans, `Position`-compatible paths, head access via `0`, more stable duplicate/overlap handling in `ReplacePart`, and delayed replacement resume coverage
- [x] Extend that structure branch through the next parity layer by fixing empty-position handling for `Extract` / `ReplacePart`, adding basic `ReplacePart[..., Heads -> True/False]` support, and preserving nested selector shape in `Part`
- [x] Extend generalized part-spec handling further by teaching `Part` / `Extract` / `ReplacePart` to honor explicit `0` endpoints inside `Span` selectors while keeping `Extract`'s documented list-of-lists ambiguity behavior aligned with Wolfram's `Position`-style interpretation
- [x] Teach `ReplacePart` to accept pattern-based position specs, including mixed exact/pattern spec lists, binding substitution into rhs expressions, explicit-`0` head matching under `Heads -> Automatic`, and suspension-safe resume coverage for both position matching and delayed rhs evaluation
- [ ] Continue widening that traversal model into future tree walkers beyond `Cases`, `Level`, and `Position` as those builtins land
- [ ] Continue the structure access/update family beyond the current integer / `All` / `Span` / pattern-position subset with additional Wolfram-compatible part specifications and any remaining edge-case parity work in `ReplacePart`
- [ ] Continue auditing any remaining held/spec-driven target families outside the message/definition/attribute cluster so new raw-form builtins reuse shared normalization helpers from the start
- [ ] Continue moving remaining host-facing inspection/debug helper payloads onto typed continuation values as more non-`Expr` flows become suspendable

### `rum-repl` and `rum-cli`

- [ ] Decide whether trace viewing belongs in the REPL and, if so, add a minimal command flow for it
- [ ] Add snapshot-style regression coverage for help output, inspection flows, and runtime-message rendering
- [ ] Add CLI regression coverage for future machine-readable output modes if and when they are introduced

### `rum-jupyter`

- [ ] Keep the kernel adapter optional and layered on top of host-facing runtime APIs rather than letting protocol details leak back into `rum-runtime`
- [ ] Continue hardening lifecycle and frontend-compatibility details beyond the current local-socket and native-ZeroMQ-backed coverage, but prioritize work that validates or exposes kernel capabilities over notebook-only surface polish
- [ ] Continue splitting oversized `rum-jupyter` transport/backend/test modules as part of the code-organization workstream

### Rust and Python Host Adapters

- [ ] Treat Rust and Python host support as peer adapters to `rum-jupyter`, all layered on the same host-facing runtime API rather than becoming special cases inside `rum-runtime`
- [ ] Use a Rust reference scenario of long-running backend services, rule engines, and DSL-oriented infrastructure to judge API ergonomics, but keep the public surface limited to kernel sessions, requests, configuration, structured outcomes, and resource-control boundaries
- [ ] Use a Python reference scenario of AI- and data-tooling-oriented applications to judge API ergonomics, but keep the public surface general-purpose and free of AI-specific kernel features
- [ ] Define which host-facing types and flows are stable public contract versus internal implementation detail so future adapters or wrappers can be added without freezing evaluator internals
- [ ] Decide how much of `Expr`, diagnostics, messages, suspended input, and structured outputs should be exposed directly versus wrapped in host-friendly facades, erring on the side of a smaller stable contract
- [ ] Provide enough extension-oriented examples and adapter seams that downstream contributors can build richer wrappers without turning `rum-runtime` into a kitchen-sink host framework
- [ ] Add adapter-focused regression and smoke coverage for parse, eval, inspect, complete, is-complete, session lifecycle, structured failures, and resumable input from Rust and Python callers
- [ ] Establish Python packaging and local-dev workflow only after the API shape is stable, keeping wheel/build concerns outside the kernel core
- [ ] Let `rum-jupyter` reuse shared host-adapter pieces later where that reduces duplication, but do not make notebook protocol concerns the center of the broader host design

### Cross-Cutting Test Suites and Conformance

- [ ] Define a project-wide testing pyramid that distinguishes:
  - [ ] direct unit tests for small helpers, planners, normalization logic, and matcher branches
  - [ ] subsystem regression tests for syntax, lowering, runtime, terminal, REPL, CLI, Rust/Python host adapters, and Jupyter
  - [ ] end-to-end behavior tests for user-facing flows and protocol-level interactions
- [ ] Strengthen boundary and edge-case coverage for parser, lowering, evaluator, matcher, scoping, threading, loader, and message semantics
- [ ] Add broader regression corpora and golden fixtures for syntax formatting, lowering, evaluation, runtime messages, and transport-facing APIs
- [ ] Establish explicit coverage goals for high-risk semantic areas instead of relying only on opportunistic test growth
- [ ] Add differential-conformance fixtures that compare selected `rum` results against Wolfram behavior for the supported subset, with same-name language features treated as compatibility targets by default
- [ ] Keep Wolfram-oracle execution environment-gated even though the differential test workstream itself is required:
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
- [ ] Keep string literal parsing in `rum-syntax`, the canonical string atom in `rum-core`, and the current first-wave evaluation semantics plus builtin registration in `rum-runtime`, revisiting any broader crate split only if the string surface later outgrows the core kernel boundary
- [ ] Do not create a dedicated string crate yet; revisit a future `rum-string` extraction only after the builtin surface, shared helpers, and performance constraints are concrete enough to justify a new boundary
- [ ] Audit the Wolfram core-language string surface and split it into explicit implementation tiers with documented priorities
- [x] Tier 0 correctness: lock the initial in-memory string model to Unicode code-point semantics and add basic escape/unicode regression fixtures, while leaving richer escape validation and diagnostics for later follow-up
- [x] Tier 1 basic construction and slicing first wave: `StringJoin`, `StringLength`, `Characters`, `CharacterRange`, `ToString`, `StringTake`, `StringDrop`, and `StringPart`
- [x] Extend Tier 1 with the remaining near-term construction builtin `StringRepeat`, keeping the current scope focused on in-memory string values rather than host-side file or encoding IO
- [x] Land the first Tier 2 exact-search wave: `StringContainsQ`, `StringStartsQ`, `StringEndsQ`, and `StringPosition` over exact string needles with Unicode code-point indexing and overlapping 1-based position ranges
- [x] Land the corresponding Tier 2 exact-transformation wave: `StringSplit`, `StringTrim`, `StringReplace`, and `StringCases` for whitespace/default handling plus exact-string delimiters and immediate exact-string rules over in-memory string values
- [ ] Widen the string search/transformation family from exact-string needles toward richer string-pattern and option semantics such as `StringExpression`, `IgnoreCase`, overlap controls beyond the current defaults, and delayed/string-pattern rule forms
- [x] Land an initial Tier 3 string-pattern subset by teaching the current string builtins to understand `StringExpression` and `Alternatives` over exact-string leaves
- [ ] Continue Tier 3 string patterns with repetition forms, start/end anchors, `Shortest` / `Longest`, `IgnoreCase`, delayed pattern rules, and `RegularExpression` interop
- [ ] Tier 4 conversions and interoperability: `CharacterCode`, `FromCharacterCode`, formatting/conversion touchpoints, file/path string conventions, and message-template compatibility
- [ ] Add golden tests that cover evaluator behavior, round-tripping, formatting, matching, and unicode-heavy cases across the implemented string subset
- [ ] Document supported string semantics, temporary limitations, and any deliberate divergence from Wolfram behavior as the subsystem grows

### Future Workstream: Practical Mathematical Surface

- [ ] After kernel stabilization and the initial string workstream, decide the practical mathematical surface needed for business, industrial, and DSL-oriented workloads
- [ ] Prioritize exact arithmetic, symbolic manipulation, and migration-relevant same-name Wolfram behavior before broader CAS-style expansion
- [x] Land the first exact arithmetic layer for integers and rationals, plus an initial `Expand` pass over rational-coefficient polynomial expressions with pattern-free indeterminates
- [x] Extend that rational-coefficient polynomial subset through `Coefficient` and `CoefficientList`, keeping them on top of the polynomial core rather than a broader rational-function layer
- [x] Land an initial `Factor` pass for rational-coefficient multivariate polynomials over the current pattern-free indeterminate model, including common-content extraction, exact linear-factor recursion, and migration-relevant identities such as `x^3 + y^3 + z^3 - 3 x y z`
- [x] Extend that same mathematical subset toward `Together`, including shared factor cancellation after common-denominator combination, without pulling the kernel into broader CAS-oriented scope
- [x] Land an initial symbolic `D` for exact arithmetic, rational functions, and abstract symbol-headed functions, using held pattern-free differentiation variables whose top-level head is a symbol and emitting Mathematica-style abstract `Derivative[...]` forms when no concrete rule is built in
- [ ] Keep the math IR layered so later work can distinguish polynomial normalization from rational-function normalization, rather than forcing negative powers or denominators directly into the polynomial core
- [ ] Treat the current exclusion of pattern-bearing indeterminates as a first-pass safety boundary for `Expand`, not as the long-term mathematical model; leave room to support Wolfram-style cases such as `(u[i_] + u[j_])^2`
- [ ] Defer full multivariate factorization over `Q` beyond the current linear-factor pass, including polynomial `GCD`, square-free decomposition, modular factorization, Hensel lifting, and recombination, until the broader kernel core priorities are further along

## External Review Intake (2026-03)

This section records the parts of the March 2026 external review round and the
immediate follow-up self-review that are worth absorbing into the roadmap after
comparing them against the current code tree. It is intentionally selective
rather than a verbatim import of every review comment.

### P0

- [x] Fix definition replacement semantics so re-defining the same `DownValue`, `UpValue`, or `SubValue` replaces the older rule instead of appending a stale earlier copy that still wins during application
- [x] Replace the current `rum-jupyter` wire-message signing with real Jupyter-compatible `HMAC-SHA256`, and add regression coverage for signed local/native transport paths
- [x] Make `rum-jupyter` honor the connection file signature scheme explicitly instead of treating any non-empty key as an implicit custom signing mode
- [x] Fix the handwritten `rum-jupyter` JSON parsers so request/response/connection-file strings round-trip real UTF-8 / Unicode data instead of decoding string contents byte-by-byte
- [x] Harden `rum-jupyter` multipart frame decoding so malformed ZeroMQ/Jupyter traffic returns structured wire errors instead of panicking the kernel

### P1

- [x] Add a more explicit conformance test set for definition precedence, tag ownership, and `HoldAllComplete` blocking boundaries
- [x] Add dedicated regression coverage for malformed `rum-jupyter` wire frames and non-ASCII JSON payloads so transport hardening stays protected
- [x] Add a dedicated regression suite for context resolution and package/context interaction instead of leaving those semantics covered only indirectly
- [ ] Expand `rum-stdlib` tests from installation smoke coverage toward builtin-registration, package-loading, and context-management behavior
- [ ] Expand `rum-terminal` tests beyond the current small baseline so terminal rendering changes are protected by direct unit coverage
- [ ] Fix REPL history persistence so multiline inputs survive reload as single logical entries instead of being split into fake per-line history items
- [ ] Fix lexer unexpected-character spans so structured diagnostics report the actual offending byte range instead of zero-width placeholders
- [ ] Keep splitting the oversized `rum-jupyter` transport/backend/test files called out by the review round until they fall back into the normal code-organization workflow

### P2

- [ ] Realign runtime test file names and contents so review-critical tests live in the modules their names imply instead of drifting into misleading buckets
- [ ] Revisit matcher compilation/storage strategy after semantics stabilize, including whether definition insertion should cache more matcher state instead of recompiling on hot paths
- [ ] Profile evaluator hot paths such as fixed-point iteration, heads lookup, and expression ordering before deciding whether additional low-level performance work is justified
- [ ] Revisit ergonomics of large request/error payloads only after measurement shows that the current shapes are a real cost rather than a static-analysis curiosity
