# Brief 204 — Protected sweep on the 11 random-family heads (unit-7 fix 1)

Implementation brief. Executes the unit-7 triage batch 1 (tracker
`docs/dev/usability-pass.md`, "Unit 7 triage"; evidence E1/E7 in
`docs/dev/usability-pass/character-classes-random-findings.md`). Read
brief-196 FIRST — it is the exact worked precedent for this ripple
(catalog attribute change → runtime mirror sync → reference regen →
attribute test). This is attribute hygiene ONLY; no `eval_*` handler
changes, no capability changes, no behavior change beyond
write-protection.

## What and why

All 11 `random` heads have EMPTY attributes today except `BlockRandom`
(which is `{HoldAll}`) — verified: `Attributes /@ {SeedRandom,
RandomInteger, RandomChoice, RandomSample, RandomString, BlockRandom,
RandomUuid, UuidQ, RandomBytes, ReseedRandom, RandomPassword}` prints ten
`{}` and one `{HoldAll}`. So every random verb head is silently
redefinable. The 5 character-class heads are ALREADY `{Protected}`
(`catalog/character_classes.rs`) — do NOT touch them. WL protects these
built-in heads. Add `Protected` to all 11, PRESERVING `HoldAll` on
`BlockRandom` and PRESERVING `Capability::Random` on the three entropy
heads (`RandomBytes`, `ReseedRandom`, `RandomPassword`).

`Protected` is NOT an attribute-driven-planning attribute: unlike
`Flat`/`Orderless`, it is not consulted in `eval_call_prepare` argument
normalization, so there is NO per-call eval cost and NO argument-ordering
change. NO microbench A/B is needed (same disposition as brief-196/193's
`Protected`). Do NOT add `ReadProtected`, `Listable`, or any other
attribute — `Protected` (plus the preserved `HoldAll` on `BlockRandom`)
ONLY.

## The 11 heads (all in `catalog/random.rs`, catalog order)

| Head | Today | After |
|---|---|---|
| SeedRandom | `ordinary` | `{Protected}` |
| RandomInteger | `ordinary` | `{Protected}` |
| RandomChoice | `ordinary` | `{Protected}` |
| RandomSample | `ordinary` | `{Protected}` |
| RandomString | `ordinary` | `{Protected}` |
| BlockRandom | `ordinary_with_hold(HoldAll)` | `{HoldAll, Protected}` |
| RandomUuid | `ordinary` | `{Protected}` |
| UuidQ | `ordinary` | `{Protected}` |
| RandomBytes | `ordinary` + `Capability::Random` | `{Protected}` + `Capability::Random` |
| ReseedRandom | `ordinary` + `Capability::Random` | `{Protected}` + `Capability::Random` |
| RandomPassword | `ordinary` + `Capability::Random` | `{Protected}` + `Capability::Random` |

## Change 1 — catalog specs (`crates/rum-stdlib/src/catalog/random.rs`)

- Imports: change `use rum_runtime::kernel_api::{BuiltinSpec, Capability,
  HoldAttribute};` to add `Attribute, AttributeSet` (mirror
  `catalog/character_classes.rs:1`). Add a file-local const beside it
  (mirror `catalog/character_classes.rs:3`):
  `const ATTR_PROTECTED: AttributeSet =
  AttributeSet::from_attribute(Attribute::Protected);`
- The 7 plain heads (`SeedRandom`, `RandomInteger`, `RandomChoice`,
  `RandomSample`, `RandomString`, `RandomUuid`, `UuidQ`): convert
  `BuiltinSpec::ordinary(name, summary, handler)` →
  `BuiltinSpec::ordinary_with_attributes(name, summary, ATTR_PROTECTED,
  handler)`.
- `BlockRandom` (currently `ordinary_with_hold(name, summary,
  HoldAttribute::HoldAll, handler)`): convert to
  `ordinary_with_attributes(name, summary,
  AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected),
  handler)`. `HoldAll` MUST be preserved (it holds the block body).
- The 3 entropy heads (`RandomBytes`, `ReseedRandom`, `RandomPassword`,
  currently `ordinary(name, summary, handler).with_capability(Capability::
  Random)`): convert the `ordinary(...)` to `ordinary_with_attributes(name,
  summary, ATTR_PROTECTED, handler)` and KEEP the trailing
  `.with_capability(Capability::Random)` exactly as-is.

No `eval_*` changes. No capability changes.

## Change 2 — runtime mirror (`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`)

