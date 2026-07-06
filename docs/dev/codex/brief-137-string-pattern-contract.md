# Brief 137 — String-pattern contract: catalog the dark surface, fix the edges

Implementation brief. Second batch of the stdlib-expansion mainline. The
brief-135 probe mapped the string-pattern engine: it supports far more than
the documentation admits. This brief turns that surface into contract and
fixes the known holes.

## Evidence (from the brief-135 matrix — reproduce any cell you rely on)

Working across StringCases/StringReplace/StringMatchQ/StringSplit/
StringPosition/StringContainsQ but largely uncataloged: literals, `~~`,
`|` alternatives, `_`, `__`, `Repeated`/`..`, `RepeatedNull`/`...`,
`DigitCharacter`, `LetterCharacter`, `WhitespaceCharacter`, `Whitespace`,
`WordCharacter`, `NumberString`, `StartOfString`, `EndOfString`,
`Except[...]`, `Shortest`/`Longest`, named binding (`x_`, `Pattern`).

Inert everywhere: `PunctuationCharacter`, `StartOfLine`, `EndOfLine`,
`WordBoundary`. `IgnoreCase -> True` works everywhere EXCEPT `StringSplit`
(stays inert unevaluated).

Bug-like edges:
1. `StringReplace["abc", EndOfString -> "$"]` → `"abc"` (no insertion) while
   `StringPosition["abc", EndOfString]` → `{{4, 3}}` (an out-of-range,
   inverted span).
2. `StringCases["", "a"...]` → `{""}` but `StringReplace["", "a"... -> "x"]`
   → `""` (inconsistent empty-match handling at the empty string).
3. `StringMatchQ["aaa", Shortest["a"..]]` → `False` (Longest → True;
   MatchQ is whole-string — laziness must not change whole-string truth).

## Goal

1. **Implement the four inert atoms**: `PunctuationCharacter` (define the
   class over ASCII punctuation + document the exact set), `StartOfLine`,
   `EndOfLine` (anchors at string start/end and around `\n`), `WordBoundary`
   (transition between `WordCharacter` and non-word/edges).
2. **`StringSplit` accepts `IgnoreCase -> True`** like its five siblings.
3. **Fix the three edges** so that: `EndOfString`/`StartOfString` (and the
   new line anchors) behave as zero-width anchors in ALL six functions —
   replacement inserts at the anchor, positions report zero-width spans in a
   consistent `{pos, pos-1}`-or-documented shape (pick one, document it,
   test it); empty-string empty-match behavior is consistent between Cases
   and Replace (decide the contract, document, test); `Shortest`/`Longest`
   never change `StringMatchQ`'s whole-string verdict.
4. **Catalog + guide**: every construct above gets a reference entry (the
   pattern atoms are symbols — register them so the generator picks them up,
   or extend docgen with a documented string-pattern section if
   registration is wrong for inert pattern atoms; choose the mechanism that
   keeps the drift test meaningful and say why). `docs/guide/syntax.md` or a
   new short `docs/guide/string-patterns.md` chapter documents the string
   pattern language with real transcripts (follow the guide's
   examples-are-real rule).

## Scope

- MAY modify: the string-pattern engine and string builtins under
  `crates/rum-stdlib/src/strings/**` and (if the matcher plumbing lives
  kernel-side) the string-pattern compiler under `crates/rum-runtime/src/**`
  — but NOT the general expression matcher; catalog/attribute/vocabulary
  registrations in BOTH crates for new symbols (learn from brief-136: new
  stdlib-visible symbols must also be added to
  `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`);
  tests in both crates; `docs/reference/` via the generator +
  `crates/rum-stdlib/src/docgen.rs` if a string-pattern section is added;
  ONE new guide chapter file (or edits to `syntax.md`) + a link line in
  `docs/guide/README.md`.
- MUST NOT touch: the general expression matcher's semantics, evaluator
  dispatch, other guide chapters, scripts.

## Behavior discipline

- Existing green behavior is contract: everything marked W in the matrix
  must keep byte-identical results (the existing string tests + new
  matrix-derived regression tests enforce this).
- Fixes only change the cells listed under Goal 3 plus the inert→working
  atoms. Anything else that looks wrong: REPORT, do not fix.
- Zero-width anchor semantics: write the contract down first (in the guide
  section), then make all six functions agree with it.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
# The fixed edges, verbatim:
cargo run -q -p rum-cli -- --eval 'StringReplace["abc", EndOfString -> "$"]'
cargo run -q -p rum-cli -- --eval 'StringPosition["abc", EndOfString]'
cargo run -q -p rum-cli -- --eval '{StringCases["", "a"...], StringReplace["", "a"... -> "x"]}'
cargo run -q -p rum-cli -- --eval '{StringMatchQ["aaa", Shortest["a"..]], StringMatchQ["aaa", Longest["a"..]]}'
# The new atoms, one probe each:
cargo run -q -p rum-cli -- --eval 'StringCases["ab, cd! ef", PunctuationCharacter]'
cargo run -q -p rum-cli -- --eval 'StringReplace["x\ny", StartOfLine -> "> "]'
cargo run -q -p rum-cli -- --eval 'StringCases["one two", WordBoundary ~~ WordCharacter]'
cargo run -q -p rum-cli -- --eval 'StringSplit["AxbXc", "x", IgnoreCase -> True]'
git status --short
```

Plus a matrix re-run: re-execute the brief-135 probe script shape for all
six functions × all constructs and paste the updated table (target: no `I`
rows left except deliberately unsupported ones, which must be listed).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- No regex engine, no new dependencies.
- No new `unsafe`. String functions are not the eval hot path, but IF any
  change touches shared matcher/dispatch code paths, STOP and report
  instead (that would need an A/B and a narrower design).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The anchor/empty-match contract as written (quote the guide text).
2. What changed — per file; the catalog/vocabulary registrations listed.
3. Verification — all commands verbatim + the updated matrix.
4. Rule violations you almost made / edges you left alone (report-only).
