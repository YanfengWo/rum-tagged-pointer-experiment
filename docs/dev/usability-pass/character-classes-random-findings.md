# Character Classes + Random Usability Audit Findings

Coverage statement: findings-table rows cover 16 distinct catalog names: 5 from
`character_classes.rs` and 11 from `random.rs`. The tracker still says 15, but
that predates the brief-201 split that kept `SeedRandom` deterministic and added
`ReseedRandom`; the authoritative count for this unit is 16. The coverage check
printed `COVERAGE-OK 16/16`.

The high-value safety boundaries held. The RandomPassword arc from briefs
157-159 still holds: policy rules are honored, unsatisfiable or malformed
policies fail closed with `$Failed` plus a message, and entropy-backed password
draws are not affected by `SeedRandom` or `BlockRandom`. The briefs 200-202
stateful-vs-pure separation also held: deterministic PRNG heads are `Pure`,
`RandomBytes`/`ReseedRandom`/`RandomPassword` are `Capability::Random`, and
`sandboxed()` blocks only the entropy heads. No fail-closed or capability-label
hole was found.

## Top Gaps By Script-User Value

1. **Safety:** all 11 random-family heads lack `Protected`; `BlockRandom` must
   preserve its existing `HoldAll` when fixed.
2. `RandomInteger[spec, {dims}]`, `RandomChoice[list, {dims}]`, and
   `RandomString[spec, {dims}]` are missing the cheap nested-output forms.
3. `RandomChoice[weights -> elements, n]` is missing weighted choice.
4. `RandomPassword` policy keys are documented and implemented, but
   `Options[RandomPassword]` is empty.
5. `RandomString[]` is rejected even though `RandomInteger[]` already has its
   WL-documented zero-argument convenience form.
6. The ASCII character-class constants are useful and protected, but there is no
   `WhitespaceCharacters` sibling despite the adjacent `WhitespaceCharacter`
   string-pattern atom.

Operator-form disambiguation: no random operator-form gaps are filed. The
candidate unary calls are already data forms or effect forms in this family:
`RandomInteger[n]`, `RandomChoice[list]`, `RandomSample[list]`,
`RandomString[n]`, `UuidQ[str]`, `RandomBytes[n]`, `SeedRandom[n]`,
`BlockRandom[body]`, and the zero-argument `RandomUuid[]`/`ReseedRandom[]`
forms. The gaps below are output-shape and weighted-choice forms, not curried
operators.

## Findings Table

### Character Classes

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Alphanumeric | rum | gap | convenience | The protected ASCII constants compose cleanly, and `Alphanumeric` is the 62-character default random alphabet. The adjacent string-pattern surface has `WhitespaceCharacter`, but this constant family has no protected `WhitespaceCharacters` string, so whitespace fuzz/test alphabets still need magic literals (Evidence E2; `crates/rum-stdlib/src/catalog/strings.rs:84`; `crates/rum-stdlib/src/catalog/character_classes.rs:5`). | Add a protected `WhitespaceCharacters` ASCII string constant with a documented set; do not broaden this unit into Unicode alphabet machinery. | S |
| DigitCharacters | rum | OK | boundary | ASCII digits `0-9` are protected, complete for the documented folk scope, and compose with `RandomString`; non-ASCII decimal digits are an intended future Unicode boundary (Evidence E1, E2; `crates/rum-stdlib/src/character_classes.rs:6`). | No change. | |
| LowercaseLetters | rum | OK | boundary | ASCII `a-z` is protected and complete for this unit; language-specific alphabets remain in the future Unicode line (Evidence E1, E2; `crates/rum-stdlib/src/character_classes.rs:5`). | No change. | |
| PunctuationCharacters | rum | OK | boundary | The 32-character ASCII punctuation set is protected and matches `PunctuationCharacter`; broader Unicode punctuation is intentionally out of scope (Evidence E1, E2; `crates/rum-stdlib/src/character_classes.rs:7`). | No change. | |
| UppercaseLetters | rum | OK | boundary | ASCII `A-Z` is protected and complete for this unit; language-specific uppercase alphabets remain out of scope here (Evidence E1, E2; `crates/rum-stdlib/src/character_classes.rs:4`). | No change. | |

