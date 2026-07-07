# Codecs + Bytes + Datetime Usability Audit Findings

Coverage statement: findings-table rows cover 34 distinct catalog names: 2
from `json.rs`, 2 from `csv.rs`, 2 from `ini.rs`, 2 from `toml.rs`, 3 from
`xml.rs`, 2 from `yaml.rs`, 2 from `schema.rs`, 11 from `bytes.rs`, and 8 from
`datetime.rs`. The coverage check printed `COVERAGE-OK 34/34`.

No silent-wrong parser/decoder safety gap was found in this unit. The malformed
JSON/CSV/INI/TOML/XML/YAML/date/schema/byte probes either returned `$Failed`
with a message or, for plain ragged CSV rows, matched the documented accepted
surface. Full-spec omissions, non-terminating rationals, deterministic key
ordering, calendar Months/Years, XML DTDs, YAML anchors, and no ambient clock
are intended boundaries, not gaps.

## Top 10 Gaps By Script-User Value

1. **Safety:** digest heads silently return an inert call for an unknown second output-format argument such as `Sha256["abc", "Base64"]`.
2. **Safety:** all 34 unit heads lack `Protected`, so codec/parser/date verb heads and the `XmlElement` constructor are redefinable.
3. **Safety:** `Crc32` lacks the reference warning that `Md5` and `Sha1` have: checksum/legacy only, not a security primitive.
4. Implemented options are not discoverable through `Options[...]` for `ToJson`, `FromCsv`, `ToCsv`, `ToXml`, `ConformSchema`, and `SchemaErrors`.
5. `DateString` lacks a cheap format selector such as ISO date-only or time-only output.
6. `DayOfWeek` returns only weekday symbols and lacks an integer/symbol output selector.
7. `DatePlus[spec][date]` is missing despite the unary spec form being unambiguous.
8. `ConformSchema[schema][value]` is missing for validation pipelines.
9. `SchemaErrors[schema][value]` is missing for validation pipelines.
10. Digest output format discovery is weak: the implemented `"Bytes"` format is not documented by `Options[...]`, and there is no explicit `"Hex"` default alias.

## Findings Table

### JSON

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromJson | rum | gap | safety | Malformed JSON and duplicate keys fail closed, and exact terminating rationals round-trip; full JSON beyond the documented subset is not a gap. `Attributes[FromJson]` is empty, leaving a parser head unprotected (Evidence E1, E2). | Add `Protected`; keep current fail-closed JSON behavior unchanged. | S |
| ToJson | rum | gap | convenience | Deterministic output, exact-rational round-trip, non-terminating-rational failure, and `"Indent" -> n` work as intended. The implemented option is not discoverable (`Options[ToJson] == {}`), and `Attributes[ToJson]` is empty (Evidence E1, E2, E10; `crates/rum-stdlib/src/json.rs:62`). | Register `Options[ToJson] = {"Indent" -> None or Automatic}` and add `Protected`; behavior unchanged. | S |

### CSV

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromCsv | rum | gap | convenience | Plain ragged rows are accepted by contract, while `Headers -> True` ragged rows, duplicate headers, and malformed quoting fail closed. The implemented `"Delimiter"`/`"Headers"` options are not discoverable, and the head is unprotected (Evidence E1, E3, E10; `crates/rum-stdlib/src/csv.rs:120`). | Register option defaults and add `Protected`; keep accepted ragged plain rows unchanged. | S |
| ToCsv | rum | gap | convenience | String-row rendering and delimiter selection work, and non-string fields fail closed. The implemented `"Delimiter"`/`"Headers"` options are not discoverable, and the head is unprotected (Evidence E1, E3, E10; `crates/rum-stdlib/src/csv.rs:150`). | Register option defaults and add `Protected`; behavior unchanged. | S |

### INI

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromIni | rum | gap | safety | The documented INI dialect parses successfully, while duplicate sections and malformed lines fail closed; no full-INI feature gap is filed. `Attributes[FromIni]` is empty (Evidence E1, E4). | Add `Protected`; keep the folk INI subset unchanged. | S |
| ToIni | rum | gap | safety | Association-of-associations rendering works, and non-string values fail closed as unrepresentable in the dialect. `Attributes[ToIni]` is empty (Evidence E1, E4). | Add `Protected`; keep current representability rules unchanged. | S |

