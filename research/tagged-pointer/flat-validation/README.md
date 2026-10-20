# Uninstrumented Flat candidate validation

This stage tests the safe prototype identified by the
[synchronous work ledger](../rewrite-work-ledger/README.md). Four implementations
are built from the same pinned source: base, wide-copy reuse, Flat Vec reuse,
and their combination. No ledger module, event calls or environment-gate checks
are present in these timing binaries. Default runtime source remains unchanged.

## Protocol fixed before timing

Formal timing uses an idle-desktop window on one host's WSL/Linux environment,
Rust 1.85.0, one source root, one target directory and fixed CPU affinity.
Every baseline denominator comes from local measurements interleaved with the
candidates. Background operating-system work and thermal/frequency drift remain
possible.

Each implementation is saved before changing source for the next build. Old
production binaries and default source hashes are preserved. Every mode runs
format checking, the full 2241-pass workspace (7 ignored, 29 groups), the prior
copy/alias/guard-order acceptance program and separate resource observations.
Compilation and testing finish before formal timing starts. Two jobs limit
build concurrency; every long child process has a time limit.

Timing inputs and live output are kept on the native Linux filesystem, then
copied to the workspace after timing completes. A fixed 30-second idle interval
separates untimed preflight from the timed protocol. This reduces activity in
the synchronized workspace during measurement; it does not control every
background process on the host.

Two byte-identical executable copies per implementation supply A/A controls.
Eight treatments run in eight serial Williams blocks, balancing every position
and all 56 ordered predecessor pairs within blocks. Seven block-boundary
transitions are recorded separately and are not balanced. These aliases test
timing variation; they are not independently compiled replicas.

Every treatment runs the unchanged 112-label microbenchmark at scale 5/reps 9,
the 12 focused rewrite cases, and six synthetic complete pipelines. The focus
families are increment, reversed increment, identity, miss, opaque body and
actual nested Flat, at N=1024 and 16000. Pipelines include eight rewrite passes
plus sorting/Part, one rewrite plus 64 Part writes, and a no-rewrite control.
Expected full results, input aliases and result-alias mutation are checked
outside timed windows. Separate untimed preflight additionally checks RHS
side-effect order, Flat/Orderless evaluation and Nothing handling. Preflight
records bind the exact binaries and generated sources.

The original suite retains its existing per-label timing method (mostly the
minimum of nine). Focus/pipeline cases retain eleven samples after a warm
evaluation; their process median is one observation. Within each block, the
implementation estimate is the geometric mean of its two aliases, then the
candidate/baseline ratio is computed. Results retain all eight paired ratios,
all four A/A controls and every original label. A/A variation is not subtracted
from an observed difference, and small slower medians are not exempted by a
5% threshold. In-process repetitions are not extra independent blocks. No
equal-label average is called whole-program performance.

The primary comparisons are wide/base, Flat/base and combined/base. The
combined/wide comparison isolates adding Flat reuse to the earlier candidate;
combined/Flat shows the incremental wide-copy change. Removed Vec counts do
not predict net speed: a new pre-scan and repeated nested-head comparisons can
offset savings. Complete pipelines and unrelated suite labels must be examined.

## Semantic and deployment boundary

The local argument preserves element values, order and child ownership when
no immediate argument has the outer head. Actual nested flattening keeps the
original loop. This adds no unsafe Rust and changes no pointer representation.
It does not remove the preceding two-slot shared copies.

ByteCount, MemoryInUse and cooperative modeled-memory limits can observe
different Vec capacities. The unchanged original already gives different
ByteCount values for SameQ-equal expressions built differently. This supports
representation-sensitive accounting, but does not prove equivalent resource
branches or tight-limit behavior. See the prior
[value argument and limits](../rewrite-work-ledger/SEMANTICS.md). Default
integration is not selected by this experiment.

## Replaying

The build script assumes the preceding offline Rust cache and saved production
binaries. Machine paths in published evidence are redacted; independent replay
requires resolving equivalent local cache paths. This is a laboratory replay,
not a portable one-command build package.

```bash
python3 research/tagged-pointer/flat-validation/build.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/flat-validation/run-native.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/flat-validation/check-trim-values.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/flat-validation/static_review.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/flat-validation/run-fresh-trim.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/flat-validation/analyze.py /path/to/NEW_EVIDENCE
```

