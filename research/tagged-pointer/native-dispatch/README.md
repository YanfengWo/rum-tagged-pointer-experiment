# Native integration: decisive performance experiment

This study inserts native keys into the actual builtin dispatch cache and tests
wide symbolic Sort temporary vectors. It uses separately patched source copies;
the default checkout and its 32-byte Expr remain unchanged. There is no recursive
conversion on the evaluated path. The pinned original is
`77029938f999b14dbe5b0755bebdf98e51517ae6`.

The result does **not** justify a general tagged-expression migration. No mode
passes a broad no-regression gate. A specialized **24-byte** symbolic Sort gives
the clearest local benefit; neither 16-byte sort representation reliably improves
on it. See [the Sort study](../native-sort/README.md),
[semantic argument](SEMANTICS.md), and [complete evidence](../evidence/native-dispatch/summary.json).

## Controls and interpretation

| Mode | Actual integration |
| --- | --- |
| base | Unmodified original |
| api | Safe symbol accessors and inline attributes only |
| raw24 | Native hashbrown cache retaining original Symbol |
| safe24 | Same cache with a safe enum atom key |
| packed16 | Same cache with two safe integer identity words |
| tagged16 | Same cache with the previous Typed16 symbol atom |
| sort-api | Safe opaque-text comparator API only |
| sort-24 | Specialized Sort retaining original Symbol |
| sort-safe16 | Specialized Sort retaining safe text reference and identity |
| sort-tagged16 | Specialized Sort retaining tagged static text and identity |

Cache lookup uses safe `hashbrown::Equivalent`, the exact original Symbol hash
stream, and full generated identities. The wrapper forbids unsafe code; tagged16
depends on the earlier external owner. Cached `None` and absent entries remain
distinct. Forced collision and replacement/clear tests cover the map contract.

Selected time ratios are medians of three paired process runs. Below 1 is faster.
These are full language operations, including actual evaluation overhead.

| Label | api/base | packed16/base | tagged16/base |
| --- | ---: | ---: | ---: |
| do-count-null | See complete data | 0.940 | 1.011 |
| association-part width 32000 | 1.093 | See complete data | 1.091 |

The safe packed cache has a local loop gain, but also consistent StringTrim
regressions of about 11–14%. The API-only control already slows the association
Part labels by about 9–10%; this cannot be attributed to pointer tagging alone.
There is no demonstrated general native dispatch benefit.

## Validation and timing limits

All nine changed modes pass 2242 workspace tests (7 ignored; 29 test groups).
Each of four cache modes passes 3 native tests and 3 tests under each of strict
Stacked/Tree Miri. Each of three sort modes passes 2 native tests and 2 under each
Miri model. All ten modes pass the full-result language acceptance probe.
Miri and the conditional manual arguments are bounded evidence, not a complete
safe API soundness proof.

The unchanged `core-language-microbench.sh` has 112 labels. Each mode has three
serial CPU-affinity process runs, scale 5/reps 9, rotating order and reversed
second order. The suite reports the **minimum** of nine repetitions for most
labels, not a median; the special GC lane retains its original convention.
There are 3360 aggregate timing rows and three process ratios per label, not
27 independently retained samples. Nominal iteration counts on once-only labels
must not be used to infer per-iteration throughput.

All slower medians are recorded, and a >=5% list highlights regressions without
declaring smaller changes acceptable. Three runs do not establish significance,
cross-machine portability, or overall application acceleration. Existing
instrumented self-time diagnostics informed path selection; their overlapping
categories are not unprofiled bottleneck fractions or cache-miss measurements.

Rust sources were formatted after semantic tests using a preserved format-only
diff; release binaries were rebuilt before timing. Source overlays, binary hashes,
failed setup attempts, test logs and all raw timings remain in the evidence.

## Reproduce

On Linux x86_64 GNU install Rust 1.85.0 with rustfmt and nightly-2026-10-05 with
Miri/rust-src. Provide available dependency downloads (hashbrown 0.16.1 is pinned)
and two separate empty output directories. From this directory run:

```bash
bash run-study.sh /path/to/new-sources /path/to/new-evidence
```

The script prepares the pinned copies, builds/tests each feature, runs both Miri
models, acceptance and full timing, then analyzes the evidence. This complete
script has been syntax checked; the recorded study used equivalent bounded
incremental runners. A fresh end-to-end run on another machine remains pending.
