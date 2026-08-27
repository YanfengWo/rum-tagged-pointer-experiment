# brief-373 — stdlib continuation ownership

## Goal

Step 4 of plan-369, and the last ownership step before the cutover. Convert
`rum-stdlib` continuation state so a rooted frame can hold it without a deep
copy, using crate-local immutable and persistent carriers.

**The suspension mechanism does not change here.** The closure queue stays the
only one, `suspend_with_result_frame_lazy` stays the construction path, and no
frame type, `FrameAction`, or Session slot is introduced — those are step 374.
Steps 370-372 are landed and their gains must survive this one.

## The partition

Measured at `55edea50`: **111 suspension sites across 43 files** in
`crates/rum-stdlib/src`. That is the whole remaining ownership surface; the
runtime side is closed by steps 371 and 372.

The families, by where the work concentrates: associations (basic, grouping,
transforms, resumable key-sort, missing, aggregate), functional operations
(folds, arrays, iteration, sequence), structure operations (ordering sets,
collection stats, selectors, the resumable merge sort), strings (patterns,
text builtins, the regex AST step 2 pulled in), and predicates.

## What "ownership conversion" means here

Same shapes as steps 371 and 372, applied crate-locally:

- **Call arguments keep their `SharedArgs` handle** wherever brief-370 made it
  available — do not degrade it and rebuild.
- **Fixed items, entries, and specs** — `Arc` slices plus a cursor rather than
  a consumed `Vec` or a consuming iterator.
- **Outputs, histories, decorated and grouped state** — persistent
  family-local carriers with O(1) resumed updates.
- **Freeze happens only after an actual `InputRequired`**, the discipline
  brief-370 established and 371/372 kept.

Step 372's binding solution is the model worth copying: rather than building
the overlay plan-369 sketched, it froze into an `Arc` only once matching had
actually suspended, so the ordinary path kept one `HashMap` probe. **Prefer
the simplest carrier that keeps the ordinary path unchanged**, and say per
family what you chose and why.

## Semantics that must not move

1. **Comparator callback order and exact output ordering** for the sort family.
   `Sort`, `SortBy`, `ReverseSort`, `KeySort`, and `KeySortBy` run on one
   iterative resumable merge engine (brief-339's line); a user comparator
   observes a specific sequence of comparisons, and that sequence is
   observable behavior.
2. **Association ordering** — insertion order, and the documented key
   semantics.
3. **String pattern semantics**, including the regex AST's matching order and
   overlap rules.
4. **`Missing` and default-value behavior** in the association family.

## Scope

MAY modify: the 43 in-scope `rum-stdlib` files, family-local state modules
they need, and focused stdlib tests.

You MAY add the **minimum** doc-hidden `kernel_api` re-export if a carrier
genuinely needs a runtime type that is not yet reachable — but name it in the
report with the reason, and add nothing beyond what compiles.

MUST NOT touch:

- `crates/rum-runtime/src/runtime/suspension/**`, `model/session.rs`
  pending/resuming fields, `scoping/gensym_gc.rs`,
  `debug_tools/resources/memory.rs` — step 374;
- the runtime files converted by steps 371 and 372, except a mechanical
  update a shared signature forces — say so explicitly if that happens;
- `crates/rum-core/src/expr.rs` representation and public serde shapes;
- `crates/rum-derive/**`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`,
  `.git`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Do not publish generic continuation machinery as a supported embedding
   API.** These carriers are crate-local implementation detail. rum-stdlib's
   public surface is its builtins, not its resume plumbing; anything that
   escapes into `kernel_api` becomes a compatibility obligation.
2. **Do not centralize unrelated state.** Association, string, and sort
   carriers stay separate even where they look similar. A shared module here
   would couple three families and grow past the file-size line for no gain —
   scope is set by the problem.
3. **No `Owned(Vec<_>)` escape variant** in any state type.
4. **Do not introduce `FrameAction` here, even partially.**
5. **Rejected ownership answers stay rejected**: `Arc::make_mut` on a shared
   handle, `Mutex`, `RefCell::take`, deferred sweep, pinning, snapshot roots.
6. **No COW deep clone while a root is retained.** If a mutation would force
   `make_mut` on a handle a frame also holds, change the representation.
7. **Watch the file-size line.** `ordering_sets.rs` is 29,341 bytes with 5
   sites, `collection_stats.rs` 25,244 with 3, `associations/basic.rs` 25,106
   with 5. Put family state in a sibling module rather than growing these past
   35k, and report the final size of anything you grew.
8. **Stop-loss is real.** Two failures of the same mechanism, on correctness
   or performance, means STOP and report with the tree as it stands.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-stdlib --offline
cargo test -p rum-host --offline
cargo test --workspace --offline
```

Baseline is `55edea50`: **2154 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`.

**Counters first, timings second.**

- Numeric suspend/resume liveness probes for at least: a callback sort, an
  association grouping, a fold, and a string pattern operation. Exact resumed
  values, never an absence of error.
- **An ordinary-path allocation comparison**, in the shape that settled step
  372 at the gate: run a non-suspending stdlib workload against `55edea50` and
  against your change, and report `alloc count`, `bytes_total`, and the repr
  counters for both. The ordinary path must not get more expensive; step 372
  made it cheaper.
- `into_vec_clone` is the COW/unwrap-fallback detector — any nonzero value
  needs an explanation.

Then the microbench A/B, profiling env OFF, three runs per arm:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief373-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief373-new
```

Affected labels: `sort-callback-small`, `sort-callback-large`,
`sortby-callback-small`, `sortby-callback-large`,
`reversesort-callback-small`, `reversesort-callback-large`,
`keysort-callback-small`, `keysort-callback-large`,
`keysortby-callback-small`, `keysortby-callback-large`,
`stringreplace-pattern-unguarded`, `stringreplace-single-literal-unguarded`,
plus any association or fold label your conversions touch — name them.

Named controls, chosen because this brief cannot reach them:
`matcher-single-tiny`, `matcher-single-wide-small`,
`matcher-single-wide-large`, `downvalue-guarded-hit`,
`subvalue-unguarded-hit`, `default-unguarded-hit`.

Report the per-label spread across the three runs and judge against it. Never
normalize against the suite median. **If the controls move as much as the
affected labels, that is a systemic shift and the honest verdict is "no
regression", not a speedup** — that is exactly how step 372 read, and
reporting it as an improvement would have been wrong.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`).
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- No new dependency.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; name
  the prior same-shape duration each timeout is derived from. Persist raw
  microbench output before parsing.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per family, with the carriers introduced and why each shape
   was chosen.
2. Any `kernel_api` addition, with its justification.
3. Liveness probes — exact resumed values.
4. The ordinary-path allocation comparison, base against change.
5. Correctness — commands, totals, `removed_or_changed=`.
6. The A/B — affected rows, controls, three-run spread, verdict.
7. File sizes for anything you grew.
8. Stop-loss / open questions (empty if none).
