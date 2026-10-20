# Rebuild controls and synthetic pipeline verification

This stage reevaluates the small safe-Rust `Replace` candidate from
[copy attribution](../copy-attribution/README.md). It changes no runtime source
or candidate implementation. The candidate still is not selected for default
integration: no-match replacement is consistently faster, but Part/Sort
regressions persist, other suite medians regress by up to11%, and complete
synthetic programs do not show a stable gain.

## Replayed builds and static code

In one retained source root and one target directory, rebuild base, candidate,
base, candidate, saving each binary separately before measuring. The first
build forces all local Rust crates to recompile; dependencies are reused. The
two base binaries are byte-identical and equal the previous primary baseline
(`9327fee2...`). Both candidates are byte-identical and equal the previous
primary candidate (`435719a5...`). Full hashes, .text hashes, build logs and
acceptance logs are retained. Original source bytes and all three earlier
binaries are preserved. This rules out observed repeated-build variation in
this replay, not every possible build difference.

The read-only ELF comparison selects 21 functions. Base versus primary
candidate has one differing normalized instruction sequence in that set,
`replace_call_children_owned` (2790 to 2637 bytes); total .text is 352 bytes
smaller. `eval_sort` and `eval_sort_by` have identical raw function bytes and
addresses. Part entry functions have unchanged sizes/addresses and equal
address-normalized instruction sequences. `compare_expr_canonical` moves by
352 bytes and has equal normalized instructions. Normalization discards
address text, RIP displacements and compiler suffixes: it is lossy and proves
neither transitive data/callee equality nor a cause of timing differences.
The independent candidate from a different source directory also is recorded;
its binary/text are not assumed equal to the primary candidate.

## Primary design and complete results

Eight serial four-mode blocks use two byte-identical baseline copies and two
byte-identical candidate copies. The four-treatment Williams design balances
positions and immediate predecessor pairs **inside blocks**. Seven cross-block
transitions are separately recorded and are not balanced. CPU affinity and
environment are fixed. No project build or other project benchmark runs in
parallel. Timing uses the runtime's monotonic `AbsoluteTiming`.

Each mode runs the unchanged 112-label `core-language-microbench.sh` at scale5,
reps9, then standalone controls and six synthetic pipelines. This gives32
full-suite processes (3584 aggregate rows),32 standalone processes and32
pipeline processes (4224 retained individual timings). Most original-suite
labels retain the minimum of nine, while new corpora retain eleven samples
after a warm evaluation and use the process median.

Within each block, candidate/base is the geometric mean of the two candidate
mode medians divided by the geometric mean of the two baseline mode medians.
The reported value is the median across eight blocks. Both same-binary A/A
ratios and all individual samples are retained; they are not subtracted from
the treatment ratio. Internal repetitions are not independent blocks. No
equal-label average is presented as whole-program performance.

| Primary measurement | Candidate/base elapsed | Faster blocks |
| --- | ---: | ---: |
| Original suite: replace-wide-no-match | 0.9150 | 8/8 |
| Original suite: appendto-list-incremental-small | 1.1095 | 0/8 |
| Original suite: keysort-default-wide | 1.0941 | 2/8 |
| Original suite: set-part-fixed100-len800 | 1.0568 | 0/8 |
| Original suite: set-part-fill-large | 1.0359 | 0/8 |
| Original suite: sort-default-wide | 1.0263 | 0/8 |
| Standalone integer no-match16000 | 0.9074 | 8/8 |
| Standalone integer increment16000 | 0.9937 | 5/8 |
| Eight increment passes +Sort +Part4000 | 0.9978 | 4/8 |
| Eight increment passes +Sort +Part16000 | 0.9974 | 5/8 |
| One increment +Sort +64Part writes4000 | 0.9935 | 6/8 |
| One increment +Sort +64Part writes16000 | 0.9986 | 4/8 |

The larger newly observed regressions are about11.0% for incremental AppendTo
(8/8 slower),9.4% for KeySort (6/8), and5.7% for fixed-position Part writes
(8/8). Their cause and stability across further runs remain unestablished.
All55 slower suite medians, including smaller regressions, are retained in the
[complete report](../evidence/regression-isolation/report.md) and
[summary](../evidence/regression-isolation/summary.json). No5% exemption is used.
The increment gain seen in the preceding study is not stable across these
contexts. The stable no-match gain is a narrow operation result.