### TOML

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromToml | rum | gap | safety | Duplicate keys and TOML `inf`/`nan` fail closed; the exact-rational and folk-subset boundaries are intended. `Attributes[FromToml]` is empty (Evidence E1, E4). | Add `Protected`; do not broaden to full TOML. | S |
| ToToml | rum | gap | safety | Deterministic exact output works, and non-terminating rationals fail closed. `Attributes[ToToml]` is empty (Evidence E1, E4). | Add `Protected`; keep current representable subset unchanged. | S |

### XML

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromXml | rum | gap | safety | Value-oriented XML parsing works, and mismatched tags plus DTD/DOCTYPE input fail closed; namespace resolution and full XML processing are intended non-goals. `Attributes[FromXml]` is empty (Evidence E1, E5). | Add `Protected`; keep DTD rejection and value-oriented XML scope unchanged. | S |
| ToXml | rum | gap | convenience | Compact and `"Indent" -> n` output work, and invalid XML names fail closed. The implemented option is not discoverable, and `Attributes[ToXml]` is empty (Evidence E1, E5, E10; `crates/rum-stdlib/src/xml.rs:53`). | Register the `"Indent"` option default and add `Protected`. | S |
| XmlElement | WL-ish (`XMLElement`) | gap | safety | The inert XML constructor behaves as data and round-trips through `FromXml`/`ToXml`, but `Attributes[XmlElement]` is empty, so the constructor head is redefinable (Evidence E1, E5). | Add `Protected`; no evaluator behavior needed. | S |

### YAML

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FromYaml | rum | gap | safety | The documented block subset parses; duplicate keys and anchors fail closed. Anchors, aliases, tags, flow collections, and merge keys are intended non-goals. `Attributes[FromYaml]` is empty (Evidence E1, E5). | Add `Protected`; keep fail-closed folk YAML scope unchanged. | S |
| ToYaml | rum | gap | safety | Representable Associations/Lists render deterministically, and non-terminating rationals fail closed. `Attributes[ToYaml]` is empty (Evidence E1, E5). | Add `Protected`; behavior unchanged. | S |

### Schema

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| ConformSchema | rum | gap | convenience | Valid schemas conform and fill defaults; malformed schemas fail closed. The implemented `"AllowExtraKeys"` option is not discoverable, and the head lacks `Protected` (Evidence E1, E6, E10; `crates/rum-stdlib/src/schema.rs:296`). | Register `"AllowExtraKeys" -> False` and add `Protected`; keep held schema semantics unchanged. | S |
| ConformSchema | rum | gap | flexibility | `ConformSchema[schema][value]` fails at wrapper construction; the unary schema form is not a data form and would be useful after codec decode pipelines (Evidence E11). | Add an operator form for schema-first validation; this touches call planning and needs the A/B gate. | M |
| SchemaErrors | rum | gap | convenience | Structured error-data output works, and malformed schemas fail closed. The implemented `"AllowExtraKeys"` option is not discoverable, and the head lacks `Protected` (Evidence E1, E6, E10; `crates/rum-stdlib/src/schema.rs:296`). | Register `"AllowExtraKeys" -> False` and add `Protected`; behavior unchanged. | S |
| SchemaErrors | rum | gap | flexibility | `SchemaErrors[schema][value]` fails at wrapper construction; the unary schema form is not a data form and would be useful after codec decode pipelines (Evidence E11). | Add an operator form parallel to `ConformSchema[schema]`; this touches call planning and needs the A/B gate. | M |

