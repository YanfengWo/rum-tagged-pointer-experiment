# Brief 009 — Representation-cost counters (Phase 0 scaffold, implementation)

## Goal

Phase 0 of the representation rewrite (`docs/representation-rewrite-plan.md`):
make the two representation flaws directly countable so every later step has an
objective gate. Add opt-in, env-gated counters in `rum-core` for (1) how often
`SharedArgs::into_vec` / `SharedExpr::into_owned` actually CLONE a shared `Arc`
versus free-unwrap a unique one (flaw 1, the owned round-trip), and (2)
call-node `Arc` allocations (flaw 2). Surface them through `rum-runtime`'s
existing profile summary. The call-stability `miss_absent` counter (flaw 3)
already exists in `evaluation_cache.rs`; no new work there.

**Critical:** these counters sit on the hottest path in the evaluator. When
disabled they MUST be zero-cost — the disabled path is a single inlined atomic
load that returns, no out-of-line call (this is exactly the lesson from
brief-005, which removed ~12% of per-call cost from instrumentation that was not
zero-cost when off). Use the `#[inline]` guard + `#[cold] #[inline(never)]`
slow-path pattern.

## Context pointers

- Read `docs/representation-rewrite-plan.md` (Phase 0) and
  `docs/performance-stage0-baseline.md` first.
- The zero-cost pattern to copy is `record_evaluation_cache_counter` in
  `crates/rum-runtime/src/runtime/evaluation_cache.rs` (`#[inline]` guard +
  `#[cold] #[inline(never)]` `_slow`).
- The hot conversions are in `crates/rum-core/src/expr.rs`:
  `SharedArgs::into_vec` (~line 488) is
  `Arc::try_unwrap(self.0).unwrap_or_else(|args| args.as_ref().clone())` — the
  `unwrap_or_else` closure runs ONLY on the shared (clone) path;
  `SharedExpr::into_owned` (~line 435) is analogous. `SharedArgs::new` (~line
  484) and `SharedExpr::new` (~line 431) are the `Arc::new` allocations.
- `rum-runtime` already prints a profile summary under
  `RUM_PROFILE_EVALUATION_CACHE=1` (see `print_*_summary` in
  `crates/rum-runtime/src/runtime/profiling.rs`).

## Scope

- Files you MAY modify:
  - `crates/rum-core/src/expr.rs` (the counter increments at the 4-6 sites),
  - a new `crates/rum-core/src/profiling.rs` (the env-gated counter facility +
    a public snapshot/report API) and `crates/rum-core/src/lib.rs` (module +
    re-export),
  - `crates/rum-runtime/src/runtime/profiling.rs` (call the rum-core report
    from the existing summary path).
- MUST NOT change `Expr`/`SharedArgs`/`SharedExpr` public types, behavior,
  serde, `Ord`/`Hash`/`PartialEq`, any `mma-hol` clone, docs, `.git`.

## Design

1. **`rum-core` counter facility** (new `profiling.rs`): an `AtomicU8` enabled
   flag initialized once from env `RUM_PROFILE_REPR` (non-empty, != "0"), the
   same cold-init pattern as rum-runtime. Counters (all `AtomicU64`,
   `Relaxed`):
   - `into_vec_total`, `into_vec_clone`
   - `into_owned_total`, `into_owned_clone`
   - `shared_args_alloc` (one per `SharedArgs::new`)
   - `shared_expr_alloc` (one per `SharedExpr::new`)
   Expose `pub fn repr_profile_enabled() -> bool` (`#[inline]`),
   `#[inline] pub fn record_into_vec(cloned: bool)` etc. (guard returns
   immediately when disabled; the actual `fetch_add` lives in a `#[cold]`
   companion), and `pub fn repr_profile_snapshot() -> ReprProfile` returning the
   current counts (a small `Copy` struct).

2. **Increment sites** in `expr.rs`:
   - `into_vec`: record `(cloned = Arc::strong_count or the try_unwrap branch)`.
     Restructure to `match Arc::try_unwrap(self.0) { Ok(v) => { record(false); v
     } Err(a) => { record(true); a.as_ref().clone() } }` so the clone branch is
     known. Keep the non-disabled fast path identical to today when the counter
     is off.
   - `into_owned`: same shape.
   - `SharedArgs::new` / `SharedExpr::new`: record one alloc each.

3. **rum-runtime summary**: in the existing summary print path (the one gated by
   `RUM_PROFILE_EVALUATION_CACHE`), also call `rum_core::repr_profile_snapshot()`
   and print `rum-profile repr into_vec_total=... into_vec_clone=...
   into_owned_total=... into_owned_clone=... shared_args_alloc=...
   shared_expr_alloc=...`. (So one env var, `RUM_PROFILE_EVALUATION_CACHE=1`,
   also enables and dumps the repr counters — set `RUM_PROFILE_REPR=1` too, or
   have rum-runtime enable the rum-core flag when its own profiling is on; your
   call, but document which env var the user sets.)

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-core
cargo test -p rum-runtime
```

All green. Then PROVE the counters are correct and the disabled path is
zero-cost:

1. Counter correctness — a loop of `n` identical calls should show ~`n`
   `into_vec_clone` (the body's args Arc is shared, so every iteration clones):

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]' 2>&1 \
  | grep 'rum-profile repr'
```

Report the counts and confirm `into_vec_clone` is on the order of the 1,000,000
iterations (not ~1, not 0).

2. Zero-cost when OFF — sample `if-true` with profiling OFF and confirm the new
   rum-core counter functions do NOT appear with meaningful self time:

```
# background the probe, sample it, reduce with scripts/sample-self-time.py
# (profiling env OFF), grep for rum_core::profiling / record_into_vec etc.
```

Confirm no `rum_core::profiling::*` / `record_into_*` frames show meaningful
self time, and that the `if-true` microbench ratio is unchanged versus before
this brief (no default-path regression — clean same-session A/B or note it is
cross-run).

## rum / Rust pitfalls (read twice)

1. Disabled path = one inlined atomic load + branch, no call, no `fetch_add`.
   Verify by sampling (step 2). A counter that taxes the hot path defeats its
   own purpose.
2. Do not change `into_vec`/`into_owned` behavior or the unique-`Arc`
   fast-unwrap; only add the branch-aware record.
3. `rum-core` must stay free of `rum-runtime` deps; the facility is
   self-contained in `rum-core`.

## Hard rules

- Do NOT `git commit`/branch/push. Leave the working tree.
- Only the listed files. Full machine access, repo + `/tmp` only, no network.
  Commands: `cargo`, release `rum-cli`, `sample`, `scripts/sample-*.py`,
  `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: same test failure twice, or the disabled path is not zero-cost
  after one attempt → STOP and report.
- Minimal diff. Paste the real test summaries, the counter line, and the
  zero-cost sample check; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-core` and
   `-p rum-runtime` summaries, the `rum-profile repr` counter line, and the
   zero-cost-when-off sample evidence (+ unchanged if-true ratio).
3. The Phase-0 baseline counts for `fExact[1]` (or note Claude will capture the
   full baseline at gate time).
4. Stop-loss / open questions (empty if none).
