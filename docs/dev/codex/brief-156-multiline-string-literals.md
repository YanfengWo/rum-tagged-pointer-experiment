# Brief 156 — Multiline string literals: `"""…"""` (escaped) + `'''…'''` (raw)

Implementation brief (B2) — a SYNTAX CONTRACT addition, decided by the
user: rum source gains the same two multiline string forms as rum's own
TOML subset. Parser-level blast radius: lexer + parser + formatter + guide.
Zero evaluator changes (a multiline literal lowers to an ordinary String
atom).

## The contract

- `"""` … `"""`: multiline string, ESCAPES PROCESSED (exactly the escape
  set of ordinary `"…"` strings). May contain `"` and `""` freely; ends at
  the first unescaped `"""`.
- `'''` … `'''`: multiline RAW string — no escape processing at all,
  content verbatim; cannot contain `'''` (document; no escape hatch in
  v1). May contain `"` , `\`, newlines freely.
- BOTH forms: if the first character after the opening delimiter is a
  newline (`\n` or `\r\n`), that first newline is DROPPED. No automatic
  dedent — `StringDedent` is the explicit companion (document the
  pairing).
- The literal is INPUT SYntax only: it lowers to a normal String atom;
  canonical output rendering (`ToString`, value printing) remains the
  single-line escaped form. The FORMATTER, however, preserves multiline
  literals verbatim (trivia/source-shape preservation, like comments).
- Unterminated literal → graceful parse error with a span (same style as
  existing string errors); the parse-depth guard is unaffected.

## Phase A — the `'` ambiguity checkpoint (do this FIRST)

`'` exists in the token taxonomy (postfix derivative territory in this
notation family). Determine what `'` does in rum TODAY (lexer + parser +
any evaluator meaning):

- If postfix `'` is live syntax (e.g. `f'''` must keep meaning a triple
  application), `'''` may ONLY open a string where an OPERAND is expected
  (prefix/value position), never in postfix position. Determine whether
  the lexer can make this distinction (position-aware lexing or lexer
  feedback from the parser). Write the disambiguation rule down as part of
  the contract and PIN BOTH SIDES with tests (`f'''[x]`-style postfix
  still parses as today; `x = '''…'''` opens a raw string).
- If `'` is currently inert/unused in practice, say so with evidence and
  keep the simple rule (still pin what `f'''` does after the change).
- If the disambiguation would require invasive lexer restructuring, STOP
  and report options — do not force it.

## Tests

- Lexer/parser: both forms, empty body, leading-newline drop (`\n` and
  `\r\n`), internal quotes, escapes processed vs raw verbatim (`\t` stays
  two characters in `'''`), unterminated → error with span, delimiter-
  adjacent edges (`""""` = `"""` + `"` cases — pin the greedy rules),
  the `'`-ambiguity pins from Phase A.
- Lowering: multiline literal === the equivalent ordinary string
  (`"""\nab""" === "ab"` style assertions through the evaluator).
- Formatter: a file containing both forms round-trips VERBATIM (source
  in = source out); mixed with comments and ordinary strings.
- End-to-end: a script embedding multiline inline data (the brief-135
  friction case) — define a small CSV/log block inline with `"""`,
  process it with `StringLines`/`FromCsv`, assert exact results.
- REPL/CLI: multi-line input completeness — `is_complete` must report
  incomplete while a multiline literal is open (test through the host
  `is_complete` request), so the REPL keeps reading.

## Docs

`docs/guide/syntax.md` §Strings: the two forms, the leading-newline rule,
the no-dedent decision with the `StringDedent` pairing, the raw-form
`'''`-content limitation, and the input-sugar/canonical-output note — real
transcripts. One line in `docs/guide/getting-started.md` if its examples
benefit.

## Scope

- MAY modify: `crates/rum-syntax/src/**` (lexer, parser, trivia/formatter,
  tests), `crates/rum-runtime/src/**` ONLY if lowering/`is_complete`
  plumbing requires it (report what), `crates/rum-repl`/`rum-host` tests
  for the is-complete pin, the two guide files, `docs/reference/` ONLY if
  the generator output shifts (unlikely — no new builtins).
- MUST NOT touch: evaluator semantics, stdlib, `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval 'StringLines["""
a,1
b,2
"""]'
printf '' | cargo run -q -p rum-cli -- --eval $'x = \'\'\'\nC:\\path \\t raw\n\'\'\'; StringLength[x]'
printf '' | cargo run -q -p rum-cli -- --eval '"""line1\nline2""" === "line1\nline2"'
# formatter round-trip on a file with both forms:
#   write /tmp/rum-156-fmt.wl containing both literals + a comment, run
#   --format (or the file formatter path), paste in==out evidence
# the Phase A postfix pin (whatever f''' means, before == after):
#   paste the probe and both-side outputs
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Existing sources must parse EXACTLY as before — the whole test corpus is
  the regression net; any change in an existing test's expectation is a
  defect, not an update.
- Formatter verbatim-preservation is a hard requirement (trivia rule).
- Stop-loss: Phase A says invasive → STOP with options; same failure
  twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. Phase A finding: what `'` is today, the disambiguation rule as landed.
2. The contract as landed + greedy-delimiter edge rules.
3. What changed — per file.
4. Verification — all commands verbatim incl. formatter round-trip.
5. Stop-loss / open questions (empty if none).
