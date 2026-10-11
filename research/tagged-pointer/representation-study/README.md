# Mixed-value regression study and borrowed interface

Date: 2026-10-10. This study compares the pinned published V1, an inline-only
control, V2, a safe enum control and original rum Expr in the same process.
The fixed rum upstream remains 77029938; runtime source has not changed.

## Change and diagnosis

V1 uses integer tag 001 and heap tag 010, recovering the original heap pointer
by masking low bits. V2 uses integer tag 001 and an unchanged aligned heap
pointer with low bits 000. Both remain 8-byte owners with the same 61-bit inline
integer range and immutable Arc payload. All other values, including full i64
and manually constructed Big integers, retain the original representation.

The pinned V1 source is byte-identical to the previously published commit
ff3327b74b0381bd1dce7ff51bb5b92f37c69251. The inline-only control changes only
#[inline] on its private heap_ptr helper. PIN.json and CONTROL.json record these
facts; package names/path metadata are adjusted to allow side-by-side compilation.

Assembly inspection of equivalent string-length projections shows:

* V1 still calls heap_ptr, although its public view method has an inline hint.
* Hinted V1 removes that call but emits pointer masking plus extra shifts and
  conditional selection in this projection.
* V2 has neither the helper call nor the masking operation in this projection.

This is evidence about the inspected functions, not a complete CPU profile of
every timing loop. The inline-only control shows that adding more inline hints
is insufficient: pure-integer scan regresses substantially in that control.
The encoding simplification lets this compiler generate a simpler projection;
benefits vary with heap fraction and arrangement. It does not eliminate Arc's
atomic-count costs or extra payload allocations.

## Borrowed interface

V2 adds borrow() returning ExprRef / AtomRef / IntRef, constructed entirely in
safe Rust. It projects every original Expr and Atom variant. Primitive integers
and symbols are copied; strings, BigInt and rationals borrow original storage;
Call borrows original SharedExpr and SharedArgs. Projection performs no allocation
or owned clone. Manual Big(7) stays Big, and IntRef::as_i64 returns None for it.

The references are bounded by the owner's lifetime. Two compile-fail doc tests
check Expr and typed-string escape. Tests also check string-buffer identity,
generated symbol identity, child-sharing identity and Weak liveness. This is a
read-only migration interface: Call children still use original Expr; no recursive
compact tree, replacement serializer, mutable projection or kernel migration exists.
See [the current manual semantics](../compact-expr/SEMANTICS.md).

## Validation and methods

Rust 1.85.0: 13 prototype unit tests, two compile-fail doc tests and two cross-
representation compatibility tests passed. Strict-provenance Miri on nightly-
2026-10-05 passed 13 prototype tests plus two compatibility tests under each of
Stacked Borrows and Tree Borrows. The unchanged workspace passed 2238 tests with
seven existing ignored tests in this stage; fixed upstream source hashes match.
These checks do not constitute a complete or machine-checked soundness proof.

Three process runs each have one warmup plus 15 measured rounds. Five
representations rotate order within each round on one allowed Linux CPU. Each
dataset has 100000 slots; scan makes 50 passes, clone/drop ten passes and
construct/convert one pass. Construction clones an input Expr before adapter
round trip; original uses an identity adapter. String values contain seven bytes.
All results' checksums match. Default standalone release configuration is used.

There are 45 measured samples per label. Ratios below are medians of same-round
paired timings pooled over the three runs; ratios above one mean slower. Full
times, per-run paired ratios and min/max values are in evidence. CPU affinity
does not eliminate host/VM interference; no statistical significance is claimed.

| Dataset / operation | V2 / V1 | Hinted V1 / V1 | V2 / hinted V1 | V2 / safe |
|---|---:|---:|---:|---:|
| heap00 / scan | 0.991 | 1.790 | 0.549 | 1.035 |
| heap00 / clone_drop | 0.736 | 0.953 | 0.780 | 0.837 |
| heap00 / construct_convert | 0.998 | 1.000 | 0.998 | 1.003 |
| heap25-blocks / scan | 0.745 | 1.020 | 0.740 | 0.940 |
| heap25-blocks / clone_drop | 0.893 | 0.923 | 0.968 | 0.931 |
| heap25-blocks / construct_convert | 1.007 | 1.001 | 1.003 | 1.008 |
| heap25-strided / scan | 0.669 | 0.943 | 0.711 | 0.893 |
| heap25-strided / clone_drop | 0.902 | 0.908 | 0.995 | 0.977 |
| heap25-strided / construct_convert | 0.982 | 0.980 | 1.002 | 0.990 |
| heap25-shuffled / scan | 0.842 | 0.882 | 0.963 | 1.000 |
| heap25-shuffled / clone_drop | 0.780 | 0.807 | 0.967 | 0.936 |
| heap25-shuffled / construct_convert | 0.996 | 0.987 | 1.000 | 1.013 |
| heap50-blocks / scan | 0.722 | 0.749 | 0.969 | 0.940 |
| heap50-blocks / clone_drop | 0.923 | 0.925 | 1.000 | 0.974 |
| heap50-blocks / construct_convert | 0.994 | 0.992 | 1.002 | 1.007 |
| heap100 / scan | 0.803 | 0.794 | 1.008 | 0.944 |
| heap100 / clone_drop | 0.928 | 0.922 | 1.007 | 0.994 |
| heap100 / construct_convert | 1.004 | 0.998 | 1.004 | 1.005 |

V2 improves the measured mixed scan labels relative to V1. It still has small
regressions against the safe control, including pure-integer scan and some
construction/conversion labels. Do not treat ratios near one as established
speedups or as proof of no regression. Original Expr clones string buffers,
whereas all wrappers clone Arc counts; that ownership-policy difference is
separate from pointer packing. Allocation/RSS and full evaluator A/B gates
remain outstanding. The earlier V1 timing table remains historical; its different
harness and scheduling do not predict these new ratios.

The typed borrow API was also timed against V2's existing lightweight view using
the same values. Their order alternates before/after the V2 scan by round, and
checksums match. Both make 50 passes; the following ratios compare those paired
scans. These measurements cover integer/string reads, not every Atom family.

| Dataset | Typed borrow / lightweight view |
|---|---:|
| heap00 | 0.993 |
| heap25-blocks | 1.037 |
| heap25-strided | 1.100 |
| heap25-shuffled | 0.953 |
| heap50-blocks | 0.942 |
| heap100 | 1.018 |

The typed interface can still cost more, including approximately 10% on the
25%-heap strided data in this run, despite performing no allocation. Keep its
readability benefit separate from performance acceptance. Its hot-path use needs
local measurement; the lightweight view remains available.

## Reproduce

On Linux with Rust 1.85.0, Python 3 and taskset: bash run-study.sh.
Set RUN_MIRI=1 after installing the specified nightly's miri and rust-src.
CARGO_TARGET_DIR may select a native temporary build directory. Outputs default
to target/study-evidence, leaving committed evidence untouched. The portable
script checks formats, values and timings; assembly and full workspace logs were
also collected by the local bounded research runner. No unsafe was added to
rum-core or rum-runtime. The next integration gate is rum-vs-rum evaluation,
per-label timings, allocation/actual memory and embedding behavior checks.
