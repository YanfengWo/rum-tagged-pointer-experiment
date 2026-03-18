# Runtime Evaluation Audit

This document captures the current `rum-runtime` evaluation sequence as it
exists today, and the main semantic gaps that still separate it from the
intended Mathematica-style core language.

It is an engineering audit of the current implementation, not a promise that
every listed behavior is final.

## Current Evaluation Pipeline

At a high level, `Session::eval_text` does:

1. parse source into `SyntaxExpr`
2. lower syntax into core `Expr`
3. evaluate to a fixed point
4. return the final value plus runtime messages emitted during that top-level
   evaluation

Within `Session::evaluate`, the current runtime behaves like this:

1. Repeatedly call `evaluate_once` until the expression stops changing.
2. Abort with `EvaluationPassLimitExceeded` if the fixed-point loop exceeds the
   configured pass limit.
3. Abort with `EvaluationCycleDetected` if the same expression reappears in the
   same fixed-point loop.
4. Inside `evaluate_once`, guard against recursive re-entry with an evaluation
   stack and a recursion limit.
5. Dispatch atoms and calls separately.

### Symbol Evaluation

Symbol evaluation currently supports:

- system session variables such as `$Context`, `$ContextPath`, `$Packages`,
  `$Path`, and `$MessageList`
- immediate ownvalues
- delayed ownvalues
- context-path-aware symbol resolution at parse/lowering time and inspection

The current behavior for both immediate and delayed ownvalues is:

1. look up the stored value
2. feed that stored expression back into `evaluate`

That gives `rum` a simple and predictable "definitions re-enter the standard
evaluation loop" model.

### Call Evaluation

Call evaluation currently follows this order:

1. Evaluate the head.
2. Run a staged call-dispatch plan that can inspect raw arguments, prepare
   evaluated arguments, apply attribute-driven preprocessing, and then try
   definitions, builtins, and pure functions in order.
3. Special forms still see the original raw arguments.
4. Ordinary calls evaluate arguments according to the evaluated head's hold
   attribute.
5. Ordinary-call preprocessing now handles `Sequence`, `Flat`, `Orderless`,
   and recursive `Listable` threading with mismatch messages.
6. Unless the head has `HoldAllComplete`, try upvalues attached to symbols that
   appear in the evaluated arguments.
7. Try downvalues on the evaluated head plus evaluated arguments.
8. Try subvalues keyed by the leftmost base symbol of a nested evaluated head.
9. Try builtin dispatch for non-special system builtins.
10. Try pure-function application.
11. Reconstruct the call if nothing fires.

This gives the runtime a workable precedence order for the currently supported
subset:

- head evaluation happens before almost everything else
- special forms, argument preparation, ordinary builtins, definitions, and pure
  functions now share one staged dispatch pipeline
- hold-sensitive builtins get partially evaluated arguments
- upvalues run before downvalues on ordinary calls, unless `HoldAllComplete`
  blocks them
- downvalues see evaluated arguments
- subvalues run after downvalues once the nested head has stabilized
- pure functions run after ordinary builtin dispatch

### Currently Implemented Hold Behavior

The runtime now has a first-class session-level attribute table for the current
early subset:

- `Attributes`, `SetAttributes`, and `ClearAttributes`
- `HoldAllComplete`
- `Unevaluated`
- `HoldPattern`
- `ReleaseHold`
- `Hold`, `HoldFirst`, `HoldRest`, `HoldAll`
- `SequenceHold`
- `Function[..., attrs]` for explicit application-time hold behavior
- `Evaluate` in held positions
- `Sequence` splicing in non-held positions unless `SequenceHold` or
  `HoldAllComplete` is active
- held control constructs such as `If`, `Which`, `With`, `Module`, `Block`,
  `Check`, and `Quiet`

This is enough for the current core rule language, but it is still a narrow
subset of the broader Wolfram attribute model.

### Currently Implemented Definition and Rule Behavior

The current runtime supports:

- immediate ownvalues
- delayed ownvalues
- immediate and delayed downvalues
- immediate and delayed subvalues
- immediate and delayed tagged definitions via `TagSet` / `TagSetDelayed`
- explicit tagged removal via `TagUnset`
- upvalue inspection via `UpValues`
- subvalue inspection via `SubValues`
- `Unset`, `Clear`, and `ClearAll` for the currently supported definition
  categories
- dedicated tag lifecycle messages for missing tags and invalid tag positions
- immediate and delayed rewrite rules
- compiled matcher dispatch for the currently supported pattern subset
- `ReplaceAll`, `ReplaceRepeated`, `Cases`, and `MatchQ`

`ReplaceRepeated` already has its own iteration limit, and fixed-point
evaluation now has a separate pass limit. Non-loader evaluation failures for
cycle/limit conditions also emit language-level `General::*` messages.

## What Is Already Coherent

The following parts of the evaluation model are now reasonably coherent and are
good enough to build on:

- fixed-point top-level evaluation with dedicated pass/cycle/recursion guards
- head-first call evaluation
- builtin hold handling for the implemented subset
- attribute-driven call normalization for `Sequence`, `Flat`, `Orderless`, and
  recursive `Listable` threading
