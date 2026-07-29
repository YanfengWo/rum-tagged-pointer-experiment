# Brief 294 — conservative in-place AppendTo fast path for plain Lists

Step 3 of `docs/dev/codex/plan-291-assoc-append-ownership.md`. **This is the
first step that touches a live evaluation path and the first that should
actually be faster.**

## Goal

`AppendTo[lst, x]` in a loop is O(n^2): each iteration clones the whole
argument vector. Steps 1–2 established the baseline and proved the blocker —
after taking the definition value AND removing the exact call-stability cache
entry, the `SharedArgs` becomes exclusive (`brief-293`, test
`append_to_args_become_exclusive_after_definition_take_and_cache_entry_removal`).

Use that to make flat list accumulation amortized O(n), **without changing any
observable behavior**, by mutating in place only when a conservative predicate
proves it is unobservable.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy hard lines).
- Plan §3 (per-operator safety table) and §5 Step 3 have the design. Follow
  **this** brief where they differ.
- Primitives from brief-293: `SharedArgs::get_mut`/`make_mut`, and the
  mutation-proof facility in `evaluation_cache/mutation_proofs.rs`.

### The safety boundary, measured

`AppendTo` is NOT generally safe to fast-path. Measured on the release CLI:

```
u = u; u /: Plus[u, 1] := (seen1 = "RAN"; 10); AddTo[u, 1]     -> seen1 = "RAN"
lst2 = f[]; f[z___] := (seen2 = "RAN"; g[z]); AppendTo[lst2, 1] -> seen2 = "RAN", lst2 = g[1]
lst3 = {};                                    AppendTo[lst3, 1] -> nothing ran, lst3 = {1}
```

