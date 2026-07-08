# Brief 207 — WhitespaceCharacters constant + RandomPassword option discovery (unit-7 fix 4)

Implementation brief. Executes the unit-7 triage batch 4 (tracker
`docs/dev/usability-pass.md`, "Unit 7 triage"; findings gaps #4 and #6 in
`docs/dev/usability-pass/character-classes-random-findings.md`). Two small,
independent convenience additions — both compat, both byte-identical for
existing calls. No behavior change to any draw or password evaluation; NO
microbench A/B.

Part A adds one protected ASCII character-class constant. Part B makes the
already-implemented `RandomPassword` policy option NAMES discoverable via
`Options[...]`. Read brief-196 (Protected character-class precedent) and
brief-197 (option-discovery registration precedent) first.

---

## Part A — `WhitespaceCharacters` constant

A sixth ASCII character-class constant, siblings with the existing five
(`Alphanumeric`, `DigitCharacters`, `LowercaseLetters`,
`PunctuationCharacters`, `UppercaseLetters`). Value = the ASCII whitespace
set that the `WhitespaceCharacter` string-pattern atom matches: verified in
`crates/rum-stdlib/src/strings.rs:279`, `is_string_whitespace` is
`matches!(ch, ' ' | '\t' | '\n' | '\r')` — exactly SPACE, TAB, LINE FEED,
CARRIAGE RETURN. So the constant is the 4-character string `" \t\n\r"`.
ASCII-only, folk scope — do NOT pull in Unicode whitespace (the family is
ASCII by design; the future unicode line owns non-ASCII).

### A1 — `crates/rum-stdlib/src/character_classes.rs`

- Add the const (mirror the existing five at lines 4-9):
  `pub(crate) const WHITESPACE_CHARACTERS: &str = " \t\n\r";`
- Add it to `CHARACTER_CLASS_CONSTANTS` (line 11) as
  `("WhitespaceCharacters", WHITESPACE_CHARACTERS)` — this drives the
  own-value install (the `install` loop at line 20).

### A2 — `crates/rum-stdlib/src/catalog/character_classes.rs`

Add a sixth `BuiltinSpec::with_attributes` entry to
`CHARACTER_CLASS_BUILTINS`, same shape as the others, `ATTR_PROTECTED`:

```rust
BuiltinSpec::with_attributes(
    "WhitespaceCharacters",
    "Evaluates to the ASCII whitespace character-class string.",
    ATTR_PROTECTED,
),
```

Catalog order is not alphabetical here (it is source order); place it last
after `UppercaseLetters` is fine.

### A3 — runtime mirror (`.../builtin_catalog/stdlib_attributes.rs`)

The five character-class heads are `Spec::new("Name", ATTR_PROTECTED)` in
`STDLIB_NON_EMPTY_ATTRIBUTE_SPECS` (e.g. line 98 `Alphanumeric`). Add
`Spec::new("WhitespaceCharacters", ATTR_PROTECTED)` in the correct
ALPHABETICAL position (after `UppercaseLetters`). The drift test
(`crates/rum-stdlib/src/tests.rs`, "kernel attribute vocabulary drift")
enforces catalog↔mirror agreement — keep the array sorted.

### A4 — regenerate reference

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

Adds a `### WhitespaceCharacters` entry (Attributes `Protected`, Capability
`Pure`) in alphabetical position. Do NOT hand-edit.

---

## Part B — `RandomPassword` option discovery

`RandomPassword` already accepts and enforces four string-keyed policy
rules (`"Require"`, `"NoAdjacentRepeats"`, `"Exclude"`, `"IgnoreCase"` —
`crates/rum-stdlib/src/random/password_policy.rs`), but
`Options[RandomPassword]` returns `{}`, so the option names are not
machine-discoverable. Register their defaults for discovery, exactly the
brief-197 pattern (the FromCsv/ConformSchema precedent in
`crates/rum-runtime/src/runtime/options/defaults.rs`).

### B1 — `crates/rum-runtime/src/runtime/options/defaults.rs`

Add a match arm in `default_system_options` beside the existing ones
(e.g. near `ConformSchema`):

```rust
"RandomPassword" => vec![
    system_rule_option("Require", Expr::call(system_expr("List"), Vec::new())),
    system_rule_option("NoAdjacentRepeats", bool_option(false)),
    system_rule_option("Exclude", Expr::call(system_expr("List"), Vec::new())),
    system_rule_option("IgnoreCase", bool_option(false)),
],
```

