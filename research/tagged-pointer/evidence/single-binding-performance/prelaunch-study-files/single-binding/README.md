# Borrowed single-binding RHS candidate

Correctness validation completed; performance measurements are deferred.
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
points or every tight resource threshold. No full contextual-equivalence
proof or new speedup is claimed.

The prepared performance protocol compares original/matcher/single with two
A/A labels each and twelve balanced blocks. It retains all 112 original labels
and the same 30 focused/pipeline/tree cases; each process statistic is one
observation. No performance samples have been collected. First priority is to
measure the increment and complete-program cases against the already faster
matcher candidate, then inspect every original-suite regression.

`build.py` and `audit.py` use the preceding offline experiment environment.
The timing drivers expect the outer monitoring launcher's actual-date and
session-state metadata. The patch and regression source can be used independently
with the pinned upstream source and its standard toolchain.
