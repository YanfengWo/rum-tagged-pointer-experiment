# Brief 239 — rum-repl dogfood port + contract inspection surface (lower_dump, message_catalog, terminal twins)

Implementation brief — embedding API line, item a2c (see
`docs/dev/embedding-api-line.md`; dogfood ruling). Port `rum-repl` onto
`HostSession`, add the two contract methods the user ruled IN
(2026-07-16: `:lower` → contract as diagnostic-grade `lower_dump`;
`:messages` → contract as `message_catalog`), give `rum-terminal`
contract-type render twins, and unify `rum-cli`'s dual session
construction. All REPL-visible output stays byte-identical — the
existing `rum-repl` tests pin it and their expectations are FROZEN
(mechanical construction-migration edits are fine; expected strings are
not).

## Pre-diagnosed facts (verified this session)

- REPL command surface (`rum-repl/src/commands.rs`): `:parse`/`:format`
  use `parse_request` only for syntax (no session state needed —
  `rum_syntax::parse_with_diagnostics` + `debug_dump`/
  `format_parsed_source`, the brief-238 CLI precedent); `:complete` →
  contract `complete` exists; `:inspect` → contract `inspect_symbol`
  exists; `:lower` → `lower_request` (returns
  `SessionLowerResponse { expr }`, rendered `expr.to_string()`,
  `session_requests.rs:95`); `:messages` → `inspect_message_catalog`
  (`session_eval.rs:122`, returns `Option<Vec<MessageCatalogEntry>>`,
  `None` = invalid selector).
- `MessageCatalogEntry` (`runtime/model/definitions.rs:37`) maps
  field-for-field onto the EXISTING `HostMessageTemplate`
  (`rum-host/src/model/inspection.rs:79`: message_name /
  qualified_symbol_name / tag / template(rendered) / delayed) — reuse
  it; do NOT invent a new type. Follow however the inspect path renders
  `template: Expr` to `String` today.
- `HostSymbolInfo` (`inspection.rs:22`) carries 11 of the 13 fields
  `rum-terminal`'s `render_inspect_response` reads; the two absent ones
  are `attribute_count`/`message_count`. Check how
  `SessionInspectResponse` computes them: if they equal
  `attributes.len()`/`message_tags.len()` (no truncation), derive them
  in the twin renderer; if they can differ, add the two counts to
  `HostSymbolInfo` (MECHANICAL: list every construction site you touch).
- REPL main loop (`rum-repl/src/lib.rs:116-130`) uses `parse_request`
  for the is-incomplete continuation decision
  (`request_failure_is_incomplete`, `:280`); the contract has
  `is_complete` (`rum-host/src/session.rs:422`) — read its actual
  signature/semantics and use it. Syntax-failure rendering has
  `HostFailure.syntax_rendered` since a2b.
- `rum-cli` keeps a raw-`Session` constructor only for the REPL branch
  (`build_repl_session`, marked a2c) — this brief removes it.

## Required change

### Part A — contract methods (`crates/rum-host`)

1. **`HostSession::lower_dump(&mut self, source: &str) -> Result<String, HostFailure>`**
   — guarded (poisoned precheck + `guard`, like `parse`), forwards to
   `lower_request`, returns the lowered expression's canonical
   rendering (`response.expr.to_string()`), maps failures through
   `host_failure_from_runtime`. Rustdoc MUST state: diagnostic-grade
   surface for developer tooling; the rendered format is NOT a
   stability promise; lowering resolves against the session's current
   context state (user ruling 2026-07-16).
2. **`HostSession::message_catalog(&self, query: Option<&str>) -> Option<Vec<HostMessageTemplate>>`**
   — plain forward like `definitions()` (read-only), mapping
   `MessageCatalogEntry` → `HostMessageTemplate`; `None` preserves the
   invalid-selector semantics. Rustdoc: global/selector message-catalog
   query; the per-symbol view remains on
   `HostInspectResponse.message_catalog`.
3. Tests (`rum-host/src/tests.rs`): `lower_dump` returns non-empty
   canonical text for a simple form and a syntax failure carries
   `syntax_rendered`; `message_catalog(None)` lists a user-defined
   message, a symbol query narrows to it, an invalid selector returns
   `None`.

### Part B — terminal twins (`crates/rum-terminal`)

Add `rum-host` to its dependencies. Add public renderers taking
contract types, byte-identical to the existing runtime-typed ones
(the frozen REPL tests are the oracle):

- `render_host_message`/`render_host_messages` (`! {symbol}::{tag}: {text}`
  — `rum-cli` has private copies from a2b; move/share so CLI and REPL
  both use these),
