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
- [ ] Resolve the 2026-05 source-size audit against `docs/code-organization-guidelines.md`: current Rust source count is 9 files over `35k` chars, 25 files in the `20k`-`35k` split-plan range, and 66 files in the `10k`-`20k` warning range
  - [ ] Split or document explicit exceptions for the `>35k` architecture-debt files: `crates/rum-runtime/src/runtime/debug_tools.rs` (`96,923` chars), `crates/rum-runtime/src/runtime/pattern_matching/sequence.rs` (`72,861`), `crates/rum-runtime/src/runtime/rewrite_builtins.rs` (`61,372`), `crates/rum-runtime/src/runtime/tests/associations.rs` (`55,351`), `crates/rum-runtime/src/runtime/options.rs` (`45,163`), `crates/rum-runtime/src/runtime/tests/holding_and_control_part7.rs` (`40,464`), `crates/rum-runtime/src/runtime/strings/text.rs` (`38,929`), `crates/rum-runtime/src/runtime/associations/transforms.rs` (`35,966`), and `crates/rum-runtime/src/runtime/namespace_tools.rs` (`35,671`)
  - [ ] Add split plans for the `20k`-`35k` files: `crates/rum-runtime/src/runtime/builtin_catalog.rs`, `crates/rum-runtime/src/runtime/tests/debug_tools.rs`, `crates/rum-runtime/src/runtime/predicates.rs`, `crates/rum-runtime/src/runtime/tests/definitions_and_rules_part3.rs`, `crates/rum-runtime/src/runtime/suspension.rs`, `crates/rum-runtime/src/runtime/tree_walkers.rs`, `crates/rum-runtime/src/runtime/associations/resumable.rs`, `crates/rum-runtime/src/runtime/control_flow/mod.rs`, `crates/rum-runtime/src/runtime/strings/patterns/matcher.rs`, `crates/rum-runtime/src/runtime/control_flow/confirmation.rs`, `crates/rum-runtime/src/runtime/structure_ops/replace.rs`, `crates/rum-jupyter/src/zeromq_native.rs`, `crates/rum-runtime/src/runtime/control_flow/branching_loops.rs`, `crates/rum-jupyter/src/transport/json/parse.rs`, `crates/rum-runtime/src/runtime/functional_ops/callable.rs`, `crates/rum-runtime/src/runtime/functional_ops/sequence.rs`, `crates/rum-jupyter/src/transport/codec.rs`, `crates/rum-jupyter/src/adapter/mod.rs`, `crates/rum-runtime/src/runtime/tests/session_requests/transforms_resume.rs`, `crates/rum-runtime/src/runtime/tests/session_requests/matcher_resume.rs`, `crates/rum-runtime/src/runtime/polynomials/core.rs`, `crates/rum-runtime/src/runtime/structure_ops/selectors.rs`, `crates/rum-runtime/src/runtime/strings/patterns/search.rs`, `crates/rum-runtime/src/runtime/definition_catalog.rs`, and `crates/rum-jupyter/src/tests/adapter.rs`
  - [ ] Revisit the 66 `10k`-`20k` warning-range files opportunistically when touching nearby subsystems, prioritizing active runtime files over stable tests

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
- [x] Document a first-pass `rum-host` design so Rust embedding can be implemented against an explicit crate boundary instead of growing ad hoc from `rum-runtime`
- [x] Document a first-pass Python adapter design so the eventual binding layer has an explicit `rum-host`-first boundary before implementation starts
- [ ] Define and document a minimal stable host-facing public contract around sessions, requests, configuration, structured outcomes, and suspend/resume while keeping evaluator internals and experimental abstractions out of the public API
- [ ] Add a first-class Rust host adapter or embedding crate with a narrow public surface shaped for long-running backend, rule-engine, and DSL-service embedding rather than broad internal exposure
- [ ] Add an optional Python host adapter with a narrow public surface shaped for AI- and data-tooling-oriented integration, likely using `pyo3` behind a thin project-owned boundary instead of leaking Python-binding concerns into `rum-runtime`
- [ ] Continue hardening `rum-jupyter` as one optional capability-validation adapter for the kernel, prioritizing features that prove or exercise kernel maturity over notebook-only polish

### Phase 8: String Language and Text Processing

