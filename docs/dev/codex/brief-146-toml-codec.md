# Brief 146 — TOML codec (`FromToml` / `ToToml`)

Implementation brief. Second step of `docs/dev/codec-family-plan.md`. The
largest codec of the line: a folk subset of TOML 1.0, spec-precise inside
the subset, fail-closed outside it. Pure, zero dependencies.

## Decode contract (`FromToml[str]` → `Association`)

Supported (spec-precise):
- bare and quoted keys; dotted keys; `[table]` headers; `[[array-of-
  tables]]`; inline tables `{k = v, …}`; arrays (heterogeneous allowed, as
  in TOML 1.0); comments.
- strings: basic `"…"` (full escape set incl. `\u`/`\U`), literal `'…'`,
  multiline basic `"""…"""` (line-ending backslash trimming per spec),
  multiline literal `'''…'''`.
- integers: decimal with underscores, `0x`/`0o`/`0b` → exact Integer.
- booleans → `True`/`False`.
- floats: decimal/exponent text → EXACT base-10 Rational (the family's
  number philosophy; `0.1` → `1/10`); underscores allowed; `inf`, `-inf`,
  `nan` fail closed (no float type in rum — document).
- datetimes, mapped onto the brief-141 value shapes:
  - offset date-time → exact rational Unix-second timestamp (offset
    folded);
  - local date-time → `{y, mo, d, h, mi, s}` date list;
  - local date → `{y, mo, d}`;
  - local time → `{h, mi, s}`;
  document this asymmetry (only offset date-times are instants).

Fail-closed with byte offsets: duplicate keys/tables, dotted-key
conflicts, redefining an inline table, invalid escapes, bad dates
(validate via the brief-141 calendar), `inf`/`nan`, anything outside the
subset. Duplicate-key policy matches FromJson (reject).

## Encode contract (`ToToml[assoc]` → String)

- Top level must be an Association with String keys; nested Associations
  emit as sub-tables (`[a.b]` headers, depth-first, insertion order);
  Lists of Associations emit as `[[array-of-tables]]`; other Lists as
  inline arrays; scalars: Integer, terminating Rational (as decimal;
  non-terminating fail closed, as ToJson), `True`/`False`, String (always
  basic-quoted with escapes).
- brief-141 values encode back: rational Unix timestamp CANNOT be
  distinguished from a plain number — so numbers always emit as numbers,
  and date lists `{y,mo,d}` / `{y,mo,d,h,mi,s}` are ALSO just lists (emit
  as arrays). Document plainly: `ToToml` does NOT synthesize TOML datetime
  syntax in this folk version (round-trip of datetime values is
  decode-only); a future option can add it if demanded. This keeps encode
  unambiguous — no guessing what a list of six integers means.
- Keys quote themselves when not bare-safe; keys containing `.` are
  quoted (never split). Empty Association → `{}`? No: an empty nested
  Association emits an empty `[a]` header; empty top level → `""`.

## Round-trip contract

`FromToml[ToToml[x]]` === x for every value ToToml accepts (datetime
value shapes excluded by construction — they encode as arrays/numbers and
decode back as the same). Pin with tests.

## Scope

- MAY create: `crates/rum-stdlib/src/toml.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/toml.rs`, tests
  `crates/rum-stdlib/src/tests/codecs_toml.rs`.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs` vocabulary,
  docgen if needed, `docs/reference/` via generator,
  `docs/guide/structured-data.md` (TOML section: the mapping table, the
  datetime asymmetry, transcripts).
- MUST NOT touch: other codecs' behavior, kernel crates beyond vocabulary,
  `Cargo.*`, evaluator/dispatch.

## Tests

- A realistic config end-to-end: title string, `[server]` table with
  dotted keys, ports array, `[[servers]]` array-of-tables, inline table,
  multiline basic string with escapes, offset datetime, local date —
  asserting the full decoded Association.
- Number exactness pins (`0.1` → `1/10`; `1_000_000`; hex/oct/bin;
  exponent).
- Datetime mapping pins in all four forms; offset folding equals the
  brief-141 `FromDateString` result for the same instant.
- Multiline string edge: leading-newline trimming, backslash line
  continuation.
- Fail-closed negatives: duplicate key, dotted conflict, `inf`, invalid
  date (Feb 30), bad escape, table redefinition — each with message
  assertion.
- Encode: sub-table vs array-of-tables selection, key quoting, insertion
  order, non-terminating rational rejection; round-trip pins.
- Cross-codec: `FromToml` → `ToJson` on the realistic config (datetime-free
  variant), `FromJson` → `ToToml` inverse.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 590
printf '' | cargo run -q -p rum-cli -- --eval 'FromToml["title = \"x\"\n[db]\nports = [1, 2]\npi = 3.14\nwhen = 1970-01-01T01:00:00+01:00\n"]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromToml["[[s]]\nn = 1\n[[s]]\nn = 2\n"]'
printf '' | cargo run -q -p rum-cli -- --eval 'ToToml[<|"a" -> <|"b" -> {1, 1/4, True}, "c" -> "x.y"|>|>]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromToml[ToToml[<|"k" -> <|"x" -> 1|>|>]]'
printf '' | cargo run -q -p rum-cli -- --eval 'FromToml["a = inf"]'          # fail-closed
printf '' | cargo run -q -p rum-cli -- --eval 'FromToml["d = 2026-02-30"]'   # fail-closed (calendar)
printf '{"srv": {"port": 8080}}' | cargo run -q -p rum-cli -- --eval 'ToToml[FromJson[$ScriptInput]]'
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- Spec-precise INSIDE the subset; fail-closed OUTSIDE it; never guess.
  Encode never synthesizes datetime syntax (documented folk boundary).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed + decisions beyond this brief (each a limits
   note).
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
