# TODO

This file tracks the project roadmap in a milestone-oriented way, with the
syntax frontend broken down into concrete engineering tasks that can be checked
off as work lands.

## Phase 0: Repository Foundation

- [x] Initialize a dedicated Git repository for `rum`
- [x] Create a Rust workspace with core crates
- [x] Add bootstrap architecture and README documentation
- [x] Introduce exact integer support with `num-bigint`
- [x] Make the workspace compile and test successfully

## Phase 1: Syntax Frontend

- [x] Establish a stable syntax crate layout with `ast`, `lexer`, and `parser`
- [x] Support grouped expressions and list literals
- [x] Support infix operators for rules, assignments, arithmetic, and replace/condition forms
- [x] Support pattern syntax for `_`, `__`, `___`, and named patterns such as `x_Integer`
- [x] Support slots and pure functions with `#`, `#n`, and `&`
- [x] Preserve exact integer literals as source text until lowering
- [x] Expand parser test coverage around precedence and nested constructs
- [ ] Introduce span-carrying syntax nodes or a green tree strategy
- [ ] Add richer diagnostics with labels and recovery

## Phase 2: Core Lowering and IR

- [ ] Lower syntax constructs into core `Expr` forms systematically
- [ ] Define core heads for rules, patterns, slots, and scoping constructs
- [ ] Separate syntax-level representations from runtime-level canonical forms
- [ ] Add golden tests for syntax-to-core lowering

## Phase 3: Definitions and Evaluation

- [ ] Implement symbol tables and context-aware symbol resolution
- [x] Add definition storage for `OwnValues` and `DownValues`
- [ ] Implement standard evaluation sequencing
- [ ] Implement `Set` and `SetDelayed`
- [x] Add evaluation control attributes such as `HoldFirst`, `HoldRest`, and `HoldAll`

## `rum-runtime` Detailed Breakdown

### Session state

- [x] Keep builtin metadata separate from user definitions
- [x] Add a definition table for symbol-bound values
- [x] Distinguish immediate and delayed own values
- [x] Add explicit storage for downvalues and pattern-based definitions
- [ ] Add context-aware symbol interning and resolution rules

### Evaluation loop

- [x] Evaluate bare symbols through definition lookup
- [x] Evaluate general call heads and arguments recursively
- [x] Add special-form dispatch for held assignment builtins
- [ ] Add a standard evaluation loop closer to Mathematica semantics
- [ ] Add recursion limits and cycle detection

### Assignment and definitions

- [x] Implement `Set` for simple symbol left-hand sides
- [x] Implement `SetDelayed` for simple symbol left-hand sides
- [x] Leave unsupported assignment targets unevaluated for now
- [x] Support pattern-based `SetDelayed` as stored downvalues
- [ ] Add `OwnValues` / `DownValues` inspection helpers

### Builtins and inspection

- [x] Keep a builtin registry in `Session`
- [x] Support a minimal non-held builtin dispatch path
- [x] Extend symbol inspection to show definition presence
- [x] Add builtin attribute metadata such as `HoldFirst` and `HoldAll`
- [ ] Add structured evaluation trace events

### Tests

- [x] Test simple symbol assignment and lookup
- [x] Test delayed own values
- [x] Test unsupported assignment fallback behavior
- [x] Add evaluator tests for held arguments and sequencing

## Phase 4: Rules and Pattern Matching

- [ ] Implement blank-based matcher compilation
- [ ] Add named pattern binding and environment capture
- [x] Implement `Rule`, `RuleDelayed`, `ReplaceAll`, and `ReplaceRepeated`
- [x] Add condition handling with `/;`
- [ ] Add `MatchQ`, `Cases`, and related core traversal tools

## Rules and Replacement Breakdown

### Matching core

- [x] Match simple blanks against atomic and call expressions
- [x] Bind named pattern variables during matching
- [x] Support condition-aware matching with `/;`
- [x] Support `BlankSequence` and `BlankNullSequence` across argument lists

### Rule objects

- [x] Preserve immediate vs delayed rule behavior for replacement workflows
- [x] Normalize single rules and rule lists for replacement
- [ ] Add better inspection/debug support for rule expressions

### Replacement engine

- [x] Implement `MatchQ`
- [x] Implement `ReplaceAll` with a single rule
- [x] Implement `ReplaceAll` with rule lists
- [x] Traverse nested expressions during replacement
- [x] Implement `ReplaceRepeated`

### Traversal tools

- [x] Implement `Cases` on top of the matcher subset already supported today
- [ ] Add level specifications and head-traversal options for `Cases`

## Phase 5: Scoping and Runtime Control

- [x] Add `If`, `Which`, and `Function` control-evaluation basics
- [x] Implement `Module`
- [x] Implement `With`
- [ ] Implement `Block`
- [x] Add `Evaluate` and `Sequence`
- [ ] Add execution tracing hooks and resource limits

## Control Evaluation Breakdown

- [x] Implement `If` with held branches and selected-branch evaluation
- [x] Implement `Which` with sequential condition evaluation
- [x] Implement `Function` application for `Slot`-based pure functions
- [x] Implement `With` with lexical substitution over simple binding lists
- [x] Implement `Module` with fresh lexical symbols and sequential initializers
- [ ] Support explicit parameter lists, named slots, and `##`

## Phase 6: REPL and Developer Tooling

- [ ] Upgrade the REPL to support multiline input
- [ ] Add history, completion, and symbol inspection improvements
- [ ] Add parse tree and lowering debug views for development
- [ ] Add snapshot tests for REPL behavior

## Phase 7: Embedding and Jupyter Preparation

- [ ] Define a stable host embedding API around `Session`
- [ ] Add structured diagnostics and machine-readable outputs
- [ ] Separate terminal REPL concerns from protocol-facing kernel concerns
- [ ] Create a future `rum-jupyter` crate for Jupyter kernel integration

## `rum-syntax` Detailed Breakdown

### Crate structure

- [x] Split the current prototype into `ast.rs`, `lexer.rs`, and `parser.rs`
- [x] Re-export a stable public syntax API from `lib.rs`
- [x] Keep syntax-layer concepts independent from runtime semantics where practical

### Lexing

- [x] Lex integers without width loss
- [x] Lex strings with escape handling
- [x] Lex symbols with Wolfram-style context markers
- [x] Lex delimiters for calls, lists, and grouping
- [x] Lex multi-character operators such as `:=`, `->`, `:>`, `/.`, and `/;`
- [x] Add lexer tests for ambiguous prefixes and operator boundaries

### AST

- [x] Add AST nodes for lists
- [x] Add AST nodes for prefix, infix, and postfix operators
- [x] Add AST nodes for blanks and named patterns
- [x] Add AST nodes for slots and pure functions
- [x] Establish AST display/debug helpers for readable snapshots

### Parsing

- [x] Parse grouped expressions with parentheses
- [x] Parse lists with `{...}`
- [x] Parse call syntax with nested arguments
- [x] Parse precedence for `+`, `-`, `*`, `/`, and `^`
- [x] Parse `=`, `:=`, `->`, `:>`, `/.`, and `/;`
- [x] Parse `_`, `__`, `___`, and named pattern shorthand
- [x] Parse `#`, `#n`, and `expr &`
- [x] Add parser tests for precedence, associativity, and nesting

### Follow-up syntax work

- [ ] Add spans to syntax nodes
- [ ] Add parser recovery for REPL-friendly diagnostics
- [ ] Add support for comments and additional trivia
- [ ] Add more Wolfram operators as the evaluator roadmap demands them