- [x] Land a substantial `CL-07` string-language surface covering exact-string manipulation, first-pass string-pattern semantics, templating, conversions, text analysis, similarity/diff helpers, and current byte-array/hash/compress/encrypt/nearest/pluralization subsets
- [x] Close the current `CL-07` mainline with a substantially richer symbolic string-pattern subset, input-like `ToString` / `TextString` rendering, first-pass `Compress` / `Uncompress` / `Encrypt` / `Decrypt` round-tripping, broader `Nearest` dataset/result support, and a wider English `Pluralize` subset; treat any remaining language-data-backed pluralization, fuller crypto envelope compatibility, and unicode-heavy golden polish as non-blocking long-tail work
  - [x] extend the current string-pattern subset with character-class atoms such as `LetterCharacter`, `DigitCharacter`, `WordCharacter`, and `WhitespaceCharacter` so the existing string builtins gain broader shared matcher coverage
  - [x] extend the current string-pattern subset with `RegularExpression[...]` interop plus a first richer `ToString` form-argument subset and a current `Nearest` / `Pluralize` parity bump through `DistanceFunction -> EditDistance|HammingDistance` and common irregular English plurals
  - [x] extend the current string-pattern replacement/query subset with `RuleDelayed` handling, deepen the byte-array/crypto layer so `Hash` / `Compress` / `Encrypt` consume `ByteArray[...]` payloads rather than only textual proxies, and widen the current `Nearest` / `Pluralize` subset through radius-style list specs plus more common English irregular and word-ending forms
  - [x] extend the symbolic string-pattern subset with `_` / `__` / `___`, `Pattern[x, ...]` capture bindings, and current `Rule` / `RuleDelayed` substitution over string-pattern matches
  - [x] tighten the symbolic matcher and string-interoperability layer with repeated named-capture consistency in predicate/query paths, `Encrypt` support for `ByteArray[...]` keys, `Nearest[..., {spec, Infinity}]`, and broader case-preserving / irregular `Pluralize` coverage
  - [x] upgrade `ToString` so the default / `InputForm` / `OutputForm` / `StandardForm` path renders a current input-like textual subset for common arithmetic and structural expressions instead of always falling back to canonical `FullForm` text
  - [x] align `TextString` with the current input-like renderer and extend the formatting-symbol subset through `TraditionalForm`
  - [x] deepen the current input-like renderer through slots, one-argument pure functions, assignment / replacement syntax, `Pattern` / `Blank*` / `Condition` / `Alternatives`, improve generic-expression byte-array / crypto transport to use the same renderer, and surface `Nearest[..., Automatic]` plus a broader irregular-`Pluralize` subset
  - [x] extend the current input-like renderer with `Part` / `Map` / `Apply` / `MapApply` syntax, add first-pass `Decrypt`, let `Nearest` consume `Association[...]` datasets, and widen the current irregular-`Pluralize` subset again
  - [x] deepen the current formatting/interoperability layer with `PatternTest` / `Optional` renderer syntax, current `Decrypt` round-trips for richer parseable input-form payloads, singleton-rule `Nearest` datasets, and phrase-level `Pluralize` over the final English word in simple multiword strings
- [ ] Decide when the string subsystem is large and stable enough to justify a dedicated crate boundary

### Phase 9: Verification, Coverage, and Conformance

- [ ] Build a deliberate, cross-cutting test strategy instead of relying only on feature-local regressions
- [ ] Expand the project toward a fuller test suite covering unit behavior, boundary conditions, semantic regressions, and host-facing conformance
- [ ] Build differential checking against Wolfram behavior into the verification strategy while keeping local Wolfram-kernel execution environment-gated rather than mandatory for every developer or CI run

## Active Workstreams

### Code Organization and Testability

- [ ] Keep turning large branch-heavy functions into smaller helpers when that makes important policy or normalization behavior reachable from direct unit tests
- [ ] Keep separating pure planning/normalization logic from state mutation and rendering paths when that improves testability without creating micro-modules
- [x] Split the old `functional_ops` and `strings` implementation/test monoliths into feature-oriented directory modules:
  - [x] `crates/rum-runtime/src/runtime/functional_ops.rs` -> `crates/rum-runtime/src/runtime/functional_ops/`
  - [x] `crates/rum-runtime/src/runtime/tests/functional_ops.rs` -> `crates/rum-runtime/src/runtime/tests/functional_ops/`
  - [x] `crates/rum-runtime/src/runtime/strings.rs` -> `crates/rum-runtime/src/runtime/strings/`
  - [x] `crates/rum-runtime/src/runtime/tests/strings.rs` -> `crates/rum-runtime/src/runtime/tests/strings/`
- [x] Complete the first major large-file refactor wave across the most painful runtime, syntax, and Jupyter monoliths:
  - [x] `crates/rum-runtime/src/runtime/control_flow.rs` -> `crates/rum-runtime/src/runtime/control_flow/`
  - [x] `crates/rum-runtime/src/runtime/structure_ops.rs` -> `crates/rum-runtime/src/runtime/structure_ops/`
  - [x] `crates/rum-runtime/src/runtime/pattern_matching.rs` -> `crates/rum-runtime/src/runtime/pattern_matching/`
  - [x] `crates/rum-runtime/src/runtime/functional_ops/arrays.rs` -> `crates/rum-runtime/src/runtime/functional_ops/arrays/`
  - [x] `crates/rum-runtime/src/runtime/strings/patterns.rs` -> `crates/rum-runtime/src/runtime/strings/patterns/`
  - [x] `crates/rum-runtime/src/runtime/strings/similarity.rs` -> `crates/rum-runtime/src/runtime/strings/similarity/`
  - [x] `crates/rum-runtime/src/runtime/eval_dispatch.rs` -> `crates/rum-runtime/src/runtime/eval_dispatch/`
  - [x] `crates/rum-runtime/src/runtime/builtin_catalog.rs` partially sliced into feature-oriented submodules
  - [x] `crates/rum-runtime/src/runtime/polynomials.rs` -> `crates/rum-runtime/src/runtime/polynomials/`
  - [x] `crates/rum-runtime/src/runtime/associations.rs` -> `crates/rum-runtime/src/runtime/associations/`
  - [x] `crates/rum-runtime/src/runtime/tests/session_requests.rs` -> `crates/rum-runtime/src/runtime/tests/session_requests/`
  - [x] `crates/rum-runtime/src/runtime/tests/structure_ops.rs` -> `crates/rum-runtime/src/runtime/tests/structure_ops/`
  - [x] `crates/rum-jupyter/src/transport.rs` -> `crates/rum-jupyter/src/transport/`
  - [x] `crates/rum-jupyter/src/tests.rs` -> `crates/rum-jupyter/src/tests/`
  - [x] `crates/rum-jupyter/src/zeromq.rs` split into a smaller core plus `zeromq/tests/`
  - [x] `crates/rum-jupyter/src/zeromq/tests/native/frontend.rs` -> `crates/rum-jupyter/src/zeromq/tests/native/frontend/`
  - [x] `crates/rum-jupyter/src/adapter.rs` -> `crates/rum-jupyter/src/adapter/`
  - [x] `crates/rum-syntax/src/formatter.rs` -> `crates/rum-syntax/src/formatter/`
