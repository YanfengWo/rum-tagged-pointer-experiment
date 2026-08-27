# brief-365 — the field-derived root-traversal substrate

## Goal

Step 1 of plan-364. Add a `rum-derive` proc-macro crate and a runtime-owned
root-traversal contract, `FrameRootTrace`, that a `#[derive]` generates by
recursing over a type's named fields. Two root kinds exist and both matter:
`Expr` and bare `Symbol`.

**Nothing about suspension changes in this brief.** No frame becomes a named
struct, no closure API is deleted, no queue is touched, and `gensym_gc.rs`
keeps its current root walk. This step builds the substrate only, so it can be
gated on its own before anything depends on it.

The property being built: a type that gains a new expression-bearing field is
covered automatically, and a type that gains a field whose type has no
traversal fails to compile. Later steps rely on both.

## Context pointers

- Read `AGENTS.md` (operational notes, Testing Policy) first.
- `docs/dev/codex/plan-364-frame-roots.md` — the full plan, its 187-type
  capture closure, and the director's rulings at the bottom. **Read the
  rulings**: two of Codex's own recommendations were overturned there.
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:236` —
  `for_each_symbol_in_expr`, and `:249` `seed_live_gensyms_from_expr`. These
  already do the recursive symbol scan inside an `Expr`. The new traversal
  hands whole `Expr` values to a visitor; it does **not** reimplement that
  descent.
- `crates/rum-runtime/src/runtime/suspension.rs:1` — the module that gains a
  `trace` submodule.
- `Cargo.toml:3` — the nine workspace members; `:25-35` — the ten direct
  dependencies.
- Acceptance is mechanical: the named tests decide, not prose.

## The two rulings that override the draft

1. **`rum-derive` is NOT published and carries no version metadata.** rum's
   release form is source-available GitHub only, with no crates.io
   publication; no crate in this workspace has a `publish` field. Add it as a
   plain **path** dependency. Do not add `version`, do not add publish
   metadata, do not reason about publication ordering.
2. **Bare `Symbol` is a root, never a no-op.** A `Block` frame captures a
   `Symbol` with no `Expr` alongside it, and treating `Symbol` as an empty
   visit reproduces a demonstrated leak (plan-364, path four). The visitor has
   two methods and both are load-bearing.

## Scope

Files you MAY modify:

- `Cargo.toml` — add the workspace member and the workspace dependency entry.
- `Cargo.lock` — **MECHANICAL ONLY**, the result of adding the crate. The Goal
  is a dependency change, so this file is in scope for that reason alone.
- `crates/rum-derive/**` — new crate.
- `crates/rum-runtime/Cargo.toml` — add the path dependency.
- `crates/rum-runtime/src/runtime/suspension.rs` — register the new submodule.
- `crates/rum-runtime/src/runtime/suspension/trace.rs` — new, the trait and
  the base implementations.
- Tests for the above, sited per this repo's existing convention.

Files/areas you MUST NOT touch:

- `crates/rum-runtime/src/runtime/suspension/core.rs` — no frame, queue, or
  API change in this brief;
- `crates/rum-runtime/src/runtime/scoping/gensym_gc.rs` — the root walk is
  wired up in a later step, not this one;
- `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`;
- any of the 187 capture types — applying the derive to them is step 2;
- `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation and any public
  type/serde shape;
- `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`.
- Do NOT add unconditional cost to the default evaluation path.

If you conclude a file outside this list must change, STOP and report it
rather than widening the diff.

## Deliverable

1. **`crates/rum-derive`** — a `proc-macro = true` crate, edition 2024, MSRV
   1.85, depending only on `syn`/`quote`/`proc-macro2` (already in the lock
   file via `serde_derive`; confirm the resolved versions and report them).
2. **`#[derive(FrameRootTrace)]`** handling: named-field structs, tuple
   structs, unit structs, enums with all three variant shapes, and generics —
   where a generic parameter that appears in a traversed field gets a
   `T: FrameRootTrace` bound. Unions are rejected with a clear compile error.
3. **The trait and its visitor**, owned by `rum-runtime`, with two visit
   methods — one for `Expr`, one for `Symbol`. Traversal **borrows**; it never
   clones an `Expr`, a container, or anything else.
4. **Base implementations** for what the 187-type closure actually needs:
   `Expr`, `Symbol`, the shared-expression types, the containers in use
   (`Vec`, `VecDeque`, `Option`, `Box`, `Arc`, `Arc<[T]>`, `HashMap`,
   `BTreeMap`, tuples), and exact no-op implementations for the scalar and
   function-pointer leaves. **No blanket implementation for `Fn`, `Any`, or
   any trait object** — those are precisely the shapes the design forbids.
5. **The manual-implementation allowlist**, sorted, with the mechanical test
   that reads it. Per the user's ruling this allowlist is the accepted
   enforcement boundary, so it must be a real gate from the day it lands, even
   though nothing is on it yet beyond the base implementations this brief
   writes.
6. **Tests**, including the negative calibration below.

## rum / Rust pitfalls (read twice)

1. **Do not reimplement the symbol descent.** `Expr` is visited once, as a
   whole, by `visit_expr`; the recursive walk into its arguments already
   exists at `gensym_gc.rs:236` and stays there. A derive that descends into
   `Expr` internals duplicates a mechanism — the plan's explicit
   "one root walk, one traversal" rule.
2. **`Symbol` is a root.** See ruling 2. A no-op implementation for `Symbol`
   would compile, pass every test you are likely to write, and silently
   preserve a demonstrated leak.
3. **Borrowed traversal only.** `Expr`/`SharedExpr`/`SharedArgs` are
   `Arc`-shared. No deep clone, no `to_vec()`, no collecting into an owned
   container anywhere in a traversal path.
4. **Negative calibration is required, not optional.** A traversal test that
   passes because it visits nothing is the failure mode this whole line
   exists to prevent. Prove your tests can fail: take a nested type, remove
   one field from the traversal by hand, show the test goes RED, restore it,
   show it goes GREEN, and paste both. A test suite nobody has seen fail is
   not evidence.
5. **Generic bounds must be necessary, not blanket.** A generic parameter that
   never appears in a traversed field must not acquire a `FrameRootTrace`
   bound, or unrelated types stop compiling in later steps.
6. **This step changes no behavior.** If a workspace test's result changes,
   something is wrong — investigate rather than adjusting the test.
7. **Stop-loss is real.** If the derive does not resolve after two serious
   attempts at a shape, STOP and report which shape and why.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste the output verbatim:

```
cargo fmt --check
cargo test -p rum-derive
cargo test -p rum-runtime
cargo test --workspace
```

The workspace total must be greater than or equal to today's, with **zero**
failures, and no existing test may change its result. State today's baseline
total before you start and the new total after.

Then paste, verbatim:

- the resolved `syn`/`quote`/`proc-macro2` versions from `Cargo.lock`;
- the negative-calibration RED run and the restored GREEN run (pitfall 4);
- the `Cargo.toml` diff for the workspace member and dependency entries,
  showing there is no `version` and no publish metadata on `rum-derive`.

No microbench gate: this brief changes no evaluation path. Say that explicitly
in your report rather than running one — and if you believe you touched a hot
path, stop and say so instead.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave all changes in
  the working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write
  that in the report instead.
- Touch nothing outside this repo except `/tmp`; no network. `cargo` must run
  `--offline`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only; no external-engine comparisons or outputs.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are zero-`unsafe`.
- **No new third-party dependency beyond `syn`/`quote`/`proc-macro2`**, which
  are already in the lock file. If you believe another is needed, STOP and
  report.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside
  the task.
- Bounded runs: `scripts/run-bounded.py --timeout SECONDS -- <command>`; state
  the bound and where you derived it from a prior measured run.
- You MUST paste real verbatim output. If a run did not reach green, say so
  explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command, with verbatim output, including the baseline
   and new workspace totals.
3. The negative calibration — the RED output and the GREEN output.
4. Derive-contract findings — any shape that resisted, any bound you had to
   add and why.
5. Stop-loss / open questions (empty if none).