### Bytes

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Sha256 | rum | gap | safety | The default hex digest and `"Bytes"` output work, but an unknown second format argument silently returns inert (`Sha256["abc", "Base64"]`). `Attributes[Sha256]` is empty (Evidence E1, E7; `crates/rum-stdlib/src/bytes/digest.rs:75`). | Fail closed on unknown format and add `Protected`; optionally accept explicit `"Hex"` and `"Base64"` output names. | S |
| Sha512 | rum | gap | safety | Shares the generic digest format parser: unknown second format arguments return inert instead of a message. `Attributes[Sha512]` is empty (Evidence E1, E7; `crates/rum-stdlib/src/bytes/digest.rs:75`). | Use the same fail-closed format handling as `Sha256`; add `Protected`. | S |
| Sha1 | rum | gap | safety | The reference correctly warns that SHA-1 is for interoperability/checksums, not collision resistance, but the generic digest format parser silently returns inert on unknown formats. `Attributes[Sha1]` is empty (Evidence E1, E7, E13; `crates/rum-stdlib/src/bytes/digest.rs:75`). | Fail closed on unknown format and add `Protected`; keep the legacy warning. | S |
| Md5 | rum | gap | safety | The reference correctly warns that MD5 is for interoperability/checksums, not collision resistance, but the generic digest format parser silently returns inert on unknown formats. `Attributes[Md5]` is empty (Evidence E1, E7, E13; `crates/rum-stdlib/src/bytes/digest.rs:75`). | Fail closed on unknown format and add `Protected`; keep the legacy warning. | S |
| Crc32 | rum | gap | safety | CRC-32 computes correctly, but the reference lacks the explicit non-security warning present for MD5/SHA-1. `Attributes[Crc32]` is empty (Evidence E1, E7, E13; `docs/reference/builtins.md:781`). | Add a Limits note that CRC-32 is for checksums/compatibility, not security; add `Protected`. | S |
| ToHexString | rum | gap | safety | String/ByteArray encoding works and invalid ByteArray payloads fail closed; the head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:16`). | Add `Protected`; behavior unchanged. | S |
| FromHexString | rum | gap | safety | Odd-length and non-hex input fail closed with a message; the head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:36`). | Add `Protected`; behavior unchanged. | S |
| ToBase64 | rum | gap | safety | String/ByteArray encoding works and invalid ByteArray payloads fail closed; the head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:59`). | Add `Protected`; behavior unchanged. | S |
| FromBase64 | rum | gap | safety | Invalid padding and invalid alphabet input fail closed; the head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:79`). | Add `Protected`; behavior unchanged. | S |
| ByteArrayQ | WL | gap | safety | Valid ByteArray payloads return `True` and invalid payloads return `False`, so classification is correct. The predicate head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:102`). | Add `Protected`; behavior unchanged. | S |
| ByteArrayToString | WL | gap | safety | Valid UTF-8 payloads convert and invalid UTF-8 fails closed; the head is unprotected (Evidence E1, E7; `crates/rum-stdlib/src/bytes.rs:115`). | Add `Protected`; behavior unchanged. | S |

### Datetime

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| DateList | WL | gap | safety | Deterministic `DateList[0]` works; malformed date lists fail closed; zero-argument now is correctly `Capability::Clock` gated in sandboxed hosts. The head is unprotected (Evidence E1, E8, E9, E12; `crates/rum-stdlib/src/datetime.rs:17`, `crates/rum-stdlib/src/datetime.rs:171`). | Add `Protected`; keep the current clock gate and UTC/proleptic-Gregorian boundary. | S |
| DateString | WL | gap | flexibility | Zero-argument now is correctly clock-gated, deterministic timestamp formatting works, and non-terminating or malformed dates fail closed. `DateString[date, format]` is rejected, so there is no cheap date-only/time-only format selector; the head is unprotected (Evidence E1, E8, E9, E12). | Add a small documented format enum such as `"ISODateTime"`/`"ISODate"`/`"ISOTime"` and add `Protected`; do not add locale or time-zone databases. | M |
| FromDateString | WL | gap | safety | ISO UTC and fixed-offset strings parse exactly, and invalid strings/out-of-range dates/leap seconds fail closed. The head is unprotected (Evidence E1, E8). | Add `Protected`; keep the narrow ISO subset. | S |
| DatePlus | WL | gap | flexibility | Fixed-duration units work, and Months/Years fail closed as an intended folk boundary. `DatePlus[spec][date]` is missing even though unary spec is not a data form; the head is unprotected (Evidence E1, E8, E11; `crates/rum-stdlib/src/datetime/calendar.rs:337`). | Add `DatePlus[spec]` as an operator form; this touches call planning and needs the A/B gate. Add `Protected`. | M |
| DateDifference | WL | gap | safety | Fixed-duration differences work, and Months/Years fail closed as intended. The head is unprotected (Evidence E1, E8; `crates/rum-stdlib/src/datetime/calendar.rs:337`). | Add `Protected`; keep fixed-duration scope unchanged. | S |
| DayOfWeek | WL | gap | flexibility | The current symbol result works, but `DayOfWeek[date, "Integer"]` is rejected, so scripts cannot request ISO integer weekdays directly; the head is unprotected (Evidence E1, E8). | Add a cheap second argument for `"Name"`/`"ISOInteger"` or equivalent documented output selection; add `Protected`. | S |
| LeapYearQ | WL | gap | safety | Exact integer years work and non-integer years fail closed. The head is unprotected (Evidence E1, E8). | Add `Protected`; behavior unchanged. | S |
| DaysInMonth | WL | gap | safety | Valid months work and out-of-range months fail closed. The head is unprotected (Evidence E1, E8). | Add `Protected`; behavior unchanged. | S |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin
to `cargo run`, except for the targeted host sandbox test.

### E1: Attribute And Option Survey

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Attributes /@ {FromJson, ToJson, FromCsv, ToCsv, FromIni, ToIni, FromToml, ToToml, FromXml, ToXml, XmlElement, FromYaml, ToYaml, ConformSchema, SchemaErrors, Sha256, Sha512, Sha1, Md5, Crc32, ToHexString, FromHexString, ToBase64, FromBase64, ByteArrayQ, ByteArrayToString, DateList, DateString, FromDateString, DatePlus, DateDifference, DayOfWeek, LeapYearQ, DaysInMonth}, Options /@ {ToJson, FromCsv, ToCsv, ToXml, DateString, DayOfWeek, Sha256, Md5, Crc32}}'\'''
List[List[List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[HoldRest], List[HoldRest], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[], List[]], List[List[], List[], List[], List[], List[], List[], List[], List[], List[]]]
```

