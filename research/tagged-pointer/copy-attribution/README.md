# Scoped copy attribution and safe Replace experiment

This study identifies the size and location of the previously observed N
`into_vec_clone` events. It then builds a small safe-Rust candidate against
`77029938f999b14dbe5b0755bebdf98e51517ae6`. The default checkout is unchanged.
The candidate improves the measured Replace operations, but remains an
experiment because two unrelated labels repeatedly regress by about 2–3%.

## What was copied

Separate diagnostic binaries record safe ownership conversions, vector lengths,
shallow slot bytes, reference counts and bounded stack traces. The N=64/256/1024
hit probes all show:

- One N-element parent traversal vector copy in replace/once.rs.
- N−1 two-element copies in attribute normalization of `Plus[n,1]`.
- One `SharedExpr::into_owned` clone.

The small stack probe confirms both traversal and normalization call chains.
There are not N copies of the full wide input. Vector slots total 3N−2 before
the change and 2N−2 afterward. The removed shallow payload is 32N bytes on this
target; this is not total allocation bytes, peak memory, recursive clone cost,
or measured memory bandwidth.

The four-site [candidate patch](candidate.patch) retains read-only SharedArgs
during ordinary Call traversal and materializes its source Vec only when first
suspending. Association, matching, RHS construction/evaluation, normalization,
and existing suspended-state types are unchanged. [SEMANTICS.md](SEMANTICS.md)
explains borrowing, roots, scope and the large-file split assessment.

## Measured results

Production baseline and candidate were built at the same local source root,
using separate output artifacts; baseline bytes were restored afterward. An
independent source copy tests the same candidate source bytes, and separate
diagnostic copies confirm the single wide clone is removed. Timing never uses
a diagnostic binary. An older original binary supplies same-binary A/A and
rebuild controls.

Eight position-balanced blocks each run four modes. The complete unchanged
112-label suite contributes 32 processes/3584 aggregate rows. Eight focused
labels add 32 processes/2816 retained individual timings. Focused results use
the median of eleven samples within a process, then eight paired process ratios;
the original suite retains its min9 and final-GC-batch conventions.

| Focused operation | Size 4000 | Size 16000 |
| --- | ---: | ---: |
| Integer increment rule hit | 0.934 | 0.944 |
| Identity rule hit | 0.972 | 0.960 |
| Integer list rule miss | 0.911 | 0.908 |
| Shared compound list rule miss | 0.865 | 0.898 |

Ratios are candidate/fresh baseline elapsed time; below 1 is faster. The
unchanged suite's `replace-wide-no-match` ratio is **0.903**, faster in all
eight blocks. Focused increment and compound-miss cases are also faster in all
eight blocks at both sizes. These are local language-operation results, not
application-wide speedup or parity with another implementation.

All 112 labels remain reported. **29 slower medians** include
`set-part-fill-large` **1.030** and `sort-default-wide` **1.023**, both slower
in all eight blocks; `cases-wide-no-match` is 1.023, slower in five blocks.
No median regression reaches 5%, but that cutoff is not a reason to ignore
smaller regressions. Their cause is unestablished. Keep the candidate isolated
until the repeated neighbor costs are explained or independently resolved.

Both baseline and candidate pass **2241 workspace tests**, with 7 ignored and
29 result groups: the pinned original's 2238 plus three new regression tests.
The earlier symbol-API stages' extra four tests are absent here. New checks
cover source aliases/rule reuse, duplicate compound binding isolation, and
guard/body evaluation order; existing workspace checks cover Sequence, Heads,
errors, resource budgets and suspension/resumption. Initial candidate testing
hit two Jupyter asynchronous deadline failures; five isolated pump tests and
the full subsequent run pass. Initial failure logs are retained without a
proven root-cause attribution or weakened timeout.

An initial fixture attempted Part assignment through a non-List and was
rejected by the original documented contract. The corrected List fixture
passes on both originals and candidate; debug/failure evidence is retained.

[Full summary](../evidence/copy-attribution/summary.json) and
[all-label report](../evidence/copy-attribution/report.md) retain raw observations,
source/binary hashes, all regressions and limitations. Eight blocks do not mean
88 independent samples; quantiles are descriptive, not confidence intervals.
One machine, toolchain and structured corpus do not establish a universal
no-regression guarantee, full semantic-equivalence proof or safe-API soundness.
No custom unsafe or general tagged-expression migration is introduced.

Organization note: the source/test/trace audit module is about 10k and is kept
cohesive around one evidence boundary for this experiment. TODO before expanding
its cases: move source and test provenance into a sibling analysis_sources
module, leaving event/window accounting here. The timing and reporting modules
are already separate. The larger existing traversal file is assessed in
SEMANTICS.md rather than silently expanded.

## Fresh primary replication

Linux x86_64 GNU, Rust 1.85.0 and rustfmt are required. Provide dependency caches
for the locked offline build (or fetch dependencies separately). Use two new
empty directories outside the checkout:

```bash
bash reproduce.sh /new/source-copy /new/evidence
```

This prepares the pinned source and identical regression fixtures, tests both
variants, builds both binaries at one source root, restores the original bytes,
then collects six fully permuted base/candidate/base blocks. It writes original
suite files, retained focused samples and `primary-replication.json`. This is a
fresh primary replication, not an assertion that historical binary hashes will
recur. The assembled script and embedded Python have syntax checks; its
individual build/test/timing steps were exercised in the recorded study, while
a fresh complete invocation of this wrapper remains pending.

The historical four-mode runners retain their fixed artifact paths to audit
the exact recorded experiment. Re-analyze its evidence with
`python3 analyze.py /path/to/evidence/copy-attribution`; neighboring prior
hotspot/native evidence and the pinned source checkout are also needed.
