# Three expression layout prototypes

Recorded 2026-10-11 on pinned rum `77029938f999b14dbe5b0755bebdf98e51517ae6`.
Three alternatives remove the previous per-symbol Arc allocation. Tagged16 also removes the outer Call Arc.
The default kernel is unchanged; these are external types operating on frozen expression snapshots.

| Layout | Bytes | Direct values | Additional allocation |
| --- | ---: | --- | --- |
| Original Expr | 32 | Original atom/Call fields | Original Head/Args and atom internals |
| Safe24 | 24 | Full i64, static text plus full symbol identity, Head/Args | Arc for other atoms |
| Tagged16 | 16 | Same policy as Safe24; discriminant in low pointer bits | Arc for other atoms |
| Tagged8 | 8 | Signed 61-bit integers and plain static symbols | Body Arc for Calls, generated symbols, other atoms and out-of-range Small |
| Previous-shared8 | 8 | Signed 61-bit integer | Previous bounded Symbol cache and broad Body Arc policy |

Safe24's storage enum uses safe Rust. Common borrowed Arc projections contain private unsafe code; the whole package is not zero-unsafe.
All callers use safe constructors/views/COW/consumption. Layouts are checked for the pinned 64-bit target.
[Design and costs](DESIGN.md), [conditional semantics and ownership argument](SEMANTICS.md), [safe Symbol access proposal](symbol-access.rs).

## Retained heap requests

Fresh graphs preserve Head/Args aliases and use normalized argument-vector capacities. Original and new layouts use one common single-pass construction algorithm.
Bytes exclude source graphs, root stack, the permanent interner and allocator metadata; they are System requests, not RSS.

| Snapshot | Original | Safe24 | Tagged16 | Tagged8 | Previous-shared8 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Integer list (1,600) | 51,288 | 38,480 | 25,672 | 12,912 | 12,960 |
| Distinct symbols (16,000) | 512,088 | 384,080 | 256,072 | 128,112 | 896,160 |
| Association (32,000) | 5,888,088 | 4,864,080 | 3,840,072 | 4,352,112 | 4,352,208 |
| ReplacePart rules (1,600) | 294,608 | 243,384 | 192,160 | 294,584 | 294,680 |
| Small rules | 640 | 536 | 432 | 520 | 760 |
| Repeated symbols (16,000) | 512,088 | 384,080 | 256,072 | 128,112 | 128,208 |
| Generated symbols (16,000) | 512,088 | 384,080 | 256,072 | 896,112 | 896,160 |
| Strings (16,000) | 624,088 | 1,264,080 | 1,136,072 | 1,008,112 | 1,008,160 |
| Deep chain (256) | 30,720 | 26,624 | 22,528 | 30,720 | 30,768 |
| Shared argument vector (200 calls) | 38,576 | 28,960 | 19,344 | 19,376 | 19,472 |

All 132 snapshots satisfy the independently counted physical cost equations. Tagged16 saves 16 bytes per physical child slot but adds 48 requested bytes for each owned non-Small/non-Symbol atom.
Tagged8 saves 24 bytes per slot but adds 48 bytes for every physical fallback node, including Call and generated Symbol.
The 128 historical original/shared8 retained-byte results match exactly; historical peak/call counts include a different bridge/cache algorithm.

## Operation timings

Ratios are medians of 27 within-round pairs; values above 1 are slower than Original. Three runs each exclude one warmup round and retain nine rounds.
Cases/setup allocation rotate; run 2 reverses datasets/cases. Counters are absent; one CPU is selected, without claiming host isolation.

| Snapshot | Tagged16 build+drop | Tagged16 scan | Tagged16 COW | Tagged8 build+drop | Tagged8 scan | Tagged8 COW |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Integer list (1,600) | 0.863 | 1.005 | 0.418 | 0.601 | 0.945 | 0.205 |
| Distinct symbols (16,000) | 0.906 | 1.733 | 0.679 | 0.557 | 1.617 | 0.197 |
| Association (32,000) | 0.915 | 1.082 | 0.911 | 1.014 | 1.059 | 0.404 |
| ReplacePart rules (1,600) | 1.029 | 1.111 | 0.955 | 1.188 | 1.087 | 0.482 |
| Small rules | 1.071 | 1.061 | 0.962 | 1.093 | 1.086 | 0.842 |
| Repeated symbols (16,000) | 0.902 | 1.307 | 0.665 | 0.560 | 1.120 | 0.230 |
| Generated symbols (16,000) | 0.919 | 1.336 | 0.753 | 3.251 | 1.327 | 3.332 |
| Strings (16,000) | 1.885 | 1.260 | 0.441 | 1.731 | 1.264 | 0.412 |
| Deep chain (256) | 1.071 | 1.031 | 0.767 | 1.133 | 1.287 | 0.746 |
| Shared argument vector (200 calls) | 0.944 | 0.803 | 0.969 | 0.979 | 0.765 | 0.600 |