### E2: JSON Fail-Closed, Determinism, And Round-Trip Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''a = <|"b" -> {True, Null, 3/2}, "a" -> 1|>; {ToJson[a], ToJson[a], FromJson[ToJson[a]], SameQ[FromJson[ToJson[a]], a], ToJson[a, "Indent" -> 2], FromJson["{"], FromJson["{\"a\":1,}"], FromJson["{\"a\":1,\"a\":2}"], ToJson[1/3]}'\'''
! FromJson::json: Invalid JSON at byte offset 1: object keys must be double-quoted strings.
! FromJson::json: Invalid JSON at byte offset 7: object keys must be double-quoted strings.
! FromJson::json: Invalid JSON at byte offset 7: duplicate object key "a".
! ToJson::json: Cannot encode non-terminating Rational 1/3 as JSON; convert it explicitly before calling ToJson.
List["{\"b\":[true,null,1.5],\"a\":1}", "{\"b\":[true,null,1.5],\"a\":1}", Association[Rule["b", List[True, Null, 3/2]], Rule["a", 1]], True, "{\n  \"b\": [\n    true,\n    null,\n    1.5\n  ],\n  \"a\": 1\n}", $Failed, $Failed, $Failed, $Failed]
```

### E3: CSV Fail-Closed And Options Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{FromCsv["a,b\n1,2"], FromCsv["a,b\n1", "Headers" -> True], FromCsv["a,b\n1"], FromCsv["a,a\n1,2", "Headers" -> True], FromCsv["\"unterminated"], FromCsv["a;b\n1;2", "Delimiter" -> ";"], ToCsv[{{"a", "b"}, {"1", "2"}}], ToCsv[{{"a", "b"}}, "Delimiter" -> ";"], ToCsv[{{"a", 1}}]}'\'''
! FromCsv::csv: Invalid CSV at byte offset 4: row 2 has 1 fields but the header has 2 fields.
! FromCsv::csv: Invalid CSV at byte offset 2: duplicate header name "a".
! FromCsv::csv: Invalid CSV at byte offset 0: unterminated quoted field.
! ToCsv::csv: Non-String CSV field at row 1, column 2.
List[List[List["a", "b"], List["1", "2"]], $Failed, List[List["a", "b"], List["1"]], $Failed, $Failed, List[List["a", "b"], List["1", "2"]], "a,b\n1,2", "a;b", $Failed]
```

