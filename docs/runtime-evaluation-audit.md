# Runtime Evaluation Audit

Status: refreshed 2026-08-23 against committed runtime code through `d97abd7`.
Uncommitted performance candidates are deliberately excluded.

This document maps the current evaluator implementation and its explicit
boundaries. It is not a second language specification: the language guide, the
generated builtin reference, and `rum-host` rustdoc are the normative contract.
Architecture, testing policy, and active sequencing live in
[ARCHITECTURE.md](../ARCHITECTURE.md),
[testing-strategy.md](./testing-strategy.md), and [TODO.md](../TODO.md).

## Request And Evaluation Pipeline

For a source request the runtime:

1. parses source into `rum-syntax` nodes under lexer/parser depth guards;
2. lowers syntax into canonical `rum-core::Expr` values using the session's
   context state;
3. evaluates each expression through the kernel fixed-point loop;
4. returns a structured response containing the final value, messages, prints,
   traces, termination classification, or an input suspension/failure.

`rum-cli`, `rum-repl`, `rum-host`, and `rum-jupyter` ultimately use this same
kernel request machinery. Terminal rendering and protocol envelopes are not
alternate evaluators.

Within evaluation, the kernel:

- repeatedly evaluates unstable expressions until they become stable;
- detects same-epoch re-entry cycles and enforces the evaluation-pass limit;
- guards evaluator recursion and periodically checks native-stack consumption;
- polls armed cancellation and elapsed-time deadlines at amortized points;
- enforces active generated-element, integer-work, iteration-work, and modeled-
  memory limits;
- dispatches symbols, calls, and stable non-symbol atoms separately.

Stable atoms and symbols use fast paths backed by session-local stability and
definition-version caches. Diagnostic counters/timers are env-gated and do not
add allocation or locking to the default path.

## Symbol And Definition Evaluation

Symbol evaluation covers session variables, immediate and delayed ownvalues,
context-path resolution, removed-symbol generations, and sealed identities.
Stored ownvalues re-enter ordinary evaluation rather than using a separate
mini-evaluator.

Definitions are stored by category and applied in the kernel's documented
precedence: argument-owned UpValues, head-owned DownValues, and nested-head
SubValues. Immediate and delayed definitions share matching machinery but keep
their distinct right-hand-side evaluation rules. `TagSet`, `TagUnset`,
`UpSet`, `Clear`, `ClearAll`, `Remove`, attributes, options, defaults, message
definitions, and inspection all operate on the same session definition state.

Wide definition tables use derived dispatch metadata and adaptive indexes for
ground candidates while retaining ordered fallback scans for patterns and small
tables. The indexes are cache data, not semantic state: definition order and
first-applicable-rule behavior remain authoritative.

## Call Dispatch

Call evaluation is staged:

1. evaluate the head;
2. build a `CallDispatchPlan` from builtin metadata, attributes, definition
   presence, and callable-head traits;
3. dispatch special forms against raw arguments;
4. otherwise prepare arguments according to hold attributes and normalize
   `Evaluate`, `Unevaluated`, `Sequence`, `Flat`, and `Orderless` behavior;
5. apply recursive `Listable` threading when requested;
6. unless `HoldAllComplete` blocks them, try UpValues;
7. try DownValues, then SubValues;
8. dispatch an ordinary registered builtin;
9. try callable heads such as pure functions, compositions, associations, and
   registered operator wrappers;
10. reconstruct the call when no stage produces a result.

Capability policy is checked at registered-builtin dispatch. This keeps policy
enforcement effective for dynamically constructed calls and keeps protocol or
source-text scanning out of the security boundary.

## Matching, Replacement, And Traversal

The compiled matcher supports the pattern forms documented in the reference,
including blanks and sequence blanks, named patterns, conditions, tests,
alternatives, optionals/defaults, exceptions, repetitions, pattern sequences,
`Verbatim`, shortest/longest selection, and the implemented `Flat`,
`Orderless`, and `OneIdentity` interactions.

The same matcher feeds definitions and the public matching/replacement family:
`MatchQ`, `Cases`, `DeleteCases`, `Count`, `MemberQ`, `FreeQ`, `Position`,
`Replace`, `ReplaceAll`, `ReplaceRepeated`, `ReplaceList`, `ReplaceAt`, and
association-aware traversal. Rule order, levels, head inclusion, held forms,
and result limits follow each builtin's generated reference entry.

Post-`v0.1.0` changes replaced many suffix copies and repeated wide scans with
cursors, shared rule tables, prefilters, tries, and adaptive indexes. These are
derived execution plans only; they do not change the documented result order or
matcher semantics. The remaining performance candidates are listed in
[TODO.md](../TODO.md).

## Normalization Boundaries

Two input-normalization families prevent held syntax from being confused with
resolved values:

- resolved-argument normalization evaluates the portions a builtin contract
  designates before compiling a traversal, loader, threading, attribute, or
  option specification;
- raw-form normalization parses held or syntax-shaped inputs such as assignment
  targets, definition owners, scoping binding lists, message selectors, and
  operator forms without ordinary argument evaluation.

Shared normalizers and planners are preferred to builtin-local parsing. A new
held/spec-driven builtin should join one of these families rather than create a
third evaluation convention.

## Suspension And Host Input

Suspension is kernel infrastructure. An evaluation that reaches `Input` or
`InputString` stores typed continuation frames and returns a structured input
request. `rum-host` resumes a single-source request with
`resume_with_input`; it does not replay the source.

Continuation state now covers the public callback-loop families exercised by
folds, windows, functional iteration, targeted mapping, tensors, strings,
matching/replacement, tracing, repeated timing, test reports, resource
constraints, and `Reap` handlers. State is carried as typed values rather than
encoded into rendered expressions.

Multi-form `eval_script`/`eval_file` requests support pre-queued input. If the
queue is empty and a script form suspends, the script-shaped request currently
cannot resume through `resume_with_input`; remaining forms are not evaluated.
That limitation is documented in the embedding guide.

## Scoping And Mutable State

`With` performs lexical substitution, `Module` mints fresh session identities,
and `Block` dynamically restores the documented symbol-state categories.
`DefinePackage` builds a sealed unit whose private symbols are not entered into
ordinary name-resolution tables; snapshot restore re-registers the unit and
re-links permitted leaked references.

Assignment supports symbols, definition forms, associations, exact `Part`
targets, and positional list `Part` targets. Incremental mutation uses
copy-on-write only when ownership and evaluator-cache proofs make in-place
updates semantically equivalent; otherwise it follows the rebuilding path.

## Current Boundaries And Known Residuals

- Numeric atoms are exact integers and rationals. There is no floating-point
  atom or host typed-float contract.
- `Factor` is intentionally a bounded rational-polynomial helper: it extracts
  monomial factors and recursively discovered linear factors. Complete
  factorization is deferred until finite-field polynomial support is designed
  as a coherent subsystem.
- `$ModuleNumber` is not part of the public surface. Freshness is provided by
  session identity allocation rather than a writable language counter.
- The same-expression cycle guard can reject state-progressing recursive
  definitions whose visible call shape repeats while other session state moves.
  Such flows need a more explicit progress contract before relaxing the guard.
- `Monitor` refresh and `ValueQ` rollback remain best-effort helper behavior
  outside their documented result contracts.
- A host can construct a pathologically deep `Expr` directly through the Rust
  API and overflow the native stack during recursive drop. Source-built trees
  are guarded; directly built trees must be depth-limited by the host.

These are implementation or product boundaries, not compatibility gaps against
another engine. Promote a boundary only by updating the rum documentation and
tests in the same change.
