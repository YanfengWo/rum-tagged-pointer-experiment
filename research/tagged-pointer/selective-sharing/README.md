# Selective Symbol sharing: bounded prefix study

An opt-in safe policy over the unchanged conversion APIs from c62eb6ba.
The kernel, earlier research sources and evidence remain unchanged. No new
unsafe is introduced; allocation accounting reuses the existing counter source.
This experiment measures a decision heuristic, not a default evaluator change.

`selective::<Compact>(&expr)` returns a graph and decision report. Safe follows
the same policy. Atoms and Calls with fewer than 64 root Args take Plain without
allocating probe scratch. Other roots sample at most 256 physical Expr slots in
preorder, counting Calls and deduplicating Head/Args storage. Lazy iterators do
not enqueue a whole wide vector. Eight repeated full Symbol values select
Shared with a 128-entry local cache; budget/exhaustion selects Plain. Scratch
is gone before construction; every call probes afresh. Context and generated
identity use full Symbol Eq/Hash. Constants were fixed before measurements.

The probe can miss late repetition, be misled by early repetition, skip narrow
roots, or detect a key beyond the full cache's first 128 entries. It supplies no
profitability guarantee. SEMANTICS.md gives the manual conditional argument and
the bounded-work reasoning; it does not prove the complete unsafe owner sound.

## Verification

Nine tests pass in default/all-feature native configurations and under strict
provenance Stacked/Tree Borrows Miri. They cover the root gate, exact node budget,
early exit, complete Symbol identities, duplicate storage, Head/Args aliasing,
COW/Weak, late misses and early false positives. The Call-only probe test uses
10,000 levels natively and 300 under Miri, then drains the original iteratively.
It does not build a compact deep graph. Ordinary Call Drop remains recursive.

The study reuses 112 lowered input bodies and 15 evaluated setup results, and
adds three explicitly synthetic controls. All 130 snapshots check three policies
and two owners: 780 full value equalities, scan checks and append-COW/source
stability for Call roots. All 910 allocation windows return to their baseline.
Selective retained storage equals the chosen branch on all 260 owner cases;
Shared savings equal 48 bytes per hit on all 260. All 128 preceding retained
layouts match exactly, including the earlier repeated-symbol control. Safe and
Compact decisions/stats agree. Both prior stages' source hashes, the original
owner hashes and seven core hashes remain fixed. Miri runs the library tests,
not the full Session/stdlib snapshot program. No evaluator A/B is claimed.

In 130 snapshots the decisions are 117 SmallRoot, 7 Plain and 6 Shared. These
counts include synthetic controls and are not weighted by evaluator frequency.

## Retained requested heap

System request bytes, not RSS. Root stack (32/16/8), source graph, existing symbol
intern pool, allocator metadata/fragmentation are excluded. All layouts normalize
spare capacities and preserve Head/Args aliases. Original normalized construction
uses a Safe roundtrip; its peak/calls contain two bridges and are not comparable
construction costs. Probe scratch is gone at the retained sample, but included
in peak/call accounting. Full seven-layout measurements are in allocations.json.

| Snapshot | Original normalized | Compact Plain | Compact Shared | Compact Selective | Decision |
| --- | ---: | ---: | ---: | ---: | --- |
| Integer list, 1,600 | 51,288 | 12,960 | 12,960 | 12,960 | Plain |
| Distinct symbols, 16,000 | 512,088 | 896,160 | 896,160 | 896,160 | Plain |
| Association, 32,000 rules | 5,888,088 | 5,888,160 | 4,352,208 | 4,352,208 | Shared |
| ReplacePart rules, 1,600 | 294,608 | 371,480 | 294,680 | 294,680 | Shared |
| Three simple rules | 640 | 856 | 760 | 856 | SmallRoot |
| Repeated symbols, 16,000 (synthetic) | 512,088 | 896,160 | 128,208 | 128,208 | Shared |
| Late repeats, 16,000 (synthetic) | 512,088 | 896,160 | 142,560 | 896,160 | Plain |
| Early repeats, 16,000 (synthetic) | 512,088 | 896,160 | 895,776 | 895,776 | Shared |

Association hits 31,999 times and saves 1,535,952 bytes in both owners. Compact
Selective remains 26.1% below normalized original; Safe Selective is 5,376,216
bytes (8.7% below). The late control has 300 distinct symbols followed by 15,700
head-symbol f copies: the probe misses 753,600 bytes of available savings. The
early control starts with eight f copies then 15,992 distinct symbols: only
384 bytes are saved, while lookups continue over the whole graph. The small-root
gate also deliberately forgoes 96 bytes saved by full sharing in three rules.

