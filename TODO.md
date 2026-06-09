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
- [ ] Resolve the 2026-05 source-size audit against `docs/code-organization-guidelines.md`: current Rust source count is 0 files over `35k` bytes, 1 file in the `20k`-`35k` split-plan range, and 137 files in the `10k`-`20k` warning range
  - [x] Split or document explicit exceptions for the `>35k` architecture-debt files; no Rust source files remain over `35k` bytes after the 2026-05 split pass
  - [x] Add split plans for the original Rust files in the `20k`-`35k` split-plan range; no Rust source files remained in that range after the initial split pass
  - [ ] Names/SyntaxInformation test split follow-up: split the regrown `crates/rum-runtime/src/runtime/tests/names_and_syntax_information.rs` coverage along real behavior boundaries
  - [x] CL-05 association split follow-up: split the remaining oversized association test file along real conceptual boundaries; `crates/rum-runtime/src/runtime/tests/associations.rs` is down to warning range after moving key transforms, key sets, mutation surface, `KeyValuePattern`, aggregate, missing, and diagnostics coverage into focused test modules
  - [x] Debug tools split follow-up: replace the oversized `crates/rum-runtime/src/runtime/debug_tools.rs` file with a `debug_tools/` module tree split by basic display helpers, trace, message switches, reaping, resource limits, monitor, evaluation data, test reports, property projection, and shared render helpers
  - [x] Sequence matcher split follow-up: keep the shared sequence dispatcher and consume-length helpers in `crates/rum-runtime/src/runtime/pattern_matching/sequence.rs`, and move Orderless candidate/backtracking logic plus Optional/SequencePattern recursion into focused `sequence/` child modules
  - [x] Rewrite builtins split follow-up: keep continuation payload types in `crates/rum-runtime/src/runtime/rewrite_builtins.rs`, and move Cases/DeleteCases/MatchQ plus Replace/ReplaceAll/ReplaceList/ReplaceRepeated flows into focused child modules
  - [x] Holding/control test split follow-up: split the oversized `holding_and_control_part7.rs` test file into abort/assert, host I/O, cleanup, Success, Failure property, Failure message-template, and Failure predicate/termination modules
  - [x] Structure selector split follow-up: keep the `Part` / `Extract` evaluator entry points in `structure_ops/selectors.rs`, and move selector parsing, diagnostics, selection application, span expansion, and extract position-spec parsing into focused child modules
  - [x] Options split follow-up: keep option model types in `options.rs`, and move builtin entry points, option storage/default lookup, `OptionsPattern` scope handling, default option tables, and `ArgumentsOptions` / `CheckArguments` diagnostics into focused child modules
  - [x] String text split follow-up: keep the `strings::text` facade small, and move text builtin entry points, input-like rendering, string value helpers, and diacritic folding into focused child modules
  - [x] Builtin catalog split follow-up: keep `builtin_catalog.rs` focused on catalog assembly and default-attribute lookup, and move numeric, logic/predicate, rule/part, and namespace/message/option catalog slices into child modules
  - [x] Namespace tools split follow-up: keep shared namespace-tool wiring in `namespace_tools.rs`, and move `Names` enumeration/pattern matching, the current regex subset, and `SyntaxInformation` defaults into focused child modules
  - [x] Control-flow split follow-up: keep `control_flow/mod.rs` as a facade and split check/quiet, abort/cleanup, assertions, failsafe wrappers, nonlocal control flow, confirmation builtins, and Do/Table iteration into focused modules
  - [x] Definitions/rules test split follow-up: split the oversized `definitions_and_rules_part3.rs` coverage into Replace/Rule, Cases/DeleteCases, and pattern/downvalue modules
  - [x] Session-request transform test split follow-up: split resumable transform coverage into rewrite/deletecases, association transform, and pattern/thread modules
  - [x] Debug-tools test split follow-up: split trace/display, dialog, resource, evaluation-data, and test-report coverage into focused test modules
  - [x] Rewrite cases split follow-up: keep `rewrite_builtins/cases.rs` as a facade and split `MatchQ` normalization, `Cases` collection, and `DeleteCases` traversal/mutation into focused child modules
  - [x] Definition/rule lifecycle test split follow-up: split core definition inspection, lifecycle/remove, tag/upvalue/subvalue, and rewrite-inspection coverage into focused test modules
  - [x] Predicate implementation split follow-up: keep shared predicate helpers in `predicates.rs` and split truth aggregates, basic structural predicates, numeric predicates, list containment, ordering, and array-shape predicates into focused child modules
  - [x] Definition catalog split follow-up: keep `definition_catalog.rs` as a facade and split traversal normalization, argument resolution, rewrite-rule compilation, continuation codecs, and catalog/value-list rendering into focused child modules
  - [x] Tree walker split follow-up: keep `tree_walkers.rs` as a facade and split count-like walkers, position collection, and level collection into focused child modules
  - [x] Orderless sequence matcher split follow-up: keep `pattern_matching/sequence/orderless.rs` as a facade and split entry dispatch, optional omission handling, candidate partitioning, collect-all matching, first-match backtracking, and selected-value matching into focused child modules
  - [x] Suspension split follow-up: keep `suspension.rs` as a facade and split primitive continuation values, rewrite-rule payloads, traversal payloads, collection outcomes, and suspension-frame/session driving into focused child modules
  - [x] String pattern matcher split follow-up: keep `strings/patterns/matcher.rs` as a facade with match-state shape and split public matching API, split/trim boundary helpers, and the recursive pattern engine into focused child modules
  - [x] Association GroupBy split follow-up: keep `associations/grouping.rs` focused on spec parsing and dispatch, and split list grouping plus association-preserving grouping into focused child modules
  - [x] Structure replace split follow-up: keep `structure_ops/replace.rs` as a facade with shared `ReplacePart` types, and split `ReplaceAt`, `ReplacePart`, rule/position parsing, and replace-plan/pattern-position helpers into focused child modules
  - [x] Native ZeroMQ split follow-up: keep `zeromq_native.rs` as a facade and split socket facade/wrappers, worker handles, async worker loops, lifecycle commands, frame conversion, and worker-thread helpers into focused child modules
  - [x] Rewrite Replace split follow-up: keep `rewrite_builtins/replace.rs` as a facade and split `Replace`, `ReplaceAll`, `ReplaceList`, and `ReplaceRepeated` flows plus ReplaceList matching helpers into focused child modules
  - [x] Functional callable split follow-up: keep `functional_ops/callable.rs` as a facade and split composition, curried operators, reverse/comparison/nearest/key wrappers, and `Through` / `Operate` head operations into focused child modules
  - [x] Association resumable split follow-up: keep `associations/resumable.rs` as a facade and split `KeySelect`, `KeySortBy` and ordering sort, `AssociationMap`, plus `Merge` / `CountsBy` resumable flows into focused child modules
  - [x] Structure sequence split follow-up: keep `structure_ops/sequence.rs` as a facade and split `Take` / `Drop` / `TakeDrop`, `Insert`, `Delete`, and `TakeList` sequence operations into focused child modules
  - [x] Scoping split follow-up: keep `scoping.rs` as a facade and split callable head application, `With` / `Module` / `Block` scope execution, and `Unique` generated-symbol support into focused child modules
  - [x] Jupyter JSON parse split follow-up: keep `transport/json/parse.rs` as a facade and split wire-message parsing, field/object decoding, and status enum decoding into focused child modules
  - [x] Debug resource split follow-up: keep `debug_tools/resources.rs` as a facade and split resource-limit control flow from memory-proxy accounting helpers
  - [x] Debug test-report split follow-up: keep `debug_tools/test_report.rs` as a facade and split `TestCreate`, report aggregation/projection, test execution capture, and test/report parsing models into focused child modules
  - [x] Logic split follow-up: keep `logic.rs` as a facade and split truth-combinator evaluation, comparison-chain evaluation, and shared boolean/relation helpers into focused child modules
  - [x] Structure selector test split follow-up: move the large `Part` exact-index/path coverage from `tests/structure_ops/selectors.rs` into a focused `selectors/part_paths.rs` child module
  - [x] Functional sequence split follow-up: keep `functional_ops/sequence.rs` as a facade and split window/accumulate/take-while sequence operations from Fold/FoldWhile/SequenceFold operations
  - [x] Optional sequence matcher split follow-up: keep `pattern_matching/sequence/optional_sequence.rs` as a facade and split Optional default/omission logic, SequencePattern choice retry, collect-all sequence matching, and sequence item/pattern matching into focused child modules
  - [x] Jupyter wire codec split follow-up: keep `transport/codec.rs` focused on public wire encode/decode APIs and move delimiter, session, signature, and channel validation into a focused child module
  - [x] Jupyter adapter split follow-up: keep `adapter/mod.rs` focused on adapter state, request dispatch, and response flow, and move IOPub envelope construction into a focused child module
  - [x] Logic test split follow-up: keep `runtime/tests/logic.rs` as a facade and split truth-combinator coverage from comparison-chain and operator-form coverage
  - [x] Association transform split follow-up: keep `associations/transforms.rs` focused on map/sort/thread operations and move KeyUnion / KeyIntersection / KeyComplement keyed-collection transforms plus their resumable fill helpers into a focused child module
  - [x] Context loader split follow-up: keep `builtins_context_loader.rs` focused on context/package/load operations and move file-name/path builtins plus wildcard/path helpers into a focused child module
  - [x] Sequence matcher entry split follow-up: keep `pattern_matching/sequence.rs` focused on arg-and-rest recursion plus consume helpers and move public match/collect entry wrappers into a focused child module
  - [x] Matcher resume test split follow-up: keep `session_requests/matcher_resume.rs` focused on matcher/traversal resume coverage and move logic/truth short-circuit resume coverage into a focused child module
  - [x] Polynomial core split follow-up: keep `polynomials/core.rs` focused on `Polynomial` algorithms and move `Term`, `Monomial`, `VariableBasis`, factor normalization, and coefficient-list monomial parsing into a focused child module
  - [x] Association basic predicate split follow-up: keep `associations/basic.rs` focused on association construction, lookup, key/value projection, key take/drop, and `Normal`, and move `KeyMemberQ` / `KeyFreeQ` plus keyed predicate diagnostics into a focused child module
  - [x] Holding/control part6 test split follow-up: keep abort/checkabort, exit, loop-control, and `Failsafe` coverage in `holding_and_control_part6.rs`, and move `WithCleanup` plus uncaught `Throw` request diagnostics into a focused child module
  - [x] String search split follow-up: keep `strings/patterns/search.rs` focused on string search/replace/count/delete/apply entry points and move delayed replacement/case rule output evaluation helpers into a focused child module
  - [x] Debug trace split follow-up: keep `debug_tools/trace.rs` focused on `Trace` / `TracePrint` capture, filtering, and rendering, and move structured trace tree construction into a focused child module
  - [x] Jupyter adapter test split follow-up: keep `tests/adapter.rs` focused on direct adapter execute, inspect, completion, tracing, and completeness APIs, and move envelope dispatch / IOPub lifecycle coverage into a focused child module
  - [ ] Revisit the 134 `10k`-`20k` warning-range files opportunistically when touching nearby subsystems, prioritizing active runtime files over stable tests

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
- [x] Define and document a minimal stable host-facing public contract around sessions, requests, configuration, structured outcomes, and suspend/resume while keeping evaluator internals and experimental abstractions out of the public API
- [x] Add an initial first-class Rust host adapter crate with a narrow public surface shaped for long-running backend, rule-engine, and DSL-service embedding rather than broad internal exposure
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