- [ ] Keep the remaining `20k+` files under active review and only continue the large-file workstream when one of these files is touched enough to justify the next conceptual split:
  - [ ] `crates/rum-runtime/src/runtime/options.rs` (`44003` bytes)
  - [ ] `crates/rum-runtime/src/runtime/rewrite_builtins.rs` (`38341` bytes)
  - [ ] `crates/rum-runtime/src/runtime/predicates.rs` (`25943` bytes)
  - [ ] `crates/rum-jupyter/src/zeromq_native.rs` (`25373` bytes)
  - [ ] `crates/rum-runtime/src/runtime/control_flow/branching_loops.rs` (`25237` bytes)
  - [ ] `crates/rum-runtime/src/runtime/control_flow/confirmation.rs` (`24919` bytes)
  - [ ] `crates/rum-runtime/src/runtime/suspension.rs` (`24874` bytes)
  - [ ] `crates/rum-runtime/src/runtime/builtin_catalog.rs` (`24421` bytes)
  - [ ] `crates/rum-jupyter/src/transport/json/parse.rs` (`24369` bytes)
  - [ ] `crates/rum-runtime/src/runtime/tree_walkers.rs` (`24253` bytes)
  - [ ] `crates/rum-runtime/src/runtime/functional_ops/sequence.rs` (`23386` bytes)
  - [ ] `crates/rum-jupyter/src/transport/codec.rs` (`23115` bytes)
  - [ ] `crates/rum-jupyter/src/adapter/mod.rs` (`23007` bytes)
  - [ ] `crates/rum-runtime/src/runtime/control_flow/mod.rs` (`22282` bytes)
  - [ ] `crates/rum-runtime/src/runtime/polynomials/core.rs` (`21551` bytes)
  - [ ] `docs/core-language-support-matrix.md` (`20475` bytes)
  - [ ] `crates/rum-jupyter/src/tests/adapter.rs` (`20336` bytes)

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
- [x] Start the structure access/update branch on top of the shared traversal groundwork by landing first passes of `Part`, `Extract`, and `ReplacePart`, including exact paths, `Part` / `Extract` `All` / `Span` selectors, `[[...]]` / `;;` sugar with omitted-endpoint spans, `Position`-compatible paths, head access via `0`, more stable duplicate/overlap handling in `ReplacePart`, and delayed replacement resume coverage
- [x] Extend that structure branch through the next parity layer by fixing empty-position handling for `Extract` / `ReplacePart`, adding basic `ReplacePart[..., Heads -> True/False]` support, and preserving nested selector shape in `Part`
- [x] Extend generalized part-spec handling further by teaching `Part` / `Extract` to honor explicit `0` endpoints inside `Span` selectors while keeping `Extract`'s documented list-of-lists ambiguity behavior aligned with Wolfram's `Position`-style interpretation
- [x] Teach `ReplacePart` to accept pattern-based position specs, including mixed exact/pattern spec lists, binding substitution into rhs expressions, explicit-`0` head matching under `Heads -> Automatic`, and suspension-safe resume coverage for both position matching and delayed rhs evaluation
- [x] Tighten part-spec parity by letting `Part` / `Extract` produce `Head[]` for atom `All` and empty positive-span selections, aligning `Span` endpoint boundary handling so `0` inside `;;` is not treated as head access, and keeping `ReplacePart` from treating atom heads as replaceable exact targets
- [x] Tighten `ReplacePart` position-spec parity by no longer treating `All` / `Span` as expandable exact position selectors while preserving integer, keyed, mixed exact/pattern, and explicit-zero head-target behavior
- [x] Tighten truth/comparison breadth by reducing exact-numeric order chains around symbolic gaps and short-circuiting known-false numeric neighbor pairs while keeping `Equal` / `Unequal` and string-chain cases conservative
- [x] Add the current unary comparison operator forms `LessThan` / `GreaterThan` / `LessEqualThan` / `GreaterEqualThan` / `EqualTo` / `UnequalTo` / `SameAs` as callable wrappers that apply a single argument on the left while keeping direct malformed forms inert
- [x] Align first-pass order-comparison parity by fixing `Order`'s sign convention, using exact numeric order for numeric ordering predicates, and adding current two-argument `LexicographicOrder` / `NumericalOrder` / `AlphabeticOrder` coverage
- [x] Extend `Part` / `Extract` span endpoint parity with the current `UpTo[n]` / `UpTo[Infinity]` subset while keeping direct `UpTo[...]` part specifications inert
- [x] Add the first `NearestTo` callable wrapper subset by routing `NearestTo[target][data]` and positive-count / radius-limited forms through the existing `Nearest[data, target, ...]` evaluator while keeping unsupported selection forms inert
- [x] Add first-pass algebraic expression predicates by covering current `PolynomialQ`, two-argument `PolynomialExpressionQ`, and two-argument `RationalExpressionQ` over exact variables / expression variables while leaving richer coefficient-domain forms inert
- [x] Add structural `DateObjectQ` / `TimeObjectQ` recognition for System `DateObject[...]` and `TimeObject[...]` wrappers without attempting date/time object normalization
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
- [x] Add a minimal `rum-jupyter-kernel` launcher plus temporary-kernelspec and one-shot smoke scripts so the adapter can be exercised against a real local Jupyter client without global installation
- [ ] Continue hardening lifecycle and frontend-compatibility details beyond the current local-socket and native-ZeroMQ-backed coverage, but prioritize work that validates or exposes kernel capabilities over notebook-only surface polish
- [ ] Continue splitting oversized `rum-jupyter` transport/backend/test modules as part of the code-organization workstream

