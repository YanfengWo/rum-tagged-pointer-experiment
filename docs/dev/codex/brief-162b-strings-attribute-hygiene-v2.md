# Brief 162b — Strings attribute hygiene, v2 (threading decision resolved)

Implementation brief. Supersedes brief-162, whose stop-loss was correct:
adding `Listable` changes nested/mixed-list INERT forms. That change is
now DECIDED and intended (tracker `docs/dev/usability-pass.md`, unit-1
triage item 1): rum already committed to `Listable` threading via
`ToLowerCase`/`StringReverse`; the case-conversion family aligns upward.
brief-162 left one pinning test in the working tree — you will UPDATE it
to the new contract (see Changes #5).

## Changes

1. **`Protected` on the string-pattern symbols** (18, audit E1 list):
   `Alternatives`, `Blank`, `BlankNullSequence`, `BlankSequence`,
   `DigitCharacter`, `EndOfLine`, `EndOfString`, `Except`,
   `LetterCharacter`, `Pattern`, `PunctuationCharacter`,
   `RegularExpression`, `StartOfLine`, `StartOfString`,
   `StringExpression`, `WhitespaceCharacter`, `WordBoundary`,
   `WordCharacter`. Mechanism: brief-159's
   `BuiltinSpec::with_attributes(..., ATTR_PROTECTED)` as in
   `crates/rum-stdlib/src/catalog/character_classes.rs`. Pattern
   construction/matching is unaffected; only assignment/definition on
   these heads is blocked.
2. **Case-conversion alignment**: `ToUpperCase` gains
   `Listable` + `Protected`; `Capitalize` and `Decapitalize` gain
   `Listable`. Contract after the change (mirrors `ToLowerCase`):
   - plain strings and FLAT string lists: byte-identical to today;
   - nested lists now thread: `ToUpperCase[{{"a"}, {"b"}}]` →
     `{{"A"}, {"B"}}`, `Capitalize[{{"ab"}, {"cd"}}]` →
     `{{"Ab"}, {"Cd"}}`, `Decapitalize[{{"Ab"}, {"Cd"}}]` →
     `{{"ab"}, {"cd"}}`;
   - mixed lists now thread with inert non-string leaves:
     `ToUpperCase[{"a", 1}]` → `{"A", ToUpperCase[1]}`,
     `Capitalize[{"ab", 1}]` → `{"Ab", Capitalize[1]}`,
     `Decapitalize[{"Ab", 1}]` → `{"ab", Decapitalize[1]}`.
   If the implementation's internal list-mapping now conflicts with
   kernel threading (double handling), simplify the implementation to
   the per-string core and let `Listable` do the mapping — provided
   flat-list results stay byte-identical.
3. **Kernel attribute vocabulary**: update the rum-runtime vocabulary
   so `catalog_attributes_match_kernel_vocabulary` passes (the test
   points you at the file).
4. **Regenerate the builtin reference**:
   `cargo run -p rum-stdlib --bin generate-builtin-reference`.
5. **Tests**: update brief-162's
   `case_conversion_list_mapping_does_not_thread_nested_or_mixed_lists`
   (in `crates/rum-stdlib/src/tests/strings/surface.rs`, currently
   uncommitted in the working tree) to assert the NEW threaded results
   and rename it accordingly (e.g.
   `case_conversion_threads_nested_and_mixed_lists_like_listable`).
   Add: attribute assertions for every symbol touched; a
   Protected-rejection test (`DigitCharacter = 5` and `Blank[x_] := x`
   emit the protection message, symbol behavior intact — mirror
   brief-159's shape); flat-list byte-identity pins
   (`ToUpperCase[{"a","b"}]` → `{"A","B"}`, etc.).

## Watch out

- Protecting `Pattern`/`Blank`/`Alternatives`/`Except` touches heads
  used by every expression pattern. Construction and matching are not
  affected by `Protected`, but run the FULL workspace suite; if
  anything in-tree legitimately assigns to one of these heads, STOP
  and report instead of unprotecting ad hoc.
- The threading upgrade is scoped to the three named symbols. If any
  OTHER symbol's results change, that is fallout — STOP and report.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Attributes /@ {DigitCharacter, Pattern, StringExpression, WordBoundary}, Attributes /@ {ToUpperCase, ToLowerCase, Capitalize, Decapitalize}}'
printf '' | cargo run -q -p rum-cli -- --eval '{DigitCharacter = 5, Blank[x_] := x}'
printf '' | cargo run -q -p rum-cli -- --eval '{ToUpperCase[{"a", "b"}], ToUpperCase[{{"a"}, {"b"}}], ToUpperCase[{"a", 1}], Capitalize[{{"ab"}, {"cd"}}], Decapitalize[{"Ab", 1}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 shows
`Protected` on the pattern symbols and `Listable`+`Protected` on all
four case functions; probe 2 shows two protection messages; probe 3
shows `{"A","B"}`, `{{"A"},{"B"}}`, `{"A", ToUpperCase[1]}`,
`{{"Ab"},{"Cd"}}`, `{"ab", Decapitalize[1]}`.

## Scope

- MAY modify: `crates/rum-stdlib/src/catalog/strings.rs`, the
  case-conversion implementation file(s) under
  `crates/rum-stdlib/src/strings/` (ONLY if needed per Changes #2),
  the kernel attribute vocabulary file in `crates/rum-runtime`,
  `crates/rum-stdlib/src/tests/**` and
  `crates/rum-stdlib/src/tests.rs`, `docs/reference/builtins.md`
  (regenerated only).
- MUST NOT touch: other catalog families, other implementations, the
  guide, `TODO.md`, the tracker, the findings file.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Behavior changes are EXACTLY: (a) assignment to the 18 protected
  symbols now messages, (b) nested/mixed-list threading for the three
  case functions. Everything else byte-identical.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; state whether Changes #2's implementation
   simplification was needed.
2. Any in-tree fallout from `Protected` (should be none).
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
