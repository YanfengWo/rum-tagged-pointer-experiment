# Brief 295 — extend the AppendTo fast path through one exact Part spine

Step 4 of `docs/dev/codex/plan-291-assoc-append-ownership.md`. Builds directly
on Step 3 (`cc86cdd`), which made flat `AppendTo` amortized O(n).

## Goal

`AppendTo[outer[[1]], x]` still rebuilds the whole outer container on every
append. Measured on the current tree, with an outer list of 1000 and 200
appends:

```
into_vec_total=400 into_vec_clone=400 shared_args_alloc=630
```

400 = 200 iterations x 2 whole-vector rebuilds each (outer, then inner). The
Part path takes none of Step 3's fast path.

Extend the proved mechanism through a **single exact positional Part spine**:
compute the path without retaining a cloned selected value, take the leaf
stability proof, evict the cache entries for each ancestor, `make_mut` each
args vector from the definition root down to the selected list, append under
the same undo guard, and seed the leaf proof. Everything else stays on the
existing path.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- Plan §5 Step 4 has the design; §1 finding E has the measurements. Follow
  **this** brief where they differ.
- Step 3's module is `definitions/incremental_append.rs`. Reuse
  `exact_position_paths_from_part_target_selector_exprs`
  (`kernel_data/structure_data/mod.rs:35`, already `pub(in crate::runtime)` —
  no visibility change needed). **Do not change the general
  `replace_exact_path`**; the generic path must stay exactly as it is.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` (main)
  - `crates/rum-runtime/src/runtime/definitions/definition_updates.rs`
    — routing only, keep it small
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs`
    — only if ancestor eviction needs an API that Step 2 did not provide
  - `crates/rum-runtime/src/runtime/tests/mutation_operator_forms.rs` — tests
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` — tests
- Files you MUST NOT touch: everything else. Specifically **do not** touch
  `kernel_data/structure_data/**`, `association_data.rs` (Step 5), any other
  compound assignment, `scripts/**`, `docs/**`, `Cargo.*`.
- Only `AppendTo`, only a **single exact positional** Part target. Bulk,
  multiple, duplicate, and association-key Part targets stay on the generic
  path.

## THE ACCEPTANCE SIGNAL — read carefully, the obvious metric is useless here

**The `appendto-part-outer1000-large/-small` ratio will NOT move, and chasing
it is a mistake.** Its baseline is 2.10x, which is already correct and
already linear: with the outer container fixed at 1000, cost is `n*m + n^2/2`,
dominated by the linear `n*m` term. Step 4 removes the constant `m` factor; it
does not change the order. **A ratio near 2.0 before and after is the expected
result, not a failure.**

Use these three instead.

**1. Counters (primary, immune to timing noise).** Probe:
`outer = Table[{}, {1000}]; Do[AppendTo[outer[[1]], i], {i, 1, 200}]` with
`RUM_PROFILE_REPR=1`. Baseline above. Expected: `into_vec_total` and
`into_vec_clone` fall toward 0, `shared_args_alloc` falls sharply. If they do
not move, the fast path is not firing — diagnose that before quoting any time.

**2. The Part-vs-flat quotient (primary timing evidence, immune to the per-run
whole-run offset because both labels come from the same run).** Currently:

```
appendto-part-outer1000-large (n=200) / appendto-list-incremental-small (n=250)
  = 18.2x    (5.15 ms vs 0.283 ms, median of three runs)
```

If the outer rebuild disappears, the Part row should approach flat-row cost
for its size, i.e. this quotient should fall from 18.2x toward roughly 1.
Report it for 3+ runs before and after. Getting it into the single digits is
success.

**3. Step 3 must not regress.** The flat path shares this module. Re-verify:
flat counters still `into_vec_total=0, into_vec_clone=0, shared_args_alloc=519`
on the 500-iteration flat probe, and the flat `large/small` ratio still ~1.95x.
A Part win that costs the flat win is not acceptable.

## Deliverable

The fast path applies only when, in addition to every Step 3 condition on the
leaf list:

- the target is a **single exact positional** Part path on a plain symbol;
- every container along the spine is a plain `List`-headed call;
- no ancestor or leaf is aliased in a way that mutation could expose — where
  it is, `make_mut` copies, which is correct but must still be tested;
- the exact leaf proof is present and valid, and every ancestor cache entry
  can be evicted.

Anything else — bulk/multi/duplicate selectors, negative or out-of-range
indices, association-key selectors, non-`List` containers, or any Step 3
disqualifier — falls through with **no behavior change whatsoever**.

## Verification (MANDATORY — you run it; iterate to green)

### Correctness — multi-level aliasing is where this breaks

The single-level alias test from Step 3 is not sufficient. Add tests for each
level independently:

- **outer aliased**: `o = {{1},{2}}; a = o; AppendTo[o[[1]], 3]` leaves `a`
  as `{{1},{2}}`;
- **inner aliased**: `o = {{1},{2}}; b = o[[1]]; AppendTo[o[[1]], 3]` leaves
  `b` as `{1}`;
- **deeper spine**: at least three levels, with an alias taken at the middle
  level;
- **same list at two positions**: `i = {1}; o = {i, i}; AppendTo[o[[1]], 2]`
  must leave `o[[2]]` as `{1}`.

Plus: bulk/duplicate/negative/out-of-range selectors still behave identically
and produce the same messages; a non-`List` container along the spine still
runs its definitions; `Protected` still refuses; tracing and monitors still
observe what they did; the returned value is unchanged.

**Rollback across multiple levels**: if the append fails after some ancestors
have already been `make_mut`'d (resource limit, interrupt, injected test-only
failure), the whole structure must be restored — no partially mutated spine.
This is harder than Step 3's single-level case; show the test explicitly.

### Standard gates

`cargo fmt --check`; `cargo test --workspace` — report totals before/after and
tests added. The nine `rum-jupyter` ZeroMQ tests fail under the dispatch
sandbox (loopback binding forbidden); report them as the known limitation.

Sentinels: `part-list`, `set-ownvalue`, `do-iterator-null`,
`list-literal-small`, and both `association-set-key-incremental-*`. Normalize
per run, **excluding the labels you expect to change from the normalization
denominator** — whole-suite normalization is biased once target labels move a
lot (this cost 5.5 percentage points of artifact in Step 3; see `cc86cdd`).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- **Semantics are the hard constraint.** If you cannot prove a spine shape
  unobservable, leave it on the generic path. A narrow fast path is fine.
- If the predicate ends up rejecting the benchmark workload, STOP and report
  why. Do not widen it to make the number move.
- Do not touch the generic `replace_exact_path` or anything under
  `kernel_data/structure_data/`. If the spine work seems to require it, STOP
  and report — that is a design change, not an implementation detail.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (your final message)

1. Part-probe counters before/after, and your reading.
2. The Part-vs-flat quotient, 3+ runs before and after.
3. Confirmation that Step 3 did not regress (flat counters + flat ratio).
4. The predicate: every condition and where it is checked.
5. The multi-level alias tests and the multi-level rollback test, called out
   explicitly.
6. Sentinel movement (normalized, target rows excluded from the denominator).
7. Test totals, tests added, `cargo fmt --check`.
8. What you left on the generic path and why.
9. Confirmation that `git status` shows only Scope files.
