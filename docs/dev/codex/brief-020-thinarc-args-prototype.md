# Brief 020 — THROWAWAY prototype: SharedArgs = thin-pointer inline slice (triomphe::ThinArc)

## Goal (measurement prototype — will be reverted)

Test the one untried args-node layout: a THIN-pointer inline slice. B1b's
`Arc<[Expr]>` is a FAT pointer (16B) that grew the `Expr` enum and regressed
everything; the current `Arc<Vec<Expr>>` is a thin pointer (8B) but uses two
allocations (Arc box + separate Vec buffer). A `triomphe::ThinArc<(), Expr>`
stores the length in the header so the pointer stays THIN (8B — no `Expr`
growth, unlike B1b) while inlining the elements into ONE allocation (no separate
Vec buffer). It keeps the `Expr::Call { head, args }` variant shape and Layer A's
independent head/args sharing.

Prediction to test: this is likely NO-GO because building the inline slice
requires MOVING the elements out of the transient `Vec` (memcpy of `Expr`
structs) that the construction pipeline builds, which `Arc::new(Vec)` avoids, and
the saved allocation is largely already eliminated by Layer A's reuse (the
`Arc<CallData>` prototype showed only `-30` allocs). Measure it.

## Design

1. `crates/rum-core/Cargo.toml`: add `triomphe = "0.1"` (needs network to fetch;
   the proxy env is set). Use its `serde` feature if it has one, else manual serde.
2. `crates/rum-core/src/expr.rs`: `pub struct SharedArgs(triomphe::ThinArc<(), Expr>);`
   Read triomphe's docs for the exact API. Roughly:
   - `new(args: Vec<Expr>)`: build the ThinArc from `((), args.iter())` /
     `ThinArc::from_header_and_iter((), args.into_iter())` (an `ExactSizeIterator`),
     then `record_shared_args_alloc()`.
   - `Deref`/`AsRef` → the inline `[Expr]` slice (triomphe ThinArc derefs to a
     `HeaderSlice` whose `.slice` is the `[T]`).
   - `into_vec(self) -> Vec<Expr>`: copy the slice to a `Vec` (`self.<slice>.to_vec()`),
     `record_into_vec(true)`.
   - `Default` (empty), `ptr_key`/`ptr_eq` (ThinArc exposes a pointer), `PartialEq`
     (`ptr_eq || slice ==`), `From<Vec<Expr>>`, `IntoIterator` (`into_vec().into_iter()`).
   - serde: must ROUND-TRIP (serialize→deserialize→equal) so the `DumpSave` test
     passes; need not match the old wire bytes. Serialize the slice as a sequence;
     deserialize into a `Vec` then `SharedArgs::new`.
   `SharedExpr` is UNCHANGED (stays `Arc<Expr>`); only `SharedArgs` changes.
3. Fix compile fallout from the `Deref` target (`Vec<Expr>` → `[Expr]`) as in
   B1a/B1b: use slices (`as_ref`/`&x[..]`), never clone on a hot path.

## Verification (MANDATORY — paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

All green incl. the `DumpSave`/`Get` round-trip (name it). Then the DECISIVE
machine-independent gate — does it actually reduce allocations, or is it
reuse-minimized to ~nothing like `Arc<CallData>`?

```
cargo build --release -p rum-cli --features alloc-count
# exact-hit (A3 reuses args -> expect ~no change):
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'fExact[1]=2; Do[Do[fExact[1],{i,1,100000}],{j,1,10}]; 1' 2>&1 | grep 'rum-profile alloc'
# pattern-hit (rebuilds Plus[1,1] each call -> CONSTRUCTS args, the real test):
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'fP[x_Integer]:=x+1; Do[Do[fP[1],{i,1,100000}],{j,1,10}]; 1' 2>&1 | grep 'rum-profile alloc'
# held large structure:
RUM_PROFILE_ALLOC=1 target/release/rum-cli --eval 'x = Range[100000]; Length[x]' 2>&1 | grep 'rum-profile alloc'
```

Report count/bytes/peak_live for all three, vs the `Arc<Vec>` baseline measured
the SAME way (build the baseline by `git stash` of this change → `--features
alloc-count` → measure → restore). The key number: does **pattern-hit's alloc
count drop meaningfully**, or stay flat (reuse already minimized it)?

Then a rough timing back-to-back A/B (default build, stash baseline), min-of-2:

```
cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh --engine both --output-dir <dir>
```

Paste `if-true`, `downvalue-exact-hit`, `downvalue-pattern-hit`, `sameq-integer`,
`length-list`, `head-list`, `part-list`, `list-literal-small` for both halves.
(The director will re-run a rigorous order-alternated interleaved A/B; your
back-to-back is the first signal.)

## Hard rules

- Do NOT `git commit`/branch/push; leave the working tree (stash-pop to restore).
  Throwaway prototype — correctness (tests + serde round-trip) + faithful
  measurement matter, not elegance. Only `crates/rum-core/{Cargo.toml,src/expr.rs}`
  + compile-fallout files. Remove any `.rum_history`. No `mma-hol` edits.
- Stop-loss: same `cargo test` failure twice, OR the serde round-trip changes, OR
  triomphe's API can't express the interface cleanly after one real attempt →
  STOP and report what blocked you.
- Paste real test summary, the THREE alloc lines (+ baselines), and the A/B rows.

## Report format

1. What changed (SharedArgs backing, constructors, serde, fallout).
2. The alloc evidence: pattern-hit / exact-hit / Range counts vs Arc<Vec>
   baseline — did pattern-hit's count actually drop?
3. Verification — fmt, `cargo test --workspace`, the serde test, alloc lines, A/B.
4. Your read: did allocations drop, and did the hot tier move?