These operations are construction/conversion, complete logical scan, and root append-COW, not evaluation, sorting or Association lookup.
Clone/drop results and all per-run medians/ranges are in [the timing report](../evidence/layout-design/timing-report.md) and [summary](../evidence/layout-design/summary.json). Ranges are descriptive, not confidence intervals.
Original wrappers retain disabled profiling branches that research wrappers omit. String COW includes the original deep String clone versus new Arc sharing; Call root clones can acquire two counts or one Body count. These differences are not exclusively tag-width gains.

## Code generation control

The first traversal passed every atom through a generic checksum helper with Big/Rational formatting branches. Counter-off disassembly confirmed an extra call and stack AtomRef.
Ordinary inline hints retained that call; an `inline(always)` annotation on the benchmark consumer eliminated it. The library atom projection has an inline hint.
All three batches keep the same algorithm, values, allocation policy and checksum. Initial source, raw rows and counter-off assembly are preserved.

| Snapshot | Tagged16 scan initial/original | Hint/original | Forced-inline/original |
| --- | ---: | ---: | ---: |
| Integer list (1,600) | 1.981 | 1.977 | 1.005 |
| Distinct symbols (16,000) | 4.597 | 4.468 | 1.733 |
| Association (32,000) | 2.000 | 2.089 | 1.082 |
| ReplacePart rules (1,600) | 2.106 | 2.106 | 1.111 |
| Small rules | 1.521 | 1.514 | 1.061 |
| Repeated symbols (16,000) | 4.403 | 4.414 | 1.307 |
| Generated symbols (16,000) | 4.413 | 4.394 | 1.336 |
| Strings (16,000) | 1.856 | 1.860 | 1.260 |
| Deep chain (256) | 1.557 | 1.590 | 1.031 |
| Shared argument vector (200 calls) | 2.492 | 2.624 | 0.803 |

The remaining symbol-read regression is consistent with both projection/code generation and losing the cached dense id: rebuilding a Symbol reads id through SymbolText. This is a structural candidate explanation, not a quantified attribution.
Forced inlining is a control of this consumer, not a guarantee that every future kernel caller will be optimized automatically.

## Validation and limits

- Native: 10 library cases plus 4 compile-fail borrowing cases; all-features also pass.
- Strict-provenance Miri: 10 library cases in both Stacked Borrows and Tree Borrows.
- Safe additive Symbol API: 10 core cases; the isolated workspace passes 2,241 tests with 7 pre-existing ignored cases, in 29 result groups.
- 660 complete roundtrips/scans, 620 full edited/source COW comparisons and 660 allocation/reclamation windows.
- Three timing batches (initial/hint/forced), each 7,500 validated rows and 6,750 measured rows. Frozen source/corpus hashes and physical memory equations pass.

The first parallel workspace attempt had one loopback address-in-use failure; serial execution passes. The failed log is retained. Existing runtime test-helper dead-code warnings are retained.
The Symbol extension is two safe accessors appended only to a separate pinned source copy. It makes permanent opaque text lifetime an explicit contract; the default core files remain byte-identical.
The manual argument and Miri are not complete API soundness or contextual-refinement proofs. Default Drop remains recursive; deep explicit release is separately tested.
No replacement evaluator, full core-language A/B, C-engine comparison, second-machine result or universal LLVM/target result exists.

## Current choice and next design questions

Tagged16 is the main general research candidate: it retains full generated identity and avoids outer Call allocation. It cuts Association retained requests by about 35% and distinct-symbol requests by about 50%; snapshot build/COW usually improve, while reads can regress.
Keep Safe24 as the policy control and Tagged8 as the wide ordinary-symbol/integer specialization. Generated symbols and Call-heavy structures expose Tagged8's fallback cost. Owned strings remain expensive in all three layouts.
Next investigate cheap symbol observations without rebuilding the larger Symbol, selective retention of a cached id, and typed ownership of already heap-backed atoms. A new projection must retain opaque provenance and complete identities.
Only after those costs are resolved should a small evaluator integration be measured label-by-label with the repository's rum-vs-rum microbenchmark gate.

## Reproduce on Linux x86_64

Use Git, Python 3, taskset, Rust 1.85.0 plus rustfmt, and nightly-2026-10-05 plus miri/rust-src. Dependencies require download access unless already cached. Paths below must be absent or empty; use persistent local storage.

```bash
bash research/tagged-pointer/layout-design/run-study.sh \
  --overlay /var/tmp/rum-layout-source-new \
  --evidence /var/tmp/rum-layout-evidence-new
```

The script builds the final annotated code, applies the disclosed safe API only in the copy, runs all checks and the final three-run timing plan. It does not regenerate the historical initial/hint batches.
`prepare-overlay.py` also prepares a reviewable source copy independently. Analyze from the unchanged repository so pinned core hashes and historical evidence remain valid controls.
