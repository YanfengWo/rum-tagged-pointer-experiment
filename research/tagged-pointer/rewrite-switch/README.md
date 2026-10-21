# Same-executable rewrite control

Correctness and same-executable performance validation completed; this remains
an isolated diagnostic, with no default integration selected.
This diagnostic holds executable bytes and relative code offsets fixed while
selecting base, matcher, depth or combined behavior once at process startup.
It complements the completed [separate-build comparison](../rewrite-shortcuts/README.md).

All four modes passed **2256 workspace tests, 7 existing ignores, 29 groups**,
formatting, complete value/alias preflight and the existing resource/capacity
checks. Eight independent launch checks passed: unset, four valid names, empty,
unknown and leading-space input. Dedicated non-Unicode, case-variant and
concurrent/repeated-Session checks have not been added.

The saved files have the same SHA-256:
`cd38f13ddd988ce453e40ae12dd28309e7fb5a824b554243fe0547dc4adfde1c`.
The timing driver points every mode and A/A label at **one saved file**;
it does not give aliases different page-cache objects. It records each selected
mode in the journal and plan. Environment parsing happens in Session construction,
outside timed operations; hot sites query the initialized cache.

All modes include the cached-query and branch code. Pure shape classification
may be moved by the compiler, and actual branch costs need not be identical.
Shared overhead must not be subtracted as a constant. ASLR can still change
absolute addresses; one image fixes relative layout, not every execution state.
The off mode is an experimental control and does not replace an untouched
upstream baseline. See [design and limits](switch-design.md).

The completed protocol retains the original 112 labels and the same 30 focused,
pipeline and positive/negative-tree cases. Eight serial Williams blocks contain
64 full suites (7168 rows), 192 additional processes and 21120 retained inner
samples. All 675 Windows session samples were locked, without a sampled
transition. These are observations under that state, not a lock-causal test.

## Measured results

Elapsed ratios below one are faster; base here is the switch-off control,
not an untouched upstream executable.

| Case | Matcher/base | Depth/base | Combined/base |
| --- | ---: | ---: | ---: |
| Increment, 16000 | 0.8821 | 1.0022 | 0.8773 |
| Identity, 16000 | 0.6017 | 0.9955 | 0.6026 |
| Opaque RHS, 16000 | 0.7173 | 0.9980 | 0.7191 |
| Eight-pass pipeline, 16000 | 0.8755 | 0.9922 | 0.8793 |
| Mixed-hit pipeline, 16000 | 0.8732 | 1.0081 | 0.8806 |
| Chain64, positive levels | 0.9988 | 0.8420 | 0.8343 |
| Chain64, negative levels | 1.0012 | 0.9933 | 1.0004 |

Matcher hit-case and pipeline gains persist in all eight blocks. Depth favors
chain32/64 and binary7/9 positive controls in all blocks, with much smaller
changes on the corresponding negative controls. Very short mixed-control and
chain16 cases have large A/A dispersion and are not useful speedup evidence.

Original-suite slower medians number 45/56/55 for matcher/depth/combined against
off. Matcher table-count-null is 1.92% slower in 8/8 blocks; combined literal
string is 6.59% slower, Do-count-null 2.50% slower, and tiny UpValue candidates
0.76% slower in 8/8. All other slower rows and A/A observations remain in the
[full report](../evidence/rewrite-switch-performance/report.md).

Holding the image fixed supports the shortcut's targeted benefit. It does not
identify a unique cause for the older regressions: the off control includes
switch machinery, absolute addresses and host state still vary, and numerical
differences remain even on unrelated labels. Do not subtract absolute times
across this study and the separate-build study to estimate a constant switch
cost. The borrowed-single candidate is not a mode in this diagnostic.

`audit.py` verifies saved builds, source snapshots, inputs, launch checks and
raw outputs without running the interpreter. Its
[correctness record](../evidence/rewrite-switch/correctness.json) preserves the
earlier deferred status at its snapshot time. The performance archive retains
the prelaunch source bytes and executed monitor. The host-specific drivers require
the outer monitoring launcher to supply `session-protocol.json` with its actual
client date and to retain session flags. These drivers assume the preceding
offline build environment; the patch can be reviewed and applied independently
to the pinned Rum source.
