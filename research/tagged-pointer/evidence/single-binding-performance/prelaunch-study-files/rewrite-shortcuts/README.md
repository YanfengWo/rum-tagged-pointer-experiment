# Safe rewrite shortcuts

Two independent, safe-Rust candidates are tested against pinned Rum
`77029938f999b14dbe5b0755bebdf98e51517ae6`. No default runtime source is edited.

- `matcher.patch`: reuse the existing definition leaf matcher only for guardless
  `Blank` and `Named(Blank)`, starting with empty bindings. Other matcher forms
  retain the original matcher and suspension path.
- `depth.patch`: for nonnegative Replace level ranges, use one only when deciding
  whether the input level is selected. All returned expression depths remain
  exact; negative and mixed ranges retain their existing scans.

See [matcher design](matcher-design.md), [matcher argument](matcher-proof.md),
and [depth argument](depth-proof.md). These are scoped source arguments with
finite regression evidence, not a complete contextual-equivalence proof.

## Validation protocol

The four uninstrumented variants are base, matcher, depth and combined. Each
receives the same three earlier copy regressions and fifteen new regressions,
including guards/RHS ordering, first-rule priority, typed bindings, trace,
complex-pattern fallback, real host suspension/resume, capability policy,
recursion limits, signed level ranges, head traversal, associations and Sequence.
The initial baseline attempt exposed a test-fixture name-resolution assumption;
the corrected tests distinguish an ordinary Global call from System Association.

Each variant must pass formatting, all workspace tests and the existing full
value/alias and resource probes before timing. The expected workspace result is
2256 passed, 7 ignored, 29 groups. Selected resource and capacity observations
must agree with base. Time and transient allocation histories are not required
to agree.

The timing protocol is declared before observing candidate times: eight serial
Williams blocks of four implementations with two byte-identical aliases each;
fixed CPU and environment; the unchanged 112-label original suite at scale 5
and minimum of nine; twelve focused and six pipeline cases with eleven retained
samples each. Twelve additional tree-shape cases compare positive and negative
level specifications on chains of depths 16/32/64 and binary trees of heights
5/7/9. The rules miss, so all values and input aliases must remain unchanged.
Negative ranges retain depth scans and provide a mechanism control. Shape
construction and value validation are outside the timed window.

Each process contributes one statistic. Eight paired process blocks, rather
than the retained inner repetitions, are the comparison units. Every original
label and every slower median is retained; A/A observations are controls and
are never subtracted. Timing uses Linux-native inputs/output and is separated
from builds, diagnostics and uploads. Windows lock flags are sampled separately;
these samples do not prove continuously idle or unlocked operation.

## Reproduction and status

The host-specific `build.py` checks and preserves the three earlier cached
binaries, uses Rust 1.85.0 with the existing offline dependency cache, and
creates fresh Linux source/target directories. `run-native.py` performs untimed
preflight followed by timing. `analyze.py` audits the saved outputs. These
drivers require the preceding experiment environment; the two patches and
regression sources can also be applied independently to the pinned upstream
source using its normal toolchain.

## Results

All four variants passed 2256 workspace tests (7 existing ignores, 29 groups),
the preflight and all selected resource/capacity checks. The independent 96-root
logical ledger confirmed that Named Blank hit cases bypass 2N general matcher
and snapshot entries while retaining N explicit leaf matches and the same RHS
work. Depth specialization removes exactly 2N+1 input-depth visits on Range[N].
These are logical events, not allocation counts.

The completed timing evidence contains 64 full original suites (7168 rows),
192 additional processes and 21120 retained inner samples. All 654 Windows
session samples were locked, with no sampled transition. Results are conditional
on that observed state; this is not an unlocked measurement or a causal study
of locking. See [all results](../evidence/rewrite-shortcuts/report.md).

| Case | Matcher/base elapsed | Depth/base elapsed | Combined/base elapsed |
| --- | ---: | ---: | ---: |
| Increment, 16000 | 0.8556 | 0.9912 | 0.8564 |
| Identity, 16000 | 0.6053 | 0.9974 | 0.5880 |
| Opaque RHS, 16000 | 0.7272 | 0.9957 | 0.7183 |
| Eight-pass pipeline, 16000 | 0.8621 | 0.9942 | 0.8552 |
| Mixed-hit pipeline, 16000 | 0.8725 | 0.9937 | 0.8665 |
| Chain 64, positive levels | 0.9936 | 0.8245 | 0.8288 |
| Chain 64, negative levels | 0.9975 | 0.9958 | 0.9932 |

Matcher gains in the five matching focus families have the same direction in
all eight blocks. Five larger positive tree controls also consistently favor
depth specialization; negative controls show much smaller changes. Chain 16
is too short and has A/A dispersion near 0.5 to 1.9, so its apparent gain is
not useful evidence. The combined pipeline benefit mainly comes from matcher.

The original suite has 37/29/41 slower medians for matcher/depth/combined.
Matcher has no 8/8 slower label, but five labels are slower in 7/8 blocks,
including retained generated-symbol GC (+3.53%) and iterator Plus (+2.03%).
Depth's `table-small` is slower by 2.79% in all blocks. Combined has three
8/8 slower labels: repeated small associations (+1.40%), local Module (+1.08%)
and small Do (+0.95%). All remaining slower medians and A/A observations are
retained in the complete report. A/A variation is never subtracted to excuse
a regression, and aliases are not independent builds.

No default integration is selected. The matcher is a useful safe candidate
with full-program synthetic gains; it does not establish a whole-runtime or
Rust-versus-C speedup. A separate same-image switch diagnostic will investigate
the effect of executable differences and the remaining regressions.
