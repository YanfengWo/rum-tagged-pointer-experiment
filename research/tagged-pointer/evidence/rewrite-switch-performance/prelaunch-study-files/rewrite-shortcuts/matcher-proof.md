# Narrow rewrite matching: semantic argument and validation boundary

This argument concerns the safe Rust shortcut for an unguarded compiled
`Blank` or `Named { inner: Blank }` at a fresh rewrite-rule candidate. It does
not prove the complete matcher, allocator, continuation system, or the
independent tagged-pointer representation. No new unsafe code is involved.
The regression source contains seven test functions and one value helper;
it is appended unchanged to each disposable implementation's runtime tests.

## Entry and result relation

The original rewrite scanner first normalizes its candidate with
`normalize_match_expr`, scans its existing `RewriteRulePlan` cursor, and
retains the conservative shape prefilter. Matching begins with an empty
`SymbolExprMap`. A successful shortcut must feed the same existing RHS
substitution, rule-applied trace, traversal continuation, and eventual
evaluation code as an ordinary successful match. A miss continues the same
cursor. Unsupported matcher forms fall back before producing a result.

For `Blank { head_constraint }`, the general matcher calls `match_blank`
and returns the unchanged empty bindings. The shortcut uses that same
predicate. It must not replace the predicate with an integer-only test:
rum supports untyped blanks and structural custom-head constraints too.
For a named blank, the original matches its blank child, then invokes
`bind_pattern_name(name, value.clone(), bindings)`. With empty initial
bindings, a successful predicate therefore yields exactly the singleton
map `{name -> normalized_candidate.clone()}`. A failed predicate yields
empty bindings. The shortcut can construct this same map directly.

The ordinary `BindingSnapshot` at these two levels records names absent
from the initial map. Its successful path leaves the singleton map intact;
its failed path restores the empty map. It cannot expose a different
binding to the scanner in this restricted domain. This reasoning does not
license dropping snapshots in nested matching, alternatives, repeated
names, or a caller supplying nonempty initial bindings.

## Evaluation, guards, and suspension

The supported Blank and Named/Blank branches do not evaluate expressions,
call user code, poll interrupts, enforce resource limits, increment the
orderless match-depth guard, or create suspension frames. Their predicate
is structural. Eliminating their temporary snapshots therefore does not
skip one of those operations in these branches.

The shortcut must not unwrap `Condition`, `DefinitionCondition`, or
`PatternTest` in order to reach an inner blank. RHS guards are compiled as
`DefinitionCondition`, and may run user code, decline a candidate, or
suspend for host input. Immediate and delayed RHS evaluation remains in
the existing rule-construction and post-rewrite evaluation paths. The
shortcut may not eagerly evaluate an accepted RHS inside the scanner:
`Replace` rebuilds a traversal before its final evaluation, and eager RHS
evaluation can change observable side-effect order.

The suspended scanner retains its current rule index, cursor, original
expression, normalized candidate, and plan. Resumption must not redo a
previous guard, a matched prefix, or an already evaluated RHS. Complex
guarded candidates stay on the original resumable matcher. The regression
suite compares queued input and two actual suspend/resume cycles, including
a declining guard followed by a narrow fallback rule.

## Why the existing general definition shortcut is not the whole proof

`try_fast_definition_match_compiled` handles recursive Call patterns as
well as simple blanks. Its Call arm rejects non-call values and differing
argument counts before consulting attributes. The general matcher can
instead promote an atom or a different-head call under `OneIdentity`, and
has Flat grouping, Orderless selection, and optional/default semantics.
Its attribute decisions also depend on the compiled pattern head. Those
Call cases need a separate proof; they must not be silently accepted as a
safe extension of the two-node argument above. Repeated names and failed
alternatives need restoration of earlier bindings. The seven regressions
include representative fallback boundaries rather than attempting a full
matcher conformance suite.

## Preserved observations and limits of this claim

Within the restricted entry relation, the argument preserves the match
boolean, binding values, rule priority, substituted expression, existing
rule-applied trace, ordinary evaluation order, and the presence of later
capability/resource checks. Tests explicitly cover a denied interactive
guard and recursion termination in an accepted RHS. These are checks of
reachable boundaries, not proofs for every host cancellation schedule or
resource setting. The existing complete workspace tests remain necessary.

Snapshot allocation, temporary ownership counts, and execution time are
intentionally different. There is no claim of identical allocator history,
RSS, wall time, or exact success/failure at every time or memory threshold.
Unlike the prior Flat capacity prototype, this shortcut does not change
the final expression's `SharedArgs` vector or its retained capacity; that
fact does not prove arbitrary resource-context equivalence. No resource
poll may be deleted from the surrounding evaluator. A broader shortcut
that crosses a user-code call, explicit resource check, or match-depth
guard requires renewed analysis and regression coverage.

Correctness checks establish no performance improvement. Only the
uninstrumented same-machine rum-versus-rum gate, with all per-label
regressions retained and reported, can establish a timing benefit.
