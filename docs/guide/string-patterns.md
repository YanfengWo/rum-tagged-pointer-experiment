# String Patterns

String-pattern builtins share one pattern language across `StringCases`,
`StringReplace`, `StringMatchQ`, `StringSplit`, `StringPosition`, and
`StringContainsQ`.

The supported string-pattern constructs are:

- string literals, including `*` and `?` wildcard characters inside a literal
- sequence syntax with `~~`, which lowers to `StringExpression`
- alternatives with `|`
- `_`, `__`, and `___`
- named bindings with `x_` and `Pattern[x, patt]`
- `Repeated[patt]`, `RepeatedNull[patt]`, `patt..`, and `patt...`
- `Shortest[patt]` and `Longest[patt]`
- `Except[patt]` and `Except[patt, inner]`
- `RegularExpression["..."]`
- `Whitespace`, `WhitespaceCharacter`, `LetterCharacter`, `DigitCharacter`,
  `WordCharacter`, `PunctuationCharacter`, and `NumberString`
- `StartOfString`, `EndOfString`, `StartOfLine`, `EndOfLine`, and
  `WordBoundary`

`PunctuationCharacter` is exactly the ASCII punctuation set:

```text
!"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
```

`WordCharacter` is an alphabetic character, an ASCII decimal digit, or `_`.
`WordBoundary` is a zero-width match at a transition between `WordCharacter`
and a non-word character or string edge.

## Zero-Width Matches

`StartOfString`, `EndOfString`, `StartOfLine`, `EndOfLine`, and
`WordBoundary` are zero-width anchors. They match an insertion point, not a
character.

`StartOfLine` matches at the start of the string and after `\n`. `EndOfLine`
matches before `\n` and at the end of the string.

`StringPosition` reports a zero-width match as `{pos, pos - 1}`, where `pos`
is the 1-based insertion point. At the end of a string, `pos` is one greater
than the string length.

`StringCases` returns the empty string for an accepted empty match.
`StringReplace` performs one replacement at the same scan position. After a
non-terminal zero-width match, scanning advances by one code point; after a
terminal zero-width match, scanning stops.

```text
$ cargo run -q -p rum-cli -- --eval 'StringPosition["abc", EndOfString]'
List[List[4, 3]]
```

```text
$ cargo run -q -p rum-cli -- --eval 'StringReplace["abc", EndOfString -> "$"]'
"abc$"
```

```text
$ cargo run -q -p rum-cli -- --eval '{StringCases["", "a"...], StringReplace["", "a"... -> "x"]}'
List[List[""], "x"]
```

Line anchors use LF positions:

```text
$ cargo run -q -p rum-cli -- --eval 'StringPosition["x\ny", StartOfLine]'
List[List[1, 0], List[3, 2]]
```

## Character Classes

Character-class atoms match one character, except `Whitespace`, which matches
one or more whitespace characters, and `NumberString`, which matches a
decimal-style numeric run.

```text
$ cargo run -q -p rum-cli -- --eval 'StringCases["ab, cd! ef", PunctuationCharacter]'
List[",", "!"]
```

```text
$ cargo run -q -p rum-cli -- --eval 'StringCases["one two", WordBoundary ~~ WordCharacter]'
List["o", "t"]
```

## Greediness

`Shortest` and `Longest` choose which accepted match is preferred for
extracting or replacing text. They do not change whether a whole-string match
exists for `StringMatchQ`.

```text
$ cargo run -q -p rum-cli -- --eval '{StringMatchQ["aaa", Shortest["a"..]], StringMatchQ["aaa", Longest["a"..]]}'
List[True, True]
```

## IgnoreCase

`IgnoreCase -> True` is accepted by `StringCases`, `StringReplace`,
`StringMatchQ`, `StringSplit`, `StringPosition`, and `StringContainsQ`.

```text
$ cargo run -q -p rum-cli -- --eval 'StringSplit["AxbXc", "x", IgnoreCase -> True]'
List["A", "b", "c"]
```
