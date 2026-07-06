# Brief 162 — Strings usability fixes 1: attribute hygiene

Implementation brief. Unit-1 fix batch 1 of the Usability Pass
(`docs/dev/usability-pass.md`, triage item 1; evidence: rows E1/E16 in
`docs/dev/usability-pass/strings-findings.md`). All changes are
attribute-surface only — no evaluation semantics change; every result
that evaluates today evaluates to the same value.

## Changes

1. **`Protected` on the string-pattern symbols** (18, from the audit's
   E1 list): `Alternatives`, `Blank`, `BlankNullSequence`,
   `BlankSequence`, `DigitCharacter`, `EndOfLine`, `EndOfString`,
   `Except`, `LetterCharacter`, `Pattern`, `PunctuationCharacter`,
   `RegularExpression`, `StartOfLine`, `StartOfString`,
   `StringExpression`, `WhitespaceCharacter`, `WordBoundary`,
   `WordCharacter`. Mechanism: brief-159's pattern —
   `BuiltinSpec::with_attributes(..., ATTR_PROTECTED)` as in
   `crates/rum-stdlib/src/catalog/character_classes.rs`. WL protects
   all of these; using them in patterns (construction) is unaffected,
   only assignment/definition is blocked.
2. **Case-conversion attribute alignment**: `ToUpperCase` gains
   `Listable` + `Protected` (parity with `ToLowerCase`, which already
   has both); `Capitalize` and `Decapitalize` gain `Listable` on top of
   their existing attributes. E16 shows all four already map lists —
   results must stay byte-identical; only the advertised attributes and
   the threading mechanism change.
3. **Kernel attribute vocabulary**: update the rum-runtime vocabulary so
   the `catalog_attributes_match_kernel_vocabulary` cross-check passes
   (the test will point you at the file).
4. **Regenerate the builtin reference** (attributes appear in it):
   `cargo run -p rum-stdlib --bin generate-builtin-reference` so the
   drift test passes.
5. **Tests**: attribute assertions for every symbol touched; a
   Protected-rejection test (e.g. `DigitCharacter = 5` and
   `Blank[x_] := x` emit the protection message and leave the symbol's
   behavior intact — mirror brief-159's test shape); a threading test
   pinning `ToUpperCase[{"a","b"}]` etc. byte-identical to today.

## Watch out

- Protecting `Pattern`/`Blank`/`Alternatives`/`Except` touches heads
  used by EVERY expression pattern. Construction and matching are not
  affected by `Protected`, but run the FULL workspace suite — any
  in-tree code that defines rules ON these heads will surface there.
  If something in-tree legitimately assigns to one of them, STOP and
  report it instead of unprotecting ad hoc.
- If threading via `Listable` would change any existing result (e.g.
  nested lists, non-string elements now threading differently than the
  implementation's internal mapping), pin the CURRENT behavior with a
  test first and report the difference instead of silently changing it.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Attributes /@ {DigitCharacter, Pattern, StringExpression, WordBoundary}, Attributes /@ {ToUpperCase, ToLowerCase, Capitalize, Decapitalize}}'
printf '' | cargo run -q -p rum-cli -- --eval '{DigitCharacter = 5, ToUpperCase[{"a", "b"}], Capitalize[{"ab", "cd"}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals; new tests added);
first probe shows `Protected` on the pattern symbols and
`Listable`+`Protected` on all four case functions; second probe shows
the protection message with `ToUpperCase`/`Capitalize` list results
unchanged (`{"A", "B"}`, `{"Ab", "Cd"}`).

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/strings.rs`, the kernel
  attribute vocabulary file in `crates/rum-runtime` (wherever the
  cross-check test reads), `crates/rum-stdlib/src/tests/**` and
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated only).
- MUST NOT touch: any `eval_*` implementation, other catalog families,
  the guide, `TODO.md`, the tracker, the findings file.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every existing evaluation result stays byte-identical; the diff is
  attributes, vocabulary, regenerated reference, and tests.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file, incl. how the vocabulary was updated.
2. Any in-tree fallout from `Protected` (should be none — else you
   stopped and reported).
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
