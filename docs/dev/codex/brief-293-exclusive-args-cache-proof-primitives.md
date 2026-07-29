# Brief 293 — exclusive-args + cache-proof primitives (infrastructure, no win)

Step 2 of `docs/dev/codex/plan-291-assoc-append-ownership.md`. Adds the
primitives that Step 3 needs. **Expected speedup: none.** Any measured change
in the benchmark suite is noise, and any behavior change is a defect.

## Goal

`SharedArgs::into_vec` already has an `Arc::try_unwrap` fast path that never
fires on incremental updates, because three strong owners exist at that
moment: the definition table, the local `current` clone, and — the one that
blocks everything — a **call-stability cache entry**
(`model/cache.rs:105-113`), which `bump_evaluation_epoch_for_symbol` does not
clear (`evaluation_cache/stability.rs:56-88`).

Add two things:

1. Safe copy-on-write accessors on `SharedArgs` in `rum-core`.
2. A mutation-only evaluation-cache facility that can, for one
   pointer-matched stable-call entry: validate it, remove it, hand back its
   stability proof **only when the entry is currently valid**, reinsert an
   incrementally updated proof, and evict exact entries along a known
   structural path.

Nothing calls these on the evaluation path in this brief. Tests are the only
callers.

## THE STOP-LOSS CONDITION — read this before writing code

The entire plan rests on an unproven premise: that after removing the cache
entry and taking the definition value, the target `SharedArgs` actually
reaches `strong_count == 1`. **Prove or disprove that first**, with a test
that constructs the real `AppendTo` situation and asserts the strong count.

- If it reaches 1: proceed with the rest of the brief.
- **If it does not: STOP. Do not build the rest of the infrastructure.**
  Report which additional holder is still alive, with `path:line`. A negative
  answer delivered early is a complete success for this brief — it saves
  Steps 3–6 entirely. Do not try to work around it on your own initiative.

Put this result first in your report either way.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- `docs/dev/codex/plan-291-assoc-append-ownership.md` §2 has the full holder
  inventory; §5 Step 2 has the design this brief is derived from. Follow
  **this** brief where they differ.
- `SharedArgs` is `Arc<Vec<Expr>>` (`crates/rum-core/src/expr.rs:571`), already
  re-exported from `rum-core/src/lib.rs:12` — adding methods needs no export
  change.

## Scope

- Files you MAY modify:
  - `crates/rum-core/src/expr.rs` (the `SharedArgs` impl only)
  - `crates/rum-runtime/src/runtime/evaluation_cache/mod.rs` (module
    declaration + wiring only)
  - `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs`
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`
  - `crates/rum-runtime/src/runtime/model/cache.rs` (only if the proof type
    genuinely belongs next to `CallStabilityCacheEntry`)
- File you MAY create:
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs`
- Files you MUST NOT touch: everything else. Specifically **do not** modify
  `definition_updates.rs`, `association_data.rs`, any builtin, any dispatch or
  evaluation path, `scripts/**`, `docs/**`, `Cargo.*`, `.git`.
- Do not add unconditional cost to the default evaluation path. Instrumentation
  stays behind the existing env-gated `RUM_PROFILE_*` pattern.

## Deliverable

1. **The stop-loss answer** (above), as a committed test.
2. `SharedArgs::get_mut` / `make_mut`, safe, no `unsafe`, no raw pointer
   dereference. `make_mut` MUST record a `shared_args_alloc` exactly when COW
   actually allocates and not otherwise — the representation profiler is how
   Step 3 will be verified, so mis-accounting here silently breaks the next
   step's gate.
3. The mutation-proof facility, with these invariants held and tested:
   - Removing an entry is a **performance-only** change: results must be
     identical with and without the entry present.
   - A stale or dirty entry may be evicted, but must **never** yield a
     reusable proof.
   - A pointer-mismatched entry is left untouched.
   - Reinsertion uses the current generation and dirty snapshot, never a
     captured stale one.
4. **If you refactor the existing cache-validity predicate** so normal lookup
   and mutation lookup share it (the plan proposes this, and the reasoning —
   preventing drift between the two — is sound): the refactor MUST be a pure
   behavior-preserving extraction. Same predicate, same call order, same
   result, for every existing caller. State explicitly in the report that you
   verified this, and how.
5. **No dead code.** Every new public/pub(crate) item must have a test
   exercising it. `cargo build` and `cargo test` must be free of new
   `dead_code`/`unused` warnings.

## Verification (MANDATORY — you run it; iterate to green)

1. `cargo fmt --check`.
2. `cargo test --workspace` green. Report the total before and after; state
   how many tests you added.
3. No new compiler warnings. Show the relevant build output.
4. **Microbench A/B — and read it with the method below, which is not the
   obvious one.**

### How to read the A/B on this suite (mandatory method)

This suite has a **per-run whole-run offset**: every run carries an overall
scale factor that moves *all* of its labels the same way by a few percent,
varying randomly between runs. Two runs of the *identical* script measured
median −4.07% with 7/38 labels up; another identical pair, +2.57% with 30/45
up. Alternating base/candidate order does **not** cancel it.

Therefore:

- **A lopsided direction split is NOT evidence.** "Only 5 of 45 labels went
  up" is what this suite's per-run offset looks like, not a regression.
- **Do not compare raw per-label seconds across runs.**
- **Normalize per run.** Run base at least 3 times and candidate at least 3
  times, interleaved. For each run, compute that run's median label time, and
  divide every label by it. Compare the *normalized* values. This cancels the
  whole-run offset and is the only comparison that means anything here.
- **In this brief every label is a control label**, because nothing on the
  evaluation path calls the new code. So the expected result is: no label
  moves outside the spread of all the others. Report the normalized
  distribution and flag any label that stands apart from the pack — that would
  indicate you accidentally changed a hot path.

Persist raw output to files before parsing. All `RUM_PROFILE_*` off for timing.

5. **Accounting check.** With `RUM_PROFILE_REPR=1`, run the 500-iteration
   `AppendTo` probe from brief-291 before and after your change. `into_vec_total`,
   `into_vec_clone` and `shared_args_alloc` must be **unchanged**, since no
   caller is switched over yet. A change here means you altered a live path.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`. If you
  believe a primitive needs it, STOP and report — a safe alternative exists
  (`Arc::get_mut`/`Arc::make_mut` are safe).
- **No semantic change whatsoever.** rum's value semantics must hold:
  after `a = <|1 -> 2|>; b = a; b[3] = 4`, `a` is unchanged; after
  `x = {1,2}; y = x; AppendTo[y, 3]`, `x` is unchanged. Add tests for both if
  the existing suite does not already assert them.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no other-engine comparison, no oracle harnesses.
- Do not implement any part of Step 3 (the AppendTo fast path). This brief
  ends at primitives plus tests. If you find yourself editing
  `definition_updates.rs`, you have left Scope.

## Report format (your final message)

1. **The stop-loss answer first**: does the target `SharedArgs` reach
   `strong_count == 1` after cache-entry removal plus definition take? Quote
   the assertion and its result. If no, stop the report here and list the
   remaining holders with `path:line`.
2. Test totals before/after, number of tests added, `cargo fmt --check` result.
3. Compiler-warning status.
4. The normalized microbench comparison (method above): the distribution, and
   whether any label stands apart from the pack.
5. The `RUM_PROFILE_REPR` accounting check: the three counters before and
   after, confirmed unchanged.
6. Whether you refactored the shared validity predicate, and how you verified
   behavior preservation.
7. Confirmation that `git status` shows only files from the Scope list.
