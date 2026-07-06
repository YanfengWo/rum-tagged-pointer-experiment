# Structured Data

`FromJson`/`ToJson`, `FromToml`/`ToToml`, `FromCsv`/`ToCsv`, and
`FromIni`/`ToIni` convert between common structured-data text formats and the
representable subset of `rum` expressions.

They are pure string codecs. They do not read or write files, do not consult
host capabilities beyond `Pure`, and fail closed with `$Failed` plus a runtime
message when the input is outside the documented subset.

## Bytes, Digests, And Encodings

`ByteArray` represents raw byte payloads as base64 text. `StringToByteArray`
encodes a string as UTF-8 bytes, and `ByteArrayToString` decodes a `ByteArray`
back to a string only when the bytes are valid UTF-8.

Hashing yes, encryption never: digests and encodings are pure deterministic
byte functions and belong in the standard library. Encryption, signing, and key
handling do not enter the kernel; if they are ever needed, they belong in a
downstream capability crate. `Md5` and `Sha1` are present for interoperability
and checksums only, not for collision-resistant use.

Digest and encoding functions accept either a string, hashed or encoded as its
UTF-8 bytes, or a `ByteArray`.

```text
$ cargo run -q -p rum-cli -- --eval 'Sha256["abc"]'
"ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
```

`Sha256`, `Sha512`, `Sha1`, and `Md5` return lowercase hexadecimal strings by
default. The second form with `"Bytes"` returns a `ByteArray`.

```text
$ cargo run -q -p rum-cli -- --eval 'Sha256[FromHexString["00ff"], "Bytes"]'
ByteArray["But9amnuGeX733SQGNPSq/oEvL0TZdsxLrhtxxaTibg="]
```

`Crc32` returns an unsigned exact integer.

```text
$ cargo run -q -p rum-cli -- --eval 'Crc32["123456789"]'
3421780262
```

`ToHexString` and `FromHexString` provide lowercase hex encoding and
case-insensitive decoding. `ToBase64` and `FromBase64` use standard RFC 4648
base64 with padding.

```text
$ cargo run -q -p rum-cli -- --eval '{ToHexString[StringToByteArray["ab"]], FromHexString["6162"] // ByteArrayToString}'
List["6162", "ab"]
```

```text
$ cargo run -q -p rum-cli -- --eval 'FromBase64[ToBase64["héllo, 世界"]] // ByteArrayToString'
"héllo, 世界"
```

Malformed encoded text fails closed with `$Failed` and a runtime message.

```text
$ cargo run -q -p rum-cli -- --eval 'FromHexString["6"]'
! FromHexString::hex: expected an even-length hexadecimal string
$Failed
```

`ByteArrayQ` tests for a syntactically valid `ByteArray` payload.

## JSON Mapping

`FromJson[str]` parses RFC 8259 JSON into these values:

| JSON | `rum` value |
| --- | --- |
| object | `Association` with member order preserved |
| array | `List` |
| string | string atom, including JSON escapes and surrogate pairs |
| integer number | exact `Integer`, with arbitrary precision |
| decimal or exponent number | exact base-10 value; `0.1` becomes `1/10` |
| `true`, `false`, `null` | `True`, `False`, `Null` |

Duplicate object keys are rejected instead of being merged. JSON `-0` parses
to the exact integer `0`.

```text
$ cargo run -q -p rum-cli -- --eval 'FromJson["{\"name\": \"Ada\", \"scores\": [1, 2.5, true, null]}"]'
Association[Rule["name", "Ada"], Rule["scores", List[1, 5/2, True, Null]]]
```

`ToJson[expr]` renders the inverse subset:

| `rum` value | JSON output |
| --- | --- |
| `Integer` | integer literal |
| terminating `Rational` | decimal literal |
| string | JSON string |
| `True`, `False`, `Null` | `true`, `false`, `null` |
| `List` | array |
| `Association` with string keys | object |

Non-terminating rationals, non-string association keys, symbols other than
`True`, `False`, and `Null`, and ordinary calls are rejected.

