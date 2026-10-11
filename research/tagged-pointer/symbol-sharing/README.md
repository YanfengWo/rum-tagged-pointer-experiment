# Bounded Symbol payload sharing

Opt-in safe conversion policy over the unchanged Compact/Safe recursive owners
from revision 209b08af and the pinned expression corpus from revision 1f3c21be.
The kernel and all earlier source/evidence records remain unchanged. New unsafe
code is absent; the study's optional allocation counter reuses the existing
counter source. The manual conditional refinement argument is in SEMANTICS.md.

`shared::<Compact>(&expr, 128)` returns an owned graph and cache statistics;
`shared::<Safe>` uses the identical policy. The table holds at most 128 distinct
full Symbol values during one conversion. It has no global state, eviction or
post-return lifetime. Generated identity and context are part of the key.
Head/Args identities remain separate even when their Symbol payloads share.
Zero capacity uses the Plain bridge and zero diagnostic counters. HashMap
capacity is not bounded to exactly the entry count. The library is a standalone
research option; rum's default conversion and evaluator do not use this cache.

Plain and Shared use the same derived bridge, with only the atom factory changed
from the original pinned algorithm. analyze.py verifies that derivation and
unchanged dependency/source hashes. This prevents combining a changed graph
conversion algorithm with the Symbol sharing policy in the reported comparison.

## Checks and corpus

Default/all-feature tests each pass eight focused tests, as do strict-provenance
Stacked/Tree Borrows Miri. Tests exercise both owners' repeated Symbols, full
context/generated identity, zero/bounded fallback, alias classes, COW/Weak,
non-Symbol variants, owned extraction, DAG conversion and shared leaf release.
Explicit release covers 10,000 levels natively and 24 under Miri. Ordinary
Call Drop remains recursive. The new tests compose the old owner API; they
do not replace its earlier checks or establish full unsafe API soundness.

The study reuses all 112 lowered bodies and 15 evaluated setup results from
expression-census, and adds one explicitly synthetic 16,000-repeated-symbol
list. Native checks cover 512 full roundtrip equalities plus scan checks;
all Call roots also get append-COW checksum/source-stability checks.
640 fresh graph windows return to their allocation baseline. All 128 sharing
savings match 48 bytes per cache hit, with equal Safe/Compact statistics.
The three unshared retained layouts match all 127 preceding census records.
Miri checks the eight library tests, not the full Session/stdlib corpus binary.

## Retained requested heap bytes

These are System allocation requests, not RSS. Root stack (original 32 bytes,
Safe 16, Compact 8), input graphs, already interned symbol text, allocator
metadata and fragmentation are excluded. Each graph is newly built; the
original comparison is normalized through a Safe roundtrip. All layouts
discard spare Vec/String capacity and preserve Head/Args aliases. The temporary
Symbol table is gone before retained bytes are sampled, but included in peak
and allocation calls. Original peak/calls include two bridges, so they are not
a fair construction-cost comparator. Full per-snapshot measurements are saved.

| Snapshot | Original normalized | Safe Plain | Safe Shared | Compact Plain | Compact Shared |
| --- | ---: | ---: | ---: | ---: | ---: |
| Integer list, 1,600 | 51,288 | 25,768 | 25,768 | 12,960 | 12,960 |
| Distinct symbols, 16,000 | 512,088 | 1,024,168 | 1,024,168 | 896,160 | 896,160 |
| Integer Association, 32,000 rules | 5,888,088 | 6,912,168 | 5,376,216 | 5,888,160 | 4,352,208 |
| ReplacePart rules, 1,600 | 294,608 | 422,704 | 345,904 | 371,480 | 294,680 |
| Three simple rules | 640 | 960 | 864 | 856 | 760 |
| Repeated symbols, 16,000 (synthetic) | 512,088 | 1,024,168 | 256,216 | 896,160 | 128,208 |

The Association has 31,999 hits: 1,535,952 retained bytes avoided in both
representations. Compact Shared is about 26.1% below normalized original; Safe
Shared is about 8.7% below it. Compact Shared still peaks at 8,640,996 additional
requested bytes during bridging, versus 4,352,208 retained; source remains live
outside that accounting window. Compact Plain peaks at 10,176,800 bytes.

The distinct-symbol list has zero hits: it consumes the entry budget and then
15,873 misses build ordinary nodes. Retained heap stays 75% above normalized
original for Compact. ReplacePart saves 76,800 bytes relative to Compact Plain
but still slightly exceeds original; small rules remain about 18.8% larger.
The synthetic repeated-symbol result isolates a favorable mechanism and does
not establish its frequency in real evaluation. This sharing policy itself
uses safe Rust and cannot establish an unsafe-specific advantage.