- [x] Build a deliberate, cross-cutting test strategy instead of relying only on feature-local regressions; see [docs/testing-strategy.md](./docs/testing-strategy.md)
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
- [x] Complete the focused split-plan cleanup pass for the remaining `20k+`
  Rust files; the current source-size audit has no Rust source files over
  `20k`, leaving only the `10k`-`20k` warning range for opportunistic review
  when nearby subsystems are touched.

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
- [x] Tighten standard levelspec parity across the shared traversal walkers by accepting legal empty ranges, positive/negative `Infinity` list bounds, and Wolfram's mixed negative-depth / positive-level range semantics under current `Heads` traversal
- [x] Align compound-head `Heads -> True` traversal depth semantics so heads are visited as nodes without making their depth inflate the enclosing expression's negative level
- [x] Tighten `Extract` nested position-list parsing so shaped integer / `Key[...]` selectors remain valid while nested `All` / `Span` / deeper plain-list forms report Wolfram-aligned `Extract::psl1`, and carry the same compound-head negative-level rule into `Cases` / `DeleteCases`
- [x] Align `Part` / `Extract` invalid `Span` diagnostics by reporting malformed direct spans as `Part::span` / `Extract::span`, preserving `Span[..., 0]` as a selector that later fails with `take`, and emitting current `UpTo::innf` / `UpTo::argx` companion messages for invalid `UpTo[...]` span endpoints
- [x] Add the current unary `Extract[positionSpec]` operator form on top of existing `Function[...]` operator-form lowering so one-argument `Extract` no longer reports an arity error and can apply exact position specs to later targets
- [x] Align current `Part` / `Extract` argument-count diagnostics by giving `Part[]` the Wolfram-style `Part::argm` branch and `Extract[]` / over-arity calls the `Extract::argt` branch while leaving one-argument `Extract` as the operator form
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
- [x] Complete the current oversized `rum-jupyter` transport/backend/test split pass; continue only opportunistic warning-range cleanup when touching nearby adapter code

### Rust and Python Host Adapters

