# Brief 155 — Schema validation: patterns ARE the type language

Implementation brief (B1). Declarative validation for nested Association
configs — the natural companion of the codec family (`FromYaml` → validate
→ use). The rum-native design decision, made in advance: **no parallel type
DSL — rum's existing pattern language is the spec language.** A schema is
symbolic data; leaf specs are ordinary patterns evaluated through the
existing matcher (so `_Integer`, `_String?StringQ`, `x_Integer /; x > 0`,
`"a" | "b"` all work for free).

## Builtins (stdlib, Pure)

### `SchemaErrors[value, schema]` → `List` of error Associations

Each error: `<|"Path" -> "db.port", "Message" -> "expected to match
_Integer, got \"x\""|>` (path segments joined with `.`; list indices as
`[3]`, e.g. `"servers[2].host"`). Empty list = valid. Collects ALL errors
in one pass (path-ordered), capped at 64 with a final
`"Path" -> "", "Message" -> "… and N more errors"` entry.

### `ConformSchema[value, schema]` → normalized value | `$Failed`

On success returns the value with defaults filled (see `Optional` below),
key order = schema order for filled keys, original order otherwise. On any
error: one runtime message per error (same cap), `ConformSchema::schema`
tag, message text = `Path: Message`, then `$Failed`.

## Schema shapes (the whole contract — fail-closed outside it)

- **Association schema**: `<|"key" -> spec, …|>` — value must be an
  Association; each present key's value validated against its spec;
  MISSING keys error unless their spec is `Optional[...]`; EXTRA keys
  error (strict by default; `SchemaErrors[value, schema,
  "AllowExtraKeys" -> True]` / same option on ConformSchema relaxes —
  extras pass through unvalidated).
- **`Optional[spec]`**: key may be absent (absent → simply omitted).
  **`Optional[spec, default]`**: absent → `ConformSchema` fills `default`
  (the default itself must conform to `spec` — validated at use, error at
  the key's path if not).
- **List schema**: `{spec}` (exactly one element) — value must be a List,
  every element validated against `spec` (paths `key[1]`, `key[2]`, …).
- **Nested**: Association schemas and list schemas nest arbitrarily.
- **Leaf spec**: anything else is a PATTERN, tested with the matcher
  semantics of `MatchQ` (held appropriately — the schema is data; patterns
  inside must not evaluate prematurely; decide and document how the
  builtins hold their schema argument).
- Malformed schema (e.g. `{a, b}` two-element list schema, non-String
  Association keys) → fail-closed `::schema` message naming the SCHEMA
  path, `$Failed` from both builtins.

## Tests

- The brief-135 task-4 config end-to-end: nested Association, required
  keys, type predicates, ranges via `/;` conditions, defaults filled —
  asserting the exact normalized value AND the exact error list for an
  invalid variant (paths like `db.port`, `servers[2].host`).
- Pattern-leaf breadth: `_Integer`, `_String`, alternatives, PatternTest,
  Condition, literal match; held-schema semantics (a pattern with `/;`
  does not evaluate at schema-construction time).
- Optional with/without default; default-violates-spec error; extra-key
  strict vs allowed; list schema incl. empty list, nested
  list-of-associations; error cap behavior (70 errors → 64 + summary).
- Codec integration: `ConformSchema[FromYaml[…], schema]` end-to-end and
  `FromJson` variant; one `ToJson[SchemaErrors[…]]` probe (errors are
  plain data — they serialize).
- Vocabulary/catalog registrations + reference drift (expect 598).

## Scope

- MAY create: `crates/rum-stdlib/src/schema.rs` (+ submodules),
  `crates/rum-stdlib/src/catalog/schema.rs`, tests.
- MAY modify: catalog mod/install path, `stdlib_attributes.rs`, docgen if
  needed, `docs/reference/` via generator,
  `docs/guide/structured-data.md` (a "Validating Configuration" section
  with real transcripts, placed after the codec sections).
- MUST NOT touch: the matcher (CALL it, do not change it),
  evaluator/dispatch, other docs, `Cargo.*`.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
cargo run -q -p rum-stdlib --bin generate-builtin-reference && cargo test -p rum-stdlib tests::builtin_reference_docgen::builtin_reference_is_in_sync -- --exact
rg -c '^### ' docs/reference/builtins.md      # expect 598
printf '' | cargo run -q -p rum-cli -- --eval 'ConformSchema[<|"name" -> "web", "port" -> 8080|>, <|"name" -> _String, "port" -> x_Integer /; x > 0, "debug" -> Optional[True | False, False]|>]'
printf '' | cargo run -q -p rum-cli -- --eval 'SchemaErrors[<|"port" -> "x", "extra" -> 1|>, <|"name" -> _String, "port" -> _Integer|>]'
printf 'db:\n  host: h\n  port: 5432\n' | cargo run -q -p rum-cli -- --eval 'ConformSchema[FromYaml[$ScriptInput], <|"db" -> <|"host" -> _String, "port" -> _Integer|>|>]'
printf '' | cargo run -q -p rum-cli -- --eval 'ConformSchema[<|"a" -> 1|>, <|"a" -> {_Integer, _Integer}|>]'   # malformed schema, fail-closed
git status --short
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. No network, no
  new dependencies, no new `unsafe`.
- The pattern language is reused, never reimplemented — leaf validation
  goes through the existing public matcher entry points.
- Fail-closed on malformed schemas; never guess.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format (your final message)

1. The contract as landed (incl. the hold semantics decision) + anything
   beyond this brief.
2. What changed — per file.
3. Verification — all commands verbatim.
4. Stop-loss / open questions (empty if none).