### Random

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| SeedRandom | WL-inherited name, rum split | gap | safety | The settled split is clean: `SeedRandom[seed]` is deterministic-only, `Pure`, and sandbox-allowed; `SeedRandom[]` is not an entropy form. The head still lacks `Protected` (Evidence E1, E3; `crates/rum-stdlib/src/catalog/random.rs:6`; `crates/rum-host/src/tests.rs:441`). | Add `Protected` while keeping `Capability::Pure`; do not add a zero-argument entropy form. | S |
| RandomInteger | WL | gap | safety | Deterministic-by-default is correctly documented and labeled `Pure`, and malformed bounds stay inert rather than yielding a fabricated value. The head lacks `Protected` (Evidence E1, E3, E4; `docs/guide/design-boundaries.md:124`; `crates/rum-stdlib/src/catalog/random.rs:11`). | Add `Protected`; keep deterministic session-PRNG behavior and the no-`RandomReal` exact-number boundary unchanged. | S |
| RandomInteger | WL | gap | flexibility | `RandomInteger[spec, n]` works, but `RandomInteger[spec, {n1, n2, ...}]` stays inert because output counts are parsed only as scalar exact integers (Evidence E4; `crates/rum-stdlib/src/random.rs:60`; `crates/rum-stdlib/src/random.rs:396`). | Add nested `{dims}` output generation with the existing element-limit and memory-limit checks. | M |
| RandomChoice | WL | gap | safety | Empty lists fail closed with `RandomChoice::empty`, deterministic choice is `Pure`, and sandbox policy allows it as intended. The head lacks `Protected` (Evidence E1, E3, E4; `crates/rum-stdlib/src/random.rs:76`; `crates/rum-host/src/tests.rs:443`). | Add `Protected`; keep deterministic PRNG choice unchanged. | S |
| RandomChoice | WL | gap | flexibility | Scalar count output works, but `RandomChoice[list, {dims}]` and weighted `RandomChoice[weights -> elements, n]` stay inert (Evidence E4; `crates/rum-stdlib/src/random.rs:81`; `crates/rum-stdlib/src/random.rs:96`). | Add `{dims}` nested output and a WL-compatible weighted-choice form with exact non-negative weights and fail-closed malformed weight specs. | M |
| RandomSample | WL | gap | safety | Full-list and counted sampling work, and count greater than length fails closed with `RandomSample::ksamp`. The head lacks `Protected` (Evidence E1, E3; `crates/rum-stdlib/src/random.rs:107`). | Add `Protected`; keep without-replacement sampling and current `ksamp` diagnostic unchanged. | S |
| RandomString | WL | gap | safety | Deterministic default-alphabet draws are `Pure`, duplicate alphabet code points intentionally weight draws, and empty alphabets or bad counts stay inert. The head lacks `Protected` (Evidence E1, E2, E4; `crates/rum-stdlib/src/random.rs:148`; `crates/rum-stdlib/src/tests/random.rs:224`). | Add `Protected`; keep deterministic PRNG string generation and code-point alphabet semantics unchanged. | S |
| RandomString | WL | gap | flexibility | `RandomString[n]` and `RandomString[alphabet, n]` work, but `RandomString[]` emits an arity message and `RandomString[alphabet, {dims}]` stays inert (Evidence E4; `crates/rum-stdlib/src/random.rs:153`; `crates/rum-stdlib/src/random.rs:358`). | Add the documented zero-argument convenience form and nested `{dims}` output using the existing default alphabet and limits. | M |
| BlockRandom | WL | gap | safety | `BlockRandom` correctly restores the deterministic PRNG stream and does not affect entropy-backed password draws. It has `HoldAll` but lacks `Protected` (Evidence E1, E3, E5; `crates/rum-stdlib/src/catalog/random.rs:31`; `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs:105`). | Add `Protected` while preserving `HoldAll`; no call-planning behavior change. | S |
| RandomUuid | rum | gap | safety | UUID strings are deterministic under the session PRNG and `UuidQ` validates the version/variant format. That reproducibility is intended; entropy-by-default would violate the contract. The head lacks `Protected` (Evidence E1, E3, E5; `docs/guide/design-boundaries.md:124`; `crates/rum-stdlib/src/random.rs:214`). | Add `Protected`; keep reproducible PRNG UUIDs and the reference warning about entropy reseeding for global uniqueness. | S |
| UuidQ | rum | gap | safety | UUID format validation returns `True` only for version-4/variant-valid strings and remains `Pure` in sandboxed hosts. The predicate head lacks `Protected` (Evidence E1, E5; `crates/rum-stdlib/src/random.rs:230`). | Add `Protected`; behavior unchanged. | S |
| RandomBytes | rum | gap | safety | `RandomBytes` is correctly `Capability::Random`, denied by `sandboxed()`, and no-entropy paths return `$Failed` plus `General::noentropy`. The head lacks `Protected` (Evidence E1, E5; `crates/rum-host/src/tests.rs:438`; `crates/rum-host/src/tests.rs:456`; `crates/rum-stdlib/src/random.rs:242`). | Add `Protected`; keep host-entropy-only behavior and capability gating unchanged. | S |
| ReseedRandom | rum | gap | safety | The rum-native split is correct: `ReseedRandom[]` is the only entropy reseed, it is `Capability::Random`, sandbox-denied, and no-entropy paths fail closed. The head lacks `Protected` (Evidence E1; `crates/rum-stdlib/src/catalog/random.rs:53`; `crates/rum-host/src/tests.rs:438`; `crates/rum-host/src/tests.rs:456`). | Add `Protected`; do not merge it back into `SeedRandom`. | S |
| RandomPassword | rum | gap | safety | The fresh policy surface held: `Require`, `NoAdjacentRepeats`, `Exclude`, and `IgnoreCase` are enforced; impossible or malformed policies return `$Failed` plus `RandomPassword::password`; entropy errors share the fail-closed entropy path. The head lacks `Protected` (Evidence E1, E5; `crates/rum-stdlib/src/random/password_policy.rs:58`; `crates/rum-stdlib/src/random/password_policy.rs:129`; `crates/rum-stdlib/src/random.rs:167`). | Add `Protected`; keep entropy-backed rejection sampling, the 10000-attempt cap, and `Capability::Random`. | S |
| RandomPassword | rum | gap | convenience | The implemented policy keys are discoverable only in prose/reference text: `Options[RandomPassword]` returns `{}` (Evidence E1; `crates/rum-runtime/src/runtime/options/defaults.rs:20`; `docs/reference/builtins.md:2910`). | Expose the policy defaults through the options/discovery surface while preserving the current string-keyed call syntax. If string-keyed rules stay outside `SymbolOptions`, add a small discovery-only path rather than changing accepted calls. | M |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin
to `cargo run`, except where the evidence is a direct file:line citation from
the catalog, implementation, reference, or existing host tests.