- [x] Write a first-pass `rum-host` plan in [docs/host-adapter-plan.md](./docs/host-adapter-plan.md) that fixes the intended crate boundary, goals, non-goals, and Rust-first / Python-second rollout order
- [x] Write a first-pass Python adapter plan in [docs/python-adapter-plan.md](./docs/python-adapter-plan.md) that fixes the intended crate boundary, Python-facing API bias, and `rum-host` dependency rule
- [ ] Treat Rust and Python host support as peer adapters to `rum-jupyter`, all layered on the same host-facing runtime API rather than becoming special cases inside `rum-runtime`
- [ ] Use a Rust reference scenario of long-running backend services, rule engines, and DSL-oriented infrastructure to judge API ergonomics, but keep the public surface limited to kernel sessions, requests, configuration, structured outcomes, and resource-control boundaries
- [ ] Use a Python reference scenario of AI- and data-tooling-oriented applications to judge API ergonomics, but keep the public surface general-purpose and free of AI-specific kernel features
- [x] Define which host-facing types and flows are stable public contract versus internal implementation detail so future adapters or wrappers can be added without freezing evaluator internals
- [x] Decide how much of `Expr`, diagnostics, messages, suspended input, and structured outputs should be exposed directly versus wrapped in host-friendly facades; the initial contract exposes rendered values and host facades, not raw `Expr`
- [x] Create the `rum-host` crate as the primary Rust embedding boundary before starting project-owned Python bindings
- [ ] Keep the Python adapter downstream of `rum-host` even if early experiments make direct `rum-runtime` binding look tempting
- [ ] Provide enough extension-oriented examples and adapter seams that downstream contributors can build richer wrappers without turning `rum-runtime` into a kitchen-sink host framework
- [x] Split `crates/rum-host/src/model.rs` into request/outcome/inspection submodules once the host facade grew beyond the first-pass API shape
- [x] Add broader Rust adapter-focused regression coverage beyond the initial `rum-host` smoke tests for parse, eval, inspect, complete, is-complete, structured failures, structured output, builtin-discovery configuration, and resumable input
- [x] Add a minimal Rust embedding example under `crates/rum-host/examples/basic_embedding.rs`
- [ ] Add future Python adapter-focused regression coverage after a `rum-python` crate exists
- [ ] Establish Python packaging and local-dev workflow only after the API shape is stable, keeping wheel/build concerns outside the kernel core
- [x] Evaluate the first `rum-jupyter` reuse boundary: keep direct `rum-runtime` use for now, and consider reusing `rum-host` cursor helpers and host outcome conversion only after the host facade proves stable

### Samples and Showcase Packages

- [ ] Prepare a small, polished sample set once the core kernel and host-adapter surfaces are stable enough to demonstrate end-to-end value rather than isolated builtin checklists
- [ ] Later build the planned [Python AI Policy](./examples/python-ai-policy/PLAN.md) sample once `rum-python` is ready: a Python-hosted AI-infrastructure policy / rewrite / validation example that keeps provider calls mocked and uses `rum` as a symbolic middleware layer over a narrow canonical request IR
- [ ] Later build the planned [Rust Wasm Prototype](./examples/rust-wasm-proto/PLAN.md) sample once `rum-host` is ready: a Rust-hosted WebAssembly validation and semantics prototype that uses Rust for host infrastructure and `rum` for stack typing, control-flow validation, step rules, traps, diagnostics, and explanation
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

- [x] `CL-01` Expression, truth, and structural foundation
- [x] `CL-02` Definitions, attributes, assignments, and scoping
- [x] `CL-03` Rules, patterns, and tree walkers
- [x] `CL-04` Functional and list combinators
- [x] `CL-05` Associations and keyed data
- [x] `CL-06` Procedural control, exceptions, and failure flow
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
  - [x] close the current CL-09 polish pass with broader session-state accounting for removed-symbol identity / generation state in the `MemoryInUse[]` proxy, explicit `Heads -> False` override regressions for `Trace[...]` / `TracePrint[...]`, and boundary coverage for `EvaluationData[...]` / `TestReport[...]` message-property projection plus merged-report projection
- [ ] Defer reviewing any restored or regenerated `WLBuiltin/` inventory until
  a deliberate post-mainline builtin survey is useful; see
  [docs/wolfram-builtin-inventory.md](./docs/wolfram-builtin-inventory.md)
  for the regeneration method, and do not treat the full Wolfram builtin table
  as an implementation commitment

### Post-Mainline Long-Tail Priorities

