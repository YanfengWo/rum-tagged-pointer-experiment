# Brief 019 — THROWAWAY perf prototype: Arc<CallData> single-allocation node

## Goal (this is a measurement prototype — it will be reverted)

Before committing to the ~572-site `Expr::Call { head, args }` accessor migration
that a real single-allocation node needs, measure whether the single-alloc node
actually pays off. Do it WITHOUT the migration via a trick: make both
`SharedExpr` and `SharedArgs` newtypes over a SHARED `Arc<CallData>`, where
`CallData { head: Expr, args: Vec<Expr> }`. A `Call` node's `head` and `args`
fields hold (clones of) the SAME `Arc<CallData>`. The `Expr::Call { head, args }`
variant shape is UNCHANGED, so none of the 572 destructure sites change — they
read `head`/`args` through the (unchanged) `Deref`/`as_ref` accessors, which now
project onto `CallData.head` / `CallData.args`.

This faithfully reproduces the single-alloc node's allocation behavior (one
`Arc<CallData>` + the args `Vec` buffer = 2 allocs/node vs the current 3:
separate head `Arc<Expr>` + args `Arc<Vec>` + buffer), with a THIN pointer (no
`Expr` enum growth) and move-construction (no element copy — the failure modes
that killed `Arc<[Expr]>` in B1b). It is a CONSERVATIVE lower bound: the real
single-field `Expr::Call(Arc<CallData>)` would also shrink `Expr::Call` from 16
bytes to 8; this prototype keeps it at 16, so a real migration can only do
better.

**This is throwaway.** Implement, measure, report. Do NOT optimize for code
cleanliness; optimize for a faithful measurement with the variant shape intact.

## Design

In `crates/rum-core/src/expr.rs`:

1. `struct CallData { head: Expr, args: Vec<Expr> }` (private).
2. `pub struct SharedExpr(Arc<CallData>);` and `pub struct SharedArgs(Arc<CallData>);`
   — both wrap `Arc<CallData>`. A `Call` node always has `head` and `args` whose
   `Arc<CallData>` are the SAME allocation (ptr-equal).
3. Accessors (keep the EXISTING public method/trait surface so callers and the
   572 destructure sites are unchanged):
   - `SharedExpr`: `Deref<Target=Expr>` → `&self.0.head`; `AsRef<Expr>` likewise;
     `into_owned(self) -> Expr` → `self.0.head.clone()`; `ptr_key`/`ptr_eq` on
     `Arc::as_ptr(&self.0)`.
   - `SharedArgs`: `Deref<Target=[Expr]>` → `&self.0.args[..]` (or keep
     `Target=Vec<Expr>` → `&self.0.args` if simpler and the call sites need it);
     `AsRef<[Expr]>` → `&self.0.args`; `into_vec(self) -> Vec<Expr>` →
     `self.0.args.clone()` (record_into_vec(true)); `ptr_key`/`ptr_eq` on
     `Arc::as_ptr(&self.0)`; `Default`. Keep `Deref`/iteration working as today.
4. Constructors (the ONLY places that build a `Call`):
   - `Expr::call(head: Expr, args: Vec<Expr>)`: `let cd = Arc::new(CallData {
     head, args }); record_shared_args_alloc(); Expr::Call { head:
     SharedExpr(cd.clone()), args: SharedArgs(cd) }`.
   - `Expr::call_shared(head: SharedExpr, args: SharedArgs)` and
     `call_with_shared_args(head: Expr, args: SharedArgs)`: **preserve Layer A
     reuse** — if the incoming `head` and `args` already share the same
     `Arc<CallData>` (`Arc::ptr_eq`), reuse it directly (no rebuild); otherwise
     build a fresh `CallData { head: <the head Expr>, args: <the args Vec> }` and
     share it into both fields. This is the load-bearing case: A3's candidate and
     A4's no-change rebuild pass the call's own head+args, which ARE ptr-equal, so
     they must hit the no-rebuild path.
5. **serde (throwaway-grade — must ROUND-TRIP, need NOT match the old wire
   format):** the current `#[derive(Serialize, Deserialize)]` on `Expr` will not
   work (deriving the two `Call` fields would serialize the whole `CallData`
   twice and cannot merge them back on deserialize). Replace it with a **manual
   `Serialize`/`Deserialize` for `Expr`**: `Atom` as today; `Call` serialized as
   a `(head: Expr, args: Vec<Expr>)` shape projecting `self.0.head` /
   `self.0.args`, and deserialized by reading `head` + `args` and rebuilding via
   `Expr::call(head, args)` (which shares one `CallData`). It only needs to be
   self-consistent (serialize→deserialize→equal), so the existing `DumpSave`/
   `Get` round-trip test passes; it does NOT need to match the pre-prototype wire
   bytes (we are not loading old snapshots in this probe). `SharedExpr` /
   `SharedArgs` then do not need their own derived serde.
6. Fix compile fallout from the `SharedArgs` `Deref` target change exactly as in
   B1a/B1b (slices), if any.

## Verification (MANDATORY — paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

All green (especially the `DumpSave`/`Get` round-trip test — name it). Then the
allocation evidence (the prototype should reduce node allocs 3→2):

```
cargo build --release -p rum-cli --features alloc-count
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'Do[Do[fExact[1],{i,1,100000}],{j,1,10}]; fExact[1]=2; 1' 2>&1 | grep 'rum-profile alloc'
```

Report count/bytes vs the B0 baseline (`count=3001487` before). Then the decisive
hot-tier perf A/B — clean back-to-back, default build, the change is uncommitted
so `git stash` it for the baseline half, min-of-2 each half:

```
cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste `if-true`, `downvalue-exact-hit`, `downvalue-pattern-hit`, `sameq-integer`,
`length-list`, `head-list`, `part-list` for BOTH halves (baseline = stashed
Arc<Vec>, new = the Arc<CallData> prototype). This is the go/no-go number.

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (stash-pop to restore
  after the baseline half). Only `crates/rum-core/src/expr.rs` + any pure
  compile-fallout file. No `mma-hol` edits. Remove any `.rum_history`.
- This is a throwaway prototype: correctness (tests + serde round-trip green) and
  a faithful A/B matter; code elegance does not.
- Stop-loss: same `cargo test` failure twice, OR the serde round-trip changes →
  STOP and report (do not fake serde).
- Paste real test summary, the alloc line, and the back-to-back A/B rows.

## Report format

1. What you changed in expr.rs (CallData, the two newtypes, constructors,
   serde).
2. Confirmation that A3/A4 reuse hits the ptr-eq no-rebuild path (state how).
3. Verification — fmt, `cargo test --workspace`, the serde test, the alloc line,
   the back-to-back A/B rows (both halves).
4. The go/no-go read: did the hot tier improve, hold, or regress?
