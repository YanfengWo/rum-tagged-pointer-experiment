# Same-executable rewrite control

Correctness validation completed; performance measurements are deferred.
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
The prepared timing driver points every mode and A/A label at **one saved file**;
it does not give aliases different page-cache objects. It records each selected
mode in the journal and plan. Environment parsing happens in Session construction,
outside timed operations; hot sites query the initialized cache.

All modes include the cached-query and branch code. Pure shape classification
may be moved by the compiler, and actual branch costs need not be identical.
Shared overhead must not be subtracted as a constant. ASLR can still change
absolute addresses; one image fixes relative layout, not every execution state.
The off mode is an experimental control and does not replace an untouched
upstream baseline. See [design and limits](switch-design.md).

The prepared protocol retains the original 112 labels and the same 30 focused,
pipeline and positive/negative-tree cases. Eight serial Williams blocks use
mode/alias labels and process-level observations. No times have been collected
for this control. In particular, no conclusion about layout causing earlier
regressions follows from the correctness checks.

`audit.py` verifies saved builds, source snapshots, inputs, launch checks and
raw outputs without running the interpreter. Its
[correctness record](../evidence/rewrite-switch/correctness.json) explicitly
marks the performance gate deferred. The host-specific timing drivers require
the outer monitoring launcher to supply `session-protocol.json` with its actual
client date and to retain session flags. These drivers assume the preceding
offline build environment; the patch can be reviewed and applied independently
to the pinned Rum source.