### Rust and Python Host Adapters

- [x] Write a first-pass `rum-host` plan in [docs/host-adapter-plan.md](./docs/host-adapter-plan.md) that fixes the intended crate boundary, goals, non-goals, and Rust-first / Python-second rollout order
- [x] Write a first-pass Python adapter plan in [docs/python-adapter-plan.md](./docs/python-adapter-plan.md) that fixes the intended crate boundary, Python-facing API bias, and `rum-host` dependency rule
- [ ] Treat Rust and Python host support as peer adapters to `rum-jupyter`, all layered on the same host-facing runtime API rather than becoming special cases inside `rum-runtime`
- [ ] Use a Rust reference scenario of long-running backend services, rule engines, and DSL-oriented infrastructure to judge API ergonomics, but keep the public surface limited to kernel sessions, requests, configuration, structured outcomes, and resource-control boundaries
- [ ] Use a Python reference scenario of AI- and data-tooling-oriented applications to judge API ergonomics, but keep the public surface general-purpose and free of AI-specific kernel features
- [ ] Define which host-facing types and flows are stable public contract versus internal implementation detail so future adapters or wrappers can be added without freezing evaluator internals
- [ ] Decide how much of `Expr`, diagnostics, messages, suspended input, and structured outputs should be exposed directly versus wrapped in host-friendly facades, erring on the side of a smaller stable contract
- [ ] Create the `rum-host` crate as the primary Rust embedding boundary before starting project-owned Python bindings
- [ ] Keep the Python adapter downstream of `rum-host` even if early experiments make direct `rum-runtime` binding look tempting
- [ ] Provide enough extension-oriented examples and adapter seams that downstream contributors can build richer wrappers without turning `rum-runtime` into a kitchen-sink host framework
- [ ] Add adapter-focused regression and smoke coverage for parse, eval, inspect, complete, is-complete, session lifecycle, structured failures, and resumable input from Rust and Python callers
- [ ] Establish Python packaging and local-dev workflow only after the API shape is stable, keeping wheel/build concerns outside the kernel core
- [ ] Let `rum-jupyter` reuse shared host-adapter pieces later where that reduces duplication, but do not make notebook protocol concerns the center of the broader host design

### Samples and Showcase Packages

- [ ] Prepare a small, polished sample set once the core kernel and host-adapter surfaces are stable enough to demonstrate end-to-end value rather than isolated builtin checklists
- [ ] Add a Python-host sample that shows `rum` embedded from Python for an AI-adjacent workflow, such as structured symbolic preprocessing, rule-driven tool orchestration, or controllable prompt-side helper logic layered on top of the future `rum-python` adapter
- [ ] Add a Rust-host sample that shows `rum` embedded for system/ops or backend-style automation, such as policy-driven maintenance flows, config inspection/transformation, or rule-engine style control logic layered on top of the future `rum-host` adapter
- [ ] Explore a pure-Wolfram-language object/class showcase package as a sample of how much higher-level abstraction `rum` can support without new runtime mechanisms:
  - [ ] first survey existing Wolfram-language object/class implementations such as `Wolfram/Class`, `KirillBelov/Objects`, and older closure/message-passing packages from the Mathematica ecosystem
  - [ ] record the tradeoffs of reusing an existing package versus writing a project-owned sample package tuned to `rum`'s supported subset and showcase goals
  - [ ] prefer a representative OOP-style sample over a bare “class syntax demo”, so the result illustrates practical package-level encapsulation, method dispatch, and state management rather than only the mechanism itself
  - [ ] use a static-site-generator style package as the default class/object showcase candidate, keeping workflow/state-machine style samples as strong backups if that proves a better fit for the supported subset
  - [ ] keep this as a package/library sample first, not as a `CL-08` core-language feature, unless later experience shows a real missing runtime mechanism

### Cross-Cutting Test Suites and Conformance