### E4: INI And TOML Fail-Closed Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{FromIni["[s]\nk=v"], FromIni["[s]\na=1\n[s]\nb=2"], FromIni["not a pair"], ToIni[<|"s" -> <|"k" -> "v"|>|>], ToIni[<|"s" -> <|"k" -> 1|>|>], FromToml["a = 1\nb = 1.5"], FromToml["a = 1\na = 2"], FromToml["a = inf"], ToToml[<|"a" -> 1, "b" -> 3/2|>], ToToml[<|"a" -> 1/3|>]}'\'''
! FromIni::ini: Invalid INI at byte offset 8: duplicate section name "s".
! FromIni::ini: Invalid INI at byte offset 0: expected a section header or key = value line.
! ToIni::ini: ToIni value for key "k" in section "s" must be a String.
! FromToml::toml: Invalid TOML at byte offset 6: duplicate key "a".
! FromToml::toml: Invalid TOML at byte offset 4: inf and nan are outside rum's exact TOML number subset.
! ToToml::toml: Cannot encode non-terminating Rational 1/3 as TOML; convert it explicitly before calling ToToml.
List[Association[Rule["s", Association[Rule["k", "v"]]]], $Failed, $Failed, "[s]\nk = v", $Failed, Association[Rule["a", 1], Rule["b", 3/2]], $Failed, $Failed, "a = 1\nb = 1.5", $Failed]
```

### E5: XML And YAML Fail-Closed Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{FromXml["<r a=\"1\"><c/>text</r>"], FromXml["<a></b>"], FromXml["<!DOCTYPE r><r/>"], ToXml[XmlElement["r", <|"a" -> "1"|>, {"text"}]], ToXml[XmlElement["r", <||>, {XmlElement["c", <||>, {}]}], "Indent" -> 2], ToXml[XmlElement["1bad", <||>, {}]], FromYaml["a: 1\nb:\n  - true\n"], FromYaml["a: 1\na: 2\n"], FromYaml["a: &x 1\nb: *x\n"], ToYaml[<|"a" -> 1, "b" -> {True, "x"}|>], ToYaml[1/3]}'\'''
List[XmlElement["r", Association[Rule["a", "1"]], List[XmlElement["c", Association[], List[]], "text"]], $Failed, $Failed, "<r a=\"1\">text</r>", "<r>\n  <c/>\n</r>", $Failed, Association[Rule["a", 1], Rule["b", List[True]]], $Failed, $Failed, "a: 1\nb:\n  - true\n  - x", $Failed]
! FromXml::xml: Invalid XML at byte offset 3: mismatched closing tag </b>; expected </a>.
! FromXml::xml: Invalid XML at byte offset 0: DTD/DOCTYPE declarations are deliberately unsupported; FromXml never processes DTDs or external entities.
! ToXml::xml: XML tag "1bad" is outside the supported XML name subset.
! FromYaml::yaml: Invalid YAML at byte offset 5: duplicate mapping key "a".
! FromYaml::yaml: Invalid YAML at byte offset 3: YAML anchors are unsupported.
! ToYaml::yaml: Cannot encode non-terminating Rational 1/3 as YAML; convert it explicitly before calling ToYaml.
```