### E1: Attribute And Option Survey

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Attributes /@ {"Alphanumeric", "DigitCharacters", "LowercaseLetters", "PunctuationCharacters", "UppercaseLetters", "SeedRandom", "RandomInteger", "RandomChoice", "RandomSample", "RandomString", "BlockRandom", "RandomUuid", "UuidQ", "RandomBytes", "ReseedRandom", "RandomPassword"}, Options /@ {RandomPassword, RandomString, RandomChoice, RandomSample, RandomInteger}}'\'''
List[List[List[Protected], List[Protected], List[Protected], List[Protected], List[Protected], List[], List[], List[], List[], List[], List[HoldAll], List[], List[], List[], List[], List[]], List[List[], List[], List[], List[], List[]]]
```

### E2: Character-Class Values And Composition

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{StringLength /@ {UppercaseLetters, LowercaseLetters, DigitCharacters, PunctuationCharacters, Alphanumeric}, Alphanumeric === UppercaseLetters <> LowercaseLetters <> DigitCharacters, StringContainsQ[PunctuationCharacters, "!"], StringContainsQ[PunctuationCharacters, "~"], StringContainsQ[PunctuationCharacters, " "], StringJoin[StringCases[PunctuationCharacters, PunctuationCharacter]] === PunctuationCharacters, StringCases["a \t!", {LetterCharacter, WhitespaceCharacter, PunctuationCharacter}], RandomString[DigitCharacters, 6]}'\'''
List[List[26, 26, 10, 32, 62], True, True, True, False, True, List["a", " ", "\t", "!"], "409814"]
```