- `Evaluate` + `Sequence` interaction in held vs non-held positions
- `OneIdentity`-aware matching for singleton-head patterns
- upvalue application ahead of downvalues for ordinary calls
- downvalue application after argument evaluation
- subvalue application for nested-head expressions such as `f[x_][y_]`
- immediate vs delayed rule behavior
- lexical scoping via `With` and `Module`
- dynamic rebinding via `Block` for definitions, attributes, and messages on
  localized symbols
- runtime messages, `Check`, `Quiet`, and `$MessageList`

These pieces are not "complete Mathematica semantics", but together they form a
usable and internally consistent core for rule-driven DSL work.

## Known Semantic Gaps

The remaining gaps fall into a few clear buckets.

### 1. Attribute Model Is Still an Early Subset

Missing or incomplete areas:

- richer inspection and lifecycle coverage around user-defined attribute state
- fuller `Listable` semantics beyond the current recursive threading,
  scalar-broadcast, and `Thread::tdlen` mismatch subset

Practical impact:

- matcher and evaluator normalization now cover a useful subset, including a
  practical recursive `Listable` implementation, but not the broader Wolfram
  attribute surface
- sequence behavior is much better aligned with the intended long-term language
  model than before, but still not exhaustive

### 2. Definition Precedence Is Still a Narrow Slice

The current evaluator has a workable precedence order for ownvalues,
upvalues, downvalues, subvalues, builtins, and pure functions, but it does not
yet model the wider definition lattice.

Missing or incomplete areas:

- broader tag-based definition attachment such as multi-owner tag propagation
- more data-driven handler registration on top of the current staged dispatch
  pipeline

Practical impact:

- common argument-owned protocols can now be modeled with `TagSet` /
  `TagSetDelayed`, but deeper multi-owner dispatch is still absent
- host-side DSL embedding still cannot rely on the full Wolfram definition
  lattice, especially where nested-head behavior must interact with future
  `UpValues` refinements and richer tag operations
- the call path is now more uniform, but handler registration is still partly
  handwritten

### 3. Held Evaluation Is Functional but Not Yet Rich

Held evaluation currently covers the main control constructs, but not the
broader set of non-evaluation wrappers and held-pattern tools.

Missing or incomplete areas:

- more exact interaction rules among held wrappers, pattern matching, and
  substitution

Practical impact:

- some advanced rule authoring patterns still require awkward workarounds
- held pattern matching is less faithful than the intended core-language target
- host DSL authors can now choose function application attributes explicitly,
  but the surrounding held-wrapper rules are still not complete

### 4. Scoping Is Good Enough for Core Use, but Not Yet Exhaustive

`With`, `Module`, and `Block` are implemented, but they still represent a
minimal semantics slice rather than a full scoping model.

Known gaps:

- `Block` now localizes definitions, user attributes, and message templates for
  the named symbols it rebinds, but the runtime still lacks broader categories
  such as options/format values
- `Module` fresh-name allocation now skips pre-existing generated symbols, but
  supporting machinery such as `$ModuleNumber` is absent
- nested interactions among scoping, held patterns, and future attribute-driven
  evaluation have not been fully audited yet

Practical impact:

- the current behavior is strong enough for many rule DSL scenarios
- deeper meta-programming workflows may still expose capture or rebinding edge
  cases

### 5. Evaluation Sequencing Is Still Somewhat Ad Hoc Around Builtins

The runtime now has a real evaluation loop, but some sequencing rules are still
encoded directly in handlers rather than through a more general semantic model.

Known gaps:

- special forms and ordinary builtins now share one staged dispatch path, but
  builtin metadata still does not drive all evaluator decisions
- the dispatch stages are explicit, but handler registration is still hardcoded
- future features can still accidentally lean on implicit ordering inside the
  handwritten stage handlers

Practical impact:

- adding new builtins still requires careful manual placement in the dispatch
  order
- it is easy for future features to accidentally depend on implicit handler
  ordering

### 6. Pattern and Rule Semantics Still Cover Only the Intended Early Subset

The matcher IR is in place, but it still targets the current early-language
subset.

Missing or incomplete areas:

- richer pattern forms such as `Alternatives`, `Repeated`, `Optional`,
  `PatternTest`, `Except`, `Longest`, and `Shortest`
- tighter integration between held-pattern constructs and matcher lowering
- broader traversal controls beyond the currently implemented `Cases` subset

Practical impact:

- rule authoring is already useful, but still not expressive enough for many
  mature Wolfram-style idioms
- some business-rule authoring patterns will remain verbose until the matcher
  language grows

## Priority Order Suggested by This Audit

The cleanest next steps for `rum-runtime` are:

1. keep consolidating evaluator metadata so special forms and ordinary builtins
   sit on a more data-driven dispatch model instead of handwritten handler
   registration
2. extend tag-oriented lifecycle tools beyond `Unset` / `Clear` / `ClearAll`
   into richer multi-owner tag behavior
3. extend the attribute table from the current
   hold/order/associativity/list-threading subset into later families
4. continue auditing scoping and held-wrapper interactions now that
   `Function[..., attrs]`, broader `Block`, and safer `Module` freshness are in
   place
5. widen the matcher language only after the current evaluation and scoping
   semantics feel fully locked down

## Summary

`rum-runtime` now has a real evaluation core rather than a bootstrap echo
interpreter. The remaining work is no longer "make evaluation exist"; it is
"make the evaluator less ad hoc and more semantic."

That is a good place to be. The implementation is already strong enough to
support meaningful rule-based DSL experimentation, and the gaps are now narrow
enough to attack in a deliberate order.