- [x] Work through the remaining `CL-01` through `CL-09` long-tail in explicit waves instead of reopening every matrix group at once. `CL-01` through `CL-09` are now mainline-closed; the notes below are demand-driven deferred parity items, not active blockers.
  - [x] Wave 1: `CL-07` deep parity
    - Deferred: unicode-heavy / golden regression expansion remains non-blocking polish.
    - Deferred: fuller project-owned crypto envelope compatibility waits for future interop-driven work.
    - Deferred: richer `Nearest` long-tail semantics should wait for concrete caller demand.
    - Deferred: language-data-backed `Pluralize` parity can wait for a future data-backed lexical layer if user-visible need justifies it.
  - [x] Wave 2: `CL-03` and `CL-05`
    - [x] `CL-03`: close the current Wave 2 rule/pattern/traversal convenience sweep through association-aware `Cases` / `DeleteCases` / `Level` / `Count` / `MemberQ` / `FreeQ` / `Position`, the remaining `ReplaceList` root long-tail needed beyond the earlier backtracking subset, and current `Flat` / `Orderless` regrouping parity for `MatchQ` / `ReplaceAll` / `ReplaceList`; leave deeper pattern/string crossover and any remaining advanced matcher breadth as non-blocking long-tail
    - [x] `CL-05`: close the current keyed-data mainline as substantially covered by the implemented association/rule-list construction, lookup, keyed helpers, grouping/counting/merge/catenate subsets, `Missing` / `DeleteMissing`, keyed structural selectors, and held mutation support; leave the remaining items below as non-blocking long-tail rather than active Wave 2 blockers.
      - Deferred: the evaluator-coupled `Values[..., Unevaluated]` delayed-value wrapper escape belongs to later hold/evaluation-model cleanup; this is not a local association-helper blocker.
      - Deferred: remaining `GroupBy` / `Counts` / `Catenate` / `Merge` invalid-message, obscure reducer, and option-tail edges are non-blocking parity polish unless a concrete migration case needs them.
      - Deferred: fuller `Missing` forms and broader association semantics beyond the current keyed-lookup / `DeleteMissing` subset are future feature breadth, not current CL-05 mainline blockers.
      - Deferred: association source/test splitting stays architecture debt tracked by the source-size audit, not a CL-05 semantic blocker.
  - [x] Wave 3: `CL-06` and `CL-01`
    - [x] `CL-06`: close the current mainline as substantially covered by the
      implemented control-flow, cleanup/abort, confirmation, status-object,
      assertion, and host-input subsets tracked in the support matrix. The
      latest pass adds oracle-aligned generated `Confirm*` failure metadata
      for the current `MessageTemplate` / `MessageParameters` /
      `ConfirmationType` slice, `$Canceled` and all `Missing[...]`
      confirmation handling, and broader confirmation-family host-input
      resume coverage, `Confirm[Failure[...]]` wrapping with the source
      failure message including delayed source `MessageTemplate` resume, plus
      `$AssertFunction = None` no-callback behavior and discarded callback
      return values. Treat the remaining items below as non-blocking
      long-tail parity work rather than active Wave 3 blockers.
      - Deferred cleanup/control long-tail: deeper `WithCleanup` /
        `Failsafe` / `CheckAbort` / `AbortProtect` parity beyond the current
        suspension-safe cleanup, throw/abort cleanup boundaries, abort
        propagation, full option-list validation plus `Rule` / `RuleDelayed`
        option RHS handling for `CheckAbort`, and request-level diagnostic
        coverage.
      - Deferred status/failure long-tail: richer generated
        `Failure` / `Success` / `TerminatedEvaluation` parity beyond the
        current projection, metadata, message-template subset, delayed
        `MessageParameters` rendering/resume, and Failsafe multi-argument
        `MessageParameters` grouping.
      - Deferred assertion/I/O long-tail: obscure `Assert` controls and
        fuller host-I/O option, prompt, request, parse/error semantics, and
        option-list diagnostic shape.
    - [x] `CL-01`: close the current mainline as substantially covered by the
      implemented expression model, exact path selectors, current `Part` /
      `Extract` / `ReplacePart` selector subset, current structural sequence
      helpers, `Depth` / `Level` / traversal `Heads` option slice, and current
      truth/comparison semantics tracked in the support matrix. Treat the
      remaining full Wolfram-compatible part-spec parity beyond the current
      atom `All` / span-boundary selector alignment, remaining `Part`
      selector-list and `Extract` nested position-list edges beyond the locked
      subset, full `Heads` parity for future walkers and less-common option
      diagnostics beyond the current `Depth` / `Level` / traversal parser,
      any `Level` / `Depth` oddities not covered by the current
      empty-expression / association oracle cases, and deeper
      truth/comparison breadth beyond the locked `Not` / `And` / `Or` /
      `Xor` / `Equivalent` / `Implies`, `Equal` / `Unequal`, and
      `Order` / `OrderedQ` slices as non-blocking long-tail parity work rather
      than active Wave 3 blockers.
      - [x] Cover the current nonatomic `Order` / `OrderedQ` argument-order
        parity slice by replacing text-only expression ordering with recursive
        same-head argument ordering and exact numeric atom comparison.
      - [x] Tighten the current `Order` / `OrderedQ` expression-order slice
        with Wolfram-aligned symbol-name ordering, same-name context fallback,
        and common system call-head precedence for `Association`, `Power`,
        `Times`, and `Plus`.
      - [x] Tighten the current `Order` / `OrderedQ` power-ordering slice so
        negative powers compare by base and sort before their own base and
        non-negative same-base powers, matching Wolfram's `a/b` vs `a*b`
        ordering.
      - [x] Tighten the current truth/comparison ordering slice with
        Wolfram-aligned number/string/symbol/compound expression order
        categories, stable `Equivalent` orderless argument placement, and
        `Nand` / `Nor` residual handling that preserves nested `And` / `Or`.
      - [x] Tighten the current `Heads` option parser shared by `Depth` /
        `Level` / traversal walkers so string-named and nested-list option
        forms, including empty option lists at option positions, follow the
        current Wolfram-compatible boundary.
      - [x] Align the current `Depth` / `Level` `Heads` option diagnostics for
        unknown option names, non-`True` / `False` `Heads` values, and
        non-option list entries while preserving `Level`'s third-argument
        wrapper-head ambiguity.
      - [x] Carry that `Heads` option diagnostic parity into the current
        traversal-family parser users, including `Cases`, `DeleteCases`,
        `Count`, `MemberQ`, `FreeQ`, and `Position` `optx` / `opttf` /
        `nonopt` message tags through suspension-safe continuation payloads.
      - [x] Lock the current empty-expression and association-aware
        `Depth` / `Level` boundary with oracle-backed cases for empty
        `Association[]`, empty-list negative-level traversal under
        `Heads -> True`, and association head/value traversal.
      - [x] Lock the current `Part` / `Extract` selector-list branch-failure
        and nested `Extract` key / empty-selector shapes with oracle-backed
        regression tests, and align explicit invalid-span `take` diagnostics to
        emit once while leaving broader part-spec parity as long-tail.
      - [x] Align `Symbol[...]` diagnostics that surface through `Part` /
        `Extract` head-access paths, including non-string `Symbol::string`
        and zero-argument `Symbol::argx` message tags.
      - [x] Align current `Part::take` / `Extract::take` range-rendering
        diagnostics so invalid `Span[...]` selections report `start through
        end` rather than raw canonical `Span[...]` expressions.
      - [x] De-duplicate invalid `UpTo[...]` span endpoint diagnostics across
        fixed-point normalization so rational endpoints such as `UpTo[3/2]`
        emit one `UpTo::innf` plus one `Part` / `Extract` `::span`.
      - [x] Align direct empty `Span[]` as a full-range `Part` / `Extract`
      - [x] Align direct single-endpoint `Span[...]` as prefix / suffix /
        empty-range `Part` / `Extract`
        selector and render invalid two- / three-argument `Span` diagnostics
        with Wolfram-style `;;` notation while preserving direct high-arity
        `Span[...]` message shape.
      - [x] Tighten the current `Equal` chain boundary so adjacent mixed
        comparable atoms, such as integer-vs-string pairs, reduce to `False`
        while non-adjacent unknown-containing chains remain conservative.
      - [x] Tighten the current `Equal` / `Unequal` truth/comparison boundary
        with Wolfram-style boolean-pair equality plus `Unequal` known-prefix
        duplicate detection before the first unknown comparison.
      - [x] Revisit zero-argument applications of comparison operator forms,
        such as `EqualTo[3][]`; current compound-head zero-arg rendering keeps
        them inert instead of reducing the equality-like forms to `True`.
      - [x] Align current order-comparison operator-form application arity
        diagnostics so `LessThan[3][]` / `LessThan[3][2, 4]` emit the
        wrapper-head `::argx` message while staying inert.
      - [x] Align the current `Level` invalid-levelspec diagnostic with the
        Wolfram-style `Level::level` tag while preserving apply-head and
        option-position behavior.
      - [x] Carry Wolfram-style invalid-levelspec and match-limit diagnostics
        into the current traversal-family users, including `Cases`,
        `DeleteCases`, `Count`, `MemberQ`, `FreeQ`, and `Position` `::level`
        plus `Cases` / `Position` `::innf` message tags.
      - [x] Align the current `Flatten::normal` boundary for atom and
        association inputs while preserving the existing same-head /
        level-group subset.
      - [x] Align `First` / `Last` / `Rest` / `Most` empty and atom
        diagnostics for the current expression / association surface subset.
      - [x] Align current structural sequence atom diagnostics for `Take` /
        `Drop` / `TakeDrop` / `TakeList` without broadening the implemented
        sequence-spec subset.
      - [x] Align current `Insert` / `Delete` atom-input diagnostics for the
        supported integer position subset while leaving nested position-list
        breadth as long-tail.
      - [x] Align current `Insert` / `Delete` out-of-range integer position
        diagnostics for nonatomic expressions and correct the current
        `Delete` integer path / list-of-paths subset, including direct
        `Delete[..., 0]` head deletion.
      - [x] Correct the current `Insert` integer path / list-of-paths subset,
        including nested insertion and multi-position insertion ordering.
    - [x] `CL-02`: mainline closed; leave only small definition/update-family
      polish as deferred long-tail unless a downstream project exposes a
      concrete blocker.
      Closed mainline includes symbol-target `SubtractFrom`, `TimesBy`,
      `DivideBy`, `ApplyTo`, `Decrement`, `PreIncrement`, `PreDecrement`, and
      `PrependTo` plus their common update syntax lowering where applicable,
      a first-pass `Remove` subset for symbol and string-name targets, and
      same-evaluation stale `Removed[...]` symbol identity with later same-name
      reuse through runtime symbol generations, plus
      current `DefaultValues` query / `ClearAll` interaction for optional
      defaults stored through `Default[...]` and explicit `SyntaxInformation`
      entries, plus a first-pass
      `InformationData` association summary for symbol and string-name targets
      with current `InformationValueForm` wrapping for non-empty definition
      lists and Wolfram-style `HoldPattern[lhs] :> rhs` public value-list
      rules, plus a held first-pass `Definition` expression for symbol and
      string-name targets,
      a first-pass `ValueQ` based on rollback trial evaluation, and exact
      part-target updates for the held update family including bulk selectors
      with list-shaped distribution / broadcast assignment, empty bulk selector
      no-op writes, duplicate nested bulk selector leaf distribution, and
      rejection of nested selector lists that Wolfram `Part` itself rejects
      with current `Part::pkspec1` / `Set::pkspec1` diagnostics, plus
      Wolfram-style `rvalue` diagnostics for non-variable held-update targets.
      Deferred non-blocking gaps include obscure update-family diagnostics,
      exact return-form / message-count parity for invalid nested held
      part-targets,
      deeper `Removed[...]` rendering / introspection corners, and fuller
      `Definition` / `Information` print-layer parity beyond the current
      transport-safe expression shapes.
  - [x] Wave 4: polish pass across `CL-04`, `CL-08`, and `CL-09`
    - [x] `CL-04`: close the current mainline as substantially covered by
      the implemented threading / `Listable`, `Map` / `Apply` / `Comap`
      traversal, targeted mapping, fold / nest / fixed-point, callable
      operator-wrapper, and dense `ArrayFilter` / `ArrayReduce` subsets, plus
      curated Wolfram Lane A coverage for representative Thread, Comap,
      MapIndexed, ArrayFilter, and ArrayReduce cases. Treat sparse-array
      parity, broader ArrayPad-style storage / padding compatibility, and
      obscure functional-combinator diagnostic or option-shape corners as
      non-blocking long-tail work rather than active CL-04 blockers.
    - [x] `CL-08`: close the current polish pass with `Names` default options plus `IgnoreCase` matching for wildcard and current regex patterns, broader `SyntaxInformation` default shapes for `Names` / `Protect` / `Unprotect`, and package-tooling regressions for `Save` / `DumpSave` names-pattern object selection; leave deeper regex-engine parity, context-alias resolution, spelling correction, and fuller persistence compatibility as future non-blocking long-tail
    - [x] `CL-09`: close the current polish pass with removed-symbol identity
      / generation-state accounting in the `MemoryInUse[]` proxy, explicit
      `Trace[...]` / `TracePrint[...]` `Heads` override regressions, and
      `EvaluationData[...]` / `TestReport[...]` message-property and
      merged-report projection boundary coverage. Leave allocator-level
      real-memory accounting and obscure trace/report parity for future
      concrete demand rather than active Wave 4 blockers.
  - [x] finish with a roadmap/docs cleanup pass that re-syncs the top-level `CL-01` through `CL-09` checklist state and stale matrix wording with the now-completed mainlines
