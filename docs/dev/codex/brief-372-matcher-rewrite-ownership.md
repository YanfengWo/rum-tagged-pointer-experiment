# brief-372 — matcher and rewrite continuation ownership

## Goal

Step 3 of plan-369, and the hardest partition. Close the matcher/rewrite
ownership gap without deep-copying `MatcherArg`, `MatcherNode`, rule plans,
bindings, candidate queues, rebuilt outputs, or target lists.

**The suspension mechanism does not change here.** The closure queue stays the
only one, `suspend_with_result_frame_lazy` stays the construction path, and no
frame type, `FrameAction`, or Session slot is introduced — those are step 374.
Steps 370 and 371 are landed and their gains must survive this one.

## The partition

Measured at `c0510048`: **146 suspension sites** across the 29 files under
`pattern_matching/**`, `kernel_data/rewrite_data/**`,
`definitions/application/**`, `definitions/rhs_guards.rs`, and
`kernel_data/structure_data/replace/**`. That is denser than step 371's whole
50-file partition, in the hottest code in the evaluator.

Everything outside those five roots belongs to steps 371 (landed) or 373
(stdlib, not yet). Report any file you find ambiguous rather than guessing.

## The risk that makes this partition different

`SymbolExprMap` has **233 references** in the runtime, and the binding table is
read and written on the matching hot path. plan-369 proposes an immutable base
plus delta/tombstone overlay for it. That design is accepted **conditionally**:

- A lookup that was one `HashMap` probe must not silently become several.
- Backtracking must remain correct: the matcher discards and restores binding
  sets during search, and an overlay has to make that cheaper or equal, never
  a rebuild.
- **If the overlay cannot hold the matcher labels flat, that is a stop, not a
  thing to tune indefinitely.** plan-369's own rule: stop if the same overlay
  mechanism fails correctness or performance twice. Report what you measured
  and where you stopped; a matcher slowdown is not an acceptable price for
  frame ownership, and under the 2026-08-27 standard "most correct" includes
  being correct about performance.

Consider whether some binding paths can avoid the overlay entirely — for
example a fixed-arity match that never needs a delta — and say what you chose
per path rather than applying one shape everywhere.

## Semantics that must not move

1. **Ordering and backtracking** — candidate order, first-match wins, and the
   documented per-lane contracts in `docs/guide/patterns-and-rules.md`.
   `ReplacePart` runs back-to-front and does not evaluate a winner's earlier
   rivals.
2. **Right-hand-side `Condition` guards** — landed only recently in briefs
   347-354 across eight candidate-selecting lanes, on one substrate with one
   accept rule: exactly `True` accepts, anything else declines, and a decline
   is indistinguishable from the rule not matching.
3. **`OptionValue` scope** — a guard shares its body's scope.
4. **`Orderless`/`Flat`/`OneIdentity`** collection behavior, including the
   wrapped-candidate rules from briefs 275-279.

## Testing — the guard lesson applies directly here

The guard line's own failure is the sharpest warning available for this
partition: a guarded `DownValue` stopped enforcing its guard after any
attribute-revision bump, and **six green gates plus a whole-line regression
test all missed it**, because every one asserted the guard *before* the
invalidating action and never after one.

So for anything you touch that caches or compiles state derived from a
definition — matcher plans above all — **a test must act after the
invalidation**: define, bump attributes (any `SetAttributes`, or a
`ValueQ`-style snapshot restore), then assert. A test that only asserts
immediately after definition proves nothing about the cached path.

## Scope

MAY modify: the 29 in-scope files, family-local immutable state modules they
need, and focused runtime tests.

MUST NOT touch:

- `crates/rum-stdlib/**` — step 373;
- `crates/rum-runtime/src/runtime/suspension/**`, `model/session.rs`
  pending/resuming fields, `scoping/gensym_gc.rs`,
  `debug_tools/resources/memory.rs` — step 374;
- the step-371 files, except where a shared signature forces a mechanical
  update — say so explicitly if that happens;
