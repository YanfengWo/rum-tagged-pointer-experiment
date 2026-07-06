# Syntax

`rum` source lowers into a smaller core expression form before evaluation.

Lowering is syntax work. It rewrites sugar into canonical calls, but semantic
validation belongs to the evaluator and builtins.

The canonical lowering contracts are recorded in
[lowering canonical forms](../lowering-canonical-forms.md).

## Expressions

The runtime value model is expression based.

An expression is either an atom or a call.

A call has a head and zero or more arguments:

`head[arg1, arg2, ...]`

Most surface constructs are sugar for calls with system heads.

For example, a list literal lowers to `List[...]`.

```text
$ cargo run -q -p rum-cli -- --eval 'FullForm[{a, b + 1}]'
"List[a, Plus[1, b]]"
```

`FullForm` renders the expression shape as text. The `Plus` arguments appear in
the runtime's canonical order for that expression.

## Atoms

The core atom families you will use most often are:

- symbols, such as `x`, `` Global`x ``, `` System`Plus ``
- exact integers, such as `0`, `42`, `-99`
- exact rationals, such as `1/2`
- strings, such as `"hello"`

Decimal literals lower to exact rationals. There is no floating-point atom in
the current language boundary.

```text
$ cargo run -q -p rum-cli -- --eval 'FullForm[1.25]'
"5/4"
```

The design-boundaries chapter explains the exact-rational boundary in more
detail.

## Symbols

Unqualified symbols are resolved through the current context and context path.

Fully qualified symbols include a context prefix.

Examples include `` Global`x ``, `` System`Plus ``, and `` Demo`value ``.

Use the context chapter for `$Context`, `$ContextPath`, packages, and saved
sessions.

## Strings

Strings are double quoted.

They evaluate to themselves, so `"rum"` remains a string value.

String builtins such as `StringLength`, `StringSplit`, `StringJoin`, and
`StringReplace` are cataloged in the generated reference.

`rum` also accepts two multiline string input forms. Both forms lower to the
same ordinary string atom as a single-line string.

An escaped multiline string starts and ends with `"""`. It uses the same escape
forms as ordinary `"..."` strings, including `\n`, `\r`, `\t`, `\"`, `\\`,
`\u{...}`, and named characters such as `\[Alpha]`. Single `"` and doubled `""`
characters may appear directly in the body. The literal ends at the first
unescaped `"""`.

```text
$ cargo run -q -p rum-cli -- --eval '"""line1\nline2"""'
"line1\nline2"
```

A raw multiline string starts and ends with `'''`. It performs no escape
processing: backslashes, `\t`, quotes, and newlines are literal content.

```text
$ cargo run -q -p rum-cli -- --eval $'x = \'\'\'\nC:\\path \\t raw\n\'\'\'; {StringLength[x], ToCharacterCode[x]}'
List[15, List[67, 58, 92, 112, 97, 116, 104, 32, 92, 116, 32, 114, 97, 119, 10]]
```

If the first character after either opening delimiter is `\n` or `\r\n`, that
first newline is dropped. No automatic dedent is applied. Pair multiline
literals with `StringDedent` when indented source text should become
flush-left string content.

```text
$ cargo run -q -p rum-cli -- --eval 'StringLines["""
a,1
b,2"""]'
List["a,1", "b,2"]
```

The raw form cannot contain `'''` in its body; there is no raw-string escape
hatch in this version.

Apostrophe is also postfix derivative syntax. To preserve that syntax, `'''`
opens a raw string only where a non-juxtaposition operand is expected, such as
at the start of input or after `=`, `,`, `(`, `[`, `{`, and infix operators.
After an expression, consecutive apostrophes remain postfix derivative tokens.

```text
$ cargo run -q -p rum-cli -- --eval "FullForm[HoldComplete[f'''[x]]]"
"HoldComplete[Derivative[3][f][x]]"
```

Multiline literals are input syntax only. Value printing and `ToString` use the
canonical single-line escaped string form, as shown in the examples above. The
formatter preserves the original multiline literal text when formatting source.

## Calls

Ordinary calls preserve their head during lowering:

For example, `f[1, x]` remains an ordinary call headed by `f`.

The evaluator later decides whether the head has attributes, user definitions,
upvalues, downvalues, subvalues, or builtin behavior.

## Arithmetic Sugar

Arithmetic operators lower into ordinary calls:

- `a + b` lowers to `Plus[a, b]`
- `a - b` lowers to `Plus[a, Times[-1, b]]`
- `a * b` lowers to `Times[a, b]`
- `a / b` lowers to `Times[a, Power[b, -1]]`
- `a ^ b` lowers to `Power[a, b]`
- `-x` lowers to `Times[-1, x]`

The evaluator may simplify those calls.

For exact arithmetic, the result remains exact when the operation is supported.

## Lists

List syntax is sugar for `List`.

For example, `{1, 2, 3}` is list syntax. The rendered result currently uses
canonical head form, such as `List[1, 2, 3]`.

List-processing builtins include `Map`, `Apply`, `Total`, `Cases`, `Select`,
`Fold`, `Table`, and many structure operations.

## Associations

Association syntax lowers to `Association` containing `Rule` entries.

```text
$ cargo run -q -p rum-cli -- --eval '<|"name" -> "rum", "year" -> 2026|>'
Association[Rule["name", "rum"], Rule["year", 2026]]
```

Association builtins include `Keys`, `Values`, `Lookup`, `KeyTake`, `KeyDrop`,
`KeyValueMap`, `AssociationMap`, and related entries in the reference.

## Patterns

Pattern sugar lowers into canonical pattern heads.

```text
$ cargo run -q -p rum-cli -- --eval 'FullForm[x_Integer -> x + 1]'
"Rule[Pattern[x, Blank[Integer]], Plus[1, x]]"
```

The left side `x_Integer` lowers to `Pattern[x, Blank[Integer]]`.

The right side `x + 1` lowers to a `Plus` call.

The patterns-and-rules chapter covers matching behavior.

## Function And Slot Sugar

The syntax:

`# + 1 &` lowers to a pure `Function` containing `Slot[1]`.

Slot sequence forms include `##` and `##2`.

Named slots use strings, as in `#name`.

## Assignment And Rewrite Sugar

Common assignment and rewrite forms lower to these heads:

- `lhs = rhs` -> `Set[lhs, rhs]`
- `lhs := rhs` -> `SetDelayed[lhs, rhs]`
- `lhs =.` -> `Unset[lhs]`
- `lhs -> rhs` -> `Rule[lhs, rhs]`
- `lhs :> rhs` -> `RuleDelayed[lhs, rhs]`
- `expr /. rules` -> `ReplaceAll[expr, rules]`
- `expr //. rules` -> `ReplaceRepeated[expr, rules]`

Lowering does not prove that an assignment target is semantically valid.

That decision belongs to the runtime.

## Comments

Block comments use `(* ... *)`.

```text
$ cargo run -q -p rum-cli -- --eval '1 (* ignored *) + 2'
3
```

Comments are trivia. They do not become runtime expressions.

## Validation Boundary

Syntax lowering rejects malformed syntax, invalid integer literals, and parse
depth overflow.

It does not reject every expression that a builtin may later consider invalid.

For example, a tagged definition form can lower to `TagSetDelayed[...]` even if
the evaluator later reports that the tag position is invalid.

Keep this layer split in mind:

- parser and lowering decide whether source can become an expression
- evaluation decides what the expression means
