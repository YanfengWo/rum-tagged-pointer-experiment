# Brief 143 — Scripting utilities batch (Python/Perl tier-1) + StringForm contract

Implementation brief. The merged tier-1 batch from the Python-stdlib and
Perl-core sweeps, plus the StringForm mini-audit. All pure; folk scope;
rum's own documented contracts. Zero new dependencies.

## Builtins

**Text shaping** (strings family):
- `StringWrap[s, width]` → greedy word-wrap to a String with `\n` (words
  longer than `width` stay unbroken on their own line; width ≥ 1).
- `StringIndent[s, prefix]` (String prefix) and `StringIndent[s, n]`
  (n spaces) → prefix every line (including empty ones? NO — skip
  whitespace-only lines, document).
- `StringDedent[s]` → remove the longest common leading whitespace of the
  non-empty lines.
- `StringExpandTabs[s, n]` → tabs to next n-column stop (default 8 with the
  one-argument form `StringExpandTabs[s]`).
- `AlignColumns[rows]` → rows is a list of equal-or-ragged lists of
  strings; left-align each column to its max width, two-space separator,
  trailing whitespace trimmed; returns one String with `\n`.

**HTML** (strings family):
- `HtmlEscape[s]` → escape `& < > " '`.
- `HtmlUnescape[s]` → the five named entities plus numeric `&#N;` /
  `&#xH;`; UNKNOWN entities pass through verbatim (tolerant; document).

**Shell strings** (strings family):
- `ShellSplit[s]` → POSIX-style word split (whitespace, single/double
  quotes, backslash escapes outside single quotes); unterminated quote →
  fail-closed message + `$Failed`.
- `ShellQuote[s]` → single-quote wrapping with the `'\''` escape; empty
  string → `''`.

**Path strings** (join the pure path family beside `FileNameJoin`):
- `FileExtension[s]` → extension without the dot (`""` if none; dotfiles
  like `.bashrc` have no extension — document).
- `FileBaseName[s]` → last component without its extension.
- `FileNameSplit[s]` → list of components (inverse-ish of `FileNameJoin`;
  absolute-path leading `/` handling documented).

**Exact statistics** (arithmetic family, beside `Mean`):
- `Median[list]` → exact; even length averages the two middle values
  (rational). Empty → message + inert (match `Mean`).
- `Commonest[list]` → List of the most frequent elements in
  first-occurrence order (ties all included).
- `Quantile[list, q]` → nearest-rank method (no interpolation), `q` an
  exact number in [0, 1]; document the exact rank formula
  (`Ceiling[q * n]`, clamped to 1..n; `q = 0` → first sorted element).

**URL strings** (new small family or strings — your call):
- `UrlEncode[s]` → percent-encode everything outside RFC 3986 unreserved
  (`A-Z a-z 0-9 - _ . ~`); UTF-8 bytes for non-ASCII.
- `UrlDecode[s]` → inverse; invalid `%` sequences or non-UTF-8 result →
  fail-closed message + `$Failed`.
- `UrlParse[s]` → Association with keys `"Scheme"`, `"Host"`, `"Port"`
  (Integer or Null), `"Path"`, `"Query"` (raw String or Null),
  `"Fragment"` (String or Null); folk grammar
  `scheme://host[:port][/path][?query][#fragment]` plus scheme-less
  `host/path` rejected (fail-closed) — document exactly what parses.

**Args** (new `args` module or beside strings):
- `ParseArgs[args]` (args = List of strings, e.g. `$ScriptArgs`) →
  `<|"Positional" -> {…}, "Options" -> <|k -> v|>, "Flags" -> {…}|>`.
  Deterministic spec-less rules: `--key=value` → option (key without
  dashes); bare `--key` → flag; `--` ends option parsing (rest
  positional); everything else positional. No `-x` short-option
  clustering (document as out of folk scope).

**StringForm contract** (the audit):
- Diagnose why `` StringForm["`1` + `2`", 3, 4] `` is inert. Implement the
  cataloged behavior: `` `n` `` inserts the n-th argument (rendered as by
  `ToString`), ```` `` ```` inserts the next argument in sequence, backtick
  escaping documented; result is the formatted String (evaluate eagerly —
  no lazy display form; that IS rum's contract). Fix the catalog summary if
  it overpromises. Add tests.

## Scope

- MAY create/modify: `crates/rum-stdlib/src/**` (implementations, catalog
  rows, tests), `stdlib_attributes.rs` vocabulary (brief-136 lesson),
  docgen if needed, `docs/reference/` via generator; ONE new guide section
  — extend `docs/guide/string-patterns.md` or `structured-data.md` is NOT
  right for these; add a short `docs/guide/text-utilities.md` chapter
  (wrap/indent/tables/html/shell/url/args, real transcripts) + guide README
  link.
- MUST NOT touch: kernel crates beyond vocabulary registration and (if
  StringForm lives kernel-side) the StringForm implementation site;
  `Cargo.*`; evaluator/dispatch.

## Tests

Coverage axes per family (canonical/boundary/negative), plus:
- `StringDedent`/`StringWrap` idempotence checks; `AlignColumns` ragged
  rows; tab stops at column boundaries.
- `ShellSplit` round-trips with `ShellQuote` on nasty strings (spaces,
  quotes, `$`, newline).
- URL encode/decode round-trip on unicode; `UrlParse` full and minimal
  forms + fail-closed negatives.
- `Median`/`Quantile` exactness pins (rational middles); `Commonest` tie
  order.
- `ParseArgs` mixed input incl. `--` semantics.
- `StringForm` placeholders, sequence form, escaping, arity mismatch
  (message + inert or $Failed — match family; document).
- One end-to-end: pipeline-shaped —
  `ParseArgs → UrlParse → AlignColumns` style script asserting one exact
  output string.
- Vocabulary/catalog registrations + reference drift.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect committed count + 19 (state the actual numbers)
cargo run -q -p rum-cli -- --eval 'StringWrap["the quick brown fox jumps over the lazy dog", 10]'
cargo run -q -p rum-cli -- --eval 'AlignColumns[{{"name","qty"},{"apple","3"},{"kiwi","10"}}]'
cargo run -q -p rum-cli -- --eval '{HtmlEscape["a<b & c"], HtmlUnescape["a&lt;b &amp; &#233;"]}'
cargo run -q -p rum-cli -- --eval 'ShellSplit["cc -o \"my prog\" main.c"]'
cargo run -q -p rum-cli -- --eval '{FileExtension["a/b/c.tar.gz"], FileBaseName["a/b/c.tar.gz"], FileNameSplit["/a/b/c"]}'
cargo run -q -p rum-cli -- --eval '{Median[{1, 2, 3, 4}], Commonest[{a, b, a, c, b}], Quantile[Range[10], 1/4]}'
cargo run -q -p rum-cli -- --eval 'UrlParse["https://example.com:8080/x/y?q=1#top"]'
cargo run -q -p rum-cli -- --eval 'UrlDecode[UrlEncode["a b/世界"]]'
cargo run -q -p rum-cli -- --eval 'ParseArgs[{"--verbose", "--out=x.txt", "--", "--literal", "pos"}]'
cargo run -q -p rum-cli -- --eval 'StringForm["`1` + `2` = `3`", 1, 2, 3]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Fail-closed where the brief says so; tolerant ONLY where documented
  (HtmlUnescape unknown entities).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. What changed — per file; the StringForm diagnosis in two sentences.
2. Verification — all commands verbatim.
3. Contract decisions beyond this brief (each becomes a limits note).
4. Stop-loss / open questions (empty if none).