- [ ] `CL-X` Explicit exclusions for the current project phase stay documented and narrow unless they are deliberately adopted later
- [x] Define a project-wide testing pyramid that distinguishes direct unit
  tests, subsystem regressions, differential conformance, adapter /
  host-contract suites, and real-project smoke lanes; see
  [docs/testing-strategy.md](./docs/testing-strategy.md)
- [ ] Strengthen boundary and edge-case coverage for parser, lowering, evaluator, matcher, scoping, threading, loader, and message semantics
- [ ] Add broader regression corpora and golden fixtures for syntax formatting, lowering, evaluation, runtime messages, and transport-facing APIs
- [x] Establish explicit coverage goals for high-risk semantic areas instead of
  relying only on opportunistic test growth; the checked `rum-difftest`
  inventory baseline now guards source and semantic-category minima, with the
  policy documented in `crates/rum-difftest/coverage/README.md`
- [ ] Add differential-conformance fixtures that compare selected `rum` results against Wolfram behavior for the supported subset, with same-name language features treated as compatibility targets by default
  - [x] Attribute parity follow-up: normalize equivalent hold-attribute
    combinations such as `HoldFirst` plus `HoldRest` to `HoldAll`, then retire
    the `attributes_hold_first_rest_normalizes_holdall` known difference
- [ ] `CL-Y` Real-project Wolfram package/script support, using the local
  `mma-hol` project as the first concrete sample:
  - [ ] preserve the already-demonstrated `Module` closure pattern where
    exported definitions retain access to fresh private mutable state after
    the enclosing `Module` returns
  - [x] make the normal `Table` collection path avoid
    repeated accumulator cloning, then add first-pass `Flatten` support for
    `Flatten[expr]`, `Flatten[expr, 1]`, `Flatten[expr, n]`, and
    `Flatten[expr, Infinity]` over same-head expressions plus
    `Flatten[expr, n, h]` single-head specs and current level-list specs such
    as `{2, 1}` / `{{1, 2}, {3}}`, covering the nested-table `List` case as
    the preferred efficient alternative to large-loop `AppendTo`; leave only
    obscure edge-case parity for later polish
  - [x] add first-pass `Join` support for top-level same-head expressions,
    associations, later-key-wins association canonicalization, and the current
    positive-integer `Join[..., n]` level subset over homogeneous same-head
    structure; leave obscure edge-case/message parity for later structural
    polish
  - [x] add first-pass `Reverse`, `Transpose`, `DeleteDuplicates`, and
    `DeleteDuplicatesBy` support for ordinary `.wl` list-shaping and stable
    deduplication flows; leave fuller permutation/rank, custom same-test, and
    option/message parity as polish
  - [x] add first-pass `Union`, `Complement`, `Sort`, and `SortBy` support for
    ordinary same-head list/set and structural-ordering flows; leave custom
    ordering predicates, multi-key `SortBy`, and deeper option/message parity
    as polish
  - [x] deepen held mutation support with first-pass symbol-target
    `AppendTo[privateList, item]`, `i++`, and `counter += delta`; leave
    broader part-target and obscure update-family parity for later CL-02 polish
  - [x] add first-pass explicit gensym support through `Unique[]`,
    `Unique["prefix"]`, `Unique[symbol]`, list forms, and current
    attribute-setting forms so package code can construct unforgeable private
    heads without relying only on `Module` locals; leave obscure
    message/attribute edge cases as future parity polish
  - [ ] broaden source-language parsing for real Wolfram scripts, including
    Unicode identifier characters used as ordinary local names and the
    Unicode operator glyphs that package-level parsers/printers commonly keep
    in string data or symbolic constants
  - [x] add the first script-runner environment wave for multi-file projects,
    including `$InputFileName`, mutable `$Path`-driven loading,
    `FileNameJoin`, `DirectoryName`, `ParentDirectory`, `FileNames`, and
    `FileNameTake`; leave fuller path-option/message parity for later polish
  - [x] add request-boundary `Exit` control flow, including `Get` propagation
    and CLI exit-code mapping, before turning on the first real-project smoke
    lane
  - [ ] turn `mma-hol/tests/run_all.wls` into the first real-project smoke lane
    once the loader/script prerequisites land, with special attention to the
    already-matrix-tracked `Throw` / `Catch` / `Failure` / `FailureQ` /
    `Stack[]` path
  - [ ] keep small glue helpers such as `FromDigits` and text-only `Row` on the
    compatibility radar, while using test-output `ToString[..., InputForm]`
    paths as smoke coverage for the already-tracked string rendering subset
    and preferring sample-code cleanup when a dependency is accidental rather
    than philosophically important
