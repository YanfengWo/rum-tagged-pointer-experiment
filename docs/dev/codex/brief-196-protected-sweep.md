# Brief 196 — Protected sweep on the 34 codec/bytes/datetime heads (unit-6 fix 2)

Implementation brief. Executes the unit-6 triage batch 2 (tracker
`docs/dev/usability-pass.md`, "Unit 6 (codecs+bytes+datetime) triage";
evidence E1 in `docs/dev/usability-pass/codecs-bytes-datetime-findings.md`).
Read the tracker's unit-6 triage AND brief-193 first — 193 is the exact
worked precedent for this ripple (catalog attribute change → runtime
mirror sync → reference regen). This is attribute hygiene ONLY; no
`eval_*` handler changes, no behavior change beyond write-protection.

## What and why

All 34 unit-6 heads have EMPTY attributes today (`Attributes[FromJson]`,
`Attributes[Sha256]`, `Attributes[DateList]`, … all `{}`; the two schema
heads are `{HoldRest}`), so every codec/parser/date verb head and the
`XmlElement` constructor is silently redefinable. WL protects these
built-in heads. Add `Protected` to all 34, PRESERVING `HoldRest` on the
two schema heads.

`Protected` is NOT an attribute-driven-planning attribute: unlike
`Flat`/`Orderless`, it is not consulted in `eval_call_prepare` argument
normalization, so there is NO per-call eval cost and NO argument-ordering
change. NO microbench A/B is needed (same disposition as brief-193's
`Protected`/`NumericFunction`). Do NOT add `ReadProtected`, `Listable`, or
any other attribute — `Protected` (plus the preserved `HoldRest`) ONLY.

## The 34 heads (by catalog file)

- `catalog/json.rs`: FromJson, ToJson
- `catalog/csv.rs`: FromCsv, ToCsv
- `catalog/ini.rs`: FromIni, ToIni
- `catalog/toml.rs`: FromToml, ToToml
- `catalog/xml.rs`: FromXml, ToXml, XmlElement
- `catalog/yaml.rs`: FromYaml, ToYaml
- `catalog/schema.rs`: ConformSchema, SchemaErrors  **(HoldRest → HoldRest + Protected)**
- `catalog/bytes.rs` `DIGEST_BUILTINS` (`#[cfg(feature = "hash")]`): Sha256,
  Sha512, Sha1, Md5, Crc32
- `catalog/bytes.rs` `BYTE_BUILTINS`: ToHexString, FromHexString, ToBase64,
  FromBase64, ByteArrayQ, ByteArrayToString
- `catalog/datetime.rs`: DateList, DateString, FromDateString, DatePlus,
  DateDifference, DayOfWeek, LeapYearQ, DaysInMonth

32 gain `{Protected}`; the 2 schema heads gain `{HoldRest, Protected}`.
The digest 5 stay `#[cfg(feature = "hash")]`-gated (do not change the
gating).

## Change 1 — catalog specs (`crates/rum-stdlib/src/catalog/*.rs`)