- [ ] Maintain a page-driven core-language support matrix keyed to the current official Wolfram scope anchors in `Core Language & Structure` and `Language Overview`, and treat that matrix as the public-facing definition of the kernel feature target
- [ ] For every function and mechanism tracked in that matrix, record one of: implemented, partial, planned, excluded, extension, or deliberately divergent, and keep any divergence notes explicit enough to feed later user-facing documentation
- [ ] Use the matrix to drive page-by-page implementation sequencing through the `CL-*` group IDs defined there, while keeping clearly non-kernel areas linked from those pages out of scope unless they are explicitly adopted later

### Core-Language Matrix Groups

- [ ] `CL-01` Expression, truth, and structural foundation
- [ ] `CL-02` Definitions, attributes, assignments, and scoping
- [ ] `CL-03` Rules, patterns, and tree walkers
- [ ] `CL-04` Functional and list combinators
- [ ] `CL-05` Associations and keyed data
- [ ] `CL-06` Procedural control, exceptions, and failure flow
- [x] `CL-07` String language
- [x] `CL-08` Package, namespace, options, and protection tooling
  - [x] land first-pass `Protected` state on top of the shared attribute model, including `Protect` / `Unprotect` and protected-symbol write barriers across definition/message/clear paths
  - [x] land first-pass symbol-level `Options` / `SetOptions` / `OptionValue` storage and query semantics, including `Options[f] = {...}` and `Unset[Options[f]]`, before tackling `OptionsPattern`
  - [x] deepen option parity with current system default options and first-pass `OptionsPattern` / one-argument `OptionValue[...]` support inside matching definitions
  - [x] land first-pass `Names` and `SyntaxInformation`, including direct assignment / `Unset`, `Block` / loader snapshot participation, and current visible-context name matching
  - [x] extend the current package/tooling layer through broader system default option coverage for the existing `Heads` family, system default `SyntaxInformation` metadata, and first-pass `ArgumentsOptions` / `CheckArguments` / direct builtin `UpSet` support
  - [x] land first-pass package-development persistence and stub tooling through `DeclarePackage`, textual `Save` / `DumpSave` round-trips, and session-local `Once`
  - [x] deepen the current package/tooling subset with `Save` / `DumpSave` context-filter handling, `DumpSave` attribute-suppression handling, richer `Names[RegularExpression[...]]` coverage through alternation and bounded repetition, and broader package/context default `SyntaxInformation`
  - [x] deepen the current package/tooling subset with `Once` option-form parsing in second position, current `PersistenceTime -> 0` no-cache handling, `Save` unknown-option rejection, `Once` unknown-option reporting, non-capturing-group support in `Names[RegularExpression[...]]`, and broader `SyntaxInformation` defaults for optioned traversal/string/package helpers
  - [x] close the current CL-08 mainline with delayed option-rule support, richer current `SetOptions` / `OptionValue` forms, and a stable current package-tooling subset; treat any remaining deeper `Names` regex corners, broader `SyntaxInformation` properties, and fuller `DeclarePackage` / `Save` / `DumpSave` / `Once` parity as non-blocking long-tail polish rather than active mainline blockers
- [x] `CL-09` Tuning, debugging, and verification surface
  - [x] land a first CL-09 debugging wave through structured default `Trace[expr]`, print-stream `TracePrint`, exact-rational `Timing` / `AbsoluteTiming`, echo-style debugging helpers, and a current canonical-text-proxy `ByteCount` subset
  - [x] deepen the first CL-09 surface with fixed-sample `RepeatedTiming`, session-level `On` / `Off` message gating, tag-aware `Sow` / `Reap`, and a current canonical-text `Short` subset
  - [x] land a first resource/monitor/reporting wave through cooperative `TimeConstrained` / `MemoryConstrained` / `MemoryInUse`, monitor hooks with `EvaluationMonitor` / `StepMonitor` option symbols, and first-pass `EvaluationData` / `TestCreate` / `TestReport` / `Stack`
  - [x] deepen the current CL-09 subset with filtered `Trace` / `TracePrint` forms, more stable `Monitor` display behavior, `EvaluationData` property projection, and broader `TestCreate` / `TestReport` metadata through `TestID`, `SameTest`, and expected-message matching in the current first-pass subset
  - [x] deepen the current CL-09 subset with structural-proxy `ByteCount` / `MemoryInUse` semantics and list-pattern `Trace[..., {...}]` / `TracePrint[..., {...}]` filtering
  - [x] deepen the current CL-09 subset with current flat-trace levelspec forms, `EvaluationData` / `TestReport` property projection through `All` and `"Properties"`, default-option coverage for `Monitor`, and broader session-state accounting in the current `MemoryInUse[]` proxy
  - [x] deepen the current CL-09 subset with `Heads -> True|False` trace filtering in the flat-trace forms, stronger default-option coverage for `Monitor`, and broader message/request state accounting in the current `MemoryInUse[]` proxy
  - [x] deepen the current CL-09 subset with a structured default `Trace[expr]` tree model
  - [x] deepen the current CL-09 subset by carrying `Trace[..., patt]`, `Trace[..., levelspec]`, and `Trace[..., Heads -> ...]` onto the same structured tree model, and by aligning `EvaluationData["Trace"]` with that structured representation
  - [x] deepen the current CL-09 subset with indented structured `TracePrint[...]`, explicit-`None` callback overrides for `Monitor`, and broader `EvaluationData` / `TestReport` metadata through held `"Input"` plus report-level `"Success"`
  - [x] deepen the current CL-09 subset with depth-aware structured `Trace[...]` branching, more stable nested filtered-trace shells, matching nested `TracePrint[...]`, and aligned `EvaluationData["Trace"]` trees for the current structured subset
  - [x] deepen the current CL-09 subset with symbol-or-string property projection for `EvaluationData[...]` / `TestReport[...]`, input-resume coverage for `EvaluationData[...]` and `Monitor[...]`, and broader `MemoryInUse[]` growth coverage through persisted `Once[...]` state
  - [x] deepen the current CL-09 subset with held `EvaluationData["MessagesExpressions"]`, `TestReport` merging for `{report1, report2, ...}`, current report-level `SameTest` / `TimeConstraint` / `MemoryConstraint` options, and matching `SyntaxInformation` / default-option coverage
  - [x] land a first-pass `Dialog` subset with `Return`-sensitive evaluation boundaries, `DialogProlog` / `DialogSymbols` option plumbing, and input-resume coverage through the existing host-input suspension model
  - [x] deepen the current CL-09 subset with more layout-aware structural accounting for `ByteCount[]` / `MemoryInUse[]`, including builtin/session-layout coverage and active `OptionValue` scope growth in the current proxy model
  - [x] deepen the current CL-09 subset with default `Heads` option support for filtered `Trace[...]` / `TracePrint[...]`, plus broader per-test `TestReport` metadata through `ActualMessagesText`, `OutputLog`, and `Timing`
  - [x] close the current CL-09 mainline with payload-aware `ByteArray` accounting inside the layout-aware `ByteCount[]` / `MemoryInUse[]` subset plus the current structured trace/report shape coverage; treat any remaining deeper real-memory parity or trace/report edge-case polish as non-blocking long-tail work rather than active mainline blockers