- [x] Create a dedicated `rum-difftest` crate and ingest the imported `ham` seed pack plus handoff docs as a first-class conformance workstream input
- [ ] Keep `rum-difftest` split into explicit lanes instead of one monolithic diff bucket:
  - [x] Lane A: single-expression Wolfram diff fixtures and runner
  - [x] Land a first-pass Lane A runner that accepts a user-provided kernel path via CLI or config and keeps oracle artifacts transient
  - [x] Lane A follow-up: compare single-expression values, messages, and printed output instead of values only
  - [x] Lane A follow-up: curate resolved imported seed known differences such as `specific_override` into `cases/` before pruning seed-linked active ledger rows, preserving `seed/` as unchanged historical input via the retired seed manifest
  - [x] Lane A/B follow-up: add a shared InputForm-like renderer for common list, rule, delayed-rule, and association output shapes before oracle comparison
  - [x] Land a first curated `cases/wolfram/core/` fixture set covering arithmetic/structures, strings, associations, loops/control objects, functional combinators, rules/patterns/replacement/traversal, and failure/status smoke checks
  - [x] Lane A follow-up: make the default runner include both imported seed golden files and curated project-owned `cases/wolfram/core/` fixtures, while keeping `--fixture PATH` for single-file focus
  - [x] Lane B: land a first stateful session fixture format and runner for ordered definition / option / message / precedence checks
  - [x] Lane B follow-up: replace replay-based Wolfram state simulation with a single-kernel session oracle transport per fixture before relying on deeper stateful edges
  - [x] Lane B follow-up: add per-step message-stream and print-stream capture so stateful diffs compare values, messages, and prints instead of values only
  - [x] Lane A/B follow-up: surface included known-difference metadata in reports and count confirmed vs resolved known differences so closable ledger rows are visible during oracle runs
  - [x] Lane A/B follow-up: add `--summary-json PATH` machine-readable status output with oracle configuration, run options, comparison counts, known-difference counts, and seed-drift counts for scripts or CI dashboards
  - [x] Lane A/B follow-up: extend `--summary-json PATH` with bounded issue-sample locators for non-matches, seed snapshot drift, and resolved known differences, plus `--max-issue-samples N` and emitted/total/truncated metadata, without freezing full per-case/per-step report rows
  - [x] Lane B follow-up: expose oracle session batch and step counts in text/JSON reports so optional long-lived oracle process reuse has an observable throughput baseline
  - [x] Lane B follow-up: route prepared fixture groups through an `evaluate_session_batches` oracle interface, preserving the current per-fixture default while creating the API boundary needed for a later single-process multi-batch Wolfram transport
  - [x] Lane A follow-up: route prepared single-expression cases through the
    same `evaluate_session_batches` oracle interface as isolated one-case
    batches, so the local `OracleKernel` path can amortize process startup
    without sharing state across Lane A cases
  - [x] Lane B follow-up: implement the `OracleKernel` multi-batch session transport as one Wolfram process per Lane B run, with fixture-boundary cleanup of Global-context symbols, newly introduced contexts, `$Packages`, `$Context`, and `$ContextPath`
  - [x] Lane B follow-up: pin the multi-batch transport parser and per-step stream assembly with local unit tests, including malformed markers and missing-result count mismatches
  - [x] Lane B follow-up: add a fixture-boundary isolation fixture that detects leaked ordinary symbols, user contexts, and package contexts across multi-batch oracle transport batches while still proving state persists within one fixture
  - [x] Lane B follow-up: expose the concrete session-batch transport name and process count in text/JSON reports, so the current single-process `OracleKernel` path is distinguishable from generic fallback behavior
  - [x] Lane A/B follow-up: add a non-oracle `rum-difftest inventory` coverage-audit command with source-group, normal-entry, known-difference, and machine-readable summary counts for default fixture sets
  - [x] Lane A/B follow-up: add a checked inventory baseline plus `inventory --check-baseline --fail-on-regression` so fixture count, total entry, normal-entry, and known-difference regression gates are executable without Wolfram
  - [x] Lane A/B follow-up: add `inventory --write-baseline PATH` so intentional fixture additions or reshuffles can refresh the stable non-oracle coverage baseline without hand-editing threshold rows
  - [x] Lane A/B follow-up: extend `rum-difftest inventory` with stable semantic category counts in text and JSON output so high-risk fixture coverage is easier to audit without parsing file paths
  - [x] Lane A/B follow-up: extend the checked inventory baseline to source-or-category records so semantic category coverage regressions fail `inventory --check-baseline --fail-on-regression`
  - [x] Lane A/B follow-up: add focused `inventory --lane` / `--source` /
    `--category` filters that apply consistently to text output, summary JSON,
    baseline comparison, and baseline capture, with fail-fast empty-filter
    handling for typoed coverage audits
  - [x] Lane A/B follow-up: make focused inventory baseline checks consider
    only baseline records matching the active filter scope, and fail fast when
    fixtures match but no checked baseline record covers that scope
  - [x] Lane A/B follow-up: record active filters in filtered
    `inventory --write-baseline` artifact comments so intentionally scoped
    baselines are not confused with the repository-wide checked baseline
  - [x] Lane A/B/C follow-up: add `rum-difftest check` as a fast non-oracle
    aggregate gate over the checked inventory baseline, Lane C ledger
    validation, and oracle-artifact hygiene, with optional stable summary JSON
    for local preflight scripts
  - [x] Lane A/B/C/D follow-up: let `rum-difftest check --include-lane-d-smoke` opt into the local Lane D smoke baseline gate, with `--lane-d-smoke-baseline PATH` for alternate baseline validation while keeping default `check` free of timing inputs
  - [x] Lane A/B/C/D follow-up: let `rum-difftest check --include-lane-d-readiness` opt into the Lane D checked-baseline readiness gate and surface the missing runner-specific artifacts in the aggregate text/JSON summary without affecting the default local preflight path
  - [x] Lane A/B/C/D follow-up: add bounded `issue_samples` to `rum-difftest check --summary-json`, with `--max-issue-samples N` plus emitted/total/truncated metadata for local scripts that need a wider or narrower representative sample, so aggregate-gate failures expose inventory, Lane C, Lane D smoke, and Lane D readiness locators without freezing each lane's full report shape
  - [x] Lane A/B/C/D follow-up: make `rum-difftest check` report skipped optional gates in text/JSON output, so default local preflight coverage is explicit instead of inferred from missing Lane D sections
  - [x] Lane A/B/C/D follow-up: add a top-level `schema_version` to every `--summary-json` output so local scripts and dashboards can detect intentionally incompatible report-shape changes
  - [x] Lane A/B/C/D follow-up: make CLI summary-output tests parse generated
    `--summary-json` files as JSON and assert key fields structurally, so
    machine-readable report regressions cannot pass through string-fragment
    assertions alone
  - [x] Lane A/B/C/D follow-up: add untruncated aggregate `issue_kind_counts`
    to `rum-difftest check --summary-json`, so scripts can count dirty gate
    categories even when bounded `issue_samples` are intentionally capped
  - [x] Lane A/B/C/D follow-up: add untruncated aggregate
    `issue_check_counts` to `rum-difftest check --summary-json`, so scripts
    can count dirty aggregate gates directly without widening or parsing
    representative issue samples
  - [x] Lane A/B/C/D follow-up: add `--issue-sample-check` and
    `--issue-sample-kind` filters to `rum-difftest check --summary-json`,
    preserving unfiltered totals/counts while narrowing representative locators
  - [x] Lane A/B/C/D follow-up: validate aggregate `--issue-sample-kind`
    values against the known check issue-kind set so typoed filters fail fast
    instead of silently producing empty sampled locators
  - [x] Lane A/B/C/D follow-up: keep aggregate check/sample kind filter
    validation backed by the same name catalog used by `check --summary-json`
    issue-sample collection, avoiding parser/reporter drift
  - [x] Lane A/B follow-up: add `--source` / `--category` filters for focused
    oracle audits over the default fixture sets, reusing the same source labels
    and semantic category names exposed by `inventory`
  - [x] Lane A/B follow-up: expose oracle process counts in lane summary JSON
    plus concrete batch transport metadata and generic batch/input counts, and
    keep Lane A/B text output showing process, batch, and batch-input counts,
    making the current case-isolated batch Lane A transport and single-process
    Lane B batch transport auditable before any future interactive reuse work
  - [x] Lane A/B follow-up: add `--fail-on-seed-drift` as an opt-in fixture
    snapshot audit gate, keeping normal oracle conformance based on fresh
    `rum`-vs-Wolfram comparisons while letting seed refresh passes fail fast
    and recording active failure gates in summary JSON options
  - [x] Lane A/B follow-up: add `--fail-on-resolved-known-difference` as an
    opt-in known-difference cleanup gate, so oracle audits can fail when
    included active ledger rows now match fresh Wolfram behavior and should be
    retired
  - [x] Lane A/B follow-up: add `--issue-sample-kind` filtering for
    non-match, seed-drift, and resolved-known-difference locators so targeted
    oracle audits are not drowned out by unrelated issue categories, while
    keeping an unfiltered issue-count breakdown in summary JSON
  - [x] Lane A follow-up: audit curated fixture seed annotations against the
    local Wolfram oracle and refresh stale project-owned association/template
    expectations, leaving the remaining default-run seed drift isolated to the
    imported `seed/ham-import` snapshot
  - [x] Lane A/B follow-up: add `--fail-on-project-seed-drift` plus text/JSON
    project seed-drift counts, so default oracle audits can protect
    project-owned fixtures without failing on immutable imported seed snapshot
    drift
  - [x] Lane A/B follow-up: add `--fail-on-project-mismatch` plus text/JSON
    project non-match counts, so full default oracle audits can gate
    project-owned fixtures while preserving imported seed failures as
    historical-reference input
  - [x] Lane A/B follow-up: include `fixture_source` and
    `project_owned_fixture` on summary JSON issue samples, so automation can
    route imported seed, curated project, stateful, and custom fixture locators
    without reimplementing path classification
  - [x] Lane A/B follow-up: add unfiltered `issue_source_counts` to summary
    JSON so automation can see non-match, seed-drift, and resolved-known
    issue distribution by fixture source without widening sampled locators
  - [x] Lane A/B follow-up: add `--issue-sample-source` filtering for bounded
    oracle-lane issue locators so full default audits can focus samples on
    project-owned or imported fixture sources without changing unfiltered
    issue/source counts
  - [x] Lane A follow-up: add a curated structure-selector fixture for stable
    `Part` / `Extract` / `ReplacePart` / `Depth` / `Level` conformance, and
    guard it with the checked inventory category baseline
  - [x] Lane A follow-up: add a curated threading fixture for stable `Thread`
    / `MapThread` / current listable-dispatch conformance, and guard it with
    the checked inventory category baseline
  - [x] Lane A follow-up: broaden the curated definition-precedence fixture
    with stable downvalue replacement, literal-vs-typed dispatch, and upvalue
    precedence cases, and guard the higher coverage floor in the inventory
    baseline
  - [x] Lane A follow-up: add a curated options-surface fixture for stable
    `Options` / `SetOptions` / `OptionValue` / `OptionsPattern` conformance,
    and guard it with the checked inventory category baseline
  - [x] Lane A follow-up: add a curated messages/attributes fixture for stable
    `Message` / `Quiet` / `Check` and basic attribute conformance, then retire
    the `HoldFirst` plus `HoldRest` normalization known-difference row after
    runtime parity landed
  - [x] Lane A follow-up: add a curated context/namespace fixture for stable
    `Context`, `Begin` / `End`, `BeginPackage`, and `Names` conformance, and
    guard it with the checked inventory category baseline
  - [x] Resolve the project-owned Lane A known differences for exact
    `Factorial[8]`, `Abs[-7]`, and integer-list `Range[5]` by implementing
    the current exact-numeric / integer-range runtime subset and promoting
    those curated smoke cases back to normal oracle fixtures
  - [x] Resolve current project-owned Lane A known differences for same-name
    curated smoke cases that are useful compatibility targets but outside the
    current implemented subset: the `StringForm` expression/display boundary
  - [ ] Lane B follow-up: add optional long-lived interactive process reuse once oracle-run throughput needs it
  - [x] Lane C: land a checked known-difference ledger plus `rum-difftest lane-c` validation that fixture `!!! id=...` entries and ledger rows stay synchronized
  - [x] Lane C follow-up: add richer metadata links from ledger entries back to matrix/TODO items and close criteria as divergences are resolved
  - [x] Lane C follow-up: validate ledger `fixture=...` metadata against the actual fixture file containing each known-difference ID
  - [x] Lane C follow-up: validate active ledger `tracked_by=path:needle` metadata against repository files so stale ownership references fail `rum-difftest lane-c`
  - [x] Lane C follow-up: add `--summary-json PATH` machine-readable status output with stable counts and checked artifact paths for scripts or CI dashboards
  - [x] Lane D: land a local `rum-difftest lane-d` runner over the imported timing and memory seed suites, with a default smoke profile, explicit full profile, elapsed time, result node count, message count, print count, and evaluation failures without using a Wolfram oracle
  - [x] Lane D follow-up: add checked historical baseline artifacts, threshold policy, and regression-decision output before treating performance runs as pass/fail gates
  - [x] Lane D follow-up: add baseline capture workflow before tightening elapsed thresholds beyond broad smoke guardrails
  - [x] Lane D follow-up: add multi-run baseline calibration before tightening elapsed thresholds beyond broad smoke guardrails
  - [x] Lane D follow-up: add CI-machine-specific threshold profiles before tightening elapsed thresholds beyond broad smoke guardrails
  - [x] Lane D follow-up: add `--summary-json PATH` machine-readable status output with stable run counts, clean status, and optional baseline/readiness/write-baseline summary counts for scripts or CI dashboards
  - [ ] Lane D follow-up: capture checked `ci-debug` / `ci-release` baseline artifacts on the intended runners before tightening elapsed thresholds beyond broad smoke guardrails; `lane-d --check-baseline-readiness` now reports the missing runner-specific artifacts instead of relying on TODO text alone
  - [x] Lane E: keep host-contract tests in `rum`-native crate-local suites rather than Wolfram diff, with current Rust host-contract coverage owned by `cargo test -p rum-host`
  - [x] Lane F: keep adapter/protocol tests in adapter-native suites rather than Wolfram diff, with current Jupyter protocol coverage owned by `cargo test -p rum-jupyter`
