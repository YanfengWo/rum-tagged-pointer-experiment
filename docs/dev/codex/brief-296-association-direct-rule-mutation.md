# Brief 296 — direct canonical-Rule mutation for Association lvalues

Step 5 of `docs/dev/codex/plan-291-assoc-append-ownership.md`. Targets the
slowest label in the whole suite.

## Goal

`assoc[k] = v` in a loop is near-O(n^3). Two independent costs stack:

1. `canonicalize_association_entry_forms` (`association_data.rs:259-275`) does
   a linear `find` per entry — quadratic — and **both** the parse and the
   render side call it.
2. `render_association_entry_forms_preserving_heads` rebuilds **every**
   `Rule[k, v]` expression on every single update.

Cost 2 is measurable and stark. Probe `assoc = <||>; Do[assoc[i] = i, {i, 1,
200}]` under `RUM_PROFILE_REPR=1`:

```
into_vec_total=0 into_vec_clone=0 shared_args_alloc=20324 shared_expr_alloc=20322
```

Twenty thousand expression allocations for two hundred assignments. The
arithmetic is exact: update *i* rebuilds *i* Rules, and `sum(1..200) = 20100`.

Replace the parse/canonicalize/render round trip with **direct mutation of the
already-canonical top-level Rule vector**: for a single-key function-call
target and `Part[..., Key[k]]`, require a currently valid Association
stability proof (whose existing duplicate/Rule-form check is what establishes
the stored top level is already canonical, so it need not be re-parsed or
re-canonicalized), then `SharedArgs::make_mut` and replace or append exactly
one Rule under an undo guard.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- Plan §5 Step 5 has the design. Follow **this** brief where they differ.
- Primitives: `SharedArgs::make_mut` and the mutation-proof facility from
  brief-293; the same undo-guard shape used in `incremental_append.rs`.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/kernel_data/association_data.rs`
  - `crates/rum-runtime/src/runtime/kernel_data/mod.rs` — **both** the `mod`
    declaration and the glob re-export on line 10
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs`
    — only if the proof API needs extending
  - `crates/rum-stdlib/src/tests/associations/associations_set_lvalue.rs`
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`
- File you MAY create:
  - `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs`
- Files you MUST NOT touch: everything else. Specifically **do not** touch
  `definitions/incremental_append.rs` or anything else from Steps 3–4,
  `scripts/**`, `docs/**`, `Cargo.*`.
- Only the **single-key** function-call form and `Part[..., Key[k]]`. Nested
  and multi-key forms keep the existing recursive fallback.

## If you move code, keep the move auditable

The plan proposes relocating the lvalue block out of `association_data.rs`
(16,640 chars) into a new `association_lvalue.rs`. That is acceptable, but a
refactor and an optimization in one diff is hard to review, and this is a hot
correctness path.

Therefore: **relocated code must move verbatim** — byte-identical apart from
the minimum adjustments (visibility, `use` paths, module qualification). All
new logic must be *added*, not smuggled into the move. In your report, state
explicitly which functions moved unchanged and which are new. If you cannot
keep the move verbatim, do not move at all — leave the block where it is and
add the fast path in place. `association_data.rs` at 16,640 chars is nowhere
near the 35k line, so not moving is a perfectly good outcome.

## Acceptance signals

Unlike Step 4, **the large/small ratio here DOES move** and is a valid signal,
because this step changes the order.

1. **`shared_expr_alloc` (primary).** Baseline 20322 on the probe above. Direct
   mutation constructs one new Rule per update, so this should fall to roughly
   the iteration count (~200), a ~100x reduction. `shared_args_alloc` should
   fall comparably. This is the most direct evidence and is immune to timing
   noise — report it first.
2. **`association-set-key-incremental` large/small ratio.** Baseline **6.27x**
   (between n^2 and n^3). Removing both quadratic canonicalization passes
   should bring it toward **4.0x** (n^2 with a linear key scan retained).
   Same-run quotient, so immune to the per-run whole-run offset.
3. **Absolute**: `association-set-key-incremental-large` is 24.68 ms today,
   the slowest label in the suite by an order of magnitude. Report the new
   value, normalized per run with target rows excluded from the denominator.
4. **Steps 3–4 must not regress**: flat probe still `0/0/519`, Part probe still
   `0/0/~231`, flat `large/small` still ~1.95x, Part-vs-flat still ~1.4x.

## Verification (MANDATORY — you run it; iterate to green)

### Correctness — Association has more shapes than List did

Every one of these must behave **identically** to before:

- **alias**: `a = <|1 -> 2|>; b = a; b[3] = 4` leaves `a` as `<|1 -> 2|>`;
- **key order is preserved** on both update-existing and append-new;
- updating an existing key replaces in place, does not append a duplicate;
- **`RuleDelayed` becomes an immediate `Rule`** when its value is updated,
  matching current behavior;
- a duplicate-key Association literal still canonicalizes as it does now;
- a missing nested key still emits `Set::kval` with the same message;
- an invalid `Association[...]` form still falls through unchanged;
- `Part[..., Key[k]]` form behaves as before;
- RHS and selectors finish evaluating **before** any mutation;
- `Protected`, tracing, monitors, and active resource constraints all behave
  as before (these use the old path);
- **rollback**: a failure after `make_mut` restores the association intact.

Run the whole Association test family, not just the lvalue file.

### Standard gates

`cargo fmt --check`; `cargo test --workspace` — totals before/after and tests
added. The nine `rum-jupyter` ZeroMQ tests fail under the dispatch sandbox
(loopback binding forbidden); report as the known limitation.

Sentinels: `set-ownvalue`, `part-list`, `symbol-ownvalue-read`,
`list-literal-small`, `association-small-literal-repeat`, and all four
AppendTo rows. Normalize per run **excluding the labels you expect to change
from the denominator** (the correction from `cc86cdd`).

`association-small-literal-repeat` is the sentinel that matters most here: it
exists to catch exactly this step making small Associations slower to make
large ones faster.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- **Semantics are the hard constraint.** If a shape cannot be proven
  unobservable, leave it on the existing recursive path.
- If the predicate rejects the benchmark workload, STOP and report why. Do not
  widen it to move the number.
- Do not implement Step 6 (the bounded key index). A linear key scan is
  expected to remain; this step is about removing the round trip, not the
  scan.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (your final message)

1. `shared_expr_alloc` / `shared_args_alloc` before and after, with your
   reading.
2. The large/small ratio, 3+ runs before and after.
3. Absolute value of `association-set-key-incremental-large`.
4. Confirmation Steps 3–4 did not regress (both probes, both quotients).
5. **Which functions moved verbatim and which are new** — or that you chose
   not to move anything.
6. The predicate: every condition and where it is checked.
7. Correctness test list, with rollback and `RuleDelayed` called out.
8. Sentinel movement, with `association-small-literal-repeat` called out.
9. Test totals, tests added, `cargo fmt --check`.
10. Confirmation that `git status` shows only Scope files.
