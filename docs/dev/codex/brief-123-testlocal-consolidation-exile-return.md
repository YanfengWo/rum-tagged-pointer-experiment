# Brief 123 — consolidate test-local builtin registrations + repatriate exiled mechanism tests

## Goal

Close the two test-organization debts left by the rebalance line:

1. **Consolidate test-local registrations**: several runtime test modules
   register their own minimal builtins ad-hoc (`vm_rhs_template/tests.rs`:
   SameQ/CompoundExpression/If/Return; `evaluation_cache/tests.rs`:
   CompoundExpression; `tests/scoping_gensym_gc.rs`: First; …). A shared
   `#[cfg(test)]` helper already exists (`runtime/test_builtins.rs`, created
   in brief-113 with `register_test_arithmetic_builtins`). Extend it into
   THE one home for test-local minimal builtins (grouped registration fns or
   one registry fn with a needs-list — your design, reported), and convert
   every ad-hoc site to use it. Each minimal implementation stays a
   deliberately SIMPLE test double (structural equality SameQ etc.) — do not
   import stdlib semantics; document that inline in the helper's header
   comment (it already carries the brief-113 rationale).
2. **Repatriate exiled kernel-mechanism tests**: during the Option-1 mass
   migrations, a few KERNEL-MECHANISM tests were exiled to rum-stdlib solely
   because their probe expressions used stdlib builtins (tracked in
   PROGRESS; known exiles: `stdlib/src/tests/incidental_scoping_gensym_gc.rs`
   — it drives `collect_and_sweep_dead_gensyms`/GC introspection counters,
   pure kernel machinery; audit the other `incidental_*` files for the same
   character: tests whose SUBJECT is kernel machinery — GC, resource limits,
   suspension — rather than builtin behavior). Move those test fns back into
   the runtime test tree, replacing their stdlib-builtin probe usage with
   test-local registrations from the consolidated helper (this is the
   sanctioned probe-adaptation path: the exile was the workaround, the
   helper is the fix). Tests whose subject IS builtin behavior stay in
   stdlib — when in doubt, report and leave.

Workspace green; **total test-fn count conserved** (moves + zero deletions;
paste the accounting). Zero production-code change EXCEPT possibly
visibility already available (if a repatriated test needs a kernel item that
lost visibility, STOP and report rather than promoting).

## Context pointers

- `runtime/test_builtins.rs` (brief-113) — the seed helper.
- Ad-hoc registration sites (verified): `vm_rhs_template/tests.rs`,
  `evaluation_cache/tests.rs`, `tests/scoping_gensym_gc.rs`,
  `tests/registered_builtin_resolution.rs` (this last one is the RESOLUTION
  PROBE from brief-102 — its inline ZzzProbe-style registration is the
  point of the test; leave it self-contained, note it in the report).
- Known exile: `incidental_scoping_gensym_gc.rs` (its GC-introspection
  methods were made pub in brief-106 FOR the exile — repatriation may allow
  narrowing them back to pub(crate)/pub(in crate::runtime); do so ONLY if
  no other external user exists, and report either way).
- The gensym-GC promotions' fate: if narrowed, update the brief-106 note in
  your report; the drift/capability tests are untouched.

## Scope

- MAY modify: `runtime/test_builtins.rs`; the ad-hoc registration test
  modules; `runtime/tests/**` (receiving repatriated fns + mod wiring);
  `crates/rum-stdlib/src/tests/**` (removing repatriated fns; deleting
  emptied files + mod wiring); visibility NARROWING of the brief-106 GC
  accessors if repatriation permits (never widening).
- MUST NOT touch: production code otherwise, stdlib non-test code, other
  crates, any `mma-hol` clone, `docs/**` except your report, `.git`,
  `Cargo.toml`/`Cargo.lock`.

## Verification (MANDATORY)

```
# BEFORE: capture test-name lists
cargo test -p rum-runtime -- --list > /tmp/cdx_123_rt_before.txt
cargo test -p rum-stdlib -- --list > /tmp/cdx_123_sl_before.txt
# ... work ...
cargo fmt --check
cargo test --workspace
cargo test -p rum-runtime -- --list > /tmp/cdx_123_rt_after.txt
cargo test -p rum-stdlib -- --list > /tmp/cdx_123_sl_after.txt
# paste: per-crate before/after counts; the moved-fn list (name: stdlib -> runtime);
# combined-total conservation; grep for remaining ad-hoc register_builtin in test code
grep -rn "register_builtin" crates/rum-runtime/src/runtime --include="*.rs" | grep -v test_builtins.rs | grep -v registered_builtin_resolution
```

The final grep should show only non-test/production call sites (report what
remains and why each is fine).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, `grep`,
  `python3`. Never edit any `mma-hol` file. **No new `unsafe`.**
- No test deleted, weakened, or renamed beyond module-path moves.
- Stop-loss: a repatriation needing new visibility, ambiguous
  mechanism-vs-behavior classification you cannot justify, same compile
  error twice → STOP and report.

## Report format

1. The consolidation design (helper API) + converted sites; the
   left-self-contained note for the resolution probe.
2. The repatriation list (fn: stdlib file → runtime file; classification
   reasoning) + files deleted; any visibility narrowing performed.
3. Verification — verbatim outputs incl. the conservation accounting and
   the final grep.
4. Stop-loss / open questions.
