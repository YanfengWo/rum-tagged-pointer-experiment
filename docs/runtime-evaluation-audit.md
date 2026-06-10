# Runtime Evaluation Audit

This document captures the current `rum-runtime` evaluation model and the main
semantic gaps that still separate it from the intended Mathematica /
Wolfram-style core language.

It is an engineering audit of the current implementation, not a promise that
every listed behavior is final.

This document focuses on evaluator semantics. Architecture, code-organization
rules, and roadmap sequencing are tracked separately in:

- `../ARCHITECTURE.md`
- `./code-organization-guidelines.md`
- `../TODO.md`

## Current Evaluation Pipeline

At a high level, `Session::eval_text` does:

1. parse source into `SyntaxExpr`
2. lower syntax into canonical core `Expr`
3. evaluate to a fixed point
4. return the final value plus runtime messages emitted during that top-level
   evaluation

The same evaluator core is also exposed through host-facing request APIs, so
CLI, REPL, tests, and protocol adapters no longer need distinct execution
paths.

Within `Session::evaluate`, the runtime currently behaves like this:

1. repeatedly call `evaluate_once` until the expression stops changing
2. abort with `EvaluationPassLimitExceeded` if the fixed-point loop exceeds the
   configured pass limit
3. abort with `EvaluationCycleDetected` if the same expression reappears in the
   same fixed-point loop
4. guard recursive re-entry with an evaluation stack and recursion limit
5. enforce current host resource budgets for selected generated outputs,
   integer work loops, and iteration work loops
6. dispatch atoms and calls separately

### Symbol Evaluation

Symbol evaluation currently supports:

- system session variables such as `$Context`, `$ContextPath`, `$Packages`,
  `$Path`, and `$MessageList`
- immediate ownvalues
- delayed ownvalues
- context-path-aware symbol resolution at lowering and inspection time

The current model for both immediate and delayed ownvalues is:

1. look up the stored value
2. feed that stored expression back into `evaluate`

That gives `rum` a predictable "definitions re-enter the normal evaluation
loop" model.

### Call Evaluation

Call evaluation currently follows this order:

1. evaluate the head
2. assemble a staged dispatch plan using builtin metadata, attributes, and head
   traits
3. let special forms inspect the original raw arguments
4. evaluate ordinary arguments according to the evaluated head's hold behavior
5. run attribute-driven preprocessing such as `Sequence`, `Flat`, `Orderless`,
   recursive `Listable`, and shared `Thread` helpers
6. unless blocked by `HoldAllComplete`, try upvalues attached to evaluated
   arguments
7. try downvalues on the evaluated head plus evaluated arguments
8. try subvalues keyed by the leftmost base symbol of a nested evaluated head
9. try ordinary builtin dispatch for non-special system heads
10. try pure-function application when the head is itself `Function[...]`
11. reconstruct the call if nothing fires

This gives the runtime a workable precedence order for the current subset:

- head evaluation happens before almost everything else
- special forms, argument preparation, definitions, builtins, and pure
  functions share one staged pipeline
- builtin stage assembly is metadata-backed rather than entirely handwritten
- upvalues run before downvalues on ordinary calls unless `HoldAllComplete`
  blocks them
- downvalues see evaluated arguments
- subvalues run after downvalues once the nested head is stable
- pure functions run after ordinary builtin dispatch

## Normalization And Suspension Boundaries

Two design boundaries now matter a lot to evaluator coherence.

### 1. Normalization Families

The runtime now distinguishes between:

- resolved-argument normalization
  - for builtins that intentionally evaluate part of their input before
    compiling an internal config
  - examples: `Thread`, traversal config, loader config, attribute updates
- raw-form normalization
  - for builtins whose inputs are held or syntax-shaped and must be interpreted
    without ordinary evaluation
  - examples: message selectors, definition targets, assignment lhs
    classification, and scoping binding lists

This is important because it keeps the runtime from drifting back toward ad hoc
per-builtin parsers.

### 2. Suspension / Resume

Host-driven input is now part of the evaluator model rather than a front-end
shim.

That includes:

- `InputString` backed by a real runtime input facility
- request suspension and continuation-based resume instead of full request replay
- typed internal continuation support for helper paths that do not naturally
  return `Expr`

This machinery now reaches not only ordinary evaluation but also matcher and
inspection-related helper paths in the currently covered subset.

## What Is Already Coherent

The following parts of the evaluation model are now coherent enough to build on:

- fixed-point top-level evaluation with separate pass, cycle, and recursion
  guards
- head-first staged call evaluation
- metadata-backed builtin dispatch planning
- attribute-driven call normalization for `Sequence`, `Flat`, `Orderless`,
  recursive `Listable`, and `Thread`
- `Thread[expr]`, `Thread[expr, head]`, and the implemented third-argument
  sequence-spec subset
- `Evaluate` and `Unevaluated` interaction in held and non-held positions
- `HoldPattern`, `ReleaseHold`, and explicit function application attributes
- upvalues, downvalues, subvalues, tagged definitions, `TagUnset`, `Clear`,
  `ClearAll`, and definition inspection for the supported categories
- immediate vs delayed rule behavior
- compiled matcher dispatch for the current rule/pattern subset, including
  `Condition`, sequence blanks, `PatternTest`, `Alternatives`, `Optional`,
  `Except`, `Repeated`, `PatternSequence`, `OrderlessPatternSequence`,
  `Verbatim`, `Longest`, `Shortest`, `MatchQ`, `Cases`, `DeleteCases`,
  `Count`, `MemberQ`, `FreeQ`, `Position`, `Replace`, `ReplaceAll`,
  `ReplaceRepeated`, and current first passes of `ReplaceList` and `ReplaceAt`