```text
$ cargo run -q -p rum-cli -- --eval 'ToJson[<|"n" -> 1/4, "big" -> 2^20, "s" -> "héllo", "l" -> {True, Null}|>]'
"{\"n\":0.25,\"big\":1048576,\"s\":\"héllo\",\"l\":[true,null]}"
```

Non-BMP string characters are emitted as JSON surrogate-pair escapes.

## Pretty Output

Default `ToJson` output is compact.

`ToJson[expr, "Indent" -> n]` pretty-prints arrays and objects with `n` spaces
per nesting level. No other JSON options are currently accepted.

```text
$ cargo run -q -p rum-cli -- --eval 'ToJson[<|"a" -> {1, True}, "b" -> <||>|>, "Indent" -> 2]'
"{\n  \"a\": [\n    1,\n    true\n  ],\n  \"b\": {}\n}"
```

The result is still a `rum` string atom, so the CLI displays embedded newlines
as `\n` escapes.

## Fail-Closed Policy

JSON decoding rejects trailing non-whitespace, unquoted keys, single-quoted
strings, comments, `NaN`, `Infinity`, unescaped control characters in strings,
invalid escapes, lone surrogates, duplicate object keys, and excessive nesting.

Parse errors use `FromJson::json` and name the byte offset. The fixed nesting
cap is 256 and uses `FromJson::depth`.

```text
$ cargo run -q -p rum-cli -- --eval 'FromJson["{\"a\": 1,}"]'
! FromJson::json: Invalid JSON at byte offset 8: object keys must be double-quoted strings.
$Failed
```

Encoding errors use `ToJson::json`.

```text
$ cargo run -q -p rum-cli -- --eval 'ToJson[<|"x" -> 1/3|>]'
$Failed
! ToJson::json: Cannot encode non-terminating Rational 1/3 as JSON; convert it explicitly before calling ToJson.
```

The script author must convert unsupported values explicitly before calling
`ToJson`.

## Round Trips And Transforms

The JSON-representable subset round-trips through exact values:

```text
$ cargo run -q -p rum-cli -- --eval 'FromJson[ToJson[<|"k" -> {1, 2/5, "z"}|>]]'
Association[Rule["k", List[1, 2/5, "z"]]]
```

Because `FromJson` returns ordinary `rum` data, the result can be transformed
with standard library functions before encoding again:

```text
$ cargo run -q -p rum-cli -- --eval 'ToJson[Mean[Map[Lookup[#, "v"] &, First[GatherBy[FromJson["[{\"g\":\"a\",\"v\":1},{\"g\":\"a\",\"v\":3},{\"g\":\"b\",\"v\":5}]"], Lookup[#, "g"] &]]]]]'
"2"
```

## TOML Mapping

`FromToml[str]` parses rum's fail-closed TOML 1.0 folk subset into an
`Association` with insertion order preserved.

| TOML | `rum` value |
| --- | --- |
| table, dotted key, inline table | `Association` |
| array | `List` |
| array-of-tables | `List` of `Association`s |
| string | string atom |
| integer | exact `Integer` |
| decimal or exponent float | exact base-10 value; `0.1` becomes `1/10` |
| boolean | `True` or `False` |
| offset date-time | exact rational Unix-second timestamp |
| local date-time | `{y, mo, d, h, mi, s}` |
| local date | `{y, mo, d}` |
| local time | `{h, mi, s}` |

Only offset date-times are instants. Local TOML date/time values are decoded as
plain date lists because they do not carry an offset. `inf`, `-inf`, and `nan`
fail closed because rum has no inexact float value in this codec family.

```text
$ cargo run -q -p rum-cli -- --eval 'FromToml["title = \"x\"\n[db]\nports = [1, 2]\npi = 3.14\nwhen = 1970-01-01T01:00:00+01:00\n"]'
Association[Rule["title", "x"], Rule["db", Association[Rule["ports", List[1, 2]], Rule["pi", 157/50], Rule["when", 0]]]]
```

```text
$ cargo run -q -p rum-cli -- --eval 'FromToml["d = 2026-07-06\nt = 12:13:14.5\n"]'
Association[Rule["d", List[2026, 7, 6]], Rule["t", List[12, 13, 29/2]]]
```

