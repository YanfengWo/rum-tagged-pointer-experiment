# Brief 138 — In-language JSON codec (`FromJson` / `ToJson`)

Implementation brief. Cornerstone of the codec family: this brief SETS the
family conventions (`From*`/`To*` named pairs, fail-closed parsing, error
style) that INI/TOML/XML/YAML will follow later. Pure string↔expression —
no I/O, no capability beyond `Pure`, fully deterministic.

## Goal

`FromJson[str]` parses an RFC 8259 JSON text into rum values; `ToJson[expr]`
renders the inverse subset; both are cataloged, reference-documented (drift
green), guide-documented with real transcripts, and covered by tests
including fail-closed negatives. Self-implemented parser/serializer in
`rum-stdlib` — no new dependencies. Workspace green.

## Value mapping (the contract)

Decode (`FromJson`):
- object → `Association` (member order preserved; duplicate keys →
  fail-closed error);
- array → `List`;
- string → String (full escape handling incl. `\uXXXX` and surrogate pairs
  → correct UTF-8);
- number → EXACT value: integers (any size — exact big integers) →
  Integer; decimals/exponents → exact base-10 Rational (`0.1` → `1/10`,
  `1.5e3` → `1500`); never floats, never precision loss;
- `true`/`false`/`null` → `True`/`False`/`Null`.

Encode (`ToJson`):
- Integer → JSON integer; Rational with terminating decimal expansion
  (denominator `2^a·5^b`) → decimal literal; NON-terminating Rational →
  fail-closed error (message names the value; the script author converts
  explicitly);
- String → JSON string (escapes; non-BMP via surrogate pairs);
- `True`/`False`/`Null` → literals;
- List → array; Association with String keys → object (non-String key →
  fail-closed error);
- anything else (symbols, ordinary calls) → fail-closed error + `$Failed`.
- Default output compact; `ToJson[expr, "Indent" -> n]` pretty-prints with
  `n`-space indentation. No other options in v1.

## Fail-closed discipline (family convention — write it once, follow forever)

- Reject with a message + `$Failed`: trailing non-whitespace after the
  value, unquoted keys, single quotes, comments, `NaN`/`Infinity`, control
  characters in strings, invalid escapes, lone surrogates, duplicate object
  keys.
- Messages: one symbol-family, e.g. `FromJson::json` / `ToJson::json` (or a
  `General::`-style tag pair — match the existing message-style norms in the
  stdlib; state your choice). Message text names the byte/char offset for
  parse errors.
- Depth guard: a fixed nesting cap (256) with its own clear message —
  parsing is recursive; the cap must trip before any native-stack risk
  (the brief-131 stack guard is the backstop, not the primary).
- Large inputs: element/size growth counts against the session's existing
  work limits where natural; do not add new limit machinery.

## Scope

- MAY create: `crates/rum-stdlib/src/json.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/json.rs`, tests
  (`crates/rum-stdlib/src/tests/json.rs`), `docs/guide/structured-data.md`
  (new chapter: the JSON mapping table, fail-closed policy, transcripts) +
  a link line in `docs/guide/README.md`.
- MAY modify: `crates/rum-stdlib/src/catalog/mod.rs` + the install path for
  the new family; `crates/rum-runtime/src/runtime/builtin_catalog/stdlib_attributes.rs`
  (REMEMBER the brief-136 lesson: register both names; family norm =
  attribute-free `ordinary` unless you have a reason); `docs/reference/`
  via the generator.
- MUST NOT touch: evaluator/dispatch, string-pattern engine, other guide
  chapters, `Cargo.*` (zero new dependencies).

## Tests (coverage axes)

- Round-trips: nested objects/arrays, all scalar kinds, unicode incl.
  surrogate pairs, big integers (hundreds of digits), exact decimals.
- Decode-only: exponents, `-0` (document the result), whitespace forms.
- Encode-only: Indent option shape; non-terminating rational error;
  non-String key error; symbol/call error.
- Fail-closed negatives: every rejection listed above, each asserting the
  message fires and the result is `$Failed`.
- Depth cap: a 300-deep nested array trips the cap message (build the input
  by string repetition, not by literal source).
- One end-to-end: `FromJson` → transform (`GatherBy`/`Mean` from brief-136)
  → `ToJson`, asserting exact output text.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md          # expect 520
cargo run -q -p rum-cli -- --eval 'FromJson["{\"a\": [1, 2.5, true, null], \"b\": {\"c\": \"x\"}}"]'
cargo run -q -p rum-cli -- --eval 'ToJson[<|"n" -> 1/4, "big" -> 2^100, "s" -> "héllo", "l" -> {True, Null}|>]'
cargo run -q -p rum-cli -- --eval 'ToJson[<|"x" -> 1/3|>]'          # fail-closed (non-terminating)
cargo run -q -p rum-cli -- --eval 'FromJson["{\"a\": 1,}"]'          # fail-closed (trailing comma)
cargo run -q -p rum-cli -- --eval 'FromJson[ToJson[<|"k" -> {1, 2/5, "z"}|>]]'   # round-trip
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network.
- Zero new dependencies; parser/serializer are plain safe Rust in stdlib.
- No new `unsafe`. Not an eval-hot-path change; if you touch dispatch/eval
  code, STOP and report.
- Fail-closed means fail-closed: when in doubt between guessing and
  erroring, error.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed (mapping table + message tags + any decisions
   beyond this brief).
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
