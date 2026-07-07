# Brief 194 — Usability audit: codecs + bytes + datetime (34 builtins)

Audit brief — sweep unit 6 of the Usability Pass (the codec family
json/csv/ini/toml/xml/yaml/schema + bytes + datetime). Read
`docs/dev/usability-pass.md` FIRST: the three axes, the compat rule, the
coverage discipline, and the closed-unit ledgers (units 1-5) — they show
the judgment standard, the corrections method, and the disambiguation
rule for operator forms. This is an AUDIT: you change NO behavior and NO
code; the only file you create is the findings document.

## Objective

Sweep every builtin in the nine catalog families of this unit against
the three usability axes and write
`docs/dev/usability-pass/codecs-bytes-datetime-findings.md`.

## The scope guardrail (READ TWICE — this governs the whole unit)

This unit is the serialization / hashing / calendar surface. Unlike the
math unit the risk is NOT false CAS gaps; it is (a) filing false gaps for
missing FULL SPEC coverage, and (b) missing the REAL high-value axis here,
which is fail-closed parsing. Four design boundaries are INTENDED, not
gaps:

- **Folk / good-enough codecs, not full spec engines.** rum's codecs
  cover a documented, useful SUBSET (e.g. `ToJson` renders "the
  JSON-representable subset"; the XML/TOML/YAML readers are folk
  parsers). A COMPLEX or exotic input getting a fail-closed message or an
  inert result is CORRECT folk behavior. Do NOT propose "support full
  XSD / XML namespaces / YAML anchors & merge keys / every TOML 1.0
  corner / streaming parse / arbitrary schema languages" — out of scope
  by design; filing them wastes the sweep.
- **Rationals-only, no floating point (BY DESIGN).** JSON/TOML/CSV
  numbers lower to exact rationals; `1.5` becomes `3/2`. This is the
  contract, NOT a gap (see `docs/guide/design-boundaries.md`). A
  round-trip FIDELITY question (does `ToJson[3/2]` emit `1.5` or `3/2`,
  and does it re-read equal?) IS a legitimate probe; "add IEEE floats"
  is NOT a proposal.
- **Determinism by design.** Serialized output (association key order in
  JSON/TOML, etc.) is deterministic on purpose (the FS-purity line's
  "deterministic by default"). Deterministic ordering is a CONTRACT, not
  a gap. A NON-deterministic ordering hole WOULD be a real safety finding.
- **No ambient clock.** `DateList[]` / `DateString[]` (zero-arg "now"
  forms) reach `SystemTime::now()` only behind `Capability::Clock`
  (`datetime.rs:171` `clock_capability_allowed`); `sandboxed()` drops
  Clock so the now-forms fail closed there. This capability gating is the
  intended safety design — do NOT propose "make DateList[] read the wall
  clock unconditionally". Verifying the gate holds (and is labeled
  correctly) IS a finding; removing it is not.

What IS a legitimate finding here (the three axes, tuned to this unit):

- **Safety (HIGHEST value this unit):** a `From*` parser or a decoder
  that returns a SILENT wrong / partial / inert result on malformed input
  instead of failing closed with a message + `$Failed`. Prime targets:
  `FromJson` on invalid JSON (`"{"`, single-quoted keys, trailing
  commas), `FromCsv` on ragged rows, `FromToml`/`FromXml`/`FromYaml`/
  `FromIni` on malformed text, `FromHexString` on odd-length / non-hex,
  `FromBase64` on invalid padding, `FromDateString` on an unparseable or
  out-of-range string, `DateList`/`DatePlus` on out-of-range components,
  `ConformSchema`/`SchemaErrors` on a malformed schema, `ByteArrayQ`
  misclassification. Also: **round-trip corruption** (`FromJson[ToJson[
  e]]` ≠ `e` for a representable `e`) and **determinism holes**
  (`ToJson[assoc]` differing across two calls). Fail-closed +
  round-trip-faithful + deterministic beats a silent wrong answer.
- **Capability / policy labeling (safety):** confirm the `DateList[]` /
  `DateString[]` now-forms are correctly `Capability::Clock`-gated (the
  unit-4 `Exit`-mislabel check applied here) — `sandboxed()` blocks the
  zero-arg form while the with-argument (deterministic) forms stay
  available. Note the digest-policy hygiene (cf. brief-163): `Md5` /
  `Sha1` / `Crc32` are non-cryptographic / legacy — is that stated so a
  user cannot mistake them for a security primitive? (This is a
  doc/hygiene note, NOT a removal — they are legitimate for
  checksums/compat.)
- **Flexibility:** WL-documented (or obviously useful, folk-cheap)
  OPTIONS / call forms currently missing — e.g. `ToJson` compact vs
  pretty-print, `FromCsv`/`ToCsv` delimiter / quoting, `DateString`
  format spec, `DayOfWeek` symbol-vs-integer, an encoding argument. Only
  forms rum can fold cheaply; NOTE-but-do-not-propose anything needing a
  new parser or spec depth. Most heads here are ≤2-arg codec verbs;
  apply the operator-form disambiguation rule (only arities that cannot
  be a data form) — most will NOT qualify.