- `crates/rum-core/src/expr.rs` representation and public serde shapes;
- `crates/rum-derive/**`, `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`,
  `.git`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No `Owned(Vec<_>)` escape variant** in any state type. That is the deep
   copy this line removes, wearing a named type as a disguise.
2. **Do not introduce `FrameAction` here, even partially** — a half-adopted
   action protocol is the second suspension mechanism the line forbids.
3. **Rejected ownership answers stay rejected**: `Arc::make_mut` on a shared
   handle, `Mutex`, `RefCell::take`, deferred sweep, pinning, snapshot roots.
4. **No COW deep clone while a root is retained.** If a mutation forces
   `make_mut` on a handle the frame also holds, the design is wrong at that
   point — change the representation, do not pay the clone.
5. **Do not weaken a test to make a conversion pass**, and never use another
   engine as an oracle — the Testing Policy is a hard line.
6. **Matcher is the hottest path in the evaluator.** Zero ordinary-path cost
   by construction is the target; "measured as noise afterwards" is second
   best and a measured regression is a defect.
7. **Stop-loss is real.** Two failures of the same mechanism, on correctness
   or on performance, means STOP and report with the tree as it stands.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test --workspace --offline
```

Baseline is `c0510048`: **2150 passed, 0 failed**. Report the new total and
prove no existing test changed result with a sorted per-test-name diff and
`removed_or_changed=0`.

**Counters first, timings second.**

- Numeric suspend/resume liveness probes for: matcher bindings surviving a
  suspension, a guarded replacement, and nested rule application. Exact
  resumed values, never an absence of error.
- `into_vec_clone`, `shared_args_alloc`, `shared_expr_alloc`, and alloc-byte
  counters for those probes. Any nonzero `into_vec_clone` needs an
  explanation — it is now the COW/unwrap-fallback detector.
- A post-invalidation guard test as described above.

Then the microbench A/B, profiling env OFF, **three runs per arm**:

```
bash scripts/core-language-microbench.sh --output-dir /tmp/brief372-base
bash scripts/core-language-microbench.sh --output-dir /tmp/brief372-new
```

Affected labels — paste every row: `matcher-single-tiny`,
`matcher-single-wide-small`, `matcher-single-wide-large`, `replaceall-simple`,
`replace-wide-no-match`, `cases-wide-no-match`, `deletecases-wide-no-match`,
`deletecases-wide-association-no-match`, `position-wide-no-match`,
`replacepart-wide-rule-list`, `replacepart-guarded-hit`,
`replacepart-guarded-decline-fallback`, `rewrite-guarded-hit`,
`rewrite-guarded-decline-fallback`, `downvalue-guarded-hit`,
`downvalue-guarded-decline-fallback`, `upvalue-guarded-hit`,
`subvalue-guarded-hit`, `subvalue-unguarded-hit`, `subvalue-unguarded-miss`,
`default-guarded-hit`, `default-unguarded-hit`, `default-unguarded-miss`,
`symbol-ownvalue-guarded-read`.

Named controls: `sort-default-wide`, `sortby-default-wide`,
`keysort-default-wide`, `sort-symbol-default-width4000`,
`sort-symbol-default-width16000`,
`association-part-positional-read-width4000`,
`association-part-positional-read-width32000`.

Report the per-label spread across the three runs and judge against it. Never
normalize against the suite median. If the controls move as much as the
affected labels, that is the noise floor — but a matcher label moving beyond
its spread is a defect to fix or report, not to ship.

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

1. What changed — per family, with the state types introduced, and the binding
   overlay design with its per-path choices.
2. Liveness probes — exact resumed values.
3. Counters — `into_vec_clone` and the allocation rows, with explanations.
4. The post-invalidation guard test and what it asserts.
5. Correctness — commands, totals, `removed_or_changed=`.
6. The A/B — every affected row, the controls, the three-run spread, and your
   verdict against that spread.
7. File sizes for anything you grew.
8. Stop-loss / open questions (empty if none).
