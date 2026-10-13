# Symbol uniqueness operation: benefit verification

Recorded 2026-10-13.
The prior 16-byte representation is unchanged. This stage verifies a real kernel algorithm rather than a numeric scan alone.

## Conclusion

No useful general elapsed-time gain was verified for the tested ordered symbol-uniqueness operation. The lightweight key scan's earlier speedup does not carry over when hashing, adaptive deduplication and full Symbol output materialization are included.
Typed16 key with core-compatible hash takes about 1.32x Original time for distinct symbols and 1.26x for generated symbols. Repeated-symbol and context controls are near parity; Association is about 1.02x. These descriptive results do not establish a small improvement.
Including one conversion and release makes the candidate slower in all tested workloads. Eight operations per conversion still do not recover parity in this batch. Do not convert an existing Expr graph solely to accelerate this operation.

## What was measured

The pinned runtime's UniqueSymbolCollector preserves first-seen order, scans linearly below 17 unique symbols, and promotes to HashSet when the 17th unique symbol arrives. The generic port preserves every push decision, checked against the original source after disclosed variable/type/seed adaptations.
An unused safe research export in an isolated copy calls the actual, unchanged runtime collector as the result oracle. Full preorder includes heads before args. This is the collector suboperation, not the complete UpValue target planner, matcher, Session evaluation or runtime representation replacement.
All timed modes use the same generic collector with a common cloned RandomState seed within each process. They return complete ordered Vec<Symbol>; cached entries keep a borrowed source node plus its key and resolve unique outputs through the full view. Output materialization, checksum consumption and result release are included. HashSet growth and the uniqueness threshold are unchanged.
Seven modes separate Original, actual unchanged previous16, current Typed16/Safe24 full views, Safe24 cached keys, Typed16's existing derived key Hash, and a local core-compatible hash adapter. The adapter is tested for exact Symbol hasher input; the prior public SymbolKey remains unchanged and has no heterogeneous map lookup.
Prepared-graph lanes run one or eight deduplications. Fresh-graph lanes convert once, run one or eight deduplications, and release. Adoption ratios compare those candidate fresh lanes with Original prepared lanes, explicitly excluding an Original conversion. Thus the baseline does not pay an artificial deployment cost.

## Ratios versus Original (greater than 1 is slower)

| Dataset | Typed full | Safe key | Typed derived | Typed compatible key | Adopt once | Adopt eight times |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Integers1600 | 1.156 | 1.037 | 1.252 | 1.265 | 9.001 | 3.318 |
| Distinct symbols16000 | 1.209 | 1.063 | 1.341 | 1.316 | 1.860 | 1.351 |
| Association32000 | 1.068 | 1.030 | 1.040 | 1.021 | 36.212 | 5.729 |
| Rules1600 | 1.071 | 1.066 | 1.062 | 1.048 | 23.964 | 4.239 |
| Small rules | 1.044 | 1.019 | 1.147 | 1.139 | 6.542 | 2.321 |
| Repeated16000 | 0.995 | 0.979 | 0.991 | 0.998 | 4.145 | 1.401 |
| Generated16000 | 0.983 | 1.040 | 1.451 | 1.257 | 1.811 | 1.329 |
| Strings16000 | 1.295 | 1.043 | 1.550 | 1.543 | 64.929 | 10.924 |
| Contexts16000 | 1.015 | 1.008 | 1.066 | 1.005 | 1.788 | 1.110 |
| Shared200 | 1.017 | 0.997 | 1.065 | 1.092 | 1.110 | 1.118 |
| Unique16 | 1.041 | 1.037 | 1.167 | 1.247 | 3.042 | 1.537 |
| Unique17 | 0.984 | 0.993 | 1.093 | 1.105 | 2.027 | 1.239 |

## What the result explains

Lightweight scan avoids a complete Symbol observation. A real collector needs equality/hash and outputs; many distinct symbols must eventually materialize every complete Symbol. The current cached observation also retains the source-node reference, with its associated copying/index and later projection work. These are structural costs visible in the implementation, not a quantified allocation of total elapsed time.
Generated-symbol hash control: the derived-key lane is about 1.45x Original; the exact core hash adapter reduces that to about 1.26x. Removing this confounder still does not produce a net gain. Safe24-key is often closer to Original than Tagged16-key: unsafe tagging is not necessary to obtain the observation/cache policy.
The earlier retained-request savings and scan results remain valid within their stated scopes. They do not imply this operation is faster, and neither study proves overall rum acceleration.

## Evidence and limits

- Native three tests and strict-provenance Miri three tests under each borrowing model pass. Covers thresholds, empty input, shared calls, first-seen order, contexts and identity 1/u64::MAX; core-compatible hash input is tested.
- 134 snapshots: 938 complete ordered Symbol results agree with the actual runtime collector.
- Isolated workspace: 2,242 passed, seven existing ignored, 29 result groups. The runtime collector source and prior representation sources/corpus remain hash-identical.
- 10,080 timing rows, 9,072 after warmup; exact dataset/case/round order, iterations, checksums and positive finite durations validated.
- Three CPU-affinity runs with rotating setup/case order and reversed run2. Operation order is fixed. Hash seeds vary by process but are shared across representations within a process. These are descriptive paired medians and ranges, not statistical significance tests or host isolation.
- Initial compile failed due to an unqualified Symbol type in the safe export; corrected and all checks rerun. The failed diagnostic is retained. Upstream nightly/test-helper warnings are retained without editing kernel behavior.
No new unsafe is introduced by this operation adapter. Prior safe-API soundness and target limits remain. No new default evaluator hot path exists, so these timings are not a core-language A/B acceptance result or a second-machine reproduction.

## Reproduce

Linux x86_64; Git, Python3, taskset, Rust1.85.0 + rustfmt and nightly-2026-10-05 + miri/rust-src. Dependencies need download access unless cached. Paths must be absent or empty and outside the research tree.

```bash
bash research/tagged-pointer/symbol-operations/run-study.sh \
  /var/tmp/rum-symbol-ops-source-new /var/tmp/rum-symbol-ops-evidence-new
```

Portable preparation and syntax are checked locally, not reproduced on another machine.
Further work should first profile duplicate checking, hash-table operations and full-output restoration separately. A thinner borrowed observation or retaining a full observation for outputs may be worth testing. Any evaluator integration needs its actual kernel rum-vs-rum gate; this negative result does not justify a wholesale migration.