- **Convenience:** `Protected` on these heads (constant-like codec/verb
  heads should be write-protected — cite `Attributes[...]`); attribute
  parity; sibling-name ergonomics. An attribute mismatch that changes
  real behavior is a finding.

Origin note: many heads here are rum-NATIVE codec names (`FromJson`/
`ToJson`/`FromCsv`/… — WL serializes via `ImportString`/`ExportString`,
not these names), while `DateList`/`DateString`/`DayOfWeek`/`LeapYearQ`
are WL-inherited names and `XmlElement` ~ WL `XMLElement`. Mark Origin
honestly; for rum-native heads judge on the three axes intrinsically, not
on WL parity.

## Inputs

- Catalog name lists (authoritative, 34 total — findings table has AT
  LEAST one row per entry, grouped by family, catalog order):
  `crates/rum-stdlib/src/catalog/json.rs` (2: FromJson, ToJson),
  `csv.rs` (2: FromCsv, ToCsv), `ini.rs` (2: FromIni, ToIni),
  `toml.rs` (2: FromToml, ToToml),
  `xml.rs` (3: FromXml, ToXml, XmlElement),
  `yaml.rs` (2: FromYaml, ToYaml),
  `schema.rs` (2: ConformSchema, SchemaErrors),
  `bytes.rs` (11: Sha256, Sha512, Sha1, Md5, Crc32, ToHexString,
  FromHexString, ToBase64, FromBase64, ByteArrayQ, ByteArrayToString),
  `datetime.rs` (8: DateList, DateString, FromDateString, DatePlus,
  DateDifference, DayOfWeek, LeapYearQ, DaysInMonth).
- Implementations (confirm the paths): `crates/rum-stdlib/src/json/`
  + `json.rs`, `csv.rs`, `ini.rs`, `toml/` + `toml.rs`, `xml/` +
  `xml.rs`, `yaml/` + `yaml.rs`, `schema.rs`, `bytes/` + `bytes.rs`,
  `datetime/` + `datetime.rs`.
- Contracts: entries in `docs/reference/builtins.md`;
  `docs/guide/design-boundaries.md` (rationals-only + determinism +
  no-ambient-authority — the authority for what is intentionally out of
  scope); the FS-purity line's `Capability::Clock` gating in
  `datetime.rs`.
- WL comparison: documented WL surfaces from your knowledge (options,
  extra argument forms, attributes, domain-error messages). Design
  reference ONLY — never run an external engine. For rum-native codec
  names there is no WL parity target; judge intrinsically.

## Findings file format

Same as units 1-5 (the five existing findings files are the template):
the seven-column table (`Builtin | Origin | Verdict | Axis | Finding |
Proposal | Size`), `OK` rows REQUIRED (the all-rows table IS the
coverage proof), one row per axis-gap; then an **Evidence appendix**
(verbatim probe transcripts, stdin fed: `printf '' | cargo run -q -p
rum-cli -- --eval '…'`; batch related checks; `file:line` citations
acceptable for code-visible gaps) and a **Triage sketch** (2-5 candidate
fix batches ranked by script-user value, noting which touch call
planning / attribute-driven planning and therefore need the A/B gate).

For every row, mark clearly whether an apparent gap is a REAL usability
finding or an intentional design boundary (full-spec / rationals-only /
determinism / no-ambient-clock — an `OK` row with a one-line "intended"
note, NOT a gap). Getting this classification right is the point of the
unit.

## Probe budget

Cap ~40 probe evals total (34 builtins); anything needing more becomes an
open question. Expected wall time 25-40 min. Batch related checks into
one `--eval` list where possible (e.g. all the malformed-input
fail-closed probes for one codec together); feed stdin (`printf '' | …`)
so the CLI does not block. For the capability check, probe both the
trusted CLI and, if reachable, the `sandboxed()` path per the tracker's
method (or cite `datetime.rs` capability code where a runtime probe is
not cheap).

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/json.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/csv.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/ini.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/toml.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/xml.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/yaml.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/schema.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/bytes.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/datetime.rs
# one-off shell check: every catalog name from the NINE files appears as a
# table row; must print COVERAGE-OK 34/34 (paste the check script + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create:
  `docs/dev/usability-pass/codecs-bytes-datetime-findings.md` — the ONLY
  file in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`,
  the tracker, the other findings files, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new
  dependencies, no code changes.
- Every `gap` row is evidence-backed: a probe transcript or a
  `file:line` citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines.
- The design boundaries are hard lines: do NOT propose full spec
  coverage, floating point, an unconditional wall clock, or any
  crypto/encryption primitive (policy: never in kernel — cf. brief-163).
  A row that would require any of those is an `OK` boundary row, not a
  gap.
- Stop-loss: if the probe surface misbehaves, STOP after two attempts on
  the same probe and report it.

## Report format

1. Coverage statement: rows vs names, the check output verbatim
   (34/34).
2. Top 10 gaps by script-user value, one line each (mark the safety ones
   — silent-wrong / not-fail-closed parsers, round-trip corruption,
   determinism or capability holes — FIRST).
3. The triage sketch (note which batches touch call planning /
   attribute-driven planning and need the A/B gate).
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
