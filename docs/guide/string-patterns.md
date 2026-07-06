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

## Tokenization

`TokenizeWith[rules, str]` scans a string left to right using a list of
`"class" -> pattern` string-pattern rules. At each position every rule is
tried, the longest non-empty match wins, and ties go to the earliest rule in
the list. Zero-width matches are skipped so an anchor-only rule cannot stall
the scan. Input characters that no rule matches are coalesced into `"text"`
spans.

The result is a list of `{class, {start, end}}` spans. Positions are 1-based
inclusive code-point positions, the same convention as `StringPosition`.
Spans are strictly ascending, non-overlapping, and cover the entire string.
Invalid rule lists, non-string classes, invalid string patterns, and non-string
input fail closed with a message and `$Failed`.

```text
$ cargo run -q -p rum-cli -- --eval 'TokenizeWith[{"comment" -> (";" ~~ Except["\n"]...), "section" -> ("[" ~~ Except["]"]... ~~ "]"), "key" -> (LetterCharacter ~~ WordCharacter... ~~ "="), "number" -> NumberString, "value" -> (WordCharacter..)}, ";top\n[main]\nname=rum\nport=42"]'
List[List["comment", List[1, 4]], List["text", List[5, 5]], List["section", List[6, 11]], List["text", List[12, 12]], List["key", List[13, 17]], List["value", List[18, 20]], List["text", List[21, 21]], List["key", List[22, 26]], List["number", List[27, 28]]]
```

Tokenizer rule sets are ordinary rum data meant to live in packages; grammar
definitions are not stdlib code.

`RumTokens[str]` exposes rum's own lexer as `{kind, {start, end}}` spans. It
is total on string input: whitespace and comments are emitted as tokens,
unrecognized input is emitted as `"error"`, and the span union always covers
the whole input. The token kind taxonomy follows `rum-syntax` and may grow;
the contract is the result shape, total coverage, and the 1-based inclusive
code-point span convention.

```text
$ cargo run -q -p rum-cli -- --eval 'RumTokens["f[x_] := x + 1 (* c *)"]'
List[List["symbol", List[1, 1]], List["[", List[2, 2]], List["symbol", List[3, 3]], List["_", List[4, 4]], List["]", List[5, 5]], List["whitespace", List[6, 6]], List[":=", List[7, 8]], List["whitespace", List[9, 9]], List["symbol", List[10, 10]], List["whitespace", List[11, 11]], List["+", List[12, 12]], List["whitespace", List[13, 13]], List["integer", List[14, 14]], List["whitespace", List[15, 15]], List["comment", List[16, 22]]]
```

The current `RumTokens` token-kind taxonomy is `"integer"`, `"string"`,
`"symbol"`, `"whitespace"`, `"comment"`, `"error"`, and these punctuation or
operator labels: `"["`, `"]"`, `"[["`, `"]]"`, `"{"`, `"}"`, `"<|"`, `"|>"`,
`"("`, `")"`, `","`, `";"`, `";;"`, `"@"`, `"@@"`, `"@@@"`, `"@*"`, `"+"`,
`"++"`, `"+="`, `"-"`, `"--"`, `"-="`, `"*"`, `"**"`, `"*="`, `"/"`, `"/@"`,
`"/="`, `"//"`, `"//@"`, `"//="`, `"/*"`, `"~~"`, `"~"`, `"^"`, `"^="`,
`"^:="`, `"!"`, `"'"`, `"&&"`, `"||"`, `"==="`, `"=!="`, `"=="`, `"!="`,
`"<"`, `">"`, `"<="`, `">="`, `"<>"`, `":"`, `"::"`, `"/:"`, `"->"`, `":>"`,
`"="`, `"=."`, `":="`, `"/."`, `"//."`, `"/;"`, `"."`, `".."`, `"..."`,
`"&"`, `"|"`, `"|->"`, `"?"`, `"#"`, `"_"`, `"__"`, and `"___"`. The
internal end-of-input marker is not emitted.
