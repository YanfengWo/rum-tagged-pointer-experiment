# Brief 100 — make registration the ONLY builtin dispatch path (kill the static fallback)

## Goal

Stage-2 step 2 (`docs/crate-rebalance-plan.md`). Today builtin dispatch does NOT
require registration: `resolution.rs` falls back to a static
`OnceLock<HashMap>` built from `core_builtin_specs()`
(`builtin_catalog.rs:66 default_builtin_dispatch_spec_for_symbol`), so
`rum-stdlib::install_core`'s `register_builtin` loop is effectively decorative.
Builtin families cannot move out of `rum-runtime` while the kernel statically
consults its own catalog. After this brief: the session's registered spec map is
the ONLY dispatch source; the transitional bridge is that session construction
auto-installs `core_builtin_specs()` (so all behavior is unchanged); the static
fallback in resolution is DELETED. Zero behavior change; hot-path adjacent →
microbench A/B gate applies.

## Context pointers

- Read `AGENTS.md` and `docs/crate-rebalance-plan.md` first. Briefs 097/099
  landed: dispatch is fn-pointer open; `kernel_api` exists.
- Key sites (verified 2026-07-03):
  - `crates/rum-runtime/src/runtime/resolution.rs` ~lines 80-95:
    `builtin_dispatch_spec_for_symbol` consults the session's registered map,
    then falls back to `default_builtin_dispatch_spec_for_symbol` (the static
    catalog) — including a `with_fallback_dispatch` merge path. The fallback
    calls are what this brief removes.
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs`:
    `core_builtin_specs()` (stays `pub` — rum-stdlib uses it),
    `default_builtin_dispatch_spec(name)` + `default_builtin_dispatch_spec_for_symbol`
    (callers must migrate to session state; delete or reduce these statics as
    call sites allow).
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs:237` — enumerates
    `core_builtin_specs()` for name listing; must read the SESSION's registered
    specs instead (`self.builtins` map / an accessor), not the static catalog.
  - `crates/rum-runtime/src/runtime/capabilities.rs` tests use
    `default_builtin_dispatch_spec` — point them at a registered session or the
    catalog directly (test-only; keep them meaningful: they assert the CATALOG's
    capability data, so reading `core_builtin_specs()` directly in the test is
    fine).
  - `crates/rum-runtime/src/runtime/session_api.rs::register_builtin` — the real
    path; note it clears `builtin_dispatch_spec_cache` and bumps the attribute
    revision/epoch per call (fine at construction on an empty session; do NOT
    add a bulk-registration fast path unless a test forces you to).
  - Session construction: find the single constructor flow all sessions share
    (`Session::new` / `Default` / snapshot-restore) and auto-install
    `core_builtin_specs()` there exactly once. Snapshot restore
    (`restore_from_bytes`) must end with the same registered set — verify by
    test, and check whether restore rebuilds via a constructor that already
    auto-installs (do not double-register; `register_builtin` is idempotent by
    key, but assert the count once in a test rather than assuming).
- Attribute wiring: today a builtin's ATTRIBUTES flow from the spec at
  registration (`register_builtin` inserts into `self.attributes`) AND the
  static fallback path serves attributes/dispatch for unregistered sessions via
  the spec cache. With auto-install, attributes come from registration alone —
  any place that consulted the static spec for attributes must see identical
  values. `cargo test --workspace` is the referee.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/resolution.rs`
  - `crates/rum-runtime/src/runtime/builtin_catalog.rs`
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs`
  - `crates/rum-runtime/src/runtime/capabilities.rs` (tests only)
  - `crates/rum-runtime/src/runtime/model.rs` /
    `crates/rum-runtime/src/runtime/session_api.rs` (constructor auto-install +
    an accessor for registered specs if names.rs needs one)
  - `crates/rum-runtime/src/runtime/session_state.rs` ONLY if the compiler
    forces a carriage change (report it).
  - Test files under `crates/rum-runtime/**` that assert the old
    fallback behavior (list each in the report).
- MUST NOT touch: builtin family files, `kernel_api.rs`, the catalog DATA
  (specs themselves), other crates (`rum-stdlib`'s `install_core` keeps working
  unchanged — it becomes a harmless re-registration), any `mma-hol` clone,
  `docs/**` except your report, `.git`, `Cargo.toml`/`Cargo.lock`.
- No default-path cost: the dispatch-spec CACHE in front of resolution stays;
  do not add per-call work. Construction-time cost (~513 inserts) is fine.

## Deliverable

- Registration-only dispatch in the working tree (no commit).
- A test (or existing test evidence) that a fresh `Session` evaluates a builtin
  (proving auto-install) and that snapshot restore preserves the registered set.
- Verbatim gates below, including the microbench A/B.

## rum / Rust pitfalls (read twice)

1. **Zero behavior change.** Any workspace-test difference is a defect. The
   `with_fallback_dispatch` merge semantics for USER-registered overrides must
   survive: a user spec that omits dispatch/handler still inherits the
   catalog's (now via the registered base spec being overwritten — think through
   the order: auto-installed catalog spec first, user `register_builtin`
   REPLACES it; the merge helper's role may shrink; keep behavior, not code).
2. **Do not break `rum-stdlib`/hosts**: `core_builtin_specs()` stays `pub` and
   `install_core` keeps compiling and passing its tests unchanged.
3. **Suspension/eval paths untouched** — this brief is resolution/bootstrap
   only.
4. **One variable per measurement**; profiling env OFF for timing.
5. Stop-loss: same failure twice, or an unattributable microbench regression →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

BEFORE any edit (ONE run):

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_100_micro_base
```

After:

```
cargo fmt --check
cargo test --workspace
grep -rn "default_builtin_dispatch_spec_for_symbol" crates/   # expected: zero hits (or test-only, justify)
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_100_micro_new
```

Paste verbatim: workspace totals, the grep, and the per-label base-vs-new
seconds table. Judgment rule (learned in brief-097): sub-ms labels jitter
±20%+ run-to-run; only a CONSISTENT-direction hot-tier delta matters; anything
suspicious gets ONE re-run then a report, not silent re-measurement.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, the microbench script, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Probe budget: the two microbench runs + at most one suspicious-label re-run.
- Clean-check probes: use `git status --short -- crates/` (Claude may commit
  docs concurrently).
- Minimal diff; stop-loss on repeat failure; paste real verbatim outputs only.

## Report format (your final message)

1. What changed — per file with line ranges; where auto-install lives; the fate
   of `default_builtin_dispatch_spec*` and the merge helper; every touched test.
2. Verification — verbatim outputs (totals, grep, microbench table).
3. Verdict — registration is provably the only dispatch path (cite the deleted
   fallback + the fresh-session test) + the microbench delta.
4. Stop-loss / open questions (empty if none).
