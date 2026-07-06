# Brief 159 — Named character-class constants (Protected)

Small implementation brief. Ergonomic fix for `RandomPassword`/`RandomString`
policies (and useful anywhere a character set is needed): five named string
constants, so callers write `Require -> {UppercaseLetters, DigitCharacters}`
instead of spelling out `"ABCDEFGHIJKLMNOPQRSTUVWXYZ"`. Because these are
just strings, NOTHING in `RandomPassword`/`RandomString` changes — the
constants compose with the existing string-based option surface.

## The constants (bare symbols, no `$` prefix, each evaluates to its string)

| Symbol | Value |
|---|---|
| `UppercaseLetters` | `"ABCDEFGHIJKLMNOPQRSTUVWXYZ"` |
| `LowercaseLetters` | `"abcdefghijklmnopqrstuvwxyz"` |
| `DigitCharacters` | `"0123456789"` |
| `PunctuationCharacters` | the SAME ASCII punctuation set as the brief-137 `PunctuationCharacter` string-pattern atom (read that source; the two MUST match — reuse the constant/definition so they can never drift) |
| `Alphanumeric` | `UppercaseLetters <> LowercaseLetters <> DigitCharacters` = `"ABC…Zabc…z0123456789"` (this is the current default `RandomPassword`/`RandomString` alphabet, now nameable) |

## Contract

- Each is a bare symbol with an OWNVALUE that evaluates to the string
  above (no brackets: `UppercaseLetters` → the string).
- **`Protected`** — reassignment fails: `UppercaseLetters = "x"` emits the
  standard protection message (`Set::wrsym` / whatever the existing
  protected-symbol write path uses) and does NOT change the value. This is
  the security guarantee the whole design rests on; TEST it for all five.
- They install with core (`install_core`), appear in the generated
  reference (pick the registration mechanism that keeps the drift test
  meaningful — a constant/ownvalue catalog entry, or extend docgen; report
  what you chose, as brief-137 did for pattern atoms), and are `Pure`.

## Tests

- Value pins for all five (exact strings; `PunctuationCharacters` asserted
  EQUAL to the character set the brief-137 `PunctuationCharacter` matcher
  uses — cross-check, don't hardcode a second copy).
- `Alphanumeric === UppercaseLetters <> LowercaseLetters <> DigitCharacters`.
- Protection: `UppercaseLetters = "x"` (and the other four) → protection
  message + value unchanged; `Unprotect`-then-set is out of scope (do not
  test/enable a bypass).
- Composition (the whole point): 
  `RandomPassword[Alphanumeric <> PunctuationCharacters, 20, "Require" -> {UppercaseLetters, LowercaseLetters, DigitCharacters, PunctuationCharacters}]`
  produces a 20-char password containing at least one of each class (assert
  membership); `RandomString[DigitCharacters, 6]` yields only digits.
- Vocabulary/catalog registration + reference drift.

## Docs

Update the randomness section (`docs/guide/design-boundaries.md` or wherever
brief-140/157/158 put it) so the `RandomPassword` policy examples USE the
constants (the readable form above), and add a one-line list of the five
constants. Real transcripts.

## Scope

- MAY modify: the stdlib random/strings catalog + a small constants module,
  `stdlib_attributes.rs` vocabulary, docgen if needed, `docs/reference/`
  via generator, the randomness guide section; `crates/rum-runtime/**`
  ONLY if constant/ownvalue registration needs a minimal kernel hook
  (report it).
- MUST NOT touch: `RandomPassword`/`RandomString` logic (they already take
  strings), the matcher, evaluator/dispatch, `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # report the count (expect 605 if the five register as reference entries)
printf '' | cargo run -q -p rum-cli -- --eval '{UppercaseLetters, DigitCharacters, Alphanumeric}'
printf '' | cargo run -q -p rum-cli -- --eval 'UppercaseLetters = "x"'         # protected: message + unchanged
printf '' | cargo run -q -p rum-cli -- --eval 'UppercaseLetters'               # still the full alphabet
printf '' | cargo run -q -p rum-cli -- --eval 'PunctuationCharacters === StringJoin[StringCases["!\"#$%&'"'"'()*+,-./:;<=>?@[\\]^_`{|}~", PunctuationCharacter]]'
printf '' | cargo run -q -p rum-cli -- --eval 'p = RandomPassword[Alphanumeric <> PunctuationCharacters, 20, "Require" -> {UppercaseLetters, LowercaseLetters, DigitCharacters, PunctuationCharacters}]; StringLength[p]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- `PunctuationCharacters` must be the SAME set as brief-137's
  `PunctuationCharacter` — share the definition, never duplicate a literal
  that could drift.
- Protection is mandatory and tested for all five.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. The registration mechanism chosen + the reference-count change.
2. What changed — per file.
3. Verification — all commands verbatim (incl. the protection pin).
4. Stop-loss / open questions (empty if none).