## Results

All four uninstrumented implementations pass 2241 workspace checks (7 ignored,
29 groups), formatting and acceptance. All 64 complete-suite, 64 focus and
64 pipeline processes finish successfully: 7168 original-suite rows and 12672
retained focus/pipeline timings. The local host is an AMD Ryzen 7 9800X3D,
x86_64 WSL2 Linux, CPU 0. Old binaries retain their hashes; these are fresh
builds, with no assumption of byte identity to earlier source-directory builds.

| Measurement | Wide/base | Flat/base | Combined/base | Combined/wide |
| --- | ---: | ---: | ---: | ---: |
| Original wide no-match Replace | 0.9085 | 1.0043 | 0.9107 | 1.0021 |
| Increment 16000 | 0.9977 | 1.0034 | 1.0127 | 1.0137 |
| Reversed increment 16000 | 1.0005 | 1.0016 | 1.0164 | 1.0156 |
| Eight rewrite passes +Sort/Part 16000 | 0.9967 | 0.9965 | 1.0114 | 1.0159 |
| One rewrite +Sort/64 Part writes 16000 | 0.9975 | 0.9989 | 1.0134 | 1.0171 |

Ratios describe elapsed time, below one is faster. The new Flat prototype does
not give a stable increment or pipeline gain. Six original StringTrim labels
are consistently slower: Flat/base 1.1949–1.2332, combined/wide 1.2115–1.2560,
all eight blocks above one. Their A/A medians are only about 0.9894–1.0147;
those large differences cannot be dismissed as small ordinary control noise.
The wide candidate retains about 9% no-match gains, but set-part-fill-large and
sort-default-wide are 2.23% and 2.17% slower in every block.

The complete suite has 34 slower wide/base medians, 65 Flat/base, 74
combined/base, and 84 combined/wide. Every label is reported. These counts do
not make every small difference a proven causal regression. Across all 112
labels, A/A median ranges are base 0.9544–1.0267, wide 0.9512–1.0526,
Flat 0.9752–1.0705, combined 0.9738–1.0927. Some same-binary labels also show
5%–9% shifts of unknown cause. Percent-level observations must be interpreted
with those controls, without subtracting them or attributing all effects to
the local Flat algorithm.

**Neither Flat nor the combined implementation is selected for integration.**
Removed vector constructions are confirmed work reductions, not a measured
net runtime benefit. Full per-block ratios, all A/A controls and retained
samples are in the [summary](../evidence/flat-validation/summary.json) and
[per-label report](../evidence/flat-validation/report.md). Exact large-string
value checks and static review are retained separately; neither changes the
original suite's timing source or proves a unique cause for its regressions.

## Post-primary diagnostics

The original six StringTrim labels validate lengths, so four additional untimed
processes check all six full expected ASCII strings plus original-input aliases
with SameQ: 24 case checks pass. This supplements rather than changes the
original timing suite.

The same six setup expressions, benchmark wrapper and timing windows are then
extracted byte-for-byte into a fresh small session, with all eight modes and
eight balanced blocks. These 64 processes retain 384 original-wrapper minima;
the nine individual repetitions remain unreported by that unchanged wrapper.
This diagnostic was selected after seeing the primary regressions, and does
not supersede the full gate. Flat/base remains 1.1974–1.2359 and combined/wide
1.2029–1.2520, all six labels slower in every block. The large preceding suite
history is therefore not necessary for these differences. Short setup,
allocator/cache state and compilation effects remain possible.

Static source review finds no language-level Plus/Flat operation in the timed
StringTrim/Do inner loop for these inputs. Do increments Rust integers, and
StringTrim has Protected but no Flat attribute. Wrapper k=k+1 occurs outside
the measured window. The three selected trim entry/helper functions have the
same sizes and normalized instruction shapes after resolving named local and
dynamic GOT relocations; their addresses change by -0x590 from base to Flat.
Other selected one-hop functions do differ. This lossy comparison is not a
transitive code/data equivalence proof and does not establish a layout cause.
The full static records, disassembly and all diagnostics are retained.
