# Brief 017 — B1a: migrate in-place SharedArgs mutators to build-then-freeze

## Goal

Layer B B1a (`docs/layer-b-node-rewrite-plan.md`). Before the args backing can
become `Arc<[Expr]>` (B1b — one contiguous allocation instead of `Arc` box + `Vec`
buffer), nothing may mutate a `SharedArgs` **in place**, because `Arc<[Expr]>`
cannot be mutated through `DerefMut` (`Arc::make_mut` needs `T: Clone`; `[Expr]`
is unsized). This brief removes the `DerefMut for SharedArgs` impl and rewrites
every resulting in-place mutation to **build-then-freeze**: take an owned
`Vec<Expr>`, mutate the `Vec`, then reconstruct the `SharedArgs` / call. The
backing stays `Arc<Vec<Expr>>` for now (B1b does the type swap). This is a
**behavior-preserving refactor with zero perf delta** — its only purpose is to
make B1b a clean swap.

## The exact sites (compiler-enumerated)

Removing `impl DerefMut for SharedArgs` (crates/rum-core/src/expr.rs ~line 559)
produces **41 `error[E0596]: cannot borrow ... SharedArgs as mutable`** errors
across these 12 files — fix every one:

- `crates/rum-runtime/src/runtime/structure_ops/exact_paths.rs`
- `crates/rum-runtime/src/runtime/structure_ops/replace/replace_part.rs`
- `crates/rum-runtime/src/runtime/structure_ops/sequence/delete.rs`
- `crates/rum-runtime/src/runtime/structure_ops/sequence/insert.rs`
- `crates/rum-runtime/src/runtime/structure_ops/surface.rs`
- `crates/rum-runtime/src/runtime/rewrite_builtins.rs`
- `crates/rum-runtime/src/runtime/definition_updates.rs`
- `crates/rum-runtime/src/runtime/definition_catalog/traversal_continuation.rs`
- `crates/rum-runtime/src/runtime/parse_and_message_helpers.rs`
- `crates/rum-runtime/src/runtime/suspension/collections.rs`
- `crates/rum-runtime/src/runtime/suspension/rewrite_rules.rs`
- `crates/rum-runtime/src/runtime/suspension/traversal.rs`

(Read-only uses of `SharedArgs` via the immutable `Deref` are NOT affected —
this brief keeps `Deref<Target=Vec<Expr>>`, so `.iter()`, `.len()`, `&args[i]`,
`.as_slice()`, etc. all keep working. Only the ~41 MUTATION sites break.)

## Scope

- Files you MAY modify: `crates/rum-core/src/expr.rs` (remove ONLY the `DerefMut`
  impl) and the 12 runtime files above (+ a runtime test module if you add a
  regression). MUST NOT change anything else, `SharedArgs`'s other impls
  (`Deref`, `new`, `into_vec`, serde), `Expr`, any `mma-hol` clone, `.git`.

## Design (the build-then-freeze pattern)

For each mutation site, the value being mutated is a `SharedArgs` (often a field
of a suspension/continuation state, or a local). Rewrite so the mutation happens
on an owned `Vec<Expr>`, then re-wrap:

- If the site owns the `SharedArgs` and produces a new one:
  `let mut v = Expr::unwrap_shared_args(shared); v.push(x); /* ... */; let shared
  = SharedArgs::new(v);` (or build the call directly from `v`).
- If the site mutates a `SharedArgs` field in place across a loop, hoist the
  owned `Vec` out of the field, mutate the `Vec` through the loop, and store the
  re-frozen `SharedArgs` back once.
- For `iter_mut`: collect/clone into a `Vec`, mutate, re-freeze.

`Expr::unwrap_shared_args(args) -> Vec<Expr>` and `SharedArgs::new(vec)` already
exist; `Expr::call` / `call_with_shared_args` build call nodes. Keep the SAME
final values — only the carrier mechanics change.

**Why this is correct (state it in your report):** the args value is identical;
the mutation now happens on an owned `Vec` that is frozen into a `SharedArgs`
exactly where the old code finished its in-place mutation. No observable behavior
changes — it is a refactor. Removing `DerefMut` is sound because after this brief
nothing mutates a `SharedArgs` in place.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

All green (the structure-ops builtins — Insert/Delete/ReplacePart/sequence ops —
the rewrite builtins, definition updates, and suspension/resume paths are exactly
what these 41 sites implement, so the workspace suite is the behavior gate). Then
prove ZERO perf delta (this is a pure refactor) with a clean back-to-back A/B —
the change is uncommitted, so `git stash` it for the baseline half:

```
# baseline: git stash; cargo build --release -p rum-cli; microbench -> /tmp/cdx_b017_base
# new:      git stash pop; cargo build --release -p rum-cli; microbench -> /tmp/cdx_b017_new
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste `if-true`, `downvalue-exact-hit`, `downvalue-pattern-hit`, `length-list`,
`head-list`, `part-list` for both halves. They must be equal within noise (this
brief changes only cold builtin mutator paths, not the hot microbench loops).

## rum / Rust pitfalls (read twice)

1. Keep `Deref` (immutable) for `SharedArgs`; remove ONLY `DerefMut`. Read sites
   must keep compiling unchanged.
2. Each rewrite must preserve the exact final args value and ordering; the
   workspace suite is the gate (Insert position semantics, Delete, ReplacePart,
   Sequence splicing, suspension resume all live in these files).
3. Do not deep-clone on a hot path; these are cold builtin/suspension paths, so a
   `to_vec`/`unwrap_shared_args` there is fine, but do not introduce clones on the
   evaluator hot path.
4. Measure the RELEASE binary; the A/B must be back-to-back with nothing else
   running.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (stash-pop to restore
  after the A/B baseline half).
- Only the listed files; nothing outside repo + `/tmp`; no network. Commands:
  `cargo`, release `rum-cli`, the microbench script, `python3`. Remove any
  `.rum_history`. mma-hol clone read-only.
- Stop-loss: same `cargo test` failure twice, OR a mutation site that cannot be
  cleanly build-then-frozen without a behavior change → STOP and report that site
  (do NOT force it). It is fine to report partial progress; this is the careful
  migration the rewrite plan calls for.
- Minimal, mechanical diff. Paste the real `cargo test --workspace` summary and
  the back-to-back A/B rows; never fake a result.

## Report format

1. Per-file: how many sites and the build-then-freeze shape used (one line each).
2. The correctness argument (value identical; mutation moved to an owned Vec).
3. Verification — verbatim `cargo fmt --check`, `cargo test --workspace` summary,
   the back-to-back A/B rows (both halves).
4. Stop-loss / open questions (empty if none).
