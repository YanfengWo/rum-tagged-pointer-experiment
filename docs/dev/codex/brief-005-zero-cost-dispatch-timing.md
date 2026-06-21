# Brief 005 — Make dispatch timing instrumentation zero-cost when disabled (implementation)

## Goal

Brief-004 found, and the director gate independently verified, that the
per-phase timing instrumentation costs ~`12%` of the per-call work **with
profiling disabled** (a clean `if-true` sample shows `230 / 1866` self samples,
`12.3%`, in `runtime::profiling::*`). The hot timing helpers are NOT `#[inline]`,
so a disabled run still pays full call overhead ~20x per dispatch. This violates
the no-default-path-cost rule. Convert them to the existing zero-cost pattern so
a disabled run pays nothing, while enabled profiling output is byte-for-byte
unchanged.

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` first. Diagnosis is
  in `PROGRESS.md` "Brief-004 Diagnosis: shared call-dispatch overhead".
- The TEMPLATE for the fix already exists in
  `crates/rum-runtime/src/runtime/evaluation_cache.rs`:
  `record_evaluation_cache_counter` is `#[inline]` with the
  `if !..._enabled() { return; }` guard, and the actual work lives in a
  `#[cold] #[inline(never)] record_evaluation_cache_counter_slow`. Copy that
  shape.
- The hot timing helpers all live in
  `crates/rum-runtime/src/runtime/profiling.rs` and all gate on
  `lexical_profile_enabled()`.

## Scope

- Files you MAY modify: `crates/rum-runtime/src/runtime/profiling.rs` only.
  (If a helper's body needs a new private `*_enabled`/`*_slow` companion fn,
  add it in the same file.)
- MUST NOT change any call site, any other file, the enabled/output behavior,
  the `RUM_PROFILE_*` env semantics, `Expr`/`Symbol`, docs, or any `mma-hol`
  clone. The change is internal to these 8 functions.

## Design (apply the inline-guard + cold-slow-path pattern to all 8)

For each of these `pub(super)` functions in `profiling.rs`, make the function
`#[inline]`, keep ONLY the `if !lexical_profile_enabled() { return <none>; }`
guard (plus, for the `record_*` variants that take an `Option` token, the
`let Some(token) = token else { return; }` early-out MAY stay in the inline
part or move to the slow fn — your call, but the disabled path must not call
out), and move the rest of the body into a new `#[cold] #[inline(never)]`
companion that is called only when enabled:

1. `runtime_path_timing_start` (line ~960)
2. `record_profile_runtime_path_timing` (~1458)
3. `definition_dispatch_timing_start` (~926)
4. `record_profile_definition_dispatch_phase_timing` (~1370)
5. `definition_rule_timing_start` (~918)
6. `record_profile_definition_rule_timing` (~1294)
7. `builtin_timing_start` (~934)
8. `record_profile_builtin_timing` (~1423)

The disabled fast path must be: inlined atomic load of the enabled flag, branch
not-taken, return — no out-of-line call, no allocation, no thread-local touch.
When enabled, behavior is identical (same stack push/pop, same stats, same
output).

## Tests

No new semantic behavior to test; rely on the existing suite plus an
enabled-output equivalence check (below). If you add any helper, keep it private.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green. Then prove ENABLED profiling is unchanged — run the per-phase timing
on `downvalue-pattern-hit` with profiling ON, BEFORE and AFTER your change, and
confirm the same phases still print:

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 target/release/rum-cli --eval \
  'fPattern[x_Integer] := x + 1; Do[Do[fPattern[1],{i,1,100000}],{j,1,10}]' 2>&1 \
  | grep -E 'runtime-path-self-timing' | head
```

Then the performance gate (profiling OFF), with a clean same-session A/B if you
can, else paste the `if-true` ratio and note it is cross-run:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_005_micro
```

Paste the `ratios.tsv` rows for `if-true` AND its same-tier neighbors
(`block-local`, `with-local`, `downvalue-exact-hit`, `downvalue-pattern-hit`).
`if-true` should drop (target ~10%); no same-tier regression. If it does not
drop, STOP and report.

## rum / Rust pitfalls (read twice)

1. Measure the RELEASE binary; profiling OFF for any timing number.
2. The disabled path must be a single inlined branch — verify by re-sampling
   `if-true` and confirming `profiling::runtime_path_timing*` no longer appears
   with meaningful self time (was `12.3%`).
3. Do NOT change enabled behavior or output; the per-phase summary must be
   identical when `RUM_PROFILE_LEXICAL_SUBSTITUTION=1`.
4. `#[cold] #[inline(never)]` on the slow companion so the hot caller stays
   small and the branch predictor favors the disabled path.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; leave changes in the
  working tree.
- Only `profiling.rs`. Nothing else; if you think you must, STOP and report.
- Full machine access, repo + `/tmp` only, no network. Commands: `cargo`,
  release `rum-cli`, the microbench script, `python3`, `sample`.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, or no `if-true` movement → STOP
  and report.
- Minimal diff. Paste the real `cargo test` summary, the enabled-output check,
  and the `if-true` ratios; never fake a result.

## Report format

1. Per-function changes (the 8), one-line why each.
2. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, the enabled-output equivalence grep (before/after), and the
   before/after `if-true` + same-tier neighbor ratios.
3. Confirmation that a re-sampled `if-true` no longer shows the profiling
   helpers with meaningful self time.
4. Stop-loss / open questions (empty if none).
