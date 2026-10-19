# Balanced same-binary A/A and native Sort replication

The existing native experiment binaries were remeasured without rebuilding or
changing core/runtime/stdlib. This study tests repeatability on the same machine,
not independent build reproducibility or representative application speedup.

Ten serial blocks each run the complete unchanged 112-label language suite in
five positions: original base-a, API-only, original base-b, sort-API-only and
safe Symbol24 Sort. Both base IDs refer to the **same absolute binary path and
SHA**. Each mode occupies every position twice, using rotation and reversal.
Scale is 5, repetitions 9, CPU affinity fixed, stdin closed, and profiling off.
There are **50 process runs, 5600 aggregate rows and ten A/A pairs**.

Candidate ratios use the within-block geometric mean of both originals. These
are ten process-block observations, not ninety independent retained repetitions.
Most labels use the unchanged suite's minimum of nine; retained GC keeps the
original final-batch convention. The generated source hash matches the earlier
study, and every saved binary matches its prior SHA before and after running.

## Findings

| Predeclared label | API/base | Sort-API/base | Symbol24/base | Symbol24/Sort-API | Faster blocks |
| --- | ---: | ---: | ---: | ---: | ---: |
| Symbol Sort 4000 | 1.036 | 0.989 | 0.909 | 0.891 | 8/10 |
| Symbol Sort 16000 | 0.989 | 0.999 | 0.940 | 0.942 | 9/10 |
| Association keyed Part 4000 | 1.097 | 1.105 | 0.989 | 0.898 | 7/10 |
| Association keyed Part 32000 | 1.093 | 1.094 | 0.999 | 0.911 | 5/10 |

The 16000-symbol safe Sort gain repeats: about **6% less elapsed time**, with
block p10/p90 ratios 0.916/0.961. Both base-order strata have the same direction
(0.948 and 0.933). It passes the predeclared exploratory direction screen.
The 4000 case is noisier and fails that screen despite its favorable median.

There is also a clear reason to retain this as an experiment: the Symbol24 binary
slows incremental association key assignment by median **14.9% (small)** and
**13.8% (large)**; all ten block ratios are slower on both labels. Those labels
pass the exploratory regression screen. No default replacement is justified.

The API-only association-read regression also repeats in median, around 9–10%.
Both API controls are slower in nine of ten blocks. However, original A/A keyed
read ratios have extreme outliers (4000 range 0.580–1.096; 32000 range
0.495–1.094), so those effects do not pass the conservative A/A-noise screen.
Neither ignoring the regression nor assigning it a proven code-layout cause is
supported. All blocks, including outliers, remain in the analysis.

## Evidence and limits

[Full summary](../evidence/measurement-validation/summary.json) contains every
ratio, per-block seconds, order and position strata, screen outcome and hash.
[All-label report](../evidence/measurement-validation/report.md) keeps all 112
labels. The four focus labels above were fixed before running. Other labels are
exploratory; screens are not multiplicity-corrected statistical tests.

A/A measures original variability, not a bound on candidate-specific noise.
Position layers contain two observations each; treatment carryover pairs are
not fully balanced. All modes inherit one common parent environment, but
non-profile RUM variables were not individually recorded, so equality with the
previous session is not claimed. No hardware cache profile, independent rebuild,
new target/compiler, or real application corpus is covered.

The next performance step is to isolate and explain the association-assignment
regression before considering integration. Do not expand Expr migration on the
basis of this local Sort result.

## Reproduce

On Linux, provide the prior native evidence and saved binaries (or independently
reproduce those builds first). From this directory:

```bash
python3 run.py /new/evidence /prior/native-evidence --build /prior/native-builds
python3 analyze.py /new/evidence /prior/native-evidence
```

Use a fresh evidence directory and avoid other CPU-heavy work. `run.py` verifies
the original script PIN and prior binary hashes before invoking bounded serial
runs. A newly rebuilt binary with a different hash requires a separately recorded
build study rather than silently bypassing that check.
