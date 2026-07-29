# Brief 300 — in-place fast path for positional Part assignment

Step 3 (final) of `docs/dev/codex/plan-297-part-assignment.md`. `lst[[i]] = v`
works correctly since `88625df` but copies the container on every assignment.
Make the unaliased case independent of container width.

## Goal

Current baseline, committed in `885ec0f`:

| Label | n | Time |
|---|---|---|
| `set-part-fixed100-len100` | 100 | 0.194 ms |
| `set-part-fixed100-len800` | 100 | 0.558 ms |
| `set-part-fill-small` | 200 | 0.522 ms |
| `set-part-fill-large` | 400 | 1.417 ms |

Counters for `qL = Table[0,{200}]; Do[qL[[i]] = i, {i,1,200}]`:

```
into_vec_total=200 into_vec_clone=200 shared_args_alloc=226
```

`into_vec_clone=200` is the whole story: the unique-ownership fast path misses
on every single assignment, so each one copies the entire argument vector.
This is the same shape briefs 293–295 already solved for `AppendTo`; reuse
that machinery rather than inventing new.

## Context pointers

- Read `AGENTS.md`.
- Plan §6 Step 3 has the design. Follow **this** brief where they differ.
- Primitives already in the tree: `SharedArgs::make_mut`,
  `take_call_stability_proof` / `take_call_stability_proof_for_key` /
  `evict_call_stability_keys` (`evaluation_cache/mutation_proofs.rs:24,49,114`),
  and the predicate/rollback shape in `definitions/incremental_append.rs`.
- `evaluation_cache/mutation_proofs.rs` should need **no change** — the
  existing eviction API fits. Touch it only if you prove otherwise.

## Acceptance signals — note which one actually matters here

**Primary: the fixed-work length quotient.** `set-part-fixed100-len800 /
set-part-fixed100-len100` is **2.87x** today. It measures the same 100
assignments against two container widths, so it *is* the "cost depends on
container width" property, directly. If a slot replacement becomes O(depth),
this should head toward **1.0**. Same-run quotient, immune to the per-run
whole-run offset.

**Secondary: the fill quotient.** `fill-large / fill-small` is **2.72x**
today, and the target is ~2.0 — a much smaller margin. **The plan text says
"about 4"; that number is void**, it came from the failing pre-`88625df` path.
Do not treat 2.72x as a broken measurement; it is correct, and it is small
because a fixed per-operation evaluator cost dominates the O(m) copy at these
sizes.

**Counters (immune to load, report first).** `into_vec_clone` should fall from
200 toward 0. If the counters do not move, the fast path is not firing —
diagnose that before quoting any timing.

