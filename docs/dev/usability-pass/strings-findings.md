# Strings Usability Audit Findings

Coverage statement: 101 findings-table rows cover 99 distinct catalog names; the coverage check printed `COVERAGE-OK 99/99`.

## Top 10 Gaps By Script-User Value

1. String-pattern symbols are unprotected, so users can accidentally assign over core pattern atoms and anchors.
2. Common string-search functions lack unambiguous unary operator forms, forcing wrappers in higher-order scripts.
3. Malformed `Uncompress`, wrong-key `Decrypt`, missing template slots, malformed HTML numeric entities, and unequal-length `HammingDistance` stay inert instead of offering a fail-closed mode.
4. `Encrypt` exposes a first-pass `XOR-SHA256` encrypted object, which is easy to mistake for production-grade encryption.
5. `Hash` only accepts `SHA256`, with other familiar algorithm names left inert.
6. URL, CLI-argument, and path-string helpers cover common Unix-oriented cases but lack form encoding, short options, and separator control.
7. Text-shaping helpers have fixed defaults for newline preservation, blank-line indentation, and column separators.
8. Encoding helpers do not expose byte-oriented `ToCharacterCode` or common UTF-8 spelling aliases.
9. English-only lexical helpers and diacritic folding miss language/normalization escape hatches.
10. Sequence helpers use fixed contiguous/simple alignment behavior without an option for non-contiguous subsequences or scoring control.