## Fresh construction and ordinary release timing

Each sample includes fresh probe, conversion, report counters and ordinary Drop.
No allocation counter is enabled. Inputs/interning exist outside timers. Three
runs pin the first allowed CPU, with one warmup and nine measured rounds each;
six owner/policy cases rotate, and run 2 reverses dataset and case order. This
produces 1,440 raw rows and 1,296 measured rows, 27 samples per case/dataset.
Two constructions per sample (200 for small rules). Semantic/COW checks happen
separately before timing. Original double-bridge construction is not timed.

Each cell below is the median of paired within-round elapsed-time ratios.
Above 1 is slower. Per-run medians and ranges are retained in summary.json.

| Snapshot | Safe Sel/Plain | Safe Shared/Plain | Safe Sel/Shared | Compact Sel/Plain | Compact Shared/Plain | Compact Sel/Shared | Compact Sel/Safe Sel |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| gMicroAssociationPartWidth32000 | 0.922 | 0.922 | 0.997 | 0.908 | 0.911 | 0.999 | 0.920 |
| gMicroReplacePartRules | 0.979 | 0.978 | 0.987 | 0.983 | 0.975 | 0.981 | 0.926 |
| gMicroSortSymbolWidth16000 | 1.001 | 1.142 | 0.854 | 1.001 | 1.222 | 0.808 | 0.934 |
| gMicroWideTraversal | 1.090 | 1.015 | 1.074 | 1.086 | 0.967 | 1.119 | 0.846 |
| rulesSimple | 1.033 | 1.204 | 0.843 | 1.000 | 1.231 | 0.813 | 0.887 |
| synthetic-early-repeats-16000 | 1.177 | 1.156 | 1.006 | 1.246 | 1.231 | 1.010 | 0.990 |
| synthetic-late-repeats-16000 | 1.002 | 0.553 | 1.840 | 1.017 | 0.502 | 2.041 | 0.939 |
| synthetic-repeated-symbol-16000 | 0.534 | 0.539 | 0.994 | 0.477 | 0.471 | 1.004 | 0.839 |

The distinct-symbol Compact Selective/Plain per-run medians are
1.018/0.993/1.004, versus full Shared/Plain's aggregate 1.222. Association keeps
the same retained benefit and has Selective/Plain 0.906/0.915/0.907. However the
integer probe adds 1.086/1.081/1.087 even though no sharing is selected. The
early false-positive control has 1.253/1.284/1.229; the late control has
1.013/1.033/1.014 but misses full sharing's roughly halved construction time.
Safe small rules show a 1.033 aggregate despite skipping the probe; differences
in wrapper/diagnostics and measurement noise remain possible, so no zero-cost
claim is made. Host interference is not excluded and no significance claim is
made. These timings concern conversion/release, not full evaluation. Read/COW
performance of a selected graph is not remeasured or assumed faster.

## Decision

Keep this heuristic as an experimental opt-in. It improves two identified cache
regressions but introduces integer sampling cost and cannot recover all useful
late sharing. Do not enable it as a universal default. The memory savings come
from the unchanged sharing strategy and node widths; selection only chooses
which graph to build. A caller's explicit workload choice or a separately
measured fused strategy may be more useful. Any future policy must count its own
decision cost and retain these misleading-prefix controls.

Distinct Symbol storage still remains about 75% above normalized original in
Compact. Sampling does not solve that representation problem. A separate safe
interned-symbol access design is still needed before investigating a direct
representation; private-field casts are not used here.

## Reproduction

The inherited owners require 64-bit pointers; this study was run on Linux x86_64
GNU with Rust 1.85.0 and nightly-2026-10-05 Miri (miri/rust-src components).
Python 3 and taskset are required. Cargo.lock and pinned corpus are committed;
populate the dependency cache with cargo fetch before the offline script on a
new machine. Install the pinned toolchains/components first. From this directory:

```bash
bash run-study.sh /absolute/output/directory
```

Do not put output inside this source package. The script bounds child process
groups and analyzes all records. The same observation on another host has not
yet been established. Cargo outputs, caches and private paths are not committed.
Generated JSON/log evidence is large and preserved intact; handwritten source
modules remain below the preferred 10k-character threshold.
