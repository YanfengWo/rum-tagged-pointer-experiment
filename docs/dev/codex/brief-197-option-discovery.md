# Brief 197 — Register codec/schema option defaults for Options[...] discovery (unit-6 fix 3)

Implementation brief. Executes the unit-6 triage batch 3 (tracker
`docs/dev/usability-pass.md`, "Unit 6 (codecs+bytes+datetime) triage";
evidence E10 in `docs/dev/usability-pass/codecs-bytes-datetime-findings.md`).
This is the DISCOVERY-surface registration pattern from unit 2 (brief-170
Heads) / unit 1 (StringCases Overlaps): the options-defaults table feeds
`Options[...]` / `OptionValue[...]` ONLY; implementations read their
options directly from call args and NEVER consult the table. So this is
byte-identical BEHAVIOR — pure discoverability. No A/B, no reference regen
(the generated `builtins.md` does not list Options).

## The gap

Six unit-6 heads already IMPLEMENT options (parsed directly from call
args) but return `Options[head] == {}`, so a user cannot discover them:
`ToJson` (`"Indent"`), `ToXml` (`"Indent"`), `FromCsv` / `ToCsv`
(`"Delimiter"`, `"Headers"`), `ConformSchema` / `SchemaErrors`
(`"AllowExtraKeys"`). Register their real defaults.

## The real defaults (verified against the implementations — the registered value MUST match)

- `ToJson` `"Indent"` → default **`None`** (compact; `parse_to_json_args`
  returns `Option<usize>` = `None` when absent — `crates/rum-stdlib/src/
  json.rs:62`).
- `ToXml` `"Indent"` → default **`None`** (same shape — `xml.rs:53`).
- `FromCsv` `"Delimiter"` → **`","`**, `"Headers"` → **`False`**
  (`FromCsvConfig::default()` = `{ delimiter: ',', headers: false }`,
  `csv.rs:19`).
- `ToCsv` `"Delimiter"` → **`","`**, `"Headers"` → **`Automatic`**
  (`ToCsvConfig::default()` = `{ delimiter: ',', headers: None }`;
  `headers: None` means "infer from row shape" — List rows emit no header,
  Association rows emit one — so `Automatic` is the honest discovery value,
  `csv.rs:28`, `csv.rs:518`).
- `ConformSchema` / `SchemaErrors` `"AllowExtraKeys"` → default **`False`**
  (`SchemaOptions::default()` = `{ allow_extra_keys: false }`;
  `schema.rs:10`, `schema.rs:316`).

## Change — `crates/rum-runtime/src/runtime/options/defaults.rs`

Add arms to the `match symbol.name()` in `default_system_options` (use the
existing helpers `system_rule_option`, `bool_option`, `system_expr`,
`Expr::string`):

```rust
"ToJson" => vec![system_rule_option("Indent", system_expr("None"))],
"ToXml" => vec![system_rule_option("Indent", system_expr("None"))],
"FromCsv" => vec![
    system_rule_option("Delimiter", Expr::string(",")),
    system_rule_option("Headers", bool_option(false)),
],
"ToCsv" => vec![
    system_rule_option("Delimiter", Expr::string(",")),
    system_rule_option("Headers", system_expr("Automatic")),
],
"ConformSchema" | "SchemaErrors" => {
    vec![system_rule_option("AllowExtraKeys", bool_option(false))]
}
```

(Place them tidily among the existing arms; grouping/order is cosmetic.)

## Do NOT wire the table into behavior

This is discovery ONLY. Do NOT make any implementation read
`default_system_options` / `options_for_symbol` at call time, and do NOT
try to make `SetOptions[ToJson, "Indent" -> 4]` change `ToJson`'s output.
That table-vs-call-path split is the KNOWN, pre-existing rum limitation
(unit-2 note) shared by every registered option (`Cases` `Heads`,
`StringCases` `Overlaps`, …); closing it is a separate design question,
explicitly out of scope here. Registering the defaults only makes
`Options`/`OptionValue` report them.

## Tests — `crates/rum-runtime/src/runtime/tests/options_and_protection.rs`

Add one test mirroring the existing `Options[...]` assertions in this file
(`session.eval_text("Options[SYMBOL]").unwrap().value.to_string()`): assert
`Options[ToJson]`, `Options[ToXml]`, `Options[FromCsv]`, `Options[ToCsv]`,
`Options[ConformSchema]`, `Options[SchemaErrors]` each report the
registered rules (write the expected string from the actual output), and
spot-check one `OptionValue[FromCsv, "Delimiter"] == ","`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval 'Options /@ {ToJson, ToXml, FromCsv, ToCsv, ConformSchema, SchemaErrors}'
printf '' | cargo run -q -p rum-cli -- --eval '{FromCsv["a;b\n1;2", "Delimiter" -> ";"], ToJson[<|"a"->1|>, "Indent" -> 2], ConformSchema[<|"port"->1,"x"->2|>, <|"port"->_Integer|>, "AllowExtraKeys" -> True]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — one option test
added). First probe: each head reports its registered option rules (no
longer `{}`). Second probe: option-passing behavior BYTE-IDENTICAL to
today (the implementations still read options from call args — registration
changed nothing on the call path).

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/options/defaults.rs` (the six
  arms) and `crates/rum-runtime/src/runtime/tests/options_and_protection.rs`
  (the assertion test).
- MUST NOT touch: any `eval_*` / option-parsing implementation; the
  call path; `builtins.md` (Options are not in the reference — no regen);
  any other builtin; `TODO.md`, the tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- Registration is DISCOVERY ONLY — the registered default MUST equal the
  implementation's real default (verified list above); do NOT wire the
  table into any call path.
- No reference regen (Options are not listed in `builtins.md`); if a diff
  appears there, you touched the wrong thing — STOP.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the six option arms + the test; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