The mirror has two arrays the drift test cross-checks against the catalog:
`STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` (`Spec::new("Name", ATTR)`,
alphabetical) and `STDLIB_EMPTY_ATTRIBUTE_NAMES` (bare names,
alphabetical). Today `BlockRandom` is `Spec::new("BlockRandom",
ATTR_HOLD_ALL)` (line ~105); the other 10 random names live in the empty
array (`RandomBytes`, `RandomChoice`, `RandomInteger`, `RandomPassword`,
`RandomSample`, `RandomString`, `RandomUuid`, `ReseedRandom`,
`SeedRandom`, `UuidQ`).

- Add a new const beside the others (there is `ATTR_HOLD_ALL` at line ~17
  but NO plain `ATTR_HOLD_ALL_PROTECTED` — there is only the
  `..._READ_PROTECTED` variant, which is WRONG here):
  `const ATTR_HOLD_ALL_PROTECTED: AttributeSet =
  AttributeSet::from_hold(HoldAttribute::HoldAll).with(Attribute::Protected);`
  Keep the const list alphabetically ordered as the file already is.
- Change `Spec::new("BlockRandom", ATTR_HOLD_ALL)` →
  `Spec::new("BlockRandom", ATTR_HOLD_ALL_PROTECTED)`.
- MOVE the 10 other random names OUT of `STDLIB_EMPTY_ATTRIBUTE_NAMES`
  and INTO `STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` as `Spec::new("Name",
  ATTR_PROTECTED)` (the existing `ATTR_PROTECTED` const at line ~85),
  keeping BOTH arrays alphabetically sorted.

The drift test (`crates/rum-stdlib/src/tests.rs`, "kernel attribute
vocabulary drift for {name}") must stay green — it asserts the catalog and
mirror agree name-by-name. If it reports drift, you missed a name in one
array.

## Change 3 — regenerate the reference

The 11 `Attributes:` lines in `docs/reference/builtins.md` change: ten
from `None` → `Protected`, `BlockRandom` from `HoldAll` → `HoldAll,
Protected`. Regenerate; do NOT hand-edit:

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

The only `builtins.md` diff should be those 11 Attributes lines.

## Change 4 — attribute-assertion test

Add a small test in `crates/rum-stdlib/src/tests/random.rs` (co-located
with the existing random tests): assert `Attributes[RandomInteger] ==
{Protected}`, `Attributes[RandomPassword] == {Protected}`, and
`Attributes[BlockRandom] == {HoldAll, Protected}` (confirm the
`BlockRandom` hold is PRESERVED, not replaced — hold renders first, per
brief-196's `{HoldRest, Protected}` precedent; verify the exact rendered
order with a probe and match it).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval 'Attributes /@ {SeedRandom, RandomInteger, RandomChoice, RandomSample, RandomString, BlockRandom, RandomUuid, UuidQ, RandomBytes, ReseedRandom, RandomPassword}'
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[42]; a = {RandomInteger[100], RandomChoice[{x,y,z}], RandomSample[Range[4]], RandomString[4], BlockRandom[RandomInteger[9]], UuidQ[RandomUuid[]]}; SeedRandom[42]; b = {RandomInteger[100], RandomChoice[{x,y,z}], RandomSample[Range[4]], RandomString[4], BlockRandom[RandomInteger[9]], UuidQ[RandomUuid[]]}; {a, SameQ[a, b]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — drift test passes,
reference regenerated, attribute test added). First probe: ten
`{Protected}` and `BlockRandom` → `{HoldAll, Protected}`. Second probe:
behavior byte-identical to before — same seed still gives the same
reproducible stream, `BlockRandom` still restores state, `UuidQ` still
validates (`Protected` does not alter evaluation).

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/random.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`,
  `docs/reference/builtins.md` (regenerated — the 11 Attributes lines
  only), and `crates/rum-stdlib/src/tests/random.rs` (the attribute test).
- MUST NOT touch: any `eval_*` handler in `random.rs`/`password_policy.rs`;
  the 5 character-class heads (already `{Protected}`); the `WhitespaceCharacters`
  constant work (that is a later brief); any capability
  (`Capability::Random` on the three entropy heads is PRESERVED, not
  changed); `catalog_capabilities.rs` (its Pure/Random split must stay
  green untouched); `TODO.md`, the tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- `Protected` ONLY (plus the preserved `HoldAll` on `BlockRandom`). NO
  `ReadProtected`/`Listable`/`Orderless`/`Flat`/anything else. Do NOT
  touch capabilities.
- Both mirror arrays move in lockstep — the drift test is the enforcement.
  Keep both arrays alphabetically sorted.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit it.
- No A/B gate: `Protected` is not consulted in `eval_call_prepare`. If
  `cargo test` surfaces any evaluation/ordering change, or a workspace
  test goes red because it reassigns a now-`Protected` random head, STOP
  and report — do not silently work around a Protected error.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — `catalog/random.rs` (7 + 1 + 3 heads), the mirror (new
   const + 10 moved + 1 changed), the reference regen, the test; line
   ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