- [x] Keep `rum`-specific kernel extensions and runtime/session semantics in crate-local suites unless they also have a meaningful Wolfram-oracle-comparable same-name surface
- [x] Keep Wolfram-oracle execution environment-gated even though the differential test workstream itself is required:
  - [x] accept a user-configured local kernel-compatible oracle command such as `WolframKernel`, `MathKernel`, or a compatible wrapper via CLI, config, environment, `PATH`, or common install discovery
  - [x] treat the oracle environment and license as user-provided rather than project-provided
  - [x] treat connector- or MCP-wrapped Wolfram access as an optional convenience for small spot checks rather than the baseline oracle path
  - [x] do not make default development or CI flows depend on Wolfram tooling being installed
  - [x] do not commit oracle-generated expected outputs into the repository; `rum-difftest` now has an artifact hygiene test that rejects checked-in Lane A/B oracle wrapper, case/step, session, message, and print-capture artifacts
  - [x] regenerate oracle comparison material on each diff run rather than relying on checked-in snapshots; Lane A/B compare `rum` against the fresh oracle result, treat fixture `expected` fields as seed annotations, and report seed-vs-oracle drift separately
  - [x] allow oracle runs against non-pinned user versions as a compatibility check, not only against one project-standard Wolfram release; Lane A/B report the `user-provided/unpinned` version policy and config rejects oracle-version pin keys
  - [x] record the resolved oracle executable and best-effort Wolfram `$Version` in Lane A/B CLI output without making version probing a pass/fail gate
  - [x] bound every Lane A/B oracle and version-probe process with a configurable timeout, terminate timed-out kernels, and report the effective timeout in CLI output
  - [x] bound oracle stdout/stderr capture per stream, terminate over-limit processes, and retain only a capped diagnostic prefix
  - [x] record any deliberate divergences from Wolfram behavior alongside those fixtures; Lane C now verifies active fixture `!!! type=...` metadata stays aligned with ledger `type=...`, while fixture lines already require local `reason=...`