TOML decoding rejects duplicate keys or table definitions, dotted-key conflicts,
attempts to extend inline tables, invalid string escapes, invalid dates, and
unsupported TOML surface.

```text
$ cargo run -q -p rum-cli -- --eval 'FromToml["a = inf"]'
$Failed
! FromToml::toml: Invalid TOML at byte offset 4: inf and nan are outside rum's exact TOML number subset.
```

## TOML Output

`ToToml[assoc]` accepts a top-level Association with String keys. Nested
Associations emit as sub-tables, non-empty Lists of Associations emit as
arrays-of-tables, and other Lists emit as inline arrays. Scalars are exact
integers, terminating rationals, `True`/`False`, and Strings. Keys are quoted
when needed; keys containing `.` are quoted and are not split.

When a same-level scalar key follows a nested Association, `ToToml` uses an
inline table or inline array for that earlier nested value so
`FromToml[ToToml[x]]` preserves Association order.

```text
$ cargo run -q -p rum-cli -- --eval 'ToToml[<|"srv" -> <|"port" -> 8080|>|>]'
"[srv]\nport = 8080"
```

`ToToml` never guesses that a number or date-shaped List should become TOML
datetime syntax. Timestamps encode as numbers, and date lists encode as arrays.

```text
$ cargo run -q -p rum-cli -- --eval 'ToToml[<|"d" -> {2026, 7, 6}, "t" -> 0|>]'
"d = [2026, 7, 6]\nt = 0"
```

The TOML-representable subset round-trips through `FromToml[ToToml[x]]`.

```text
$ cargo run -q -p rum-cli -- --eval 'FromToml[ToToml[<|"k" -> <|"x" -> 1|>, "arr" -> {1, 1/4, True}|>]]'
Association[Rule["arr", List[1, 1/4, True]], Rule["k", Association[Rule["x", 1]]]]
```

## CSV Mapping

`FromCsv[str]` parses RFC 4180-style CSV into a `List` of row `List`s. Every
field is a String. There is no numeric, boolean, date, or expression inference;
scripts convert fields explicitly with tools such as `FromDigits` or
`ToExpression`.

Quoted fields may contain the delimiter, doubled quotes, and newlines. Rows
split on LF or CRLF, and one trailing newline is not a final empty row.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv["a,\"b,1\"\nc,d"]'
List[List["a", "b,1"], List["c", "d"]]
```

Plain CSV mode allows ragged rows because real-world CSV often has them.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv["a,b\n1"]'
List[List["a", "b"], List["1"]]
```

`FromCsv[str, "Headers" -> True]` uses the first row as header names and returns
a `List` of `Association`s. Header names must be unique, and data rows must
have the same field count as the header row.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv["name,qty\napple,3\nkiwi,10", "Headers" -> True]'
List[Association[Rule["name", "apple"], Rule["qty", "3"]], Association[Rule["name", "kiwi"], Rule["qty", "10"]]]
```

`"Delimiter" -> "\t"` selects TSV. The delimiter must be a single non-quote,
non-newline character.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv["a\tb\n1\t2", "Delimiter" -> "\t"]'
List[List["a", "b"], List["1", "2"]]
```

## CSV Output

`ToCsv[rows]` accepts either a list of list rows whose fields are all Strings,
or a list of Associations with identical String keys and String values.
Association rows imply `"Headers" -> True`; the first row's key order supplies
the output header order.

Fields are quoted only when they contain the delimiter, a quote, LF, or CR.
Quotes inside fields are doubled, and rows are joined with `\n`.

```text
$ cargo run -q -p rum-cli -- --eval 'ToCsv[{{"x","y\"z"},{"1","2"}}]'
"x,\"y\"\"z\"\n1,2"
```

```text
$ cargo run -q -p rum-cli -- --eval 'ToCsv[{<|"name" -> "apple", "qty" -> "3"|>, <|"qty" -> "10", "name" -> "kiwi"|>}]'
"name,qty\napple,3\nkiwi,10"
```