- [ ] Defer reviewing the local `WLBuiltin/` workspace until the `CL-01` through `CL-09` mainline is complete; keep it ignored for now so it does not disrupt the active kernel roadmap

### Post-Mainline Long-Tail Priorities

- [ ] Work through the remaining `CL-01` through `CL-09` long-tail in explicit waves instead of reopening every matrix group at once:
  - [x] Wave 1: `CL-07` deep parity
    - [ ] leave unicode-heavy / golden regression expansion as non-blocking polish
    - [ ] leave fuller project-owned crypto envelope compatibility for future interop-driven work
    - [ ] leave richer `Nearest` long-tail semantics for concrete caller demand
    - [ ] leave language-data-backed `Pluralize` parity for a future data-backed lexical layer if user-visible need justifies it
  - [ ] Wave 2: `CL-03` and `CL-05`
    - [x] `CL-03`: close the current Wave 2 rule/pattern/traversal convenience sweep through association-aware `Cases` / `DeleteCases` / `Level` / `Count` / `MemberQ` / `FreeQ` / `Position`, the remaining `ReplaceList` root long-tail needed beyond the earlier backtracking subset, and current `Flat` / `Orderless` regrouping parity for `MatchQ` / `ReplaceAll` / `ReplaceList`; leave deeper pattern/string crossover and any remaining advanced matcher breadth as non-blocking long-tail
    - [ ] `CL-05`: remaining keyed-data long-tail polish such as deeper association semantics beyond the current broader keyed-helper operator-form sweep, current `Lookup` / callable association / keyed `Part` alignment, and current delayed-entry preservation across `AssociationThread`, key-take/drop, keyed transform/sort helpers, set-style keyed helpers, and `AssociateTo` / `KeyDropFrom` mutation; the remaining invalid/option parity corners beyond the current `KeyUnion` / `KeyIntersection` / `KeyComplement` plus `AssociationThread` / `Catenate` / `Merge` alignment after the current direct-association `Catenate` invalid-value and invalid-value resume boundary, remaining value-consuming delayed-entry corners beyond the current `Merge` combiner-controlled delayed values and current `Catenate` delayed association-value evaluation, fuller `Missing` forms beyond the current keyed-lookup plus current `DeleteMissing` depth / scan-depth / eager-delayed target deletion / delayed-predicate resume / delayed-cleanup subset, and held-mutation follow-up beyond the current symbol plus exact part-target subset and current `KeyDropFrom` non-leaf part-target invalid boundary
  - [ ] Wave 3: `CL-06` and `CL-01`
    - [ ] `CL-06`: remaining deeper `WithCleanup` / `Failsafe` / `CheckAbort` / `AbortProtect` parity beyond the current suspension-safe cleanup, cleanup-time host-input resume, throw/abort cleanup boundaries, cleanup-`Return` preservation of the primary outcome, and nested `AbortProtect` pending-abort propagation, `CheckAbort` list plus multi-option `PropagateAborts` parsing and fallback host-input resume, true-only `Failsafe` test plus inert non-wrapper arity, oracle-aligned generated `Failsafe` failure metadata plus generated failure `Message` held-argument rendering, and current function/fail-function host-input resume, current string and `MessageName[...]`-backed `Failure` `MessageTemplate` rendering including named `MessageParameters`, valid-template-only virtual `HeldMessageTemplate` / `MessageName`, plus direct virtual `Failure[...]` `"Tag"` projection and malformed status-object inert projection boundaries, silent malformed `Failure` / `Success` constructors, `FailureQ` recognition for malformed `Failure[...]`, immediate-rule plus single-`RuleDelayed` `Failure` details normalization, association-only `Success` details projection, inert `TerminatedEvaluation` constructor/application behavior plus generated top-level/request/evaluation-data `TerminatedEvaluation` for recursion and fixed-point iteration limits, current string-tag `Success` status-object projection boundary, current `Assert` session flag plus `On[Assert]` / `Off[Assert]` controls, inert disabled `Assert`, enabled-`Assert` message tag/text parity, and `$AssertFunction` callback hook, and `Input` / `InputString` `FieldMasked`, request-kind, resumed `Input` parse diagnostics, `Pause::numnm`, plus option-error host request metadata; richer failure/status behavior, cleanup-time `Throw` during active throw requiring active-unwind / `Catch`-frame bypass state, remaining niche `Assert` controls, and fuller host-I/O semantics remain as long-tail polish
    - [ ] `CL-01`: remaining part-spec / `Heads` parity beyond the current atom `All` / span-boundary selector alignment, current `Extract` nested position-list selector subset, empty-expression plus association-aware `Depth` parity, `Level` scalar-zero levelspec parity, empty-expression negative-level depth alignment across the current structural walkers, current `Not[Not[...]]` and relational-`Not` simplifications, flat nested `And` / `Or` residual simplification, first-pass `Xor` / `Equivalent` boolean/repeated-term reductions, first-pass `Implies` simplification, current short-circuit `Nand` / `Nor`, first-pass `Boole` truth-to-integer conversion, mixed-symbolic `Unequal` chain boundary, and nonatomic `OrderedQ` argument-order parity, plus deeper truth/comparison breadth beyond the current exact-numeric order-chain reductions
  - [ ] Wave 4: polish pass across `CL-04`, `CL-08`, and `CL-09`
    - [ ] `CL-04`: remaining threading / higher-order traversal / sparse array parity beyond the current negative-level empty-expression depth alignment for `Apply` / `Map` traversal families
    - [ ] `CL-08`: broader `Names` regex, `SyntaxInformation` properties, and package-tooling compatibility polish
    - [ ] `CL-09`: deeper real-memory accounting and trace/report edge-case polish
  - [ ] finish with a roadmap/docs cleanup pass that re-syncs the top-level `CL-01` through `CL-07` checklist state and any stale matrix wording with the now-completed mainlines
