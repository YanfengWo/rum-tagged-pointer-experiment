# Brief 065 — Session snapshot / restore to-and-from an in-memory buffer

## Goal

`DumpSave`/`Save`/`Get` are WL-level AND file-based — a host cannot snapshot a session to,
or restore it from, an in-memory buffer (needed for multi-client session persistence AND
as the enabler for any no-fs host, e.g. WASM). When you are done, `HostSession` can
serialize its user definitions (+ attributes) to `Vec<u8>` and restore a session from
those bytes, with NO filesystem involved — round-tripped and tested. This reuses rum's
existing dump RENDERING (`render_saved_symbols`), which already produces the WL-source text
separately from the file write; we just render to a buffer and restore by evaluating it.

This is the Tier-2 "session snapshot / `restore_from_bytes`" item
(`docs/host-embedding-and-hardening.md`).

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-2 ("Session snapshot /
  serialize as host ops … a `restore_from_bytes(...)` … also the enabler for any no-fs
  host (WASM)").
- The reusable renderer: `crates/rum-runtime/src/runtime/package_tools.rs:335`
  `fn render_saved_symbols(&self, symbols: &[Symbol], textual_dump: bool, include_attributes: bool) -> String`
  — this is `&self`, pure, and returns the WL-source dump (Set/SetDelayed + SetAttributes);
  `eval_persist_package_state` calls it and THEN `write_saved_content` writes the file. We
  reuse the render, skip the file. Call it with `textual_dump = true, include_attributes = true`.
- "All user symbols" = `self.definitions.keys()` (the `definitions: DefinitionMap`,
  model.rs:355) — exactly the WL-level Set/SetDelayed/... definitions (native builtins are
  not here). brief-064 just exposed `definition_summaries()` over the same map; the snapshot
  is the serialization companion, and the round-trip test can VERIFY via `definitions()`.
- Restore is just evaluation: `HostSession::eval` already evaluates WL source. Restoring an
  image = reset to a clean session (brief-063 `reset()`), then eval the snapshot source.
- KNOWN existing limitation (already in TODO): rum's `DumpSave` is TEXT (verbose WL source),
  NOT a binary `.mx` — that is a separate deferred perf item; this brief uses the text form
  (correct, just larger). Do NOT implement binary serialization here.

## Design (do this; deviate only with a reported reason)

1. Runtime (`session_api.rs`): `pub fn snapshot_definitions_source(&self) -> String` —
   collect `self.definitions.keys().cloned()` (SORT them for a deterministic snapshot),
   then return `self.render_saved_symbols(&symbols, true, true)`. (`render_saved_symbols`
   is a private method on the same `impl Session` neighborhood — the new pub method calls
   it directly; do NOT change its signature.)
2. Host (`rum-host/src/session.rs`):
   - `pub fn snapshot_to_bytes(&self) -> Vec<u8>` — `self.session.snapshot_definitions_source().into_bytes()`.
     `&self`, pure (safe on a poisoned session — it just reads).
   - `pub fn restore_from_bytes(&mut self, bytes: &[u8]) -> Result<(), HostFailure>` —
     decode UTF-8 (a decode error → a `HostFailure` with `HostFailureCode::Other`/a new
     `Snapshot` code — your call, but be explicit), then RESET the session to a clean state
     (reuse brief-063 `reset()` semantics so restore is a faithful image load, not a merge
     onto existing state), then evaluate the source. If evaluation surfaces a failure,
     return it as a `HostFailure`. Document that restore REPLACES the session's user state.
3. DOCUMENT the snapshot's scope in doc-comments: it captures WL-level DEFINITIONS
   (own/delayed-own/down/up/sub values) + ATTRIBUTES of the defined symbols. It does NOT
   capture attribute-only symbols, `$`-variables / options / messages / the gensym counter,
   or contexts beyond what the source re-establishes. It is a TEXT image (not binary `.mx`).

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/session_api.rs` (the pub snapshot method);
  `crates/rum-host/src/session.rs` (snapshot/restore), `crates/rum-host/src/model/*.rs` +
  `lib.rs` ONLY if you add a `HostFailureCode::Snapshot` (+ the jupyter render mapping if a
  new code ripples); `crates/rum-host/src/tests.rs`.
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core/src/expr.rs`
  `Expr`/`Symbol`/serde shape; the `definitions` map shape; `render_saved_symbols`'s
  signature; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except your own report; `.git`,
  `Cargo.lock`.
- **No new `unsafe`.** Not a hot path — snapshot/restore are host-invoked, never per eval
  step.

## Deliverable

- Runtime `snapshot_definitions_source()`; host `snapshot_to_bytes()` + `restore_from_bytes()`.
- Round-trip test (rum-host), composing with brief-064's `definitions()`:
  - Session A: `eval("a = 1; f[x_] := x; g[2] := 20; SetAttributes[h, Orderless]")`.
  - `let bytes = a.snapshot_to_bytes();`
  - Session B (fresh): `b.restore_from_bytes(&bytes).unwrap();`
  - Assert `b.definitions()` == `a.definitions()` (Global`a own, Global`f down=1, Global`g
    down=1), `b.eval("f[7]")` → 7, `b.eval("Attributes[h]")` includes `Orderless`.
  - Restore-replaces test: define `z = 99` in B first, then restore the bytes (which don't
    contain z) → assert `z` is gone after restore (image load, not merge).
  - No filesystem is touched by either op (they are pure buffer/eval).

## rum / Rust pitfalls (read twice)

1. Reuse `render_saved_symbols(.., true, true)` — do NOT write a new serializer, and do NOT
   change its signature. Text form (correct); binary `.mx` is a separate deferred item.
2. Deterministic snapshot: sort the symbol list before rendering.
3. `restore_from_bytes` RESETS then applies (faithful image load), reusing `reset()`.
   Document that it replaces user state.
4. Not a hot path; no microbench. No fs in the snapshot/restore path (that is the WASM
   point) — snapshot returns bytes, restore takes bytes + eval; neither opens a file.
5. Minimal diff.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```
Paste the workspace summary AND the snapshot round-trip + restore-replaces test output. No
microbench (not a hot path) — state that, and state that no filesystem is touched.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes for Claude's gate.
- No new `unsafe`. Never edit any `mma-hol` file. Minimal diff.
- Do NOT change `Expr`/`Symbol`/serde, the definitions-map shape, or `render_saved_symbols`.
- Do NOT implement binary `.mx` — reuse the existing text render.
- Stop-loss: same failure twice → STOP and report.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim workspace summary + the round-trip + restore-replaces test output.
3. Finding — a session round-trips through an in-memory buffer with no fs; restore replaces
   user state; the captured-scope limitation is documented.
4. Stop-loss / open questions (empty if none).
