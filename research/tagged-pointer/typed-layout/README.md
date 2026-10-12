# Typed 16-byte payload and cached identity study

Recorded 2026-10-11.
External research only; the default rum core/runtime and evaluator remain unchanged.
This preserves the complete stage22 study and measures its actual Tagged16 as previous16.

## Changes and interpretation

- Ordinary symbols cache their authentic dense id in the unused second word. Full i64 and generated u64 identities remain intact.
- Inspect returns a lightweight equality/hash SymbolKey. Full view remains available for names, context, lexical ordering and serialization. Keys do not implement lexical Ord.
- Typed Arc<str>, Arc<BigInt> and Arc<Rational> replace the general Arc<Atom> payload. str alignment is checked with a safe fallback inside the owner.
- Direct borrowed construction avoids cloning an intermediate String/Box. The cloned-Atom move path is measured separately; neither consumes the entire source graph.
- Safe24 uses the same policy with safe enum storage. Common borrowed projections still contain unsafe internals.

Inspect symbol-list traversal is about 0.53x Original elapsed time, versus about 1.78x for full view. Safe24's key scan is nearly equal to Typed16: this gain is mainly the observation/cache policy, not proof that unsafe tagging itself is necessary.
Strings retain 640,072 bytes versus previous16's 1,136,072 and Original's 624,088. Borrow construction uses 16,021 calls versus 32,021 previously; build/drop is about 0.66x previous16 but still 1.33x Original. Final unique string drop remains about 1.80x Original.
Big/Rational retain near Original requests, replacing previous16's extra 32 bytes per owned leaf; direct construction removes one transient Box allocation per leaf. Build/drop improves versus previous16 but still regresses versus Original.
The symbol-only build/drop lanes are around 1.20x Original and 1.06–1.08x previous16 in this run. Full-view reads and default release also retain regressions. This is not a universal speedup or an evaluator acceleration claim.

## Retained requested heap bytes

Root stack, source graph, permanent symbol pool, allocator overhead and RSS are excluded.
Moved and borrowed Typed16 retain identical graphs; construction peaks/calls are separately recorded in JSON.

| Dataset | Original | Previous16 | Safe24 borrow | Typed16 borrow |
| --- | ---: | ---: | ---: | ---: |
| Integers 1600 | 51,288 | 25,672 | 38,480 | 25,672 |
| Distinct symbols 16000 | 512,088 | 256,072 | 384,080 | 256,072 |
| Association 32000 | 5,888,088 | 3,840,072 | 4,864,080 | 3,840,072 |
| Rules 1600 | 294,608 | 192,160 | 243,384 | 192,160 |
| Small rules | 640 | 432 | 536 | 432 |
| Repeated symbols 16000 | 512,088 | 256,072 | 384,080 | 256,072 |
| Generated symbols 16000 | 512,088 | 256,072 | 384,080 | 256,072 |
| Strings 16000 | 624,088 | 1,136,072 | 768,080 | 640,072 |
| Deep chain 256 | 30,720 | 22,528 | 26,624 | 22,528 |
| Shared args 200 calls | 38,576 | 19,344 | 28,960 | 19,344 |
| Manual Big(7) 16000 | 1,152,088 | 1,664,072 | 1,280,080 | 1,152,072 |
| Large Big 16000 | 1,280,088 | 1,792,072 | 1,408,080 | 1,280,072 |
| Rational 16000 | 1,792,088 | 2,304,072 | 1,920,080 | 1,792,072 |

## Paired elapsed-time ratios

Greater than 1 means slower; 27 within-round pairs per comparison, from three runs after excluding round 0.
The full/key checksum algorithms return identical values. Original and previous16 use their full interface in both lanes because they have no key projection.
Big/Rational scan includes decimal formatting allocation in every representation; it is not an arithmetic speed measurement.