- [ ] `CL-X` Explicit exclusions for the current project phase stay documented and narrow unless they are deliberately adopted later
- [ ] Define a project-wide testing pyramid that distinguishes:
  - [ ] direct unit tests for small helpers, planners, normalization logic, and matcher branches
  - [ ] subsystem regression tests for syntax, lowering, runtime, terminal, REPL, CLI, Rust/Python host adapters, and Jupyter
  - [ ] end-to-end behavior tests for user-facing flows and protocol-level interactions
- [ ] Strengthen boundary and edge-case coverage for parser, lowering, evaluator, matcher, scoping, threading, loader, and message semantics
- [ ] Add broader regression corpora and golden fixtures for syntax formatting, lowering, evaluation, runtime messages, and transport-facing APIs
- [ ] Establish explicit coverage goals for high-risk semantic areas instead of relying only on opportunistic test growth
- [ ] Add differential-conformance fixtures that compare selected `rum` results against Wolfram behavior for the supported subset, with same-name language features treated as compatibility targets by default
- [ ] `CL-Y` Real-project Wolfram package/script support, using the local
  `mma-hol` project as the first concrete sample:
  - [ ] preserve the already-demonstrated `Module` closure pattern where
    exported definitions retain access to fresh private mutable state after
    the enclosing `Module` returns
  - [ ] add the small but project-critical list/set helpers used by ordinary
    `.wl` libraries, including `Join`, `Flatten`, `DeleteDuplicates`,
    `Union`, `Sort`, `SortBy`, `Reverse`, `Transpose`, and `Complement`
  - [ ] deepen held mutation support beyond the current subset so closure-owned
    state updates such as `AppendTo[privateList, item]` work predictably
  - [ ] add explicit gensym support through `Unique[...]` so package code can
    construct unforgeable private heads without relying only on `Module`
    locals
  - [ ] fill the script-runner environment needed by multi-file projects,
    including `$InputFileName`, mutable `$Path`-driven loading, `FileNameJoin`,
    `DirectoryName`, `ParentDirectory`, `FileNames`, `FileNameTake`, and
    `Exit`
  - [ ] keep small glue helpers such as `FromDigits` and text-only `Row` on the
    compatibility radar, while preferring sample-code cleanup when a dependency
    is accidental rather than philosophically important
