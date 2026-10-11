# Recursive expression prototype and sharing study

Date: 2026-10-10. This standalone external crate implements recursive Compact
(8-byte slots) and Safe (16-byte slots) trees against pinned original Expr
(32-byte slots). All runtime code remains unchanged. This is a measured prototype,
not a complete evaluator rewrite or an established universal optimization.

## Safe client interface

```rust
use rum_core::Expr;
use rum_recursive_expr::{Compact, View};
let original = Expr::call(Expr::symbol("f"), vec![Expr::small_integer(7)]);
let compact = Compact::from_expr(&original);
if let View::Call { args, .. } = compact.view() {
    assert_eq!(args.len(), 1);
}
let (head, mut args) = compact.into_parts().unwrap();
args.make_mut().push(Compact::small(9));
let changed = Compact::call(head, args);
changed.release_iterative();
```

All unsafe ownership operations are private to owner.rs. A generic aligned Arc
payload stores atoms or Calls; only signed 61-bit Small integers are immediate.
Full i64, BigInt (including manual Big(7)), rational, string and generated symbol
variants retain their original data. Typed views borrow recursive children
without allocation. The current target is 64-bit pointers, tested on Linux
x86_64 GNU with Rust 1.85.0. Client code does not invoke Miri or select LLVM;
the tested platform/toolchain scope is still limited.

Heads remain distinct Arc<N> handles, including immediate heads. Args remains
Arc<Vec<N>>, with standard get_mut/make_mut COW and Weak liveness. Conversion
uses explicit work stacks and separate Head/Args identity memos. Shared DAGs
stay shared within each conversion, avoiding logical expansion. Weak observers
remain in their original allocation namespace; old Weak handles do not monitor
new converted allocations. Separate conversions create separate namespaces.

into_parts moves a unique Call's children or clones their counts when shared.
release_iterative explicitly drains last-owned payloads and children using a
work stack. Ordinary Drop remains recursive. Native checks cover ordinary Drop
at depth 128, benchmark cleanup at 256, and conversion plus explicit release at
10000; these are tested examples, not universal depth bounds. Arbitrarily deep
automatic cleanup and unwinding remain unresolved. Original Expr similarly has
recursive cleanup. No Hash/Eq/serde or HostSession migration is provided.

## Checks and argument

13 library tests, one complete-fixture COW test and two borrow-escape compile-fail
tests pass, both with default and allocation-count configurations. Strict-
provenance Miri passes 13 library tests plus the reduced-fixture COW test under
each of Stacked Borrows and Tree Borrows. The off-by-default allocation counter
is separately run under both models; each of its 15 reduced accounting windows
returns to its starting live-byte count. Native windows also return to baseline.
The 80-level nested-DAG test represents 2^80 logical leaves without unfolding;
Miri uses eight levels. All Atom variants, integer edges, generated identity,
sharing/COW/Weak, last ownership and cross-thread reads are exercised.

See [manual recursive semantics and ownership obligations](SEMANTICS.md).
This is not a complete or machine-checked soundness proof. The independent u64
formal model does not cover this recursive Arc implementation. The previous
2238-pass/7-ignored workspace result is reused for the unchanged kernel; this
stage tests the external crate. Original key source hashes remain identical.

## Requested-byte accounting

Retained bytes below are live heap bytes requested through System, measured
after construction with conversion temporaries gone. Root stack slots are
reported separately. Input source, static interned symbols and preexisting
allocations are excluded through a baseline window. Allocator metadata and
resident pages are not measured. This is not process RSS or the CLI's allocator.

| Dataset | Original | Safe | Compact | Compact / original |
|---|---:|---:|---:|---:|
| wide-integer | 320088 | 160168 | 80160 | 0.250 |
| wide-string | 390088 | 710168 | 630160 | 1.615 |
| balanced | 229368 | 207560 | 163872 | 0.714 |
| shared-dag | 38528 | 29008 | 19400 | 0.504 |
| deep-chain | 18480 | 26656 | 24600 | 1.331 |

Wide fixtures have 10000 integers or seven-byte strings; the balanced fixture
has depth six and fanout four; the shared DAG has 200 Call occurrences sharing
one 1000-element list; the deep chain has 256 single-argument levels and an
immediate shared head. Both controls use the same recursive sharing policy.

Slot packing gives large retained-byte savings for integer lists and shared
subtrees. Strings add an Arc payload allocation per atom; Calls add a payload
Arc above their existing head/list handles. This increases retained memory for
strings and narrow deep chains. Compact beats this Safe control's retained bytes
in every fixture, but this Safe policy is itself more costly than original on
some structures. Wrapper Clone shares immutable atom payloads; original string
Clone copies buffers. Do not attribute all Clone/COW gains to unsafe packing.

Conversion peak bytes and allocation calls are also retained in allocations.tsv.
The wide-integer compact bridge peaks at 582520 additional requested bytes while
retaining 80160; the existing original input remains live outside that window.
Memo/work/value stacks impose material transient costs. Original accounting
builds a fresh fixture directly; wrappers bridge a retained source. Thus original
construction calls/peak and wrapper bridge calls/peak involve different work;
safe-vs-compact bridge comparisons share the same strategy. Repeated full-tree
bridges inside an evaluator would be unacceptable without their own budget.