### E6: Schema Validation And Malformed Schema Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{ConformSchema[<|"name" -> "web", "port" -> 8080|>, <|"name" -> _String, "port" -> p_Integer /; p > 0, "debug" -> Optional[True | False, False]|>], SchemaErrors[<|"port" -> "x", "extra" -> 1|>, <|"name" -> _String, "port" -> _Integer|>], ConformSchema[<|"a" -> 1|>, <|"a" -> {_Integer, _Integer}|>], SchemaErrors[<|"a" -> 1|>, <|a -> _Integer|>], ConformSchema[<|"port" -> 8080, "extra" -> "kept"|>, <|"port" -> _Integer|>, "AllowExtraKeys" -> True], ConformSchema[<||>, <|"a" -> Optional[_Integer, "x"]|>]}'\'''
List[Association[Rule["name", "web"], Rule["port", 8080], Rule["debug", False]], List[Association[Rule["Path", "name"], Rule["Message", "missing required key"]], Association[Rule["Path", "port"], Rule["Message", "expected to match _Integer, got \"x\""]], Association[Rule["Path", "extra"], Rule["Message", "extra key is not allowed"]]], $Failed, $Failed, Association[Rule["port", 8080], Rule["extra", "kept"]], $Failed]
! ConformSchema::schema: Schema path a: list schema must contain exactly one element; got 2
! SchemaErrors::schema: Schema path <root>: association schema keys must be Strings; got a
! ConformSchema::schema: a: expected to match _Integer, got "x"
```

### E7: Bytes And Digest Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''ba = FromHexString["4869"]; {ToHexString["Hi"], ba, ByteArrayQ[ba], ByteArrayQ[ByteArray["!!"]], ByteArrayQ[ByteArray["AA=="]], ByteArrayToString[ba], ByteArrayToString[FromHexString["ff"]], FromHexString["f"], FromHexString["zz"], ToBase64["Hi"], FromBase64["SGk="], FromBase64["SGk"], FromBase64["@@@@"], ToBase64[ByteArray["!!"]], Sha256["abc"], Sha256["abc", "Bytes"], Sha256["abc", "Base64"], Sha1["abc"], Md5["abc"], Crc32["abc"]}'\'''
! ByteArrayToString::utf8: ByteArray payload is not valid UTF-8
! FromHexString::hex: expected an even-length hexadecimal string
! FromHexString::hex: expected an even-length hexadecimal string
! FromBase64::base64: expected standard RFC 4648 base64 with valid padding
! FromBase64::base64: expected standard RFC 4648 base64 with valid padding
! ToBase64::bytearray: invalid ByteArray payload
List["4869", ByteArray["SGk="], True, False, True, "Hi", $Failed, $Failed, $Failed, "SGk=", ByteArray["SGk="], $Failed, $Failed, $Failed, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad", ByteArray["ungWv48Bz+pBQUDeXa4iI7ADYaOWF3qctBD/YfIAFa0="], Sha256["abc", "Base64"], "a9993e364706816aba3e25717850c26c9cd0d89d", "900150983cd24fb0d6963f7d28e17f72", 891568578]
```

### E8: Datetime Deterministic And Fail-Closed Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{DateList[0], DateString[0], FromDateString["1970-01-01T00:00:00Z"], FromDateString["1970-01-01T01:00:00+01:00"], FromDateString["not a date"], FromDateString["2023-02-29T00:00:00Z"], FromDateString["2016-12-31T23:59:60Z"], DateList[{2020, 2, 30}], DatePlus[0, {1, "Days"}], DatePlus[0, {1, "Months"}], DateDifference[0, 86400, "Days"], DateDifference[0, 1, "Months"], DayOfWeek[0], DayOfWeek[0, "Integer"], DateString[0, "ISODate"], LeapYearQ[2020], LeapYearQ[3/2], DaysInMonth[2020, 2], DaysInMonth[2020, 13]}'\'''
List[List[1970, 1, 1, 0, 0, 0], "1970-01-01T00:00:00Z", 0, 0, $Failed, $Failed, $Failed, $Failed, 86400, $Failed, 1, $Failed, Thursday, DayOfWeek[0, "Integer"], DateString[0, "ISODate"], True, $Failed, 29, $Failed]
! FromDateString::date: Invalid ISO 8601 date/time at byte offset 0: expected 4 digit year.
! FromDateString::date: Invalid ISO 8601 date/time at byte offset 8: day must be in 01..28 for this month.
! FromDateString::date: Invalid ISO 8601 date/time at byte offset 17: second must be in 00..59.
! DateList::date: Date list day 30 is invalid for month 2 in year 2020.
! DatePlus::date: Calendar unit "Months" is outside rum's UTC fixed-duration date folk subset.
! DateDifference::date: Calendar unit "Months" is outside rum's UTC fixed-duration date folk subset.
! DayOfWeek::argx: expected exactly 1 argument(s), got 2
! DateString::argtu: expected between 0 and 1 argument(s), got 2
! LeapYearQ::date: LeapYearQ expects an exact integer year.
! DaysInMonth::date: Date list month must be in the range 1..12.
! DayOfWeek::argx: expected exactly 1 argument(s), got 2
! DateString::argtu: expected between 0 and 1 argument(s), got 2
```

### E9: Trusted CLI Current-Time Probe

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Length[DateList[]], StringContainsQ[DateString[], "T"], DateList[0], DateString[0]}'\'''
List[6, True, List[1970, 1, 1, 0, 0, 0], "1970-01-01T00:00:00Z"]
```

