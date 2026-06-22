# Brief 018 — B1b: swap SharedArgs backing to `Arc<[Expr]>`

## Goal

Layer B B1b (`docs/layer-b-node-rewrite-plan.md`). With B1a done (nothing mutates
a `SharedArgs` in place; `DerefMut` removed), swap the backing
`SharedArgs(Arc<Vec<Expr>>)` → `SharedArgs(Arc<[Expr]>)`. This replaces the
`Arc` box + separate `Vec` buffer (a 24-byte `Vec` struct of indirection plus the
element buffer) with a single contiguous allocation holding the refcount header
and the elements inline — fewer live allocations and one cache line instead of
two on arg access. The `Expr::Call { head, args }` variant shape is unchanged, so
Layer A's independent head/args COW (A2/A3/A4) is fully preserved.

## Honest trade-off (state it in your report)

`into_vec` can no longer free-unwrap a unique `Arc<[Expr]>` into a `Vec` (a slice
cannot be moved out of an `Arc`); it must `to_vec()` (copy). After A2 the hot
tier shows `into_vec_total = 0`, so this copy lands only on the ~18 cold builtin
callers of `into_vec` (most already clone). Confirm the hot tier does not
regress; if any hot label regresses, STOP and report.

## Expected, measurable result (the gate)

- `cargo test --workspace` green and the `DumpSave`/`Get` snapshot round-trip
  unchanged — `Arc<Vec<Expr>>` and `Arc<[Expr]>` serialize to the IDENTICAL
  sequence wire format, so this is a no-behavior-change internal swap.
- The B0 allocation counter (`cargo build -p rum-cli --features alloc-count`)
  shows **fewer live bytes** on a held-structure workload (the per-node `Vec`
  struct + buffer collapse into one `Arc<[Expr]>`): compare `peak_live_bytes`
  before/after on e.g. `x = Range[100000]; Length[x]` and a nested-structure
  hold. Report the before/after.
- The primitive tier (clean back-to-back A/B, default build) does NOT regress;
  most labels COW-reuse args after A2 so expect flat-to-slightly-better;
  `into_vec_clone` stays `0` on the hot loops (A1/A2 invariant).

## Context pointers

- All type changes are in `crates/rum-core/src/expr.rs`, the `SharedArgs` block
  (declaration ~line 496 and its impls; `DerefMut` is already gone). Read it
  fully first. `SharedArgs` derives `Clone, Debug, Eq, Hash, Serialize,
  Deserialize` — keep them all (they hold for `Arc<[Expr]>`).
- Compile fallout: changing `Deref<Target>` from `Vec<Expr>` to `[Expr]` breaks
  callers that use `Vec`-only methods through deref — chiefly `.as_slice()`
  (a `Vec` method). Fix each by `.as_ref()` / `&x[..]` (slice), NOT by cloning to
  a `Vec`. The compiler enumerates them (brief-012's earlier attempt saw ~58 such
  read sites — all mechanical now that the mutators are already migrated).

## Scope

- Files you MAY modify:
  - `crates/rum-core/src/expr.rs` (the `SharedArgs` backing + its impls),
  - any site that fails to compile ONLY because the `Deref` target changed
    `Vec<Expr>` → `[Expr]` — fix via slices (`as_ref`/`&x[..]`), list every one.
- MUST NOT change: the `Expr::Call { head: SharedExpr, args: SharedArgs }`
  variant shape, `SharedExpr`, the serde wire format, the `Expr` public API, any
  `mma-hol` clone, `.git`.

## Design (implement exactly)

1. `pub struct SharedArgs(Arc<[Expr]>);` (keep the derives).
2. `new(args: Vec<Expr>)`: `let shared = Self(Arc::from(args)); record_shared_args_alloc(); shared`.
3. `into_vec(self) -> Vec<Expr>`: `self.0.to_vec()` — always copies; call
   `record_into_vec(true)` (can no longer free-unwrap). Keep the signature.
4. `ptr_eq`: `Arc::ptr_eq(&lhs.0, &rhs.0)` (unchanged). `ptr_key(&self) -> usize`:
   the `Arc<[Expr]>` pointer is a fat pointer — return the data address, e.g.
   `Arc::as_ptr(&self.0) as *const Expr as usize`.
5. `Default`: `Self(Arc::from(Vec::new()))`.
6. `Deref`: `type Target = [Expr]; fn deref(&self) -> &[Expr] { &self.0 }`.
7. `AsRef<[Expr]>`: `&self.0`. `From<Vec<Expr>>`: `Self::new(args)`. `PartialEq`:
   `Self::ptr_eq(self, other) || self.as_ref() == other.as_ref()`.
   `IntoIterator for SharedArgs`: `self.into_vec().into_iter()`; `&SharedArgs`:
   `self.iter()`.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

All green. Then the serde / snapshot round-trip specifically — find and run the
existing `DumpSave`/`Get` or serde round-trip test (name the one you ran). Then
the allocation evidence:

```
cargo build --release -p rum-cli --features alloc-count
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'x = Range[100000]; Length[x]' 2>&1 | grep 'rum-profile alloc'
# (and compare against the same probe on the committed parent: git checkout 9d6e310^ -- crates/rum-core/src/expr.rs; rebuild; measure; restore)
```

Report the before/after `peak_live_bytes` (expect after < before). Then the tier
(default build, clean back-to-back A/B — stash this change for the baseline
half):

```
cargo build --release -p rum-cli   # default build, no feature
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste `if-true`, `downvalue-exact-hit`, `downvalue-pattern-hit`, `length-list`,
`head-list`, `part-list` for both halves. Confirm `into_vec_clone` stays 0:

```
RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_REPR=1 target/release/rum-cli --eval \
  'Do[Do[If[True,1,2],{i,1,100000}],{j,1,10}]' 2>&1 | grep 'rum-profile repr'
```

## rum / Rust pitfalls (read twice)

1. `Arc::make_mut`/`try_unwrap` do not work on `Arc<[Expr]>`; `into_vec` copies.
2. The `Deref` target changes `Vec<Expr>` → `[Expr]`; fix `.as_slice()` and any
   `&Vec` fallout with slices, never by cloning on a hot path.
3. Keep the serde derive and the wire format identical — gate on the snapshot
   round-trip.
4. Measure the default RELEASE binary (no feature) for ratios; use the
   `alloc-count` feature build only for the byte counts.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (stash-pop to restore
  after any baseline build). Only the listed files; nothing outside repo + /tmp;
  no network. Commands: `cargo`, release `rum-cli`, the microbench script,
  `python3`. Remove any `.rum_history`. mma-hol clone read-only.
- Stop-loss: same `cargo test` failure twice, OR a hot-tier label regresses, OR
  the snapshot round-trip changes → STOP and report.
- Minimal diff. Paste real `cargo test --workspace` summary, the serde test, the
  before/after peak_live_bytes, the A/B rows, and the repr counter line.

## Report format

1. Per-file changes + every compile-fallout site fixed.
2. The serde-compatibility argument + which round-trip test you ran.
3. Verification — verbatim fmt, `cargo test --workspace`, the serde test, the
   before/after `peak_live_bytes`, the A/B rows, the repr line.
4. Stop-loss / open questions (empty if none).