## Paired timing results

Three process runs, each one warmup plus nine measured rounds. Representations
rotate within each round on the first allowed CPU; no other research workload
runs concurrently. Release uses the standalone default profile and System.
The allocation feature is absent from timing builds. CPU affinity does not
exclude host/VM interference; no statistical significance is claimed.

Each label has 27 samples. Ratios below are medians of within-round pairs:
above one means slower. Scan performs ten full logical traversals, clone/drop
5000 root clones, COW ten append-and-drop operations, bridge three round trips,
and last-drop/release five fresh graphs with setup outside the timer. Original
has no bridge label. Scan checksums match; other output checksums count completed
operations. Complete roundtrip equality is asserted before timing, and a separate
fixture test checks COW's result value, unchanged source and Weak liveness.

| Dataset / operation | Compact / original | Compact / safe |
|---|---:|---:|
| wide-integer / scan | 0.866 | 1.542 |
| wide-integer / clone_drop | 0.481 | 0.999 |
| wide-integer / cow | 0.103 | 0.751 |
| wide-integer / bridge_roundtrip | — | 0.888 |
| wide-integer / last_drop | 0.204 | 1.050 |
| wide-integer / iterative_release | 0.185 | 0.972 |
| wide-string / scan | 2.014 | 0.932 |
| wide-string / clone_drop | 0.488 | 0.998 |
| wide-string / cow | 0.392 | 0.958 |
| wide-string / bridge_roundtrip | — | 0.926 |
| wide-string / last_drop | 2.551 | 1.038 |
| wide-string / iterative_release | 1.810 | 1.000 |
| balanced / scan | 0.992 | 0.939 |
| balanced / clone_drop | 0.489 | 0.999 |
| balanced / cow | 0.846 | 0.913 |
| balanced / bridge_roundtrip | — | 0.990 |
| balanced / last_drop | 1.214 | 1.010 |
| balanced / iterative_release | 0.793 | 0.961 |
| shared-dag / scan | 0.698 | 1.455 |
| shared-dag / clone_drop | 0.480 | 1.001 |
| shared-dag / cow | 0.511 | 0.982 |
| shared-dag / bridge_roundtrip | — | 0.919 |
| shared-dag / last_drop | 1.289 | 1.023 |
| shared-dag / iterative_release | 0.875 | 0.985 |
| deep-chain / scan | 1.430 | 1.423 |
| deep-chain / clone_drop | 0.488 | 0.999 |
| deep-chain / cow | 0.880 | 1.046 |
| deep-chain / bridge_roundtrip | — | 0.953 |
| deep-chain / last_drop | 1.720 | 1.015 |
| deep-chain / iterative_release | 0.575 | 0.996 |

Regressions remain: integer/shared-DAG/deep scans are slower than this Safe
control; strings and deep chains scan slower than original. Last ordinary Drop
is slower than original except for the wide-integer fixture. Ratios near one
do not establish improvement or absence of regressions. This does not pass a
universal replacement gate; there is no full-evaluator speedup result.

## Initial draft diagnosis

The initial draft's generic Tree::view boundary retained six call sites in the
benchmark assembly. Adding an inline hint to that implementation removed them;
the final diagnostic shows zero. Only that hint changed library code. The
benchmark's original iterative-release helper was also corrected to avoid
cloning shared lists, and a fixture COW test was added. The corrected helper is
restricted to the actual fixtures' atom heads and absence of Weak observers;
it is not a general-purpose rum deep-release implementation.

Initial code is reconstructible using CONTROL.json and the reverse of
initial-to-final.patch in evidence/recursive-initial. Hashes verify all changed
sources; other recorded Rust sources match. The initial original iterative-release
rows are unsuitable as direct performance comparators and are explicitly flagged.
Before/after runs are separate phases, not side-by-side binaries. Assembly
diagnostics establish call removal, not complete attribution of every timing
change; benchmark code generation and external load can affect results.

## Reproduce and next gate

On Linux with Python 3, taskset and Rust 1.85.0: bash run-study.sh.
RUN_MIRI=1 enables nightly-2026-10-05 checks with installed miri and rust-src.
CARGO_TARGET_DIR can select native temporary storage. Evidence defaults to
target/study-evidence. analyze.py checks gates, samples, accounting and source
hashes. Committed full evidence is in ../evidence/recursive/; the initial draft's
record is ../evidence/recursive-initial/. Portable reproduction does not itself
collect the optional assembly diagnostic or rerun unchanged workspace tests.

Next work should measure representative rum expression distributions and isolate
allocation/read costs before selecting a narrower migration or another layout.
Any kernel-facing migration still needs rum-vs-rum per-label evaluation A/B,
allocation/RSS, lifecycle and embedding checks. No unsafe enters rum-core or
rum-runtime, and no external engine outputs are used as correctness oracles.
