# Borrowed single-binding RHS candidate

Correctness and separate-build performance validation completed; the candidate
remains isolated and no default integration is selected.
This safe-Rust candidate removes the intermediate single-element binding map
from guardless `Named(Blank)` rewrite hits. It is layered on the existing
[leaf matcher candidate](../rewrite-shortcuts/README.md).

An internal RHS view is either a borrowed existing map or one symbol/value
pair. Its `get`, `contains_key` and `is_empty` operations feed the **existing**
scope, alpha-renaming, Sequence and shared-expression reconstruction algorithms.
The global binding container and existing map-entry signatures stay unchanged.
The borrowed value cannot enter a suspension frame or escape synchronous
substitution. Complex patterns and guards retain the owned map/continuation path.
See [the scoped semantic argument](SEMANTICS.md).

Three isolated uninstrumented variants were built from the same pinned source:
original, leaf matcher and borrowed single binding. All received the same
regressions and passed **2262 workspace tests, 7 existing ignores, 29 groups**,
formatting, complete value/alias acceptance, actual CLI semantics and the
selected resource/capacity checks. Outputs and observed resource values agree
with the original baseline. The six candidate test functions add five distinct
coverage groups for symbol identity, scope hygiene, binding priority, Sequence
and COW; one repeats an existing suspension-resume case.

The audit reconstructed the source patches in memory, checked the restored
1255-file source snapshot and all three new plus twelve preserved program
paths. Its [correctness record](../evidence/single-binding/correctness.json)
contains input, source and raw-output hashes. Fewer transient bindings and
clones are not a promise of identical allocator histories, RSS, cancellation
points or every tight resource threshold. This finite evidence does not establish
full contextual equivalence or a whole-runtime speedup.

The completed comparison contains 72 original suites (8064 rows), 216 additional
processes and 23760 retained inner samples. Original/matcher/single each have two
byte-identical A/A labels in twelve balanced blocks. All 112 original labels and
30 focused/pipeline/tree cases are retained; process blocks, rather than inner
repetitions, are the comparison units. All 730 Windows session samples were
locked, with no sampled transition. This is not an unlocked or lock-causal study.

## Measured results

Elapsed ratios below one are faster. These are paired ratios from this run;
the earlier matcher estimate is not subtracted from a different experiment.

| Case | Matcher/base | Single/base | Single/matcher |
| --- | ---: | ---: | ---: |
| Increment, 16000 | 0.8873 | 0.8215 | 0.9260 |
| Identity, 16000 | 0.5844 | 0.4187 | 0.7193 |
| Opaque RHS, 16000 | 0.7207 | 0.6204 | 0.8608 |
| Nested Flat, 16000 | 0.9180 | 0.8902 | 0.9716 |
| Eight-pass pipeline, 16000 | 0.8882 | 0.8261 | 0.9327 |
| Mixed-hit pipeline, 16000 | 0.9050 | 0.8540 | 0.9384 |

Every listed Single/matcher comparison favors Single in all twelve blocks.
For increment the block range is 0.9124-0.9404; the eight-pass pipeline range is
0.9072-0.9697. Distinct executable layout and host drift are still possible
contributors. Increment A/A medians are 1.0212/1.0399/1.0155 for
base/matcher/single; they are not subtracted to adjust results. Very short
mixed-control and chain-16 cases have large A/A dispersion and do not support
useful speedup claims.

The original suite has 57/49/45 slower medians for matcher/base, single/base and
single/matcher. Single/base key reads at width4000 are 2.47% slower in 12/12
blocks; large callback Sort is 3.11% slower in 11/12. Against matcher, callback
Sort and key-read regressions remain. Every smaller regression is retained in
the [full report](../evidence/single-binding-performance/report.md).
The result supports further work on this safe candidate, not default adoption.

A separate four-case confirmation makes a/b point to the same saved file per
implementation, rather than distinct copies. Its twelve additional blocks
(72 processes, 3168 retained samples) reproduce Single/matcher ratios of
0.9239 for increment, 0.9336 for the eight-pass pipeline, and 0.9413 for the
mixed-hit pipeline, all 12/12 faster. Increment A/A medians are now
1.0002/0.9941/1.0064. The miss control is 2.10% slower than base in 12/12 and
1.90% slower than matcher in 11/12. All 129 session samples were locked.
These blocks are kept separate and do not replace the full 112-label gate.
The changed context does not prove that file copies caused the earlier bias.
See the [confirmation](../evidence/single-binding-performance/alias-confirmation/report.md).

Prelaunch study bytes and the executed monitor are preserved in the performance
archive. Historical correctness source inventories refer to those baseline
bytes, before this results update. One outer-console log was overwritten during
the final copy after timing; its actual NUL-containing bytes and limitation are
retained. Per-case output, progress, native status and session records passed
the timing audit and are the primary evidence; the console was not reconstructed.

`build.py` and `audit.py` use the preceding offline experiment environment.
The timing drivers expect the outer monitoring launcher's actual-date and
session-state metadata. The patch and regression source can be used independently
with the pinned upstream source and its standard toolchain.
