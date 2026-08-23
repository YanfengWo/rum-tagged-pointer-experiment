# Patterns And Rules

Patterns are the main way to select expression shapes.

Rules use patterns to rewrite expressions.

Definitions use pattern-like left-hand sides to attach behavior to symbols.

The generated reference entries for this chapter include
[`MatchQ`](../reference/builtins.md#matchq),
[`Cases`](../reference/builtins.md#cases),
[`ReplaceAll`](../reference/builtins.md#replaceall),
[`ReplaceRepeated`](../reference/builtins.md#replacerepeated), and
[`Replace`](../reference/builtins.md#replace).

## Blanks

The most common pattern atoms are blanks:

- `_` matches one expression
- `_Integer` matches one expression with head `Integer`
- `__` matches one or more expressions in a sequence position
- `___` matches zero or more expressions in a sequence position

Named patterns bind a matched expression:

`x_` and `x_Integer` are named patterns.

The name can then be used on the right-hand side of a rule or definition.

## Pattern Combinators

The current matcher supports a useful subset of pattern combinators:

- `Alternatives`, written with `|`
- `Condition`, written with `/;`
- `PatternTest`, written with `?`
- `Optional`
- `Except`
- `Repeated`
- `RepeatedNull`
- `PatternSequence`
- `OrderlessPatternSequence`
- `Verbatim`
- `Longest`
- `Shortest`

```text
$ cargo run -q -p rum-cli -- --eval '{MatchQ[f[1, 2, 3], f[__, _Integer]], MatchQ[3, 1 | 2 | 3], MatchQ[4, x_ /; EvenQ[x]], MatchQ["rum", _String?StringQ]}'
List[True, True, True, True]
```

The four results demonstrate a sequence blank, alternatives, a condition, and
a pattern test.

## Defaults

`Optional` can provide a default value.

The shorthand `x_:10` means an optional named pattern with default `10`.

```text
$ cargo run -q -p rum-cli -- --eval 'p[x_:10] := x; {p[], p[7]}'
List[10, 7]
```

The first call uses the default.

The second call binds the explicit argument.

## Testing Matches

Use `MatchQ[expr, pattern]` when you need a boolean answer.

Use `Cases[expr, pattern]` when you need the matching subexpressions.

`Cases` traverses expression trees and returns matches in the current traversal
order.

The traversal subset is association-aware for the currently implemented cases,
delete, count, member, free, position, and level families.

## Replacement

Use `ReplaceAll` or the `/.` sugar for tree replacement.

```text
$ cargo run -q -p rum-cli -- --eval '{1, 2, a} /. x_Integer :> x + 10'
List[11, 12, a]
```

Only integer elements match.

The delayed rule evaluates the right-hand side after binding `x`.

Use `Replace` when you need controlled levels or one replacement at a selected
position.

Use `ReplaceRepeated` or `//.` when you want to keep applying rules until the
expression stabilizes or hits the replace-repeated iteration limit.

The current `ReplaceList` entry is a first-pass root replacement implementation
with optional max-result limiting. See the reference before depending on broad
replacement-list behavior.

## Definitions

Definition categories are stored separately:

- ownvalues for symbols
- downvalues for ordinary calls headed by a symbol
- upvalues attached to an argument-owned symbol
- subvalues for nested heads

The evaluator tries ownvalues when evaluating symbols.

For ordinary calls, it tries upvalues before downvalues, then subvalues after
nested heads become stable.

```text
$ cargo run -q -p rum-cli -- --eval 'tag /: h[tag[x_]] := x + 10; g[x_][y_] := x + y; {h[tag[5]], g[2][3]}'
List[15, 5]
```

The first definition is an upvalue attached to `tag`.

The second definition is a subvalue for the nested call shape `g[x_][y_]`.

## Immediate And Delayed

Immediate rules and definitions evaluate their right-hand side before storage:

- `Set`
- `Rule`

Delayed rules and definitions store the right-hand side and evaluate it when
the rule or definition fires:

- `SetDelayed`
- `RuleDelayed`

Use delayed forms when the right side depends on pattern bindings.

## Definition Precedence

The current implemented precedence is:

1. symbol ownvalues during symbol evaluation
2. special forms and argument preparation
3. upvalues on evaluated arguments
4. downvalues on the evaluated head
5. subvalues for nested heads
6. ordinary builtin dispatch
7. pure-function application

This is the user-facing model for the current core subset.

The generated reference defines the supported definition and pattern forms;
additional forms require an explicit rum contract rather than an assumed
definition-lattice extension.

## Limits

The matcher has a documented match-depth guard.

The current orderless sequence matcher uses `MAX_MATCH_DEPTH = 512`. When that
guard trips through a top-level recoverable evaluation path, the terminated
reason is `PatternMatchRecursionLimit`.

The previously recorded `Flat` plus `Orderless` plus `Optional` gaps are closed:
flat-wrapped singleton candidates, optional omission collection, and their
compound interactions have rum-native regression coverage. Rule sets that
depend on subtle candidate ordering should still pin the exact documented rum
behavior in their own tests.