**Regression check: briefs 293–295 must not regress.** The flat and Part
`AppendTo` fast paths share this area. Re-verify: flat probe `0/0/519`, Part
probe `0/0/~231`, flat `large/small` ~1.95x, Part-vs-flat ~1.4x.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/part_assignment.rs` (main)
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs` —
    **only** to move shared primitives out, verbatim (see below)
  - `crates/rum-runtime/src/runtime/definitions/mod.rs`
  - `crates/rum-runtime/src/runtime/tests/part_assignment.rs`
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs` —
    only if a caller-shape change is genuinely required; state why
- File you MAY create:
  - `crates/rum-runtime/src/runtime/definitions/incremental_mutation.rs`
- Files you MUST NOT touch: everything else — `definitions.rs`,
  `association_lvalue.rs`, `kernel_data/structure_data/**`, `scripts/**`,
  `docs/**`, `Cargo.*`.

**If you move shared code, move it verbatim.** Byte-identical apart from
visibility and `use` paths; all new logic added, never smuggled into the move.
Say in the report which functions moved unchanged and which are new. If you
cannot keep it verbatim, do not move — duplicating a small guard is better
than an unreviewable diff on a hot correctness path. (Same rule as
`brief-296`, which chose not to move anything and was the better for it.)

## Deliverable

A direct route attempted after Step 1's evaluate/re-fetch/re-check phase, and
only when all hold:

- plain symbol owner, one exact path of positive integer selectors, plain
  `List` at every level;
- interrupt poll passes, protection re-checked, target still present;
- every root-to-parent call-cache entry is pointer-matched and evictable;
- no tracing, monitor scope, active resource constraint, pending
  suspension/input, or dynamic binding.

Mutation: evict root-through-parent cache entries, take the definition value
under the rollback guard, `make_mut` each ancestor, replace the one parent
slot under a swap-back guard, commit, bump the owner epoch, and **leave the
changed call proofs evicted** — do not reinsert a proof for a call whose
argument shape changed, since new definitions could match it later.

**No evaluator call and no fallible resource check may occur after the first
slot mutation.** Any predicate failure falls back to Step 1's generic path
with no observable difference.

## Verification (MANDATORY — you run it; iterate to green)

### Correctness — replacement aliasing is subtler than append aliasing

- alias at the **root**: `o = {{1},{2}}; a = o; o[[1,1]] = 9` leaves `a`;
- alias at an **intermediate** List;
- alias of the **selected child** itself: `b = o[[1]]; o[[1,1]] = 9` leaves `b`;
- **same list at two positions**: `i = {1}; o = {i, i}; o[[1,1]] = 9` must
  leave `o[[2]]` as `{1}`;
- a three-level spine with an alias taken at the middle level.

Plus, all of which must behave exactly as after `88625df`:

- the callback hazards from brief-298: an rhs that reassigns the target, a
  selector that reassigns / `Protect`s / `Clear`s it, and suspension+resume;
- **protection race**: target becomes `Protected` during rhs/selector
  evaluation → still refused;
- interrupt arriving before mutation → no partial write;
- tracing on, monitor active, resource constraint active → generic path,
  identical behavior;
- diagnostics for index 0 / negative / out-of-range / non-integer unchanged
  (`Set::argform`, `Part::partw`), none mentioning `Part is Protected`;
- **rollback**: an injected failure after the slot was replaced must swap the
  old item back and restore the definition. Show this test.

### Standard gates

`cargo fmt --check`; `cargo test --workspace` — totals before/after and tests
added. The nine `rum-jupyter` ZeroMQ tests fail under the dispatch sandbox
(loopback binding forbidden); report as the known limitation.

Sentinels: `set-ownvalue`, `part-list`, `symbol-ownvalue-read`,
`list-literal-small`, `association-small-literal-repeat`, and all four
`AppendTo` rows. Normalize **against specific untouched labels, never the
suite median** (it drifts cumulatively in an optimization lane: `cc86cdd`,
`a904935`). Run base and candidate in **one session** — cross-session
comparison on this suite is unreliable even with paired ratios (`885ec0f`:
the same comparison read -4.96% across sessions and -0.16% within one).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.**
- **Semantics are the hard constraint.** If a shape cannot be proven
  unobservable, leave it on the generic path. A narrow fast path that fires
  rarely is an acceptable outcome; a wide one that changes behavior is a
  failure.
- If the predicate ends up rejecting the benchmark workload, STOP and report
  why. Do not widen it to move the number.
- Do not change `AppendTo`'s predicates. Its two fast paths must keep exactly
  their current behavior.
- Touch nothing outside this repo except `/tmp`. Generated/scratch `rum`
  sources take the `.rum` suffix (`AGENTS.md` Working Rules). No network.
- **Testing Policy is a hard line** (`AGENTS.md`).

## Report format (your final message)

1. Counters before/after (`into_vec_total`, `into_vec_clone`,
   `shared_args_alloc`), with your reading.
2. The **fixed-work length quotient** before/after, 3+ runs — the primary
   signal.
3. The fill quotient before/after.
4. Confirmation briefs 293–295 did not regress (both AppendTo probes, both
   quotients).
5. The predicate: every condition and where it is checked.
6. Which functions moved verbatim, which are new — or that you moved nothing.
7. The alias tests (all five levels) and the rollback test, called out.
8. Sentinel movement, same-session, normalized against named untouched labels.
9. Test totals, tests added, `cargo fmt --check`.
10. Confirmation that `git status` shows only Scope files.