So the head matters: appending onto a non-`List` head evaluates that head's
definitions, i.e. runs user code. **Only a plain `List` head with no
interfering definitions may take the fast path.** Every other shape must fall
through to the existing evaluated path, unchanged.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definition_updates.rs`
    — **routing call only**; policy and mutation do not go here (it is already
    19,682 chars and must not drift toward the 35k split line)
  - `crates/rum-runtime/src/runtime/definitions/mod.rs` — module declaration
  - `crates/rum-runtime/src/runtime/evaluation_cache/mutation_proofs.rs` —
    only if the proof API needs adjusting for a real caller
  - `crates/rum-runtime/src/runtime/tests/mutation_operator_forms.rs` — tests
  - `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs` — tests
  - `crates/rum-runtime/src/runtime/eval_core/mod.rs` — **only** to widen an
    existing interrupt-poll helper's visibility to `pub(in crate::runtime)`,
    if the new module must call it. No logic change there.
- File you MAY create:
  - `crates/rum-runtime/src/runtime/definitions/incremental_append.rs`
- Files you MUST NOT touch: everything else. Specifically **do not** touch
  `association_data.rs` (that is Step 5), any other compound assignment,
  `scripts/**`, `docs/**`, `Cargo.*`.
- Do not add unconditional cost to the default path. Any counter goes behind
  the existing env-gated `RUM_PROFILE_*` cold-load pattern.
- Only `AppendTo`. Not `PrependTo`, not the Part/nested case (Step 4), not any
  arithmetic compound assignment.

## Deliverable

A fast path taken **only** when all of the following hold, each checked
explicitly:

- the target is a plain symbol whose own value is a `List`-headed call;
- `List` itself has no interfering user definitions;
- nothing in the situation can observe the old value or run user code:
  no upvalues/downvalues that could match, tracing disabled, no monitor
  callback active, no active deadline or memory limit, no suspension pending;
- the appended item cannot itself trigger evaluation of the target;
- the exact call-stability proof is present and currently valid, so removing
  it yields exclusivity.

On any predicate failure, fall through to the existing path with **no
behavior change and no observable difference** — including the message,
result, and side-effect ordering.

Mutation sequence: take the exact stability proof, `make_mut` the definition's
list under a rollback guard, push, bump the symbol epoch, reinsert the proof
with the item's bloom unioned in.

## Verification (MANDATORY — you run it; iterate to green)

### 1. Correctness first — these are not optional

Add tests for every one of these; each must behave **identically** to before:

- alias: `x = {1, 2}; y = x; AppendTo[y, 3]` leaves `x` as `{1, 2}`;
- non-`List` head runs user code: the `lst2 = f[]` case above still sets
  `seen2` and yields `g[1]`;
- `AppendTo[x, Nothing]` keeps its current semantics;
- an upvalue or downvalue that could match still fires;
- tracing enabled: the trace still records what it did before;
- a monitor callback still observes what it did before;
- a `Protected` target still refuses with the same message;
- **rollback**: if the update fails after the value was taken (resource limit,
  interrupt, or an injected test-only failure), the symbol's value is restored
  — never left unset. Show the test.
- the returned value of `AppendTo` is unchanged, including when the result is
  used rather than discarded.

### 2. The hit-rate check comes BEFORE any timing claim

Timing on this suite is noisy (per-run offset ±4%, see `df9985d`). The
counters are not. With `RUM_PROFILE_REPR=1` on the 500-iteration probe
(`lst = {}; Do[AppendTo[lst, i], {i, 1, 500}]`), the baseline is:

```
into_vec_total=500 into_vec_clone=500 shared_args_alloc=1019
```

Report the same three counters after your change. Expected: `into_vec_total`
drops toward 0 (that path is no longer taken) and `shared_args_alloc` falls
sharply (in-place push does not allocate). **If the counters do not move, the
fast path is not firing — stop and diagnose that before reporting any timing
number, because a timing "win" without a counter change is noise.** Explain
whatever residual allocation remains.

### 3. The complexity ratio is the primary timing evidence

`appendto-list-incremental-large / -small` is a ratio of two labels **within
the same run**, so it is immune to the per-run whole-run offset that makes
absolute cross-run deltas unreliable. Baseline is **3.75x** (O(n^2)). If the
accumulation becomes amortized O(n), it should approach **2.0**.

Report this ratio for at least 3 runs before and 3 runs after. This, plus the
counters, is the evidence. Absolute per-label seconds are supporting material
only; normalize per run (divide each label by that run's median) if you quote
them, per the method in brief-293.

### 4. Sentinels must not regress

Same-tier controls: `do-iterator-null`, `set-ownvalue`,
`symbol-ownvalue-read`, `list-literal-small`, `part-list`, and both
`association-set-key-incremental-*` rows (untouched by this step — they are a
pure control here). Report their normalized movement. Remember: a lopsided
direction split across all labels is the per-run offset, not a regression.

### 5. Standard gates

`cargo fmt --check`; `cargo test --workspace` green — report totals before and
after and how many tests you added. The nine `rum-jupyter` native ZeroMQ tests
fail under the dispatch sandbox because it forbids binding loopback; that is a
known sandbox limitation, so report them as such rather than treating them as
regressions.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- **No new `unsafe`.** `rum-core`/`rum-runtime` are zero-`unsafe`.
- **Semantics are the hard constraint, performance is the goal.** If you
  cannot prove a case unobservable, do not fast-path it. A narrow fast path
  that fires rarely is a good outcome; a wide one that changes behavior is a
  failure, and so is one whose predicate you cannot justify from the code.
- If the predicate turns out to be so restrictive that the target labels do
  not move, STOP and report that, with the reason the predicate rejects the
  benchmark workload. Do not widen it on your own initiative to make the
  number look good.
- Touch nothing outside this repo except `/tmp` scratch files. No network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.

## Report format (your final message)

1. **The three counters, before and after** (`into_vec_total`,
   `into_vec_clone`, `shared_args_alloc`), and your reading of them.
2. **The large/small ratio**, 3+ runs before and after.
3. The predicate: every condition, and where in the code each is checked.
4. Correctness test list, with the rollback test called out explicitly.
5. Sentinel movement (normalized).
6. Test totals, tests added, `cargo fmt --check`.
7. Any case you deliberately left on the slow path and why.
8. Confirmation that `git status` shows only Scope files.