## Synthetic programs and the context followup

The pipelines time complete construction of reversed integer input, zero/one/
eight sequential Replace increment passes, Sort, one or64 List Part writes,
and result assembly. Each output includes the modified result, its pre-Part
alias, and both original-input aliases. Analytically generated full expected
lists and SameQ checks are outside timing. Warm and every retained pipeline
result are checked. These are artificial programs, not a real application
distribution or a vendor-engine comparison.

The no-Replace16000 pipeline has a primary ratio range0.754–2.014 and a
candidate same-binary A/A median0.742. The smaller no-Replace case also shows
periodic additional delays at different repetition positions. This is evidence
of material measurement-context variation, not proof of GC or allocator cause.

The initial standalone Part control returns the resulting list, retaining the
previous result as an alias across repetitions. The unchanged original suite
times Do, which returns Null. Those ownership contexts differ. The original
suite remains the authoritative per-label gate; the standalone control is not
an exact reproduction of its ownership pattern.

An exploratory followup, selected after the primary results, executes each of
five cases in a fresh process. It adds160 processes and1760 retained timings,
for5984 retained timings in total. Part returns Null and separately validates
the complete list outside timing. The heavy and no-Replace pipelines use
unchanged per-case program generation.

| Fresh-process followup | Candidate/base elapsed | Faster blocks |
| --- | ---: | ---: |
| Part400, Null result | 1.0276 | 1/8 |
| Sort8000 | 1.0153 | 2/8 |
| Integer increment16000 | 0.9833 | 5/8 |
| Eight increments +Sort +Part16000 | 1.0093 | 2/8 |
| No Replace +Sort +64Part writes16000 | 1.0066 | 3/8 |

Fresh processes do not remove every within-process delay or produce a stable
heavy-pipeline benefit. Part/Sort regressions therefore cannot be explained
away solely by earlier rewrite activity in the complete suite. Their cause
remains open; static layout differences are a hypothesis, not a conclusion.

## Validation and replay

No new unsafe, ownership implementation or kernel patch is introduced. The
production hashes equal the previous fully checked binaries, whose workspace
results were2241 passed,7 ignored on each side. That historical full suite is
not rerun or counted again here. Four rebuilt binaries pass the existing
acceptance source; both implementations accept full new expected results and
aliases in preflight and every timed process exits successfully. The independent
u64 proof work is unchanged.

The scripts separate responsibilities: `build-controls.py` replays builds;
`static_compare.py` inspects ELF functions; `control_cases.py` and
`pipeline_cases.py` generate language sources; `run-timings.py` executes the
primary design; `run-context.py` executes the exploratory followup;
`statistics_helpers.py`, `context_analysis.py` and `analyze.py` audit results.
All code modules remain below10k bytes. Every child command with runtime or
build work is bounded. Code/payload binaries are retained locally, not included
in the research export.

These replay scripts refer to retained laboratory artifacts and the preceding
study metadata. Published paths are deliberately redacted, so the exported
raw metadata is not directly executable. For an independent fresh-source
base/candidate replication use the preceding study's
[reproduce.sh](../copy-attribution/reproduce.sh), then resolve equivalent local
artifact paths before replaying this stage. This stage does not claim a newly
executed portable one-command wrapper. On the original laboratory tree:

```bash
python3 research/tagged-pointer/regression-isolation/build-controls.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/regression-isolation/run-timings.py /path/to/NEW_EVIDENCE --preflight
python3 research/tagged-pointer/regression-isolation/run-timings.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/regression-isolation/run-context.py /path/to/NEW_EVIDENCE
python3 research/tagged-pointer/regression-isolation/analyze.py /path/to/NEW_EVIDENCE
```

The build replay intentionally rejects an existing build/saved directory; a
fresh location must be resolved before another replay. Static comparison is
also required before analysis; its recorded commands and input hashes are in
the evidence. Default runtime source remains pinned and unchanged.