### E3: Deterministic PRNG And BlockRandom

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''SeedRandom[42]; seq1 = {RandomInteger[100], RandomChoice[{a,b,c}], RandomSample[Range[5]], RandomString[4], RandomUuid[]}; SeedRandom[42]; seq2 = {RandomInteger[100], RandomChoice[{a,b,c}], RandomSample[Range[5]], RandomString[4], RandomUuid[]}; SeedRandom[123]; block = {RandomInteger[1000], BlockRandom[RandomInteger[1000]], RandomInteger[1000]}; SeedRandom[123]; plain = {RandomInteger[1000], RandomInteger[1000]}; {seq1, seq2, SameQ[seq1, seq2], block, plain, UuidQ[seq1[[5]]]}'\'''
List[List[39, c, List[2, 1, 5, 4, 3], "YcYp", "86b04c36-ea93-4d05-86e8-5fdb001a57e4"], List[39, c, List[2, 1, 5, 4, 3], "YcYp", "86b04c36-ea93-4d05-86e8-5fdb001a57e4"], True, List[997, 474, 474], List[997, 474], True]
```

### E4: Malformed Inputs And Missing Cheap Forms

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{RandomChoice[{}], RandomSample[{a}, 2], RandomInteger[-1], RandomInteger[{2, 1}], RandomInteger[3/2], RandomString[-1], RandomString["", 3], SeedRandom[], SeedRandom[3/2], ReseedRandom[5], RandomBytes[-1]}'\'''
List[RandomChoice[List[]], RandomSample[List[a], 2], RandomInteger[-1], RandomInteger[List[2, 1]], RandomInteger[3/2], RandomString[-1], RandomString["", 3], SeedRandom[], SeedRandom[3/2], ReseedRandom[5], RandomBytes[-1]]
! RandomChoice::empty: expected a non-empty list
! RandomSample::ksamp: sample count cannot exceed list length
! SeedRandom::argx: expected exactly 1 argument(s), got 0
! ReseedRandom::argx: expected exactly 0 argument(s), got 1
```

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''SeedRandom[9]; {RandomInteger[], RandomString[], RandomInteger[9, {2, 3}], RandomChoice[{a, b, c}, {2, 2}], RandomString["ab", {2, 3}], RandomChoice[{1, 9} -> {a, b}, 5], RandomChoice[{a, b, c}, 0], RandomString["ab", 0]}'\'''
List[1, RandomString[], RandomInteger[9, List[2, 3]], RandomChoice[List[a, b, c], List[2, 2]], RandomString["ab", List[2, 3]], RandomChoice[Rule[List[1, 9], List[a, b]], 5], List[], ""]
! RandomString::argtu: expected between 1 and 2 argument(s), got 0
! RandomString::argtu: expected between 1 and 2 argument(s), got 0
! RandomString::argtu: expected between 1 and 2 argument(s), got 0
```

### E5: RandomPassword Policy And Entropy-Backed Heads

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''SeedRandom[7]; first = RandomString[8]; password = RandomPassword[8]; after = RandomString[8]; SeedRandom[7]; {StringLength[RandomPassword[Alphanumeric <> PunctuationCharacters, 20, "Require" -> {UppercaseLetters, LowercaseLetters, DigitCharacters, PunctuationCharacters}]], StringLength[password], first, after, RandomString[8], RandomString[8], RandomPassword["a", 2, "NoAdjacentRepeats" -> True], RandomPassword["abc", 4, "Unknown" -> True], RandomPassword["abc", 4, "Exclude" -> {""}]}'\'''
List[20, 8, "XVoZyDqB", "t0CnBKji", "XVoZyDqB", "t0CnBKji", $Failed, $Failed, $Failed]
! RandomPassword::password: could not satisfy the password policy within 10000 attempts; loosen the constraints or increase length
! RandomPassword::password: unknown RandomPassword policy option "Unknown".
! RandomPassword::password: RandomPassword excluded substrings cannot be empty strings.
```

```text
$ scripts/run-bounded.py --timeout 120 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{ByteArrayQ[RandomBytes[4]], ByteArrayQ[RandomBytes[0]], UuidQ[RandomUuid[]], UuidQ["00000000-0000-4000-8000-000000000000"], UuidQ["00000000-0000-5000-8000-000000000000"], UuidQ["00000000-0000-4000-7000-000000000000"], UuidQ["not-a-uuid"]}'\'''
List[True, True, True, True, False, False, False]
```

