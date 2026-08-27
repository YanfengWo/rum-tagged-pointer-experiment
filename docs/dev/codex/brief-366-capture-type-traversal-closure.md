# brief-366 — close the capture-type traversal graph

## Goal

Step 2 of plan-364. Apply `#[derive(FrameRootTrace)]` across the complete
187-type capture closure so that every type a suspension frame can hold has a
field-recursive traversal. **No behavior changes in this brief**: no frame
becomes a named struct, no closure API is deleted, no call site moves, and
neither the GC root walk nor the memory proxy is wired to the new traversal.
Those are steps 3 and 4.

What must be true at the end: every type in the closure implements
`FrameRootTrace`, the implementation comes from the derive rather than from
hand, and a new expression-bearing field added to any of them is traversed
without anyone editing a traversal.

## Context pointers

- Read `AGENTS.md` first.
- `docs/dev/codex/plan-364-frame-roots.md:483` — the closure: 116 types that
  carry `Expr` directly or transitively, 71 current leaf/control types. The
  itemized list with source lines starts at `:493`. **That list is the
  authoritative scope of this brief.**
- `crates/rum-runtime/src/runtime/suspension/trace.rs` — landed in step 1
  (`d13aa27f`): the trait, the visitor, and base implementations for `Expr`,
  `Symbol`, shared-expression types, containers, scalars, and the two
  function-pointer leaves. **You do not add base implementations here**; if
  you think one is missing, say so in the report rather than adding it.
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` — the step-1
  tests, including the sorted-allowlist gate.

## The prerequisite the plan's draft missed

`crates/rum-runtime/src/runtime/suspension.rs:7` currently declares:

```rust
pub(in crate::runtime) mod trace;
```

so `FrameRootTrace` is **invisible outside `rum-runtime`** — while the closure
includes types in `rum-stdlib` (`associations/aggregate.rs:6 CatenateStep`,
the fold/array/iteration state machines, and more). Nothing can be derived
there until the trait and the derive macro are reachable. `rum-stdlib`'s
manifest currently has no `rum-derive` dependency.

Settle this first, and state your choice with reasons:

- **(a) Re-export through `rum-runtime`** — the trait and a
  `#[doc(hidden)] pub use rum_derive::FrameRootTrace;`, so `rum-stdlib` keeps
  depending only on `rum-runtime` and never learns the derive crate exists.
  The known hazard: generated code must name the trait by an absolute path
  that resolves at every use site.
- **(b) `rum-stdlib` depends on `rum-derive` directly** — simpler paths, but
  it leaks an implementation detail into a second manifest.

My preference is (a); `rum-derive` is an implementation detail of the
traversal contract, not a thing stdlib should know about. Take (b) only if
(a) has a concrete problem you can show. Either way the trait itself has to
become reachable, so decide how far its visibility opens and keep it as
narrow as the closure actually requires.

## Scope

Files you MAY modify:

- the files named in the `plan-364:493` closure list, **for derive attributes
  and their imports only**;
- `crates/rum-runtime/src/runtime/suspension.rs` — module visibility;
- whichever single `rum-runtime` module is the right export surface for the
  trait and the derive (name it in the report);
- `crates/rum-stdlib/Cargo.toml` — **only if** you choose (b);
- `crates/rum-runtime/src/runtime/tests/frame_root_trace.rs` and a sibling
  stdlib-side test file if one is needed.

Files/areas you MUST NOT touch:

- `crates/rum-runtime/src/runtime/suspension/core.rs` — no frame, queue, or
  API change;
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs` and
  `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` — wiring
  the traversal into them is step 4;
- `crates/rum-derive/**` — the macro is done; if it cannot handle a shape,
  STOP and report rather than editing it;
- `crates/rum-core/src/expr.rs` representation and any public serde shape;
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`.

**Any type not on the `plan-364:493` list is out of scope.** If you find one
that belongs in the closure and is missing, report it — do not silently add
it, and do not silently skip a listed one either.

## Deliverable

1. **Derives applied across the full closure**, both the 116 expression-bearing
   types and the 71 leaves. The leaves are not optional: a derived no-op today
   is what makes a future field automatic, which is the entire point.
2. **Generic carriers bounded correctly** — `Run<T>` and the other generic
   types get `T: FrameRootTrace` only where the parameter appears in a
   traversed field, matching the rule step 1 established.
3. **A completeness check that is mechanical, not a promise.** A test that
   fails if any closure type stops implementing the trait — for example a
   list of `assert_traces::<T>()` instantiations covering all 187. State
   plainly how you verified the list matches `plan-364:493` item for item, and
   report the count you actually instantiated.
4. **Zero growth in the handwritten allowlist.** Every type here is derived.
   If some type genuinely cannot be, STOP and report it — do **not** write a
   handwritten implementation and add it to the allowlist. That allowlist is
   the enforcement boundary the user accepted on 2026-08-27; quietly widening
   it converts an accepted boundary into an eroding one.
5. **Traversal tests recording exact visited roots** for at least: a matcher
   binding table (`SymbolExprMap`), a recursive `MatcherArg`, an
   `AssociationEntryForm`, a rewrite plan, the `Block` bare-symbol state, and
   one stdlib state machine.
6. **The negative calibration below.**

## rum / Rust pitfalls (read twice)

1. **The required negative calibration proves automatic coverage, which is the
   property this step buys.** Take one already-derived type in the closure,
   temporarily add an `Expr` field to it, and show the traversal count for
   that type rises by one **without editing any traversal code**. Paste the
   before and after. Then remove the field and show the count returns. A
   derive that compiles proves nothing about what it visits.
2. **`OnceLock` must not be initialized by traversal.** Step 1's
   implementation uses `self.get()` (`trace.rs:158`); do not introduce
   anything that forces initialization while walking.
3. **Weak references are not strong roots.** `WeakSharedArgs` is a deliberate
   no-op (`trace.rs:47`). Keep that property in anything you derive.
4. **A leaf derive must stay a leaf.** Do not "helpfully" make a leaf type
   visit something it does not own.
5. **This step changes no behavior.** Workspace results must be identical plus
   the new tests. If an existing test changes result, something is wrong —
   investigate, do not adjust the test.
6. **Do not descend into `Expr`.** Step 1's contract hands a whole `Expr` to
   the visitor; the recursive symbol scan stays in `gensym_gc.rs:236`.
7. **Stop-loss is real.** If a type's shape defeats the derive twice, STOP and
   report the shape and the error.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste verbatim:

```
cargo fmt --check
cargo test -p rum-runtime --offline
cargo test -p rum-stdlib --offline
cargo test --workspace --offline
```

Baseline is `d13aa27f`: **2141 passed, 0 failed**. Report the new total, and
prove no existing test changed result the same way brief-365 did — a sorted
per-test-name diff, with `removed_or_changed=0` pasted.

Also paste:

- the negative calibration, before and after counts (pitfall 1);
- the allowlist test's output, showing the list did not grow;
- your completeness count, and how you reconciled it against `plan-364:493`.

No microbench gate: this brief adds trait implementations that nothing calls
on an evaluation path. Say that explicitly rather than running one — and if
you believe you touched a hot path, STOP and say so instead.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo except `/tmp`; no network; `cargo --offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- **No new `unsafe`**; `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No new dependency** beyond the optional `rum-derive` line in option (b).
- Minimal diff: derive attributes, imports, visibility, and tests. No
  drive-by reformatting, renames, or reorganization.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; state
  the bound and where you derived it.
- You MUST paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per file and per type group, with counts.
2. The export decision — (a) or (b), why, and the visibility you settled on.
3. Verification — each command with verbatim output, including the per-test
   diff and `removed_or_changed=`.
4. The negative calibration — before/after counts, and the restore.
5. Completeness — how many types instantiated, and the reconciliation against
   `plan-364:493`.
6. Stop-loss / open questions — any type that resisted, any closure member you
   believe is wrong (empty if none).