- `render_host_failure` (messages block, then `syntax_rendered` for
  syntax failures / `! {message}` otherwise — the `render_request_failure`
  composition),
- `render_host_eval_response` (messages block + `value.text`),
- `render_host_inspect` (twin of `render_inspect_response`),
- `render_host_message_catalog` (twin of
  `render_message_catalog_entries`).

Then grep for remaining consumers of the OLD runtime-typed renderers;
after the REPL port they should have none outside `rum-terminal`'s own
tests — delete the dead ones (workspace-internal crate, no semver
burden) and migrate their tests to the twins. If a renderer still has a
live consumer, keep it and say so in the report.

### Part C — the REPL port (`crates/rum-repl`, + CLI unification)

1. `Repl::run` takes `&mut HostSession`. Main loop: continuation
   decision via `is_complete`; evaluation via `eval`; rendering via the
   Part B twins. Preserve the exact current behaviors pinned by tests:
   incomplete input accumulates, non-incomplete syntax failure renders
   and records history, EOF flush evaluates pending input.
2. `commands.rs`: `:parse`/`:format` go straight to `rum-syntax` (no
   session argument); `:complete` → `complete`; `:inspect` →
   `inspect_symbol` + `render_host_inspect`; `:lower` → `lower_dump`;
   `:messages` → `message_catalog` + `render_host_message_catalog`.
   All rendered output byte-identical (frozen tests).
3. `rum-repl/Cargo.toml`: depend on `rum-host` (drop `rum-runtime` if
   nothing still needs it — expected after the port).
4. `rum-cli/src/main.rs`: delete `build_repl_session`; the REPL branch
   uses `build_host_session()` — the dual construction and the a2c
   marker comment go away. Drop the `rum_runtime::Session` import; drop
   the `rum-runtime` dependency from `rum-cli/Cargo.toml` ONLY if
   nothing else in the crate needs it (check `alloc-count` and tests
   first; report either way). The 27 CLI test expectations stay frozen.
5. Migrate `rum-repl` test constructions (`Session::new()` →
   `HostSession` with core builtins) mechanically; every expected
   string stays frozen.

## Scope

MAY modify:
- `crates/rum-host/src/session.rs`
- `crates/rum-host/src/model/inspection.rs` (only if the two count
  fields prove necessary; MECHANICAL construction-site list)
- `crates/rum-host/src/model/mod.rs`, `crates/rum-host/src/lib.rs`
  (re-exports/docs)
- `crates/rum-host/src/tests.rs`
- `crates/rum-terminal/src/lib.rs`, `crates/rum-terminal/Cargo.toml`
- `crates/rum-repl/src/lib.rs`, `crates/rum-repl/src/commands.rs`,
  `crates/rum-repl/src/tests.rs`, `crates/rum-repl/Cargo.toml`
- `crates/rum-cli/src/main.rs`, `crates/rum-cli/Cargo.toml`
- `docs/guide/embedding.md` (lower_dump + message_catalog notes)

MUST NOT touch: anything in `rum-runtime`/`rum-core`/`rum-syntax`/
`rum-stdlib`, `rum-jupyter`, `HostFailureCode`/`HostTyped` definitions,
CLI test expectations, REPL test expected strings.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
git status --short            # only MAY-listed files
```

- **No microbench A/B**: the REPL is not the perf vehicle and the
  `--eval`/script paths of `rum-cli` are untouched by this brief (only
  the REPL-branch construction changes); state this in the report. If
  you find yourself touching any code on the `--eval` path, STOP.

## Hard rules

- Do NOT `git commit`, branch, or push; leave changes in the working
  tree. No network, no new dependencies beyond the listed intra-
  workspace ones, no new `unsafe`.
- Output bytes are the contract: any REPL or CLI test expectation that
  needs changing means STOP and report (the only allowed test edits are
  construction/type migrations and NEW tests).
- Stop-loss: if a twin renderer cannot reproduce an output byte-for-byte
  from contract data (missing field beyond the two known counts), STOP
  and report the missing datum.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Probed facts: `attribute_count`/`message_count` derivation verdict
   (len-equal or added fields + construction list); `is_complete`
   semantics as used; dead-renderer deletion list with the consumer
   grep evidence; rum-cli `rum-runtime` dependency verdict.
3. Test survey — each new test/assertion, named; confirmation that all
   frozen expectations are unchanged.
4. Verification — commands with verbatim output (test totals).
5. The no-A/B justification (one sentence).
6. Deviations / open questions (empty if none).
