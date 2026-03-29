# Lowering Canonical Forms

This document records the current syntax-to-`Expr` lowering contract.

The purpose of lowering is to normalize parser sugar into a smaller, stable
core-language `Expr` surface before evaluation. Lowering is not evaluation:
it should preserve direct calls when no syntax sugar needs rewriting, and it
should avoid baking in runtime-only validation that belongs later in the
evaluator.

This document is not an exhaustive builtin catalog. It records the canonical
surface that lowering itself is responsible for producing or preserving.

## Related docs

- `../ARCHITECTURE.md` explains where lowering sits in the crate/layer model
- `./runtime-evaluation-audit.md` tracks evaluator sequencing and semantic gaps
  that intentionally remain outside lowering
- `../TODO.md` tracks the remaining roadmap around lowering, evaluator cleanup,
  and later host-facing API work

## Current rule of thumb

- direct symbols lower to `Expr::Atom(Atom::Symbol(...))`
- integers lower to exact `BigInt` atoms
- strings lower to string atoms
- ordinary calls preserve their head and recursively lower arguments
- direct calls that already use canonical runtime heads are preserved
- parser sugar rewrites into canonical system-head calls

## Canonical forms in the current implementation

### Structure

- `{a, b}` -> `List[a, b]`
- `<|a -> 1, b -> 2|>` -> `Association[Rule[a, 1], Rule[b, 2]]`
- `File["demo.rum"]` -> `File["demo.rum"]`

### Patterns and slots

- `x_Integer` -> `Pattern[x, Blank[Integer]]`
- `x__` -> `Pattern[x, BlankSequence[]]`
- `x___` -> `Pattern[x, BlankNullSequence[]]`
- `x_ /; test[x]` -> `Condition[Pattern[x, Blank[]], test[x]]`
- `#` -> `Slot[1]`
- `#3` -> `Slot[3]`
- `#name` -> `Slot["name"]`
- `##` -> `SlotSequence[1]`
- `##2` -> `SlotSequence[2]`
- `expr &` -> `Function[expr]`

Pattern-language constructs that are already written as canonical calls are not
rewritten further. For example:

- `Alternatives[a, b]` stays `Alternatives[a, b]`
- `PatternTest[p, test]` stays `PatternTest[p, test]`
- `Optional[p]` stays `Optional[p]`
- `Except[p]` stays `Except[p]`
- `Repeated[p]` stays `Repeated[p]`
- `RepeatedNull[p]` stays `RepeatedNull[p]`

### Assignment and rewrite sugar

- `lhs = rhs` -> `Set[lhs, rhs]`
- `lhs := rhs` -> `SetDelayed[lhs, rhs]`
- `lhs =.` -> `Unset[lhs]`
- `lhs -> rhs` -> `Rule[lhs, rhs]`
- `lhs :> rhs` -> `RuleDelayed[lhs, rhs]`
- `expr /. rules` -> `ReplaceAll[expr, rules]`
- `expr //. rules` -> `ReplaceRepeated[expr, rules]`
- `tag /: lhs = rhs` -> `TagSet[tag, lhs, rhs]`
- `tag /: lhs := rhs` -> `TagSetDelayed[tag, lhs, rhs]`
- `tag /: lhs =.` -> `TagUnset[tag, lhs]`

### Arithmetic normalization

- `f @ x` -> `f[x]`
- `f @@ expr` -> `Apply[f, expr]`
- `f @@@ expr` -> `MapApply[f, expr]`
- `f @* g` -> `Composition[f, g]`
- `f /* g` -> `RightComposition[f, g]`
- `a + b` -> `Plus[a, b]`
- `a - b` -> `Plus[a, Times[-1, b]]`
- `a * b` -> `Times[a, b]`
- `a / b` -> `Times[a, Power[b, -1]]`
- `a ^ b` -> `Power[a, b]`
- `-x` -> `Times[-1, x]`
- unary `+x` lowers to `x`

### Messages, contexts, and package constructs

- `FindFile::notfound` -> `MessageName[FindFile, "notfound"]`
- `FindFile::"notfound"` -> `MessageName[FindFile, "notfound"]`
- `Context["Identity"]` -> `Context["Identity"]`
- `Begin["Demo`"]` -> `Begin["Demo`"]`
- `BeginPackage["Demo`"]` -> `BeginPackage["Demo`"]`
- `Needs["Demo`"]` -> `Needs["Demo`"]`
- `Get["demo.rum"]` -> `Get["demo.rum"]`

### Scoping forms

The scoping heads themselves are not rewritten, but binding lists are lowered
recursively because list literals and assignments inside them are syntax sugar.

- `With[{x = 1}, body]` -> `With[List[Set[x, 1]], body]`
- `Module[{x = 1}, body]` -> `Module[List[Set[x, 1]], body]`
- `Block[{x = 1}, body]` -> `Block[List[Set[x, 1]], body]`

More generally, lowering prefers head-preserving behavior for runtime-facing
forms that are already expressed directly. It rewrites only the syntax sugar
needed to reach the canonical core surface.

## Validation boundary

Lowering and runtime validation intentionally stop at different layers.

### Lowering-time validation

Lowering should reject only issues that are local to the syntax-to-`Expr`
translation itself:

- malformed or unparseable surface syntax
- integer literals that cannot become exact numeric atoms
- parser sugar that must be rewritten into a canonical system-head form before
  evaluation can proceed
- message-tag surface forms that should become string payloads regardless of
  whether the eventual runtime call is meaningful

In other words, lowering owns *representation-local* validation: "can this
piece of syntax become a well-formed core `Expr`?"

### Runtime validation

The evaluator should continue to own semantic checks that depend on builtin
contracts, definition rules, symbol state, or evaluation context:

- whether a tagged definition uses a valid tag owner and tag position
- whether a scoping form, control form, or builtin received semantically valid
  arguments
- whether a package/file/context lookup succeeds
- whether a canonical `Expr` is meaningful for the builtin that eventually
  receives it

In other words, runtime owns *semantic* validation: "now that this is a valid
core `Expr`, does it make sense to execute?"

### Boundary examples

- `f /: g[h[f[x_]]] := 1` lowers successfully to
  `TagSetDelayed[f, g[h[f[Pattern[x, Blank[]]]]], 1]`, and only later does the
  runtime report `TagSetDelayed::tagpos`.
- `f /: g[x_] := 1` also lowers successfully to canonical `TagSetDelayed[...]`,
  but runtime later decides whether the tag owner actually appears where it
  must.
- `Context["MissingSymbol"]` lowers unchanged; runtime decides what that call
  means for the current session.

This keeps lowering deterministic and syntax-focused while leaving semantic
policy in one place: the evaluator.
