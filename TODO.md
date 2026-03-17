# TODO

This roadmap now uses two levels of detail:
- compact phase summaries for capability blocks that are already largely done
- detailed checklists only for work that is still open or likely to move soon

## Phase Summary

### Phase 0: Repository Foundation

- [x] Set up the Git-managed Rust workspace, baseline docs, and exact integer support
- [x] Keep the workspace compiling and passing tests as the project evolves

### Phase 1: Syntax Frontend

- [x] Build a stable `rum-syntax` foundation with AST, lexer, parser, core precedence, lists, calls, rules, patterns, slots, and pure functions
- [ ] Add spans, richer diagnostics, recovery, comments, and additional operator coverage

### Phase 2: Core Lowering and IR

- [x] Lower the currently supported syntax into working core `Expr` forms used by the runtime
- [ ] Formalize the lowering pipeline, canonical core-head inventory, and golden test coverage

### Phase 3: Definitions and Evaluation

- [x] Implement sessions, contexts, package loading, ownvalues/downvalues, hold-aware evaluation, and core assignment semantics
- [ ] Add first-class definition inspection and keep tightening evaluation semantics where the current interpreter is still ad hoc

### Phase 4: Rules and Pattern Matching

- [x] Implement interpreter-based matching with `Rule`, `RuleDelayed`, `MatchQ`, `Cases`, `ReplaceAll`, `ReplaceRepeated`, conditions, and sequence blanks
- [ ] Add a dedicated matcher IR, better rule inspection, and more complete traversal controls

### Phase 5: Scoping and Runtime Control

- [x] Implement `Hold*`, `Evaluate`, `Sequence`, `If`, `Which`, `Function`, `With`, `Module`, `Block`, tracing, and resource limits

### Phase 6: REPL and Developer Tooling

- [x] Ship a usable REPL baseline with evaluation, runtime-message rendering, `:inspect`, and `:messages`
- [ ] Add multiline input, richer inspection, debug views, and stronger regression coverage

### Phase 7: Embedding and Jupyter Preparation

- [ ] Stabilize a host-facing API around `Session`
- [ ] Prepare protocol-friendly diagnostics and a future `rum-jupyter` crate

## Open Workstreams

### `rum-syntax`: Spans and Diagnostics

- [ ] Choose a span model shared by lexer tokens, AST nodes, and future runtime diagnostics
- [ ] Attach spans to all syntax nodes without breaking the current parser API too abruptly
- [ ] Preserve operator and delimiter spans well enough to point at concrete source locations
- [ ] Add parser diagnostics for missing `]`, `}`, and `)` with useful labels
- [ ] Distinguish incomplete input from hard parse failure for REPL-friendly multiline handling
- [ ] Introduce limited parser recovery so one malformed form does not poison the whole script
- [ ] Add regression tests for malformed calls, lists, patterns, and pure functions

### `rum-syntax`: Comments and Surface Expansion

- [ ] Add Wolfram-style `(* ... *)` comment lexing
- [ ] Ensure comments can appear between tokens without changing parse results
- [ ] Decide whether trivia should only be skipped or also preserved for future tooling
- [ ] Add tests for comments around operators, calls, and nested expressions
- [ ] Audit the next operator set needed by the evaluator roadmap and prioritize them explicitly

### `rum-core` and Lowering Cleanup

- [ ] Audit every currently supported syntax feature and document its canonical lowered `Expr` form
- [ ] Centralize the list of canonical core heads for rules, patterns, slots, scoping, contexts, and messages
- [ ] Separate parser sugar from runtime-canonical representation more explicitly in the lowering code
- [ ] Add golden tests for syntax-to-core lowering across rules, patterns, scoping, messages, and package constructs
- [ ] Decide where lowering-time validation should stop and runtime validation should begin

### `rum-runtime`: Definition Inspection

- [x] Decide the public shape of definition-inspection builtins and their output conventions
- [x] Implement `OwnValues[symbol]` for immediate own values
- [x] Extend `OwnValues[symbol]` to surface delayed own values distinctly
- [x] Implement `DownValues[symbol]` while preserving rule order and delayedness
- [x] Keep message definitions on their own inspection path instead of merging them into `OwnValues` / `DownValues`
- [x] Extend `:inspect` to show a short summary of stored ownvalues and downvalues
- [x] Add regression tests for definition inspection on plain symbols, delayed values, and pattern-based downvalues

### `rum-runtime`: Matcher IR and Rule Debugging

- [ ] Define a matcher IR that can represent `Blank`, `Pattern`, `Condition`, `BlankSequence`, and `BlankNullSequence`
- [ ] Lower supported pattern expressions into that matcher IR
- [ ] Keep the current interpreter-based matcher as a reference path during the migration
- [ ] Route `MatchQ` through the shared compiled matcher entrypoint
- [ ] Route rule application and downvalue matching through the same compiled matcher path
- [ ] Add rule-inspection helpers that make immediate vs delayed behavior and normalized rule form easier to see
- [ ] Add tests that compare interpreter and compiled matcher behavior on the supported subset

### `rum-runtime`: `Cases` and Traversal Controls

- [ ] Decide the minimum level-spec subset to support first: `n`, `Infinity`, or `{min, max}`
- [ ] Parse and normalize level specifications into an internal representation
- [ ] Teach `Cases` to honor simple numeric level specs
- [ ] Extend `Cases` to support bounded ranges such as `{min, max}`
- [ ] Add explicit head-traversal control instead of hard-coding the current "do not visit heads" behavior
- [ ] Add traversal tests that lock down root, head, and nested-argument behavior

### `rum-runtime`: Evaluation Follow-Through

- [ ] Audit current evaluation behavior against the intended Mathematica-style sequence and list the known gaps
- [ ] Tighten the boundary between evaluation errors and language-level messages where it is still inconsistent
- [ ] Review package-loading evaluation for better failure isolation and clearer message emission
- [ ] Decide whether any additional resource limits are needed beyond recursion and `ReplaceRepeated` iteration caps

### `rum-repl` and `rum-cli`: Interactive Ergonomics

- [ ] Detect incomplete input and keep reading until a form is syntactically complete
- [ ] Add a continuation prompt for multiline entry
- [ ] Add line history persistence
- [ ] Add symbol completion from builtins, active contexts, loaded packages, and user-defined symbols
- [ ] Expand `:inspect` output with context, definition, and message-catalog details
- [ ] Add a parse debug command such as `:parse`
- [ ] Add a lowering debug command such as `:lower`
- [ ] Decide whether trace viewing belongs in the REPL and, if so, add a minimal command flow for it

### REPL and CLI Regression Coverage

- [ ] Add snapshot tests for help output, inspection flows, and runtime-message rendering
- [ ] Add multiline-session tests once multiline input exists
- [ ] Add CLI tests for `--eval`, file execution, and machine-readable output modes as they are introduced

### Embedding and Jupyter Preparation

- [ ] Define a stable session-configuration surface for hosts to set resource limits, search paths, and context defaults
- [ ] Stabilize parse/eval/inspect entrypoints that return machine-readable outcomes instead of terminal-formatted text
- [ ] Carry messages, traces, and diagnostics in a transport-friendly structure that does not depend on the REPL
- [ ] Separate terminal presentation concerns from protocol-facing kernel concerns more explicitly in crate boundaries
- [ ] Create a placeholder `rum-jupyter` crate when the host API is stable enough
- [ ] Map Jupyter-style execute, inspect, and completion requests onto the stabilized session API
