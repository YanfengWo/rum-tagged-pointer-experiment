# CompactExpr external prototype

Date: 2026-10-10. Pinned upstream: `77029938f999b14dbe5b0755bebdf98e51517ae6`.
The runtime remains unchanged. This standalone crate wraps original Expr values;
it is an experiment before evaluator migration, not a completed optimized rum.

The safe API has new, view, clone, to_expr, into_expr and automatic drop. A signed
61-bit integer is immediate; other values use an immutable Arc payload. Full i64,
BigInt, rationals, strings, generated symbol identity and original nested child
sharing are retained. There is no raw-pointer or tag constructor in the public API.
Nested children retain the original Expr layout; mutable access is through owned
conversion and rum's existing COW operations. The wrapper is Send + Sync.

Measured on x86_64 Linux: Expr 32 bytes, SafeExpr 16 bytes, CompactExpr 8 bytes,
Option<CompactExpr> 8 bytes. SafeExpr is a control with the same integer range,
heap payload and Arc policy, isolating packing from changes to ownership policy.
These numbers describe slots, not total resident memory or evaluator speed.

## Validation

Rust 1.85.0: nine unit tests and one lifetime compile-fail doc test passed.
Strict-provenance Miri nightly-2026-10-05: nine tests passed under each of Stacked
Borrows and Tree Borrows, including count transfer, final release and threads.
The unchanged rum workspace passed 2238 tests with seven existing ignored tests.
Upstream source hashes match the fixed baseline. See [SEMANTICS.md](SEMANTICS.md)
for the manual representation relation and unsafe obligations. This is not a
machine-checked or complete soundness proof.

## Representation timings

The benchmark rotates original/safe/compact order over nine samples per label.
There are 100000 slots; mixed values are 25% heap strings. Scan performs 100
passes, clone/drop 20 passes, construction/conversion one pass. Time is measured
with Instant in a default release build of this standalone crate. All scan and
conversion checksums match. Median wall time follows; ratios above one are slower.

| Dataset / operation | Original ms | Safe ms | Compact ms | Compact / safe |
|---|---:|---:|---:|---:|
| integers / scan | 2.639 | 2.405 | 2.020 | 0.840 |
| integers / clone_drop | 9.977 | 1.195 | 0.885 | 0.740 |
| integers / construct_convert | 0.823 | 0.748 | 0.755 | 1.010 |
| mixed / scan | 2.711 | 3.100 | 3.715 | 1.198 |
| mixed / clone_drop | 17.042 | 4.209 | 4.546 | 1.080 |
| mixed / construct_convert | 0.952 | 1.213 | 1.212 | 0.999 |

The safe control is the principal causal comparison. Original Expr has a different
clone policy: owned strings clone their buffers, whereas both wrappers clone Arc
counts. An improvement against original clone/drop alone therefore does not
establish a benefit from unsafe pointer tagging.

The first version lacked explicit cross-crate inline hints. Its source and timings
are retained in evidence/compact/lib-unhinted.rs and representation-unhinted.tsv.
The final version applies hints consistently to both wrappers. This reduced large
integer-scan overhead in this run, supporting call/inlining cost as a contributing
factor; it does not isolate every optimization or host-timing influence.

Some compact operations still regress against the safe control, especially mixed
scan and clone/drop. External host contention has not been ruled out. No evaluator
speedup, RSS reduction or recursively compact expression tree is demonstrated.
This candidate must not be adopted as an evaluator performance win based on these
microbenchmarks. Next work should explain the residual regressions, design a
recursive borrow-view interface, and run rum-vs-rum per-label evaluator A/B gates.

## Reproduce

From this directory: `bash run-checks.sh`. Rust 1.85.0 and Python 3 are required.
Use `RUN_MIRI=1 bash run-checks.sh` after installing the specified nightly with
miri and rust-src. CARGO_TARGET_DIR and MIRI_SYSROOT may select local caches.
All long commands run through the upstream bounded runner. Evidence is written
under target/evidence by default, without overwriting committed evidence.