- [x] Prioritize oracle-style checks for the most semantics-sensitive areas first:
  - [x] arithmetic normalization and exact-integer behavior now have an initial curated Lane A foothold
  - [x] Lane A now has a focused exact-arithmetic fixture covering negative `Mod` / `Quotient`, signed `GCD` / `LCM`, rational powers, and `PowerMod` against a real Wolfram oracle
  - [x] Extend the shared difftest InputForm-like renderer with common `Plus` / `Times` / `Power` infix output, then add a first curated Lane A polynomial/rational-expression fixture for stable `Expand` / `Together` cases; leave broader polynomial term-ordering parity for later
  - [x] rules, patterns, replacement, and traversal behavior now have an initial curated Lane A foothold
  - [x] scoping and held-evaluation constructs now have an initial curated Lane A foothold through loops/control/function cases
  - [x] Lane A now has a focused scoping/closure fixture covering stable `With`
    lexical substitution, `Module` fresh-symbol and closure-state behavior,
    `Block` dynamic scope lookup, and `Throw` restoration against a real
    Wolfram oracle
  - [x] messages, attributes, and definition precedence now have an extended Lane B stateful foothold, including exact emitted-message checks, message-list length, `SetAttributes` / `ClearAttributes`, held argument evaluation, typed/generic/literal downvalue precedence, and UpValue dispatch
  - [x] Lane B now has broader stateful smoke coverage for prints, default values, repeated delayed evaluation, context `Begin` / `End`, definition lifecycle `Clear`, and `Protect` / `Unprotect`
  - [x] Lane B now has a curated stateful scoping/control fixture covering ordered `Block` / `Module` / `With` behavior and `Throw`-mediated restoration against a real Wolfram session oracle
  - [x] Lane B now has a curated stateful mutation/update fixture covering
    ordered `AssociateTo`, `KeyDropFrom`, `AppendTo`, `Increment` / `++`, and
    `AddTo` / `+=` behavior against a real Wolfram session oracle
  - [x] close the current Lane B known differences from `definitions_messages.txt`: exact Wolfram `$MessageList` `HoldForm[...]` shape, literal downvalue priority after an existing typed downvalue, and indexed default lookup fallback
  - [x] broaden Lane B package-loading/default-value coverage with an explicit file-backed package fixture covering qualified package values/functions, package-scoped defaults, indexed default fallback, and repeated `Needs` no-reload behavior
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
- [x] Finish the oversized `rum-jupyter` transport/backend/test split pass; no current Rust source file remains in the split-plan range

### P2

- [ ] Realign runtime test file names and contents so review-critical tests live in the modules their names imply instead of drifting into misleading buckets
- [ ] Revisit matcher compilation/storage strategy after semantics stabilize, including whether definition insertion should cache more matcher state instead of recompiling on hot paths
- [ ] Profile evaluator hot paths such as fixed-point iteration, heads lookup, and expression ordering before deciding whether additional low-level performance work is justified
- [ ] Revisit ergonomics of large request/error payloads only after measurement shows that the current shapes are a real cost rather than a static-analysis curiosity