| Dataset | Full / Original | Key / Original | Build+drop / Original | Build+drop / Previous16 | COW / Original | Drop / Original |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Integers 1600 | 0.873 | 0.787 | 1.076 | 1.077 | 0.449 | 0.854 |
| Distinct symbols 16000 | 1.775 | 0.530 | 1.212 | 1.082 | 0.741 | 0.994 |
| Association 32000 | 0.925 | 0.747 | 0.874 | 0.999 | 0.902 | 0.943 |
| Rules 1600 | 1.080 | 0.823 | 1.015 | 1.023 | 0.948 | 1.238 |
| Small rules | 1.015 | 0.896 | 0.994 | 0.963 | 0.901 | 1.020 |
| Repeated symbols 16000 | 1.224 | 0.535 | 1.206 | 1.057 | 0.824 | 0.992 |
| Generated symbols 16000 | 1.222 | 0.639 | 1.202 | 1.073 | 0.762 | 0.996 |
| Strings 16000 | 1.029 | 1.046 | 1.330 | 0.656 | 0.437 | 1.798 |
| Deep chain 256 | 0.974 | 0.845 | 1.000 | 1.003 | 0.826 | 1.230 |
| Shared args 200 calls | 0.835 | 0.838 | 1.035 | 1.053 | 0.969 | 0.984 |
| Manual Big(7) 16000 | 0.988 | 1.001 | 1.163 | 0.682 | 0.268 | 1.427 |
| Large Big 16000 | 0.998 | 0.999 | 1.201 | 0.680 | 0.272 | 1.382 |
| Rational 16000 | 1.003 | 1.006 | 1.102 | 0.760 | 0.209 | 1.239 |

Original wrappers retain disabled profiling branches; research wrappers omit them. Shared leaf COW avoids Original's deep copies. Consumer checksum helpers use the disclosed inline(always) control inherited from stage22. Allocation/cache/observation policy changes contribute to timing; tag width alone does not explain it.
Ranges and per-run medians in evidence are descriptive, not confidence intervals. Compare previous16 within this batch; do not divide medians from separate historical batches.

## Validation

- 14 library tests plus 6 compile-fail borrow examples; all-features also pass.
- Strict-provenance Miri: 14 tests under each of Stacked Borrows and Tree Borrows.
- Isolated safe core extension: 11 tests; complete workspace: 2,242 passed, 7 pre-existing ignored, 29 result groups.
- 135 snapshots, 675 full projections, 1,350 full/key scans, 635 complete edited/source COW checks and 675 fresh allocation windows.
- 405 physical cost equations, 135 same-policy width equations, 135 move/borrow retained equalities. Original and previous16 exactly match all 132 historical retained-byte rows.
- 11,700 timing rows validated for exact sample order, counts, checksums and positive durations; 10,530 measured rows after warmup.
- Frozen upstream/corpus/earlier source hashes pass. Earlier research files remain preserved separately from this new stage.

See [representation](DESIGN.md) and [conditional semantic argument](SEMANTICS.md). The extension is safe and additive but requires deliberate adoption in a future integration. The default core API remains unchanged.
Manual arguments and Miri are not complete soundness proofs. The alignment fallback is manually reviewed without an injected unaligned-Arc fixture. Default Drop is recursive. There is no evaluator migration, full core-language A/B, second-machine result or universal target/toolchain guarantee.

## Reproduce

Linux x86_64 with Git, Python 3, taskset, Rust 1.85.0 + rustfmt, and nightly-2026-10-05 + miri/rust-src. Dependencies need download access unless cached. Use absent or empty persistent paths.

```bash
bash research/tagged-pointer/typed-layout/run-study.sh \
  --overlay /var/tmp/rum-typed-source-new \
  --evidence /var/tmp/rum-typed-evidence-new
```

The script prepares a pinned isolated source copy, applies only the disclosed safe core extension, runs all checks, measures counters-off timings and counters-on allocations separately, and analyzes from the unchanged repository.
Portable preparation/syntax is checked locally; this is not a second-machine reproduction.

Next: profile a real equality/matching consumer before migrating a small evaluator operation; retain lexical Symbol behavior. Address owned-leaf construction/release regressions and audit the closed ownership API before general adoption.