### E10: Implemented Options Not Registered

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Options /@ {ToJson, FromCsv, ToCsv, ToXml, ConformSchema, SchemaErrors}, ConformSchema[<|"port" -> 1, "extra" -> 2|>, <|"port" -> _Integer|>, "AllowExtraKeys" -> True], SchemaErrors[<|"port" -> 1, "extra" -> 2|>, <|"port" -> _Integer|>, "AllowExtraKeys" -> True]}'\'''
List[List[List[], List[], List[], List[], List[], List[]], Association[Rule["port", 1], Rule["extra", 2]], List[]]
```

### E11: Operator-Form Candidates

```text
$ scripts/run-bounded.py --timeout 90 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{DatePlus[{1, "Days"}][0], ConformSchema[<|"a" -> _Integer|>][<|"a" -> 1|>], SchemaErrors[<|"a" -> _Integer|>][<|"a" -> "x"|>]}'\'''
! DatePlus::argx: expected exactly 2 argument(s), got 1
! ConformSchema::schema: ConformSchema expects a value, a schema, and optional option rules.
! SchemaErrors::schema: SchemaErrors expects a value, a schema, and optional option rules.
! DatePlus::argx: expected exactly 2 argument(s), got 1
List[DatePlus[List[1, "Days"]][0], $Failed[Association[Rule["a", 1]]], $Failed[Association[Rule["a", "x"]]]]
```

### E12: Sandboxed Clock Gate

```text
$ scripts/run-bounded.py --timeout 120 -- cargo test -p rum-host sandboxed_host_config_blocks_datetime_current_readers_but_allows_pure_forms -- --nocapture
   Compiling rum-host v0.1.0 (/Users/fft/Developments/rum/crates/rum-host)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 3.22s
     Running unittests src/lib.rs (target/debug/deps/rum_host-89d65d88c1633f3d)

running 1 test
test tests::sandboxed_host_config_blocks_datetime_current_readers_but_allows_pure_forms ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 43 filtered out; finished in 0.00s

     Running tests/adapter_contract.rs (target/debug/deps/adapter_contract-c7696f9f7ce03d8f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/installer.rs (target/debug/deps/installer-f71e33ddf9baf0ec)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s

     Running tests/lifecycle_buildout.rs (target/debug/deps/lifecycle_buildout-eac6f75e3d0acb33)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.00s

     Running tests/lifecycle_contract.rs (target/debug/deps/lifecycle_contract-f026261285791092)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s

     Running tests/random_password_policy.rs (target/debug/deps/random_password_policy-d4db37d5507bd5fc)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out; finished in 0.00s

     Running tests/stress.rs (target/debug/deps/stress-b72da7158d93499f)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.00s
```

The test body also directly asserts `DateList[]` and `DateString[]` are denied
under `HostConfig::sandboxed()` while `DateList[0]` and `DateString[0]` remain
available (`crates/rum-host/src/tests.rs:346`). The runtime gate is in
`clock_capability_allowed` (`crates/rum-stdlib/src/datetime.rs:171`).

### E13: Digest Policy Reference Citations

- `Sha1` and `Md5` have a Limits note that they are for interoperability and
  checksums, not collision-resistant (`docs/reference/builtins.md:3285`,
  `docs/reference/builtins.md:2141`).
- `Crc32` is described only as the IEEE CRC-32 checksum with no explicit
  non-security warning (`docs/reference/builtins.md:781`).

## Triage Sketch

1. **Digest and checksum safety hygiene**: make unknown digest output formats
   message and fail closed; document/register digest output choices; add the
   missing `Crc32` non-security Limits note. Small stdlib/docgen/reference
   batch; no call planning.
2. **Attribute protection sweep**: add `Protected` to the 34 codec/bytes/date
   heads, preserving existing `HoldRest` on schema heads. This is attribute
   hygiene, but not attribute-driven call planning like `Flat`/`Orderless`;
   no microbench A/B expected.
3. **Option discovery registration**: register existing option defaults for
   `ToJson`, `FromCsv`, `ToCsv`, `ToXml`, `ConformSchema`, and `SchemaErrors`.
   Behavior stays byte-identical; no call planning.
4. **Small datetime ergonomics**: add cheap `DateString` format names and a
   `DayOfWeek` output selector. Keep UTC, exact seconds, no locale/time-zone
   database, and fixed-duration-only date arithmetic.
5. **Pipeline operator forms**: add `DatePlus[spec]`, `ConformSchema[schema]`,
   and `SchemaErrors[schema]`. This touches wrapper/call planning and needs the
   core-language microbench A/B gate.

