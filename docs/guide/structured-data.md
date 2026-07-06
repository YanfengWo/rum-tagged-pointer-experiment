# Structured Data

`FromJson` and `ToJson` convert between JSON text and the JSON-representable
subset of `rum` expressions.

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