CSV decoding and encoding round-trip for the representable String-field subset
in both plain and header modes.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv[ToCsv[{{"a,b","q\"r"},{"line\nx","z"}}]]'
List[List["a,b", "q\"r"], List["line\nx", "z"]]
```

Because header-mode CSV returns Associations, it composes directly with other
structured-data codecs.

```text
$ cargo run -q -p rum-cli -- --eval 'ToJson[Map[Function[row, <|"item" -> Lookup[row, "name"], "qty" -> FromDigits[Lookup[row, "qty"]]|>], FromCsv["name,qty\napple,3\nkiwi,10", "Headers" -> True]]]'
"[{\"item\":\"apple\",\"qty\":3},{\"item\":\"kiwi\",\"qty\":10}]"
```

CSV parse errors use `FromCsv::csv` and name the byte offset.

```text
$ cargo run -q -p rum-cli -- --eval 'FromCsv["a,\"unterminated"]'
! FromCsv::csv: Invalid CSV at byte offset 2: unterminated quoted field.
$Failed
```

CSV encoding errors use `ToCsv::csv` and name the offending row/column when a
field is not a String.

## INI Mapping

`FromIni[str]` parses rum's own INI dialect into an `Association` from section
name to section `Association`. Section names, keys, and values are all Strings.

The dialect has `[section]` headers and `key = value` lines. Keys and values are
trimmed of surrounding whitespace. Values are otherwise taken verbatim: quotes
are not stripped, and `;` or `#` inside a value is part of the value. Blank
lines are ignored. Comment lines start with `;` or `#`, allowing leading
whitespace. There is no nesting, no multiline value syntax, and no inline
comments.

Keys before any section header are stored under the section name `""`.

```text
$ cargo run -q -p rum-cli -- --eval 'FromIni["; c\ntop = 1\n[db]\nhost = h ; not a comment\n"]'
Association[Rule["", Association[Rule["top", "1"]]], Rule["db", Association[Rule["host", "h ; not a comment"]]]]
```

```text
$ cargo run -q -p rum-cli -- --eval 'FromIni["[s]\nk = \"v\"\n"]'
Association[Rule["s", Association[Rule["k", "\"v\""]]]]
```

## INI Output

`ToIni[assoc]` renders an Association-of-Associations with String section names,
String keys, and String values. The `""` section, when present, is emitted first
without a header.

```text
$ cargo run -q -p rum-cli -- --eval 'ToIni[<|"" -> <|"top" -> "1"|>, "db" -> <|"host" -> "h ; not a comment"|>|>]'
"top = 1\n[db]\nhost = h ; not a comment"
```

Section and key names containing `[`, `]`, `=`, LF, or CR are not representable
and fail closed. Names starting with `;` or `#` also fail closed. Because
`FromIni` trims keys and values, `ToIni` rejects keys or values whose leading or
trailing whitespace would be lost, and it rejects multiline values. An empty
default `""` section is not representable.

The representable subset round-trips exactly through `FromIni[ToIni[x]]`.

```text
$ cargo run -q -p rum-cli -- --eval 'FromIni[ToIni[<|"" -> <|"top" -> "1"|>, "s" -> <|"k" -> "v"|>|>]]'
Association[Rule["", Association[Rule["top", "1"]]], Rule["s", Association[Rule["k", "v"]]]]
```

Because `FromIni` returns ordinary Associations, scripts can edit values before
encoding again.

```text
$ cargo run -q -p rum-cli -- --eval 'ToIni[<|"s" -> <|"k" -> StringJoin[Lookup[Lookup[FromIni["[s]\nk = v\n"], "s"], "k"], "2"]|>|>]'
"[s]\nk = v2"
```

INI parse errors use `FromIni::ini` and name the byte offset.

```text
$ cargo run -q -p rum-cli -- --eval 'FromIni["[dup]\n[dup]\n"]'
! FromIni::ini: Invalid INI at byte offset 6: duplicate section name "dup".
$Failed
```

INI encoding errors use `ToIni::ini`.
