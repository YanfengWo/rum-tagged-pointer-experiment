# Brief 144 — Tokenizer mechanism: `TokenizeWith` + `RumTokens`

Implementation brief. The last batch of the stdlib-expansion line: the
error-tolerant span machinery that turns the (brief-137) string-pattern
language into a highlighting/text-tooling engine, plus rum's own lexer
exposed as spans. Grammar DEFINITIONS stay out — they are rum packages
(future dun plugins), never stdlib code.

## Builtins (both Pure)

### `TokenizeWith[rules, str]`

- `rules`: a `List` of `class -> pattern` (`class` a String; `pattern` in
  the documented string-pattern language). Held/evaluated like the string
  functions treat their pattern arguments (match the family's argument
  handling).
- Scan left to right. At each position, try every rule; take the LONGEST
  match; ties break to the EARLIEST rule. Zero-width matches are skipped
  (a rule that matches empty cannot fire — prevents non-progress; document).
- Characters no rule matches coalesce into `"text"` spans.
- Output: `List` of `{class, {start, end}}` — 1-based inclusive code-point
  spans, the `StringPosition` convention. Invariants: spans strictly
  ascending, non-overlapping, and their union covers the whole string.
- Errors fail closed: non-String class, invalid pattern, non-String input →
  message + `$Failed` (family style).

### `RumTokens[str]`

- Tokenizes `str` with the `rum-syntax` lexer (rum-stdlib already depends
  on rum-syntax). Output: `List` of `{kind, {start, end}}`, `kind` a String
  naming the lexer's token taxonomy (document: the taxonomy follows
  rum-syntax and may grow — the CONTRACT is the shape, coverage, and span
  convention, not the exact kind list).
- TOTAL and error-tolerant: trivia (whitespace, comments) are tokens;
  input the lexer cannot recognize becomes `"error"`-kind spans; the span
  union always covers the whole input; never `$Failed` on any String input.
  If the current lexer aborts on some inputs, wrap it so unrecognized tails
  become one `"error"` span — report what you had to do.
- Spans are code-point positions in the same convention as `TokenizeWith`
  (convert from byte offsets if the lexer is byte-based — test multibyte).

## Scope

- MAY create: `crates/rum-stdlib/src/strings/tokenize.rs` (or a sibling
  module — follow family layout), `crates/rum-stdlib/src/catalog/` rows,
  tests `crates/rum-stdlib/src/tests/strings/tokenize.rs`.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs` vocabulary,
  docgen if needed, `docs/reference/` via generator,
  `docs/guide/string-patterns.md` (a new "Tokenization" section: both
  builtins, the invariants, one INI-ish highlighting example with real
  transcripts, one sentence that grammar rule-sets are ordinary rum data
  meant to live in packages).
- MAY read `crates/rum-syntax/src/**`; MAY add a minimal `#[doc(hidden)]`
  read-only token-iteration API THERE if the lexer's public surface is
  insufficient (the brief-128 precedent) — no behavior change.
- MUST NOT touch: the string-pattern engine's semantics, evaluator/dispatch,
  kernel crates beyond the above, `Cargo.*`.

## Tests

- Invariant tests for both builtins: ascending, non-overlapping, total
  coverage — on empty, ASCII, multibyte (CJK + emoji), and adversarial
  inputs.
- `TokenizeWith`: longest-match pin; tie-to-earliest pin; zero-width rule
  skipped; `"text"` coalescing; fail-closed negatives; an INI grammar
  end-to-end (comment/section/key/value classes) asserting the exact span
  list.
- `RumTokens`: a rum snippet with comment + string + operator trivia
  coverage; malformed input (unclosed string, bad glyph) still totally
  covered with `"error"` spans; multibyte positions correct.
- Cross-check: `TokenizeWith` spans convertible to `StringTake` extraction
  reproducing the input exactly (round-trip by concatenation).
- Vocabulary/catalog registrations + reference drift.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 584
cargo run -q -p rum-cli -- --eval 'TokenizeWith[{"comment" -> (";" ~~ ___ ~~ EndOfLine), "number" -> NumberString, "word" -> (WordCharacter..)}, "x = 42 ; note"]'
cargo run -q -p rum-cli -- --eval 'TokenizeWith[{"a" -> "aa", "b" -> "a"}, "aaa"]'      # longest/earliest pins
cargo run -q -p rum-cli -- --eval 'RumTokens["f[x_] := x + 1 (* c *)"]'
cargo run -q -p rum-cli -- --eval 'RumTokens["\"unclosed"]'                              # total coverage with error span
cargo run -q -p rum-cli -- --eval 'StringJoin[Map[StringTake["日本 x1", #[[2]]] &, RumTokens["日本 x1"]]]'   # multibyte round-trip
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- `RumTokens` must never fail on a String input — total coverage is the
  contract.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file; whether rum-syntax needed the read-only API.
2. Verification — all commands verbatim.
3. The documented token-kind taxonomy as landed (list).
4. Stop-loss / open questions (empty if none).