For the 32 non-schema heads: convert `BuiltinSpec::ordinary(name, summary,
handler)` → `BuiltinSpec::ordinary_with_attributes(name, summary,
AttributeSet::empty().with(Attribute::Protected), handler)`. Add
`Attribute, AttributeSet` to each file's `use
rum_runtime::kernel_api::{...}` import (mirror `catalog/arithmetic.rs:1`:
`use rum_runtime::kernel_api::{Attribute, AttributeSet, BuiltinSpec};`).

For the 2 schema heads (`catalog/schema.rs`, currently
`ordinary_with_hold(name, summary, HoldAttribute::HoldRest, handler)`):
convert to `ordinary_with_attributes(name, summary,
AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected),
handler)`. Keep `HoldAttribute` in the import and add `Attribute,
AttributeSet`.

No `eval_*` changes. No capability changes. The `.with_capability(...)` (if
any head carries one) is untouched.

## Change 2 — runtime mirror (`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`)

The mirror has two arrays that the drift test cross-checks against the
catalog: `STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` (`Spec::new("Name", ATTR)`,
alphabetical) and `STDLIB_EMPTY_ATTRIBUTE_NAMES` (bare names,
alphabetical). Today the 32 non-schema heads are bare names; the 2 schema
heads are `Spec::new(..., ATTR_HOLD_REST)`.

- Add a new const beside the others:
  `const ATTR_HOLD_REST_PROTECTED: AttributeSet =
  AttributeSet::from_hold(HoldAttribute::HoldRest).with(Attribute::Protected);`
- MOVE all 32 non-schema names out of `STDLIB_EMPTY_ATTRIBUTE_NAMES` and
  into `STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` as `Spec::new("Name",
  ATTR_PROTECTED)` (the existing `ATTR_PROTECTED` const), keeping BOTH
  arrays alphabetically sorted. (The digest names Sha256/Sha512/Sha1/Md5/
  Crc32 are currently unconditional bare names — keep them unconditional
  `Spec::new` entries; the drift test only checks catalog→kernel and the
  catalog side is hash-gated, so an unconditional mirror entry is correct
  and matches today's pattern.)
- Change the two schema entries `Spec::new("ConformSchema",
  ATTR_HOLD_REST)` / `Spec::new("SchemaErrors", ATTR_HOLD_REST)` →
  `ATTR_HOLD_REST_PROTECTED`.

The drift test (`crates/rum-stdlib/src/tests.rs`, "kernel attribute
vocabulary drift for {name}") must stay green — it asserts the catalog and
mirror agree name-by-name.

## Change 3 — regenerate the reference

The 34 `Attributes:` lines in `docs/reference/builtins.md` change from
`None` (or `HoldRest`) to `Protected` (or `HoldRest, Protected`).
Regenerate; do NOT hand-edit:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

The only `builtins.md` diff should be the 34 Attributes lines.

## Change 4 — attribute-assertion test

Add a small test (co-locate with existing codec/bytes/datetime tests under
`crates/rum-stdlib/src/tests/**`; `#[cfg(feature = "hash")]` if it
references a digest head): assert `Attributes[FromJson] == {Protected}`,
`Attributes[XmlElement] == {Protected}`, `Attributes[Sha256] ==
{Protected}`, `Attributes[DateList] == {Protected}`, and
`Attributes[ConformSchema] == {HoldRest, Protected}` (confirm the schema
hold is preserved, not replaced).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Attributes[FromJson], Attributes[ToJson], Attributes[FromCsv], Attributes[XmlElement], Attributes[Sha256], Attributes[Crc32], Attributes[ByteArrayQ], Attributes[DateList], Attributes[DayOfWeek], Attributes[ConformSchema], Attributes[SchemaErrors]}'
printf '' | cargo run -q -p rum-cli -- --eval '{FromJson["{\"a\":1}"], ToJson[<|"a"->1|>], DateList[0], Sha256["abc"], ConformSchema[<|"a"->1|>, <|"a"->_Integer|>]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — drift test passes,
reference regenerated, attribute test added). First probe: all `{Protected}`
except `ConformSchema`/`SchemaErrors` → `{HoldRest, Protected}`. Second
probe: behavior byte-identical to before (parsing/serialization/date/digest/
schema all unchanged — Protected does not alter evaluation).

## Scope

- MAY modify: the nine `crates/rum-stdlib/src/catalog/{json,csv,ini,toml,
  xml,yaml,schema,bytes,datetime}.rs` files, `crates/rum-runtime/src/
  runtime/builtin_catalog/stdlib_attributes.rs`, `docs/reference/
  builtins.md` (regenerated — Attributes lines only), and one attribute
  test file under `crates/rum-stdlib/src/tests/**`.
- MUST NOT touch: any `eval_*` handler; any builtin outside the 34 listed;
  do NOT add any attribute other than `Protected` (and the preserved
  `HoldRest`); do NOT change the digest hash-gating; `TODO.md`, the
  tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- `Protected` ONLY (plus preserved `HoldRest` on the two schema heads). NO
  `ReadProtected`/`Listable`/`Orderless`/`Flat`/anything else.
- Both tables move in lockstep — the drift test is the enforcement; if it
  reports drift, you missed a name in one array. Keep both arrays
  alphabetically sorted.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- No A/B gate: `Protected` is not consulted in `eval_call_prepare`. If
  `cargo test` surfaces any evaluation/ordering change, STOP and report —
  that would mean an unexpected interaction.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the 9 catalog files (32 + 2 heads), the mirror (new
   const + 32 moved + 2 changed), the reference regen, the test; line
   ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