## Findings Table

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Alternatives | WL | gap | safety | The string-pattern head is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| Blank | WL | gap | safety | The string-pattern atom is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| BlankNullSequence | WL | gap | safety | The string-pattern atom is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| BlankSequence | WL | gap | safety | The string-pattern atom is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| DigitCharacter | WL | gap | safety | The string-pattern character-class symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| EndOfLine | WL | gap | safety | The string-pattern anchor symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| EndOfString | WL | gap | safety | The string-pattern anchor symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| Except | WL | gap | safety | The string-pattern exclusion head is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| LetterCharacter | WL | gap | safety | The string-pattern character-class symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| Pattern | WL | gap | safety | The string-pattern binding head is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| PunctuationCharacter | WL | gap | safety | The string-pattern character-class symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| RegularExpression | WL | gap | safety | The string-pattern regex wrapper head is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| StartOfLine | WL | gap | safety | The string-pattern anchor symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| StartOfString | WL | gap | safety | The string-pattern anchor symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| StringExpression | WL | gap | safety | The string-pattern sequence head is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| WhitespaceCharacter | WL | gap | safety | The string-pattern character-class symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| WordBoundary | WL | gap | safety | The string-pattern boundary anchor is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| WordCharacter | WL | gap | safety | The string-pattern character-class symbol is unprotected at runtime (Evidence E1). | Add `Protected` to the symbol; default matching behavior unchanged. | S |
| StringJoin | WL | OK | | | | |
| StringLength | WL | OK | | | | |
| StringReverse | WL | OK | | | | |
| ToLowerCase | WL | OK | | | | |
| ToUpperCase | WL | gap | safety | `ToUpperCase` maps lists but advertises no `Listable` or `Protected` attributes (Evidence E16). | Add `Listable` and `Protected`; default conversion behavior unchanged. | S |
| Capitalize | WL | gap | convenience | `Capitalize` maps lists but does not advertise `Listable` (Evidence E16). | Add `Listable`; default capitalization behavior unchanged. | S |
| Decapitalize | WL | gap | convenience | `Decapitalize` maps lists but does not advertise `Listable` (Evidence E16). | Add `Listable`; default decapitalization behavior unchanged. | S |
| StringForm | WL | OK | | | | |
| StringWrap | rum | gap | convenience | `StringWrap` collapses existing line breaks and blank lines, leaving no preservation mode (Evidence E8). | Add an option such as `"PreserveNewlines" -> True`; keep the current normalized wrapping default. | S |
| StringIndent | rum | gap | convenience | `StringIndent` skips empty and whitespace-only lines, leaving no way to indent every physical line (Evidence E8). | Add an option such as `"BlankLines" -> True`; keep the current skip-blank default. | S |
| StringDedent | rum | OK | | | | |
| StringExpandTabs | rum | OK | | | | |
| AlignColumns | rum | gap | convenience | The column separator is hard-coded to two spaces at `crates/rum-stdlib/src/strings/scripting/text_shape.rs:244`. | Add a separator option or optional second argument; keep two spaces as the default. | S |
| DecimalString | rum | OK | | | | |
| HtmlEscape | rum | OK | | | | |
| HtmlUnescape | rum | gap | safety | Malformed numeric entities are preserved verbatim with no strict failure mode (Evidence E3). | Add `"Strict" -> True` to return a message plus `$Failed` for malformed entities; keep permissive default. | S |
| ShellSplit | rum | OK | | | | |
| ShellQuote | rum | OK | | | | |
| FileExtension | WL | gap | flexibility | Path leaf detection is slash-only at `crates/rum-stdlib/src/strings/scripting/paths.rs:64`, so backslash paths are not fully handled. | Add a path-separator option or `Automatic` separator mode; default slash behavior unchanged. | S |
| FileBaseName | WL | gap | flexibility | Path leaf detection is slash-only at `crates/rum-stdlib/src/strings/scripting/paths.rs:64`, so backslash paths keep directory text in the base name (Evidence E9). | Add a path-separator option or `Automatic` separator mode; default slash behavior unchanged. | S |
| FileNameSplit | WL | gap | flexibility | `FileNameSplit` splits only on `/` at `crates/rum-stdlib/src/strings/scripting/paths.rs:77`, so backslash paths remain one component (Evidence E9). | Add a path-separator option or `Automatic` separator mode; default slash behavior unchanged. | S |
| UrlEncode | rum | gap | convenience | URL form encoding is unreachable: spaces become `%20` and plus signs become `%2B` (Evidence E7). | Add a form/query encoding option such as `"SpaceEncoding" -> "Plus"`; keep RFC 3986 percent encoding default. | S |
| UrlDecode | rum | gap | convenience | URL form decoding is unreachable: `+` is preserved instead of decoded as a space (Evidence E7). | Add an option such as `"PlusAsSpace" -> True`; keep percent-only default unchanged. | S |
| UrlParse | rum | gap | convenience | `UrlParse` only accepts `scheme://host...` at `crates/rum-stdlib/src/strings/scripting/url.rs:133`, so schemeless URLs and decoded query fields require wrappers. | Add options such as `"AssumeScheme"` and `"DecodeQuery" -> True`; keep current strict parse default. | M |
| ParseArgs | rum | gap | convenience | Short options are treated as positional arguments rather than flags (Evidence E7). | Add `"ShortOptions" -> True` for clustered and single-letter flags; keep long-only default. | M |
| Row | WL | OK | | | | |
| StringRiffle | WL | gap | flexibility | Flat-list `StringRiffle` rejects a third separator argument at `crates/rum-stdlib/src/strings/text/builtins.rs:102` (Evidence E10). | Add the documented third-argument flat-list separator form; default one- and two-argument behavior unchanged. | S |
| StringPadLeft | WL | gap | flexibility | Padding accepts only one non-empty string and rejects padding lists (Evidence E10). | Add list/cyclic padding forms as a compatible extension; default single-string padding unchanged. | M |
| StringPadRight | WL | gap | flexibility | Padding accepts only one non-empty string at `crates/rum-stdlib/src/strings/text/builtins.rs:221`. | Add list/cyclic padding forms as a compatible extension; default single-string padding unchanged. | M |
| StringLines | WL | OK | | | | |
| StringTemplate | WL | OK | | | | |
| TemplateApply | WL | gap | safety | Missing named slots leave the call inert rather than offering a fail-closed mode (Evidence E3). | Add a strict option or rum-native `TemplateApplyStrict` that emits a message plus `$Failed`; default unchanged. | S |
| TextWords | WL | OK | | | | |
| ToCharacterCode | WL | gap | flexibility | The byte-oriented encoding argument is rejected by exact arity checking (Evidence E6). | Add a second encoding argument, at least `"UTF8"` for bytes; default scalar-code-point behavior unchanged. | M |
| ToExpression | WL | OK | | | | |
| StringToByteArray | WL | gap | convenience | Only exact `"UTF8"` is accepted; common `"UTF-8"` spelling is left inert (Evidence E6). | Accept `"UTF-8"` as an alias and add future encodings behind explicit names; default unchanged. | S |
| Hash | WL | gap | flexibility | Only `"SHA256"` is accepted; `"SHA1"` and other familiar names are left inert (Evidence E12). | Add selected documented hash algorithm names with default `SHA256` unchanged. | M |
| Compress | WL | OK | | | | |
| Uncompress | WL | gap | safety | Malformed compressed strings return the inert call rather than a message plus `$Failed` (Evidence E3). | Add a strict failure option or rum-native `UncompressStrict`; default unchanged. | S |
| Encrypt | WL | gap | safety | `Encrypt` produces an `EncryptedObject` tagged `"XOR-SHA256"` (Evidence E12). | Add a real authenticated method option and keep the current first-pass method only as an explicit/default-unchanged compatibility path. | L |
| Decrypt | WL | gap | safety | A wrong key returns the inert `Decrypt[...]` call rather than a fail-closed result (Evidence E3). | Add a strict option or authenticated method that emits a message plus `$Failed` on key mismatch; default unchanged. | M |
| Nearest | WL | gap | convenience | `DistanceFunction` is accepted but `Options[Nearest]` is empty (Evidence E2). | Register `DistanceFunction -> Automatic` in `Options[Nearest]`; default nearest behavior unchanged. | S |
| Pluralize | WL | OK | | | | |
| SyntaxQ | WL | OK | | | | |
| RumTokens | rum | OK | | | | |
| TokenizeWith | rum | gap | flexibility | Token patterns are always matched with `StringPatternMatchConfig::default()` at `crates/rum-stdlib/src/strings/tokenize.rs:160`. | Add options such as `IgnoreCase -> True`; keep case-sensitive matching default. | S |
| TextString | WL | OK | | | | |
| CharacterCounts | WL | OK | | | | |
| RemoveDiacritics | WL | gap | flexibility | Decomposed combining accents are not folded (Evidence E11). | Add a normalization-aware option such as `"NormalizeCombiningMarks" -> True`; default table-driven behavior unchanged. | M |
| Alphabet | WL | gap | flexibility | Only the default English alphabet is reachable; `"Greek"` is left inert (Evidence E13). | Add a small set of named alphabets as explicit extensions; default English behavior unchanged. | M |
| AlphabeticSort | WL | gap | flexibility | Only `"English"` is accepted as the alphabet argument, so `"Greek"` is left inert (Evidence E13). | Add named alphabet support matching `Alphabet`; default English ordering unchanged. | M |
| EditDistance | WL | OK | | | | |
| HammingDistance | WL | gap | safety | Unequal-length inputs return the inert call rather than a clear failure mode (Evidence E3). | Add a strict failure path or option that emits a message plus `$Failed`; default unchanged. | S |
| LongestCommonSubsequence | WL | gap | flexibility | The implementation returns the longest contiguous slice, not a non-contiguous subsequence (Evidence E14). | Add an option such as `Contiguous -> False` or a rum-native sibling for non-contiguous LCS; default unchanged. | M |
| SequenceAlignment | WL | gap | flexibility | `SequenceAlignment` accepts exactly two arguments and uses fixed global alignment at `crates/rum-stdlib/src/strings/similarity/mod.rs:315`. | Add explicit scoring/gap options; default alignment behavior unchanged. | M |
| Diff | rum | OK | | | | |
| Characters | WL | OK | | | | |
| CharacterRange | WL | OK | | | | |
| ToString | WL | OK | | | | |
| StringRepeat | WL | OK | | | | |
| LetterQ | WL | OK | | | | |
| LowerCaseQ | WL | OK | | | | |
| UpperCaseQ | WL | OK | | | | |
| DigitQ | WL | OK | | | | |
| StringContainsQ | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringContainsQ[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringStartsQ | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringStartsQ[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringEndsQ | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringEndsQ[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringPosition | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E5). | Add `StringPosition[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringPosition | WL | gap | flexibility | `Overlaps -> False` is rejected even though default positions include overlaps (Evidence E15). | Add `Overlaps` control with the current overlapping behavior as default unchanged. | S |
| StringSplit | WL | OK | | | | |
| StringTrim | WL | OK | | | | |
| StringReplace | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringReplace[rules]` as an operator form; default direct-call behavior unchanged. | S |
| StringCases | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringCases[patt]` and `StringCases[rules]` operator forms; default direct-call behavior unchanged. | S |
| StringCases | WL | gap | flexibility | `Overlaps -> True` is rejected for overlapping case extraction (Evidence E15). | Add `Overlaps` control with current non-overlapping extraction as default unchanged. | S |
| StringApply | rum | OK | | | | |
| StringDelete | WL | OK | | | | |
| StringTake | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E5). | Add `StringTake[spec]` as an operator form; default direct-call behavior unchanged. | S |
| StringDrop | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E5). | Add `StringDrop[spec]` as an operator form; default direct-call behavior unchanged. | S |
| StringTakeDrop | WL | OK | | | | |
| StringPart | WL | OK | | | | |
| StringCount | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E5). | Add `StringCount[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringMatchQ | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringMatchQ[patt]` as an operator form; default direct-call behavior unchanged. | S |
| StringFreeQ | WL | gap | flexibility | The unambiguous unary operator form is missing and emits an arity message (Evidence E4). | Add `StringFreeQ[patt]` as an operator form; default direct-call behavior unchanged. | S |

## Evidence Appendix

Commands were run with the repository bounded wrapper while still feeding empty stdin to `cargo run`.

### E1: Pattern Symbol Attributes

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Attributes /@ {DigitCharacter, EndOfLine, EndOfString, LetterCharacter, PunctuationCharacter, StartOfLine, StartOfString, WhitespaceCharacter, WordBoundary, WordCharacter}, Attributes /@ {ToLowerCase, ToUpperCase, StringReverse}}'
List[List[List[], List[], List[], List[], List[], List[], List[], List[], List[], List[]], List[List[Listable, Protected], List[], List[Listable, Protected]]]
```

### E2: Hidden `Nearest` Option

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Options[Nearest], Nearest[{"cat", "cot", "cut"}, "cut", DistanceFunction -> HammingDistance], Options /@ {StringCases, StringPosition, StringReplace, StringCount}}'
List[List[], List["cut"], List[List[Rule[IgnoreCase, False]], List[Rule[IgnoreCase, False]], List[Rule[IgnoreCase, False]], List[Rule[IgnoreCase, False], Rule[Overlaps, False]]]]
```

### E3: Inert Malformed Or Unsafe Inputs

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Uncompress["not-an-envelope"], Decrypt["wrong", Encrypt["secret", "value"]], TemplateApply["`name`", <||>], HtmlUnescape["&#xZZ;"], HammingDistance["ab", "a"]}'
List[Uncompress["not-an-envelope"], Decrypt["wrong", EncryptedObject[Association[Rule["Cipher", "XOR-SHA256"], Rule["KeyHash", ByteArray["K7gNU3sdo+OL0wNhqoVWhr3g6s1xYv72ol/pe/Unols="]], Rule["Data", ByteArray["dylrNX0="]], Rule["OriginalForm", String]]]], TemplateApply["`name`", Association[]], "&#xZZ;", HammingDistance["ab", "a"]]
```

### E4: Search Predicate Operator Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{StringContainsQ["a"]["cat"], StringStartsQ["a"]["abc"], StringEndsQ["c"]["abc"], StringReplace["a" -> "b"]["cat"], StringCases["a"]["cat"], StringMatchQ["a"]["a"], StringFreeQ["z"]["abc"]}'
! StringContainsQ::argtu: expected between 2 and 3 argument(s), got 1
! StringStartsQ::argtu: expected between 2 and 3 argument(s), got 1
! StringEndsQ::argtu: expected between 2 and 3 argument(s), got 1
! StringReplace::argtu: expected between 2 and 4 argument(s), got 1
! StringCases::argtu: expected between 2 and 4 argument(s), got 1
! StringMatchQ::argtu: expected between 2 and 3 argument(s), got 1
! StringFreeQ::argtu: expected between 2 and 3 argument(s), got 1
List[StringContainsQ["a"]["cat"], StringStartsQ["a"]["abc"], StringEndsQ["c"]["abc"], StringReplace[Rule["a", "b"]]["cat"], StringCases["a"]["cat"], StringMatchQ["a"]["a"], StringFreeQ["z"]["abc"]]
```

### E5: Position, Count, Take, Drop Operator Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{StringPosition["a"]["banana"], StringSplit["," ]["a,b"], StringTrim["x"]["xxax"], StringCount["a"]["banana"], StringTake[2]["abcd"], StringDrop[2]["abcd"]}'
! StringPosition::argtu: expected between 2 and 4 argument(s), got 1
! StringCount::argtu: expected between 2 and 4 argument(s), got 1
! StringTake::argx: expected exactly 2 argument(s), got 1
! StringDrop::argx: expected exactly 2 argument(s), got 1
! StringPosition::argtu: expected between 2 and 4 argument(s), got 1
! StringCount::argtu: expected between 2 and 4 argument(s), got 1
! StringTake::argx: expected exactly 2 argument(s), got 1
! StringDrop::argx: expected exactly 2 argument(s), got 1
List[StringPosition["a"]["banana"], List[","]["a,b"], "x"["xxax"], StringCount["a"]["banana"], StringTake[2]["abcd"], StringDrop[2]["abcd"]]
```

`StringSplit` and `StringTrim` are not gap rows from this transcript because their unary data forms are already meaningful direct calls.

### E6: Character And Byte Encoding Arguments

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{RemoveDiacritics["é"], RemoveDiacritics["e\u0301"], ToCharacterCode["é"], ToCharacterCode["é", "UTF8"], StringToByteArray["é", "UTF-8"]}'
! ToCharacterCode::argx: expected exactly 1 argument(s), got 2
! ToCharacterCode::argx: expected exactly 1 argument(s), got 2
List["e", "eu0301", List[233], ToCharacterCode["é", "UTF8"], StringToByteArray["é", "UTF-8"]]
```

### E7: Scripting Convenience Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{ParseArgs[{"-abc", "--out=x", "pos"}], AlignColumns[{{"a","b"},{"long","c"}}], UrlEncode["a b+c"], UrlDecode["a+b%20c"]}'
List[Association[Rule["Positional", List["-abc", "pos"]], Rule["Options", Association[Rule["out", "x"]]], Rule["Flags", List[]]], "a     b\nlong  c", "a%20b%2Bc", "a+b c"]
```

### E8: Text Shaping Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{StringWrap["alpha\n\nbeta", 20], StringIndent["alpha\n\nbeta", "> "]}'
List["alpha beta", "> alpha\n\n> beta"]
```

### E9: Backslash Path Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{FileNameSplit["C:\\tmp\\a.txt"], FileBaseName["C:\\tmp\\a.txt"], FileExtension["C:\\tmp\\a.txt"]}'
List[List["C:\\tmp\\a.txt"], "C:\\tmp\\a", "txt"]
```

### E10: Riffle And Padding Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{StringRiffle[{"a", "b"}, ",", ";"], StringPadLeft["x", 4, {"0", "1"}], StringPadRight[{"x", "yy"}, 4, "."]}'
List[StringRiffle[List["a", "b"], ",", ";"], StringPadLeft["x", 4, List["0", "1"]], List["x...", "yy.."]]
```

### E11: Decomposed Diacritic Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval 'RemoveDiacritics["é"]'
"é"
```

### E12: Hash And Encrypt Surface

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Hash["abc", "SHA256", "HexString"], Hash["abc", "SHA1", "HexString"], StringContainsQ[ToString[Encrypt["k", "v"]], "XOR-SHA256"], Decrypt["bad", Encrypt["k", "v"]]}'
List["ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", Hash["abc", "SHA1", "HexString"], True, Decrypt["bad", EncryptedObject[Association[Rule["Cipher", "XOR-SHA256"], Rule["KeyHash", ByteArray["glTDKakoUPbVOd03b0gW7idkUX2l4CNVFK9DMWRIDXo="]], Rule["Data", ByteArray["pw=="]], Rule["OriginalForm", String]]]]]
```

### E13: Alphabet Scope

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Alphabet[], Alphabet["Greek"], AlphabeticSort[{"éclair", "apple"}], AlphabeticSort[{"β", "α"}, "Greek"]}'
List[List["a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s", "t", "u", "v", "w", "x", "y", "z"], Alphabet["Greek"], List["apple", "éclair"], AlphabeticSort[List["β", "α"], "Greek"]]
```

### E14: Sequence Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{LongestCommonSubsequence["abcde", "ace"], SequenceAlignment["abc", "axc"], Diff["abc", "axc"]}'
List["a", List["a", List["b", "x"], "c"], DiffObject[1, "Characters", Association[Rule["Changes", List[List[Rule[List[1, 1], "a"]], List[Rule[2, "b"], Rule[2, "x"]], List[Rule[List[3, 3], "c"]]]], Rule["View", List[Automatic]]]]]
```

### E15: Overlaps Probe

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{StringCount["aaaa", "aa"], StringCount["aaaa", "aa", Overlaps -> True], StringCases["aaaa", "aa"], StringCases["aaaa", "aa", Overlaps -> True], StringPosition["aaaa", "aa"], StringPosition["aaaa", "aa", Overlaps -> False]}'
List[2, 3, List["aa", "aa"], StringCases["aaaa", "aa", Rule[Overlaps, True]], List[List[1, 2], List[2, 3], List[3, 4]], StringPosition["aaaa", "aa", Rule[Overlaps, False]]]
```

### E16: Case Conversion Attributes

```text
$ printf '' | scripts/run-bounded.py --timeout 30 -- cargo run -q -p rum-cli -- --eval '{Attributes /@ {Capitalize, Decapitalize, ToUpperCase, ToLowerCase}, {Capitalize[{"a", "b"}], Decapitalize[{"A", "B"}], ToUpperCase[{"a", "b"}], ToLowerCase[{"A", "B"}]}}'
List[List[List[Protected, ReadProtected], List[Protected, ReadProtected], List[], List[Listable, Protected]], List[List["A", "B"], List["a", "b"], List["A", "B"], List["a", "b"]]]
```

## Triage Sketch

1. Protect and surface attributes: add `Protected` to string-pattern symbols and align case-conversion `Listable` attributes; high safety value with small blast radius.
2. Search operator and `Overlaps` parity: add unambiguous operator forms plus `Overlaps` controls for cases/positions; high scripting value and mostly parser/evaluator surface work.
3. Text/path/url/args ergonomics: add opt-in newline, blank-line, separator, path separator, form URL, and short-option modes; medium value for scripts that generate reports and parse command inputs.
4. Fail-closed strict modes and crypto/hash expansion: add strict options/siblings for malformed decode/decrypt/uncompress/template cases, register `Nearest` options, and add stronger crypto/hash surfaces; highest safety value but mixed sizes.
5. Language and sequence flexibility: add alphabet/diacritic normalization escape hatches plus non-contiguous LCS and alignment scoring options; useful but lower priority than safety and scripting ergonomics.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/strings.rs
99
```

```text
$ perl -ne 'if ($ARGV =~ /catalog\/strings\.rs$/ && /"([A-Za-z\$][A-Za-z0-9\$]*)"/) { $cat{$1}=1; $total++ } elsif ($ARGV =~ /strings-findings\.md$/ && /^\| ([A-Za-z\$][A-Za-z0-9\$]*) \|/) { $row{$1}++ unless $1 eq "Builtin" } END { @missing=sort grep { !$row{$_} } keys %cat; @extra=sort grep { !$cat{$_} } keys %row; if (@missing || @extra || $total != 99) { print "COVERAGE-FAIL total=$total missing=@missing extra=@extra\n"; exit 1 } print "COVERAGE-OK ".scalar(keys %cat)."/$total\n" }' crates/rum-stdlib/src/catalog/strings.rs docs/dev/usability-pass/strings-findings.md
COVERAGE-OK 99/99
```

```text
$ git status --short
?? docs/dev/usability-pass/
```

Plain `git status --short` collapses the new untracked directory because the parent directory did not exist before this audit. Expanded status shows the only new file:

```text
$ git status --short --untracked-files=all
?? docs/dev/usability-pass/strings-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

None.