## Timing method and interpretation

Six datasets below have three sequential runs, one warmup plus nine measured
rounds each, rotating case order. There are 2,520 raw rows and 2,268 measured
rows, with 27 samples per dataset/operation/case. The counting allocator is
absent from these binaries. CPU affinity selects one available CPU; host/VM
interference is not excluded and no statistical significance is claimed.

Scan does three full logical traversals per sample (200 for small rules).
Build/drop does two fresh conversions followed by ordinary release (200 for
small rules); every conversion starts an empty local cache. Cache diagnostics
and lookups are included. COW clones the root argument vector, appends 777 and
releases the new root, twice (200 for small rules). Scan checksums match all
cases within each run. Other checksums count completed operations; full
roundtrip and COW value/source checks run separately before the timing phase.

Timing original is the source snapshot; retained-memory original is a fresh
normalized graph. Graph construction order outside scan/COW timers is fixed,
while timed case order rotates. Zero-hit read differences can reflect allocation
placement or noise; they are not evidence of a Symbol sharing mechanism.
The complete paired medians below use within-round pairs. Above one is slower.
Ratios near one do not establish improvement or absence of regression.

| Dataset | Operation | Safe Shared / Plain | Compact Shared / Plain | Compact Shared / Safe Shared | Compact Shared / original |
| --- | --- | ---: | ---: | ---: | ---: |
| gMicroAssociationPartWidth32000 | scan | 0.849 | 0.918 | 0.916 | 1.256 |
| gMicroAssociationPartWidth32000 | build_drop | 0.962 | 0.927 | 0.897 | n/a |
| gMicroAssociationPartWidth32000 | cow | 0.973 | 1.021 | 0.950 | 0.408 |
| gMicroReplacePartRules | scan | 0.783 | 1.010 | 0.908 | 1.191 |
| gMicroReplacePartRules | build_drop | 0.972 | 1.004 | 0.937 | n/a |
| gMicroReplacePartRules | cow | 0.980 | 1.086 | 1.018 | 0.516 |
| gMicroSortSymbolWidth16000 | scan | 0.925 | 1.005 | 0.944 | 1.123 |
| gMicroSortSymbolWidth16000 | build_drop | 1.213 | 1.247 | 0.930 | n/a |
| gMicroSortSymbolWidth16000 | cow | 0.975 | 1.000 | 0.985 | 3.531 |
| gMicroWideTraversal | scan | 1.039 | 0.996 | 1.045 | 0.882 |
| gMicroWideTraversal | build_drop | 1.013 | 1.023 | 0.838 | n/a |
| gMicroWideTraversal | cow | 1.046 | 0.969 | 0.784 | 0.216 |
| rulesSimple | scan | 0.993 | 0.983 | 0.993 | 1.010 |
| rulesSimple | build_drop | 1.253 | 1.229 | 0.895 | n/a |
| rulesSimple | cow | 1.000 | 1.000 | 1.005 | 0.801 |
| synthetic-repeated-symbol-16000 | scan | 1.038 | 0.851 | 0.847 | 1.058 |
| synthetic-repeated-symbol-16000 | build_drop | 0.548 | 0.485 | 0.826 | n/a |
| synthetic-repeated-symbol-16000 | cow | 1.008 | 0.989 | 0.989 | 3.341 |

Construction/drop regresses on distinct symbols (Compact Shared/Plain about
1.247, Safe about 1.213) and small rules (about 1.229/1.253). Association's
Compact scan is about 1.256 times source original despite its memory saving;
distinct-symbol COW is about 3.531 times original. The COW improvement on integer
or Call-heavy datasets must not be generalized to cheap Copy Symbol atoms in
original rum. The synthetic repeated-symbol build/drop reduction is substantial
in this harness, but does not justify always enabling a table.

This option remains opt-in. Next work should examine selective activation and
the distinct-symbol representation cost; any adaptive heuristic needs a fresh
comparison including its decision overhead and late-repeat cases. Direct
interned-symbol packing requires a reviewed safe access/reconstruction API or
another owner design; private layout casts or id reconstruction assumptions are
not used here. Full evaluator migration still requires per-label rum-vs-rum
A/B and lifecycle/embedding tests. No external engine is used as an oracle.

## Reproduce

On Linux with Python 3, taskset, Rust 1.85.0 and nightly-2026-10-05 with Miri/
rust-src installed: `bash run-study.sh`. CARGO_TARGET_DIR may point to native
temporary storage; evidence defaults to target/sharing-evidence. The committed
corpus is sufficient and no conversation documents are needed. analyze.py
checks pinned dependencies, bridge derivation, complete samples, cache bounds
and allocation deltas. Full evidence is in ../evidence/symbol-sharing/.
