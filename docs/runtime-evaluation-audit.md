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
2. If the evaluated head is a system special form, run its dedicated handler on
   the original arguments.
3. Otherwise evaluate arguments according to the evaluated head's hold
   attribute.
4. Splice `Sequence[...]` in non-held positions.
5. Unless the head has `HoldAllComplete`, try upvalues attached to symbols that
   appear in the evaluated arguments.
6. Try downvalues on the evaluated head plus evaluated arguments.
7. Try builtin dispatch for non-special system builtins.
8. Try pure-function application.
9. Reconstruct the call if nothing fires.

This gives the runtime a workable precedence order for the currently supported
subset:

- head evaluation happens before almost everything else
- special forms get raw syntax-level arguments
- hold-sensitive builtins get partially evaluated arguments
- upvalues run before downvalues on ordinary calls, unless `HoldAllComplete`
  blocks them
- downvalues see evaluated arguments
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
- immediate and delayed tagged definitions via `TagSet` / `TagSetDelayed`
- upvalue inspection via `UpValues`
- `Unset`, `Clear`, and `ClearAll` for the currently supported definition
  categories
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
- `Evaluate` + `Sequence` interaction in held vs non-held positions
- upvalue application ahead of downvalues for ordinary calls
- downvalue application after argument evaluation
- immediate vs delayed rule behavior
- lexical scoping via `With` and `Module`
- dynamic rebinding via `Block` for ownvalue-style symbol values
- runtime messages, `Check`, `Quiet`, and `$MessageList`

These pieces are not "complete Mathematica semantics", but together they form a
usable and internally consistent core for rule-driven DSL work.

## Known Semantic Gaps

The remaining gaps fall into a few clear buckets.

### 1. Attribute Model Is Still an Early Subset

Missing or incomplete areas:

- `Orderless`
- `Flat`
- `OneIdentity`
- `Listable`
- richer inspection and lifecycle coverage around user-defined attribute state

Practical impact:

- evaluation order cannot yet be customized for host-defined DSL constructs
- matcher normalization is still unaware of order/associativity attributes
- sequence behavior is more rigid than the intended long-term language model

### 2. Definition Precedence Is Still a Narrow Slice

The current evaluator has a workable precedence order for ownvalues,
downvalues, builtins, and pure functions, but it does not yet model the wider
definition lattice.

Missing or incomplete areas:

- `SubValues`
- broader tag-based definition attachment such as `TagUnset` / `TagUnsetDelayed`
- a more declarative precedence table for user definitions vs builtin dispatch

Practical impact:

- common argument-owned protocols can now be modeled with `TagSet` /
  `TagSetDelayed`, but deeper multi-owner dispatch is still absent
- host-side DSL embedding still cannot rely on the full Wolfram definition
  lattice
- lifecycle management is usable now, but still narrower than the long-term
  target

### 3. Held Evaluation Is Functional but Not Yet Rich

Held evaluation currently covers the main control constructs, but not the
broader set of non-evaluation wrappers and held-pattern tools.

Missing or incomplete areas:

- `Function[..., attrs]`
- more exact interaction rules among held wrappers, pattern matching, and
  substitution

Practical impact:

- some advanced rule authoring patterns still require awkward workarounds
- held pattern matching is less faithful than the intended core-language target
- host DSL authors cannot yet choose function application attributes explicitly

### 4. Scoping Is Good Enough for Core Use, but Not Yet Exhaustive

`With`, `Module`, and `Block` are implemented, but they still represent a
minimal semantics slice rather than a full scoping model.

Known gaps:

- `Block` currently rebinds ownvalue-style symbol values, not the broader space
  of attributes/messages/other definition categories
- `Module` freshness is based on generated names, but supporting machinery such
  as `$ModuleNumber` is absent
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

- special forms vs held builtins are split across separate dispatch paths
- the evaluator does not yet expose a single declarative model for
  "head evaluation, attribute preprocessing, definition lookup, builtin
  dispatch, and residual reconstruction"
- more of the behavior is encoded in handwritten handler order than in reusable
  evaluator metadata

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

1. expand from `UpValues` toward `SubValues` and the rest of the definition lattice
2. make builtin dispatch order more declarative so new semantics do not depend
   on ad hoc handler placement
3. extend the attribute table beyond hold-oriented behavior into `Orderless`,
   `Flat`, `OneIdentity`, and later `Listable`
4. let `Function` accept explicit attributes / hold behavior so held evaluation
   is not limited to builtin and symbol-level attributes
5. extend regression tests around head evaluation, held arguments, scoping, and
   delayed definitions before widening the pattern language further

## Summary

`rum-runtime` now has a real evaluation core rather than a bootstrap echo
interpreter. The remaining work is no longer "make evaluation exist"; it is
"make the evaluator less ad hoc and more semantic."

That is a good place to be. The implementation is already strong enough to
support meaningful rule-based DSL experimentation, and the gaps are now narrow
enough to attack in a deliberate order.
