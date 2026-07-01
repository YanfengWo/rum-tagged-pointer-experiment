# Brief 063 — Host session reset / clear (multi-client reuse + poison recovery)

## Goal

A long-running multi-client host needs to return a `HostSession` to a clean state to reuse
it for the next client (instead of building a brand-new one), and — importantly — it is the
RECOVERY path for a session POISONED by a caught panic (brief-060): today a poisoned
session is permanently dead. When you are done, `HostSession::reset()` rebuilds the inner
session to its original post-construction state (the same config it was built with, core
builtins reinstalled, ALL user definitions / attributes / contexts / messages / gensym
state gone) and clears the poisoned flag — so a poisoned session becomes usable again, and
a clean session is wiped for the next client. Host layer only — zero runtime/core change,
not a hot path.

This is the Tier-2 "reset / clear session" item (`docs/host-embedding-and-hardening.md`).

## Context pointers

- Read `AGENTS.md`, `docs/host-embedding-and-hardening.md` §2 Tier-2 ("reset / clear
  session to a clean state").
- `crates/rum-host/src/session.rs`: `HostSessionBuilder::build()` (:45) is the construction
  to reuse — `Session::with_config(self.config.runtime_config())` then `rum_stdlib::install_core(&mut session)`
  if `install_core_builtins`. `HostSession { session, install_core_builtins, poisoned }`
  (the struct, ~:68 after brief-060). `poisoned` is set by the brief-060 `guard`.
- `crates/rum-host/src/config.rs`: `HostConfig` + `runtime_config()` (:118) is the full
  config needed to rebuild faithfully (`current_context`, `context_path`,
  `package_search_paths`, `resource_limits`, `tracing_enabled`, `asserts_enabled`,
  `install_core_builtins`). `HostConfig` is `Clone`.
- `Session::with_config` is the existing fresh-session constructor (rum-runtime) — reset
  reuses it; do NOT add a runtime-side reset.

## Design (do this; deviate only with a reported reason)

1. Store the ORIGINAL `HostConfig` on `HostSession` (add a `config: HostConfig` field; the
   builder fills it from `self.config`). `install_core_builtins` is already in the config,
   so the separate bool field can stay or be derived from `config.install_core_builtins` —
   keep the diff minimal, but the rebuild MUST use the stored ORIGINAL config (NOT
   `self.session.configuration()`, which reflects the CURRENT mutated state — wrong for a
   clean reset).
2. Factor the construction into a private helper used by BOTH `build()` and `reset()`, e.g.
   `fn build_inner_session(config: &HostConfig) -> Session` (with_config + install_core if
   configured). `build()` calls it; `reset()` does `self.session = Self::build_inner_session(&self.config);
   self.poisoned = false;`.
3. `pub fn reset(&mut self)` — note it MUST work even when `self.poisoned` (do NOT gate
   reset behind the poisoned check the other methods use; reset is the recovery). After
   reset the session is a blank slate with the original config.
4. DOCUMENT the semantics in a doc-comment on `reset`: it drops ALL user state (defs,
   attributes, contexts, loaded packages, messages, gensyms) back to construction; it
   INVALIDATES any previously-handed-out `cancel_handle()` (the rebuilt session has a new
   interrupt flag — obtain a fresh handle after reset); it clears the poisoned flag.

## Scope

- MAY modify: `crates/rum-host/src/session.rs` (the struct + builder + reset + helper),
  `crates/rum-host/src/tests.rs` (tests). Touch nothing else unless the compile forces a
  trivial follow-on (report it).
- MUST NOT touch: any `mma-hol` clone/standard; `crates/rum-core` / `crates/rum-runtime`
  SOURCE (reset reuses the existing `Session::with_config`; if you think a runtime change
  is needed, STOP and report); `Expr`/serde shape; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except your own report; `.git`, `Cargo.lock`.
- **No new `unsafe`.** Not a hot path.

## Deliverable

- `HostSession::reset()` + the stored original config + the shared construction helper.
- Tests (rum-host):
  - **clears user state**: `eval("x = 41; x")` → 41; `reset()`; `eval("x")` → `x` (the
    symbol, undefined again, NOT 41); a fresh `eval("2 + 2")` → 4 works.
  - **poison recovery (the key one)**: trigger a panic (reuse the brief-060 `#[cfg(test)]`
    panic hook) so the session is poisoned; assert a normal `eval` now returns the
    `Poisoned` outcome; `reset()`; assert `eval("1 + 1")` → 2 succeeds and the session is no
    longer poisoned.
  - (optional) config preserved: if the session was built with a non-default
    `current_context`/limits, assert reset restores that original config, not a mutated one.

## rum / Rust pitfalls (read twice)

1. Rebuild from the STORED ORIGINAL config, never from `self.session.configuration()`
   (that is the current, possibly-mutated state).
2. `reset()` must NOT be gated by `self.poisoned` — it is the recovery path; it runs
   regardless and clears the flag.
3. Not a hot path; no microbench. But do NOT call reset anywhere per eval step.
4. Minimal diff; reuse the builder's construction, don't duplicate the install-core logic
   in two places that can drift — factor the shared helper.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```
Paste the workspace summary AND the new reset tests' output (clear-user-state +
poison-recovery). No microbench (not a hot path) — state that.

## Hard rules

- Do NOT `git commit`/branch/push. Leave changes for Claude's gate.
- Do NOT modify `rum-core`/`rum-runtime` source — host layer only. If you think you must,
  STOP and report.
- No new `unsafe`. Never edit any `mma-hol` file. Minimal diff.
- Stop-loss: same failure twice → STOP and report.
- Paste REAL verbatim output.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — verbatim workspace summary + the reset/poison-recovery test output.
3. Finding — reset returns the session to the original clean state AND recovers a poisoned
   session.
4. Stop-loss / open questions (empty if none).