- lexical scoping via `With` and `Module`
- dynamic rebinding via `Block` for the currently localized categories
- runtime messages, `Check`, `Quiet`, and `$MessageList`
- request-oriented input suspension/resume for host-driven execution

These pieces do not yet amount to "full Wolfram semantics", but together they
form a real, internally consistent core-language runtime.

## Known Semantic Gaps

The remaining gaps are now narrower and more specific than they were earlier in
the project.

### 1. Attribute Model Is Still An Early Subset

Missing or incomplete areas include:

- broader attribute families beyond the current hold / flattening /
  associativity / threading subset
- richer lifecycle and inspection coverage around user-defined attributes
- fuller `Listable` semantics beyond the currently implemented recursive
  threading and mismatch-message subset

Practical impact:

- the current attribute model is already useful, but still far from the broader
  Wolfram attribute surface

### 2. Definition Lattice Is Usable But Not Exhaustive

The evaluator now has a workable precedence order for ownvalues, upvalues,
downvalues, subvalues, builtins, and pure functions, but it still does not
model the full broader definition lattice. The `CL-02` mainline is closed for
the current core-language target, with remaining work treated as long-tail
parity rather than an active blocker.

Missing or incomplete areas include:

- richer tag-based ownership and attachment edge cases beyond the current
  `TagSet` / `UpSet` subset
- definition lhs normalization still misses non-pattern own-value constants in
  held lhs positions. Wolfram evaluates `p = a; f[p] := rhs` into a definition
  for `f[a]`; the latest `mma-hol` probe showed that `rum` currently leaves the
  literal symbol in the stored pattern for shapes such as
  `const["=", $boolEqTy]`.
- exact front-end-style definition introspection and print-layer rendering
- broader metadata-driven semantics so fewer evaluator decisions remain
  handwritten

Practical impact:

- common argument-owned protocols are possible, while deeper multi-owner,
  print-layer, and future richer precedence cases remain explicit long-tail
  work

### 3. Held Evaluation Is Functional But Not Exhaustive

Held evaluation now covers the core control and wrapper subset, but not the
full intended language.

Missing or incomplete areas include:

- more exact interactions among held wrappers, pattern matching, and
  substitution
- deeper audit of wrappers that should preserve structure more faithfully

Practical impact:

- advanced metaprogramming patterns can still hit rough edges even though the
  main control-flow subset is usable

### 4. Scoping Semantics Are Good Enough For Core Use, But Not Yet Exhaustive

`With`, `Module`, and `Block` are implemented and useful, but still represent a
core slice rather than a fully mature scoping system.

Known gaps include:

- `Block` now localizes the currently covered symbol-state slice, including the
  current option / syntax-information subset, but broader state categories are
  still not exhaustive
- `Module` freshness is much better than before, but supporting machinery such
  as `$ModuleNumber` is still absent
- deeper interactions among scoping, held patterns, and future attribute-driven
  behavior have not been fully audited

### 5. Evaluator Sequencing Still Has Some Ad Hoc Edges

The evaluator is much less ad hoc than it used to be, but some sequencing rules
and builtin-specific paths still need further consolidation.

Known gaps include:

- not every evaluator decision is yet driven by unified metadata or shared
  normalization helpers
- the same-expression cycle guard is too eager for state-progressing recursive
  definitions. Hand-written parser functions such as `parseTypeExpr[]` consume
  input state before recursively calling themselves, but the current guard sees
  only a re-entered call shape and reports a cycle.
- some remaining held/spec-driven families still need the same normalization
  treatment already applied to messages, loaders, definitions, scoping, and
  traversal config

### 6. Non-`Expr` Helper Suspension Audit Is Not Fully Closed

Typed suspension/resume is now real kernel infrastructure, but the audit is not
fully complete.

Known gap:

- some remaining inspection/debug helper paths may still need to be pulled onto
  the same typed continuation driver that now covers the currently audited
  subset

### 7. Pattern And Traversal Semantics Still Cover Only The Current Early Subset

The matcher is no longer minimal, but it still does not aim at the full
language yet.

Missing or incomplete areas include:

- deeper pattern-language breadth and pattern/string crossover beyond the
  current `PatternSequence` / `OrderlessPatternSequence` / `Verbatim` /
  `Longest` / `Shortest` subset
- broader traversal semantics beyond the current association-aware `Cases` /
  `DeleteCases` / `Level` / `Count` / `MemberQ` / `FreeQ` / `Position` slice
- remaining replacement-family parity beyond the current `Replace` /
  `ReplaceList` / `ReplaceAt` first-pass subset
- later integration work with future string-pattern semantics

Practical impact:

- rule authoring is already useful across matcher, replacement, and tree-walker
  flows, but still not expressive enough for many mature Wolfram-style idioms

## Priority Order Suggested By This Audit

The cleanest next steps for `rum-runtime` are:

1. continue tightening evaluator semantics and reducing the remaining ad hoc
   sequencing edges
2. continue auditing remaining raw-form / held-spec builtin families so they
   join the shared normalization model
3. finish auditing any remaining non-`Expr` helper paths for typed
   suspension/resume
4. widen matcher and traversal semantics only after the current evaluator
   boundaries feel fully locked down
5. strengthen verification and conformance coverage as the semantic surface
   expands

## Summary

`rum-runtime` is now a real evaluator core rather than a bootstrap interpreter.
The remaining work is no longer "make evaluation exist"; it is "make the
evaluator less ad hoc, more explicit, and more complete."

That is a good place to be. The implementation is already strong enough to
support meaningful rule-based DSL experimentation, and the remaining gaps are
now specific enough to attack deliberately.
