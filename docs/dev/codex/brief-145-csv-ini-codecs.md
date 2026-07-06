# Brief 145 — CSV and INI codecs (`FromCsv`/`ToCsv`, `FromIni`/`ToIni`)

Implementation brief. First step of `docs/dev/codec-family-plan.md` (read
it and the family conventions first). Two small value-oriented codecs, both
pure, both fail-closed, zero dependencies.

## CSV contract

- `FromCsv[str]` → `List` of row `List`s of Strings. RFC 4180 core:
  double-quoted fields may contain the delimiter, quotes (doubled `""`),
  and newlines; rows split on LF or CRLF; a single trailing newline yields
  no empty final row. VALUES ARE STRINGS — no type inference (document;
  scripts convert explicitly with `ToExpression`/`FromDigits` etc.).
  Ragged rows are ALLOWED (real-world CSV; document). Fail-closed: an
  unterminated quoted field, or characters between a closing quote and the
  next delimiter/newline, reject with byte offset.
- Options: `"Delimiter" -> ","` (default; any single character, e.g.
  `"\t"` for TSV); `"Headers" -> False` (default) — when `True`, the first
  row is the header and the result is a `List` of `Association`s keyed by
  the header strings (duplicate header names fail closed; ragged data rows
  fail closed in this mode).
- `ToCsv[rows]` → String. Accepts the same two shapes (list-of-lists of
  Strings, or list-of-Associations with identical String keys +
  `"Headers" -> True` implied — infer and document). Fields are quoted
  exactly when they contain the delimiter, a quote, or a newline; quotes
  doubled; rows joined with `\n`. Non-String field → fail-closed (message
  names the position); the caller renders values first (`ToString`).

## INI contract (ONE dialect, documented as rum's own)

- `FromIni[str]` → `Association` of section-name → `Association` of
  key → value (all Strings). Grammar: `[section]` headers; `key = value`
  lines; keys/values trimmed of surrounding whitespace; values taken
  VERBATIM after trimming (quotes are NOT stripped — document); blank
  lines ignored; comment lines start with `;` or `#` (leading whitespace
  allowed); NO nesting, NO multiline values, NO inline comments (a `;` in
  a value is part of the value — document).
- Keys appearing before any section header live under the section name
  `""` (document).
- Fail-closed with offsets: a non-comment non-blank line that is neither a
  header nor `key = value`; duplicate section names; duplicate keys within
  a section; empty key; unterminated `[` header.
- `ToIni[assoc]` → String. Input must be Association-of-Associations with
  String keys and String values (the `""` section, if present, is emitted
  first without a header); anything else fail-closed. Section/key names
  containing `[`, `]`, `=`, newline, or leading `;`/`#` fail closed
  (unrepresentable in the dialect — document).

## Round-trip contract

`FromIni[ToIni[x]] === x` for every valid input shape; same for CSV in
both header modes (given String fields). Pin with tests.

## Scope

- MAY create: `crates/rum-stdlib/src/codecs_text.rs` or sibling modules
  beside `json.rs` (follow the family layout — e.g. `csv.rs` + `ini.rs`),
  catalog rows, tests.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs` vocabulary
  (standing lesson), docgen if needed, `docs/reference/` via generator,
  `docs/guide/structured-data.md` (CSV and INI sections with real
  transcripts, mirroring the JSON section's structure).
- MUST NOT touch: `json.rs` behavior, kernel crates beyond vocabulary,
  `Cargo.*`, evaluator/dispatch.

## Tests

Coverage axes per codec: canonical, quoting edges (embedded delimiter/
quote/newline; doubled quotes at field start/end), CRLF vs LF, empty
fields/rows/input, unicode content, both header modes, TSV delimiter
option, ragged rows (allowed in plain mode, rejected in header mode), all
fail-closed negatives with message assertions, round-trip pins, and one
cross-codec end-to-end: `FromCsv` (headers) → transform → `ToJson`, plus
`FromIni` → edit one key → `ToIni` asserting exact output text.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 588
printf '' | cargo run -q -p rum-cli -- --eval 'FromCsv["a,\"b,1\"\nc,d"]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromCsv["name,qty\napple,3\nkiwi,10", "Headers" -> True]'
printf '' | cargo run -q -p rum-cli -- --eval 'ToCsv[{{"x","y\"z"},{"1","2"}}]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromCsv["a,\"unterminated"]'      # fail-closed
printf '' | cargo run -q -p rum-cli -- --eval 'FromIni["; c\ntop = 1\n[db]\nhost = h ; not a comment\n"]'
printf '' | cargo run -q -p rum-cli -- --eval 'ToIni[FromIni["[s]\nk = v\n"]]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromIni["[dup]\n[dup]\n"]'        # fail-closed
printf '{"k": 1}' | cargo run -q -p rum-cli -- --eval 'ToIni[<|"" -> <|"k" -> ToString[FromJson[$ScriptInput]["k"]]|>|>]'   # cross-codec pipeline
git status --short
```

(Note the `printf '' |` stdin feeds — the CLI's filter mode waits on a
non-TTY stdin otherwise.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Fail-closed per the family convention; tolerant behavior ONLY where this
  brief documents it (ragged plain-mode CSV rows, verbatim INI values).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The two contracts as landed + any decisions beyond this brief (each
   becomes a limits note).
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