Defaults chosen to mean "no policy": `Require -> {}`, `NoAdjacentRepeats ->
False`, `Exclude -> {}`, `IgnoreCase -> False`.

**This is discovery only.** The defaults table feeds `Options` /
`OptionValue` exclusively — implementations never consult it at call time
(verified pattern: `password_policy.rs` reads the string-keyed rules
directly off the call args). So `RandomPassword[...]` behavior is
byte-identical; only `Options[RandomPassword]` changes from `{}` to the
four rules.

**Known wart (do NOT try to fix here):** like FromCsv/ConformSchema, the
registered keys render as SYMBOLS (`Require`) while the accepted call
syntax uses STRINGS (`"Require"`). This is the established precedent
(brief-197). A uniform string-keyed-discovery mechanism is a separate
cross-cutting design item (recorded in the tracker close-out); stay
consistent with the precedent here — do NOT invent a one-off mechanism for
RandomPassword and do NOT change what `RandomPassword` accepts.

No reference change for Part B (option registration is not in the
generated reference).

---

## Tests

Co-locate with the existing character-class / random coverage (the test
that references the constants is
`random_text_composes_with_character_class_constants` in
`crates/rum-stdlib/src/tests/random.rs`; put both new assertions in that
file or its module):

- Part A: `WhitespaceCharacters === " \t\n\r"` (assert equality with the
  4-char string), `StringLength[WhitespaceCharacters] == 4`,
  `Attributes[WhitespaceCharacters] == {Protected}`, and that it composes
  as an alphabet (`StringLength[RandomString[WhitespaceCharacters, 5]] ==
  5`).
- Part B: `Options[RandomPassword]` equals the four registered rules
  (order as registered), AND a policy call is byte-identical — e.g.
  `RandomPassword["ab", 4, "NoAdjacentRepeats" -> True]` under a fixed
  entropy fixture still succeeds / fails-closed exactly as before (mirror
  an existing password test in the file using
  `core_session_with_entropy`).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{WhitespaceCharacters === " \t\n\r", StringLength[WhitespaceCharacters], Attributes[WhitespaceCharacters], StringLength[RandomString[WhitespaceCharacters, 6]]}'
printf '' | cargo run -q -p rum-cli -- --eval 'Options[RandomPassword]'
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[1]; {RandomPassword["a", 2, "NoAdjacentRepeats" -> True], RandomPassword["abc", 4, "Unknown" -> True], StringLength[RandomPassword[Alphanumeric, 12]]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests + drift
test + reference regen). First probe: `{True, 4, {Protected}, 6}`. Second
probe: the four rules `{Require -> {}, NoAdjacentRepeats -> False, Exclude
-> {}, IgnoreCase -> False}` (symbol keys, per precedent). Third probe:
`RandomPassword` behavior unchanged — the `NoAdjacentRepeats` policy still
fails closed on a single-character alphabet, the unknown option still
messages + `$Failed`, and a plain 12-char draw still returns a length-12
string.

## Scope

- MAY modify: `crates/rum-stdlib/src/character_classes.rs`,
  `crates/rum-stdlib/src/catalog/character_classes.rs`,
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`,
  `crates/rum-runtime/src/runtime/options/defaults.rs`,
  `docs/reference/builtins.md` (regenerated — the new WhitespaceCharacters
  entry only), and `crates/rum-stdlib/src/tests/random.rs` (the tests).
- MUST NOT touch: the five existing character-class constants/specs; any
  `eval_*` handler; `password_policy.rs` (its parsing is unchanged); any
  other option arm; the random-family specs from brief-204; any capability;
  `TODO.md`, the tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- Part A: ASCII-only `" \t\n\r"` (exactly `is_string_whitespace`'s set) —
  no Unicode whitespace. `Protected` like its five siblings; the mirror +
  drift test must stay green (keep the array sorted).
- Part B: discovery-only — `RandomPassword[...]` behavior byte-identical.
  Do NOT change accepted call syntax; do NOT touch `password_policy.rs`.
  Follow the brief-197 symbol-keyed precedent; do NOT invent a string-keyed
  registration mechanism.
- Regenerate `builtins.md` with the generator bin — do NOT hand-edit.
- No A/B gate. If `cargo test` surfaces any behavior change to an existing
  call, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — Part A (const + array + catalog spec + mirror + reference
   + test), Part B (options arm + test); line ranges.
2. Verification — all commands verbatim (test totals + three probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