- [x] Create a dedicated `rum-difftest` crate and ingest the imported `ham` seed pack plus handoff docs as a first-class conformance workstream input
- [ ] Keep `rum-difftest` split into explicit lanes instead of one monolithic diff bucket:
  - [ ] Lane A: single-expression Wolfram diff fixtures and runner
  - [x] Land a first-pass Lane A runner that accepts a user-provided kernel path via CLI or config and keeps oracle artifacts transient
  - [x] Land a first curated `cases/wolfram/core/` fixture set covering arithmetic/structures, strings, associations, loops/control objects, functional combinators, and failure/status smoke checks
  - [ ] Lane B: stateful session fixture format and runner
  - [ ] Lane C: checked known-difference ledger
  - [ ] Lane D: performance baseline inputs and runners
  - [ ] Lane E: keep host-contract tests in `rum`-native crate-local suites rather than Wolfram diff
  - [ ] Lane F: keep adapter/protocol tests in adapter-native suites rather than Wolfram diff
- [ ] Keep `rum`-specific kernel extensions and runtime/session semantics in crate-local suites unless they also have a meaningful Wolfram-oracle-comparable same-name surface
- [ ] Keep Wolfram-oracle execution environment-gated even though the differential test workstream itself is required:
  - [ ] accept a user-configured local oracle command such as `wolframscript`, `WolframKernel`, or `MathKernel`
  - [ ] treat the oracle environment and license as user-provided rather than project-provided
  - [ ] treat connector- or MCP-wrapped Wolfram access as an optional convenience for small spot checks rather than the baseline oracle path
  - [ ] do not make default development or CI flows depend on Wolfram tooling being installed
  - [ ] do not commit oracle-generated expected outputs into the repository
  - [ ] regenerate oracle comparison material on each diff run rather than relying on checked-in snapshots
  - [ ] allow oracle runs against non-pinned user versions as a compatibility check, not only against one project-standard Wolfram release
  - [ ] record any deliberate divergences from Wolfram behavior alongside those fixtures
- [ ] Prioritize oracle-style checks for the most semantics-sensitive areas first:
  - [x] arithmetic normalization and exact-integer behavior now have an initial curated Lane A foothold
  - [ ] rules, patterns, replacement, and traversal behavior
  - [x] scoping and held-evaluation constructs now have an initial curated Lane A foothold through loops/control/function cases
  - [ ] messages, attributes, and definition precedence
  - [x] string semantics now have an initial curated Lane A foothold that covers multiple CL-07 subfamilies

### Long-Tail Workstream: Full String Language Parity

- [x] Treat full string support as a first-class roadmap item rather than an incidental builtin grab bag
- [ ] Keep string literal parsing in `rum-syntax`, the canonical string atom in `rum-core`, and the current first-wave evaluation semantics plus builtin registration in `rum-runtime`, revisiting any broader crate split only if the string surface later outgrows the core kernel boundary
- [ ] Do not create a dedicated string crate yet; revisit a future `rum-string` extraction only after the builtin surface, shared helpers, and performance constraints are concrete enough to justify a new boundary
- [ ] Audit the Wolfram core-language string surface and split it into explicit implementation tiers with documented priorities
- [x] Tier 0 correctness: lock the initial in-memory string model to Unicode code-point semantics and add basic escape/unicode regression fixtures, while leaving richer escape validation and diagnostics for later follow-up
- [x] Tier 1 basic construction and slicing first wave: `StringJoin`, `StringLength`, `Characters`, `CharacterRange`, `ToString`, `StringTake`, `StringDrop`, and `StringPart`
- [x] Extend Tier 1 with the remaining near-term construction builtin `StringRepeat`, keeping the current scope focused on in-memory string values rather than host-side file or encoding IO
- [x] Land the first Tier 2 exact-search wave: `StringContainsQ`, `StringStartsQ`, `StringEndsQ`, and `StringPosition` over exact string needles with Unicode code-point indexing and overlapping 1-based position ranges
- [x] Land the corresponding Tier 2 exact-transformation wave: `StringSplit`, `StringTrim`, `StringReplace`, and `StringCases` for whitespace/default handling plus exact-string delimiters and immediate exact-string rules over in-memory string values
- [x] Widen the string search/transformation family from exact-string needles toward richer string-pattern and option semantics such as `StringExpression`, `IgnoreCase`, and overlap controls beyond the earliest defaults
- [x] Land an initial Tier 3 string-pattern subset by teaching the current string builtins to understand `StringExpression` and `Alternatives` over exact-string leaves
- [x] Continue Tier 3 string patterns through repetition forms, start/end anchors, `Shortest` / `Longest`, bounded repetition, `Except`, and broader zero-length/match-edge handling
- [ ] Continue Tier 3 string patterns through any remaining symbolic string-pattern parity cleanup beyond the now-landed `RegularExpression[...]` interop, symbolic blank/capture bindings, and current delayed-rule handling
- [ ] Continue Tier 4 conversions and interoperability beyond the now-landed `StringForm`, `StringTemplate`, `TemplateApply`, `TextWords`, `ToCharacterCode`, `ToExpression`, `SyntaxQ`, `TextString`, `FileTemplateApply`, `CharacterCounts`, `RemoveDiacritics`, `Alphabet`, `AlphabeticSort`, `EditDistance`, `HammingDistance`, `LongestCommonSubsequence`, `SequenceAlignment`, `Diff`, `StringToByteArray`, `Hash`, `Compress`, `Encrypt`, `Nearest`, and `Pluralize`, including richer `ToString` formatting, deeper nearest-neighbor semantics beyond the current radius/result/`Infinity` subset, fuller byte-array/crypto compatibility beyond the now-landed `ByteArray` payload/key support, and more language-data-like pluralization
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