### E6: Capability And Entropy Tests Already Cover The Sandbox Line

`crates/rum-stdlib/src/tests/catalog_capabilities.rs:18` asserts the intended
catalog split: `RandomInteger`, `RandomChoice`, `RandomSample`, `RandomString`,
`BlockRandom`, `RandomUuid`, `UuidQ`, and `SeedRandom` are `Pure`, while
`RandomBytes`, `RandomPassword`, and `ReseedRandom` are `Random`.

`crates/rum-host/src/tests.rs:432` checks `HostConfig::sandboxed()` denial for
`ReseedRandom[]`, `RandomBytes[8]`, and `RandomPassword[8]`, and checks that
`SeedRandom[42]` plus the deterministic PRNG heads still runs without sandbox
messages. `crates/rum-host/src/tests.rs:451` checks that `RandomBytes[8]`,
`RandomPassword[8]`, and `ReseedRandom[]` return `$Failed` plus
`General::noentropy` when no entropy provider is installed.

### E7: Code-Visible Gaps

`crates/rum-stdlib/src/catalog/character_classes.rs:5` registers the five
character-class constants with `ATTR_PROTECTED`. `crates/rum-stdlib/src/catalog/random.rs:5`
registers all 11 random heads through `ordinary`/`ordinary_with_hold`; only the
three entropy heads add `Capability::Random`, and no random head adds
`Protected`. The runtime mirror reflects that split:
`crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs:105` gives
`BlockRandom` only `HoldAll`, while `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs:448`
lists the other random names in the empty-attribute set.

`crates/rum-stdlib/src/random.rs:396` parses output counts only as scalar exact
integers, which explains the inert `{dims}` probes. `crates/rum-runtime/src/runtime/options/defaults.rs:20`
has no `RandomPassword` default-options entry.

## Triage Sketch

1. **Random Protected sweep** (safety, S): add `Protected` to all 11 random
   heads, preserving `HoldAll` on `BlockRandom` and syncing the runtime
   attribute mirror. This is the brief-196 class of change and does not require
   the microbench A/B gate.
2. **Nested random output shapes plus `RandomString[]`** (flexibility, M): add a
   shared `{dims}` parser/builder for `RandomInteger`, `RandomChoice`, and
   `RandomString`, with existing generated-element and memory limits. This is
   per-dispatch generation code, not call planning or attribute-driven planning;
   no A/B gate expected.
3. **Weighted `RandomChoice`** (flexibility/safety, M): add
   `RandomChoice[weights -> elements, n]` with exact non-negative weights,
   length checks, zero-total fail-closed messaging, and unbiased draw selection.
   No call-planning touch.
4. **RandomPassword option discovery** (convenience, S/M): expose the existing
   policy keys and defaults through `Options[RandomPassword]` or a small
   discovery-only path that preserves string-keyed rules. No runtime capability
   or call-planning change.
5. **`WhitespaceCharacters` sibling constant** (convenience, S): add one
   protected ASCII constant and generated reference entry. No A/B gate.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/character_classes.rs
5
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/random.rs
11
```

```text
$ python3 - <<'PY'
from pathlib import Path
import re
files = [Path('crates/rum-stdlib/src/catalog/character_classes.rs'), Path('crates/rum-stdlib/src/catalog/random.rs')]
expected = []
for path in files:
    text = path.read_text()
    for match in re.finditer(r'BuiltinSpec::[A-Za-z_]+\(\s*"([^"]+)"', text):
        expected.append(match.group(1))
body = Path('docs/dev/usability-pass/character-classes-random-findings.md').read_text()
missing = [name for name in expected if f'| {name} |' not in body]
if missing:
    print('MISSING ' + ', '.join(missing))
else:
    print(f'COVERAGE-OK {len(expected)}/{len(expected)}')
PY
COVERAGE-OK 16/16
```

```text
$ git status --short
?? docs/dev/dun-embedding-plugin-requirements.md
?? docs/dev/dun-general-editor-plugin-requirements.md
?? docs/dev/usability-pass/character-classes-random-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

None. Worktree note: `git status --short` includes two unrelated untracked
`docs/dev/dun-*` files outside this audit; the only file created for this audit
is `docs/dev/usability-pass/character-classes-random-findings.md`.
