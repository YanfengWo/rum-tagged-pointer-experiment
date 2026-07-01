# Brief 064 — Whole-session introspection: enumerate user definitions + contexts

## Goal

`HostSession::inspect` is per-symbol — you must already know the name. A host (a REPL's
"defined symbols" panel, a debugger, the enabler for a session snapshot) needs to
ENUMERATE what a session contains without naming each symbol. When you are done, the host
can list every symbol that carries a definition (own / delayed-own / down / up / sub
values), context-qualified and with the per-kind counts, plus the session's contexts —
via a native structured API (not by evaluating `Names[...]` and re-parsing). Native
builtins are NOT definitions and are correctly excluded (they live in the builtins
registry, not the `definitions` map).

This is the Tier-2 "whole-session introspection / enumeration" item
(`docs/host-embedding-and-hardening.md`).

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-2 ("Whole-session
  introspection / enumeration … no host-level 'enumerate all user definitions / contexts'").
- The definition source: `Session.definitions: DefinitionMap` (model.rs, `pub(super)`).
  It iterates `(symbol, &SymbolDefinition)` (see the pattern in the memory proxy:
  `self.definitions.iter().map(|(symbol, definition)| …)`). `SymbolDefinition` (model.rs)
  = `{ own_value: Option<Expr>, delayed_own_value: Option<Expr>, up_values: Vec<UpValue>,
  down_values: Vec<DownValue>, sub_values: Vec<SubValue> }`. A symbol is in this map iff a
  WL-level `Set`/`SetDelayed`/`TagSet`/`UpSet`/… gave it a definition — i.e. user work +
  any loaded-package definitions. Native builtins are NOT here.
- `Symbol::fully_qualified_name()` gives the context-qualified name (used in names.rs).
- Contexts: `namespace_tools/names.rs` has private `visible_contexts(&self) -> BTreeSet<&str>`
  (:224) and `enumerated_names_symbols` (:192). For the context list, prefer the distinct
  contexts actually present (the current context + context_path + the contexts of defined
  symbols) — reuse `visible_contexts` if it fits, else derive from the defined symbols'
  contexts. Keep it simple and correct.
- Per-symbol inspect already exists: `session_requests.rs:268 inspect_request(name)` — this
  brief is the ENUMERATION companion, not a replacement.

## Design (do this; deviate only with a reported reason)

1. Runtime (`session_api.rs` + a public summary type in model.rs or session_state.rs):
   - A public struct `SessionDefinitionSummary { pub name: String, pub has_own_value: bool,
     pub down_value_count: usize, pub up_value_count: usize, pub sub_value_count: usize }`
     (derive `Clone, Debug, PartialEq, Eq`). `has_own_value` = `own_value.is_some() ||
     delayed_own_value.is_some()`.
   - `pub fn definition_summaries(&self) -> Vec<SessionDefinitionSummary>` — iterate
     `self.definitions`, one summary per symbol, `name = symbol.fully_qualified_name()`.
     Sort by name for a deterministic result.
   - `pub fn context_names(&self) -> Vec<String>` — the distinct contexts present, sorted,
     deterministic.
2. Host (`rum-host`): mirror with host-owned types + methods:
   - `HostDefinitionSummary { name, has_own_value, down_value_count, up_value_count,
     sub_value_count }` (a host type, consistent with the existing host/runtime split), a
     `From<SessionDefinitionSummary>` mapping.
   - `HostSession::definitions(&self) -> Vec<HostDefinitionSummary>` and
     `HostSession::contexts(&self) -> Vec<String>`. These are read-only (`&self`); if the
     brief-060 poison model forces `&mut self` for consistency, keep them `&self` (pure
     reads, no eval, no poison risk) — a poisoned session may still be introspected.
   - Export the new host type from `lib.rs`.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/session_api.rs` (the two pub methods),
  `crates/rum-runtime/src/runtime/model.rs` OR `session_state.rs` (the public
  `SessionDefinitionSummary` type), and — only if needed to name the summary type — the
  runtime re-export; `crates/rum-host/src/session.rs`, `crates/rum-host/src/model/*.rs`,
  `crates/rum-host/src/lib.rs` (host type + methods + export), `crates/rum-host/src/tests.rs`.
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core/src/expr.rs`
  `Expr`/`Symbol`/serde shape (you MAY call `Symbol::fully_qualified_name()`, not change
  it); the `definitions` map's shape; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except
  your own report; `.git`, `Cargo.lock`.
- **No new `unsafe`.** Not a hot path — enumeration is a host-invoked read, never per eval
  step.

## Deliverable

- Runtime `definition_summaries()` + `context_names()` + the public summary type; host
  `definitions()` + `contexts()` + `HostDefinitionSummary`.
- Tests (rum-host):
  - fresh session: `definitions()` does NOT list core builtins (assert a known builtin like
    `Plus` is absent — it is native, not a definition).
  - after `eval("a = 1; f[x_] := x; f[2] := 9")`: `definitions()` lists `Global\`a` with
    `has_own_value = true`, and `Global\`f` with `down_value_count = 2` (the `f[x_]:=` and
    `f[2]:=` rules), `has_own_value = false`. Deterministic order.
  - `contexts()` includes `Global\`` (and `System\`` per the context path); deterministic.
  - a poisoned session (brief-060 test hook) can still be introspected (`definitions()`
    does not error / does not require un-poisoning).

## rum / Rust pitfalls (read twice)

1. Only symbols in the `definitions` map are enumerated — native builtins are excluded by
   construction (they are not definitions). State this in the summary's doc.
2. Deterministic output: sort by name (a `BTreeMap`/`BTreeSet` or an explicit sort) — tests
   and hosts want stable ordering.
3. `has_own_value` must cover BOTH `own_value` and `delayed_own_value` (`=` and `:=`).
4. Keep `definitions()`/`contexts()` `&self` reads — no eval, no poison gating.
5. Not a hot path; no microbench. Minimal diff.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```
Paste the workspace summary AND the new introspection test output (the definitions list
with counts + the contexts). No microbench (not a hot path) — state that.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes for Claude's gate.
- No new `unsafe`. Never edit any `mma-hol` file. Minimal diff.
- Do NOT change the `Expr`/`Symbol`/serde shape or the `definitions` map shape — only READ
  them. If you think a shape change is needed, STOP and report.
- Stop-loss: same failure twice → STOP and report.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim workspace summary + the introspection test output.
3. Finding — the host can enumerate definitions (with per-kind counts) + contexts; builtins
   excluded; poisoned session still introspectable.
4. Stop-loss / open questions (empty if none).
