# brief-366 — close the capture-type traversal graph

## Goal

Step 2 of plan-364. Apply `#[derive(FrameRootTrace)]` across the whole capture
closure — the 116 seed types plus everything the compiler shows they reach —
so that every type a suspension frame can hold has a field-recursive
traversal. **No behavior changes in this brief**: no frame
becomes a named struct, no closure API is deleted, no call site moves, and
neither the GC root walk nor the memory proxy is wired to the new traversal.
Those are steps 3 and 4.

What must be true at the end: every type in the closure implements
`FrameRootTrace`, the implementation comes from the derive rather than from
hand, and a new expression-bearing field added to any of them is traversed
without anyone editing a traversal.

## Context pointers

- Read `AGENTS.md` first.
- `docs/dev/codex/plan-364-frame-roots.md:493` — the itemized list of the
  **116 seed types** that carry `Expr` directly or transitively. This is a
  seed set, **not** the full scope: plan-364 also counts 71 leaf/control
  types, and those were never itemized anywhere. See "How scope is
  determined" below — they are reached by expansion, not by a list.
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

**Take (a), through `rum-runtime::kernel_api`.** That was my preference and
you reached the same conclusion independently on the stopped run, so it is
settled — do not spend the run re-deciding it. `rum-derive` is an
implementation detail of the traversal contract, not a thing stdlib should
know about. Open the trait's visibility only as far as the closure actually
requires, and if (a) turns out to have a concrete problem you can show, STOP
and report it rather than switching to (b) unilaterally.

## How scope is determined (this replaces the previous brief's list rule)

The first version of this brief named `plan-364:493` as the authoritative
scope and simultaneously demanded all 187 types. That is unsatisfiable and
Codex correctly stopped: the list holds exactly 116 entries and is titled
"Expression-bearing custom closure"; the 71 leaf/control types were never
itemized, and `Run<T>` is required by `plan-364:487` while appearing on no
list. My error, not a gap in the plan.

The fix is not a longer list — it is to stop using a list as the authority.
The closure is mechanically derivable and **the compiler is the boundary**:

1. Start from the 116 seeds at `plan-364:493`.
2. Derive `FrameRootTrace` on each.
3. Where a field's type has no traversal, the build fails. That failure names
   the next member of the closure. Derive it too.
4. Repeat until the workspace compiles. The closure is then closed by
   construction, with no list to keep in sync.

This is the same property the whole line is buying — a fact the compiler
enforces rather than one people maintain — so it applies to the scope
boundary as well. **187 is plan-364's estimate, not an acceptance
criterion.** Report the number you actually reach. A different number is a
finding to state, not a discrepancy to hide or to pad toward.

If expansion pulls in a type that clearly has nothing to do with suspension
state, stop and report it rather than deriving your way across the codebase.

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

**Scope is the transitive closure of the seeds, and the compiler decides its
boundary** — see "How scope is determined". A type reached by expansion from a
seed is in scope even though no list names it. A type reachable from neither is
out of scope; if you believe one belongs, report it rather than adding it.

## Deliverable

1. **Derives applied across the whole closure** — the 116 seeds and every
   type the expansion reaches, leaves included. The leaves are not optional: a
   derived no-op today is what makes a future field automatic, which is the
   entire point. `Run<T>` (`crates/rum-stdlib/src/structure_ops/resumable_merge_sort.rs:31`)
   is a known example of a required type that no list names.
2. **Generic carriers bounded correctly** — `Run<T>` and the other generic
   types get `T: FrameRootTrace` only where the parameter appears in a
   traversed field, matching the rule step 1 established.
3. **A completeness check that is mechanical, not a promise.** A test that
   fails if any closure type stops implementing the trait — a list of
   `assert_traces::<T>()` instantiations covering every type you derived.
   Report the count you reached and how it compares to plan-364's estimate of
   187, **without treating 187 as a target**: see "How scope is determined".
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
- your final type count, the seed count you started from, and any type the
  expansion reached that surprised you.

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
5. Completeness — the seed count, the final count, and any type the expansion
   reached that no list named.
6. Stop-loss / open questions — any type that resisted, any closure member you
   believe is wrong (empty if none).
