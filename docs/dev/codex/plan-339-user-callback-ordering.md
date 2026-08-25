# Plan 339 — user-callback ordering paths

This is a design-only result. No Rust, tests, scripts, generated reference, or
existing documentation was changed. The only repository change is this plan
and its draft follow-up briefs.

## 1. Hypothesis verdict: corrected

The hypothesis contains one real symptom but the proposed common cause is
wrong. Resumability did not generally turn short-circuiting equality scans
into exhaustive scans, and the custom sort path did not begin as a
divide-and-conquer sort that was later degraded. There are three distinct
implementations:

| Claim | Verdict | Evidence |
|---|---|---|
| The custom `Sort`, `SortBy`, and `ReverseSort` paths are quadratic on every input order. | **Confirmed.** They allocate an `n x n` Boolean relation table and evaluate every directed non-reflexive pair before sorting indexes from that table. That is exactly `n(n-1)` callback calls and `O(n^2)` relation storage. | `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:89-117,122-165,702-913`; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:210-240,363-454`; the exact pair-count tests are at `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs:75-94` and `crates/rum-stdlib/src/tests/structure_ops/collection_stats.rs:109-124`. |
| Resumability changed `DeleteDuplicates`, `Tally`, or the set operations from a short-circuiting scan to an exhaustive one. | **Refuted.** The resumable paths already finish the current candidate on the first exact `True`: `DeleteDuplicates` advances to the next input, `Tally` increments the first matching representative, `Union`/`Complement` finish the candidate, and `Intersection` advances to the next group. | `crates/rum-stdlib/src/structure_ops/dedup.rs:206-263`; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:92-151`; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:464-568,594-645`. |
| The custom sort lost an earlier divide-and-conquer algorithm because such state was hard to suspend. | **Refuted as history and as a machinery constraint.** The original custom-comparator design deliberately chose the all-pairs relation table as the simplest continuation shape and explicitly deferred merge sort; it was not a regression from a previous merge sort. The runtime can store an arbitrary owned `Send + 'static` state in a one-shot frame and carries remaining frames through a nested resuspension. | Historical design record `dca37c40:docs/dev/codex/brief-034-sort-comparator.md:59-74`; current frame representation and bounds at `crates/rum-runtime/src/runtime/suspension/core.rs:5-43,109-180,202-260`. |
| No `SameTest` path exits early. | **Refuted broadly; confirmed only for one shared family.** `DuplicateFreeQ` exits on the first duplicate, and the de-dup/tally/set paths above also exit at the first decisive match. In contrast, eight collection relations share `SameTestMatrixState`, fill every `lhs x rhs` cell, and compute their answer only after the matrix is complete. | `crates/rum-stdlib/src/predicates/collections.rs:331-399` versus `:24-32,63-239,401-515`; `ContainsExactly` additionally runs perfect bipartite matching at `:529-669`. |
| Association custom sorting shares the all-pairs-table defect. | **Refuted, but a worse independent defect exists.** `KeySort` and three-argument `KeySortBy` use recursive insertion sort. Before every comparison they eagerly clone the whole continuation state even when evaluation does not suspend. The result is quadratic comparisons and movement, heavy cloning, native recursion, and a release-build stack overflow at modest widths. | `crates/rum-stdlib/src/associations/transforms.rs:83-129`; `crates/rum-stdlib/src/associations/resumable/key_sort.rs:68-190,243-427`, especially the eager clones at `:112-136,259-280,342-368`. |

The corrected diagnosis is therefore: the expression-sort callback family was
implemented directly as a full relation table; the Association family was
implemented directly as recursive insertion; most equality callbacks already
have the right short-circuit state transition; and only the collection
relation matrix is deliberately exhaustive. Suspension is the context in
which these choices were made, but the runtime continuation representation is
not what prevents a merge sort (`crates/rum-runtime/src/runtime/suspension/core.rs:109-180`).

### Local probe method and load

The director's five-size measurements in brief 339 were accepted as given and
were not re-derived. The local work was the required inventory, callback-count,
call-order, and boundary probing. Every timed probe source was a separate rum
script invocation, run in its own release-CLI process with stdin redirected
from `/dev/null`; potentially long calls used
`scripts/run-bounded.py --timeout 5 -- ...`. Timed statements used in-kernel
`AbsoluteTiming`; collection operands were constructed before the timed
statement. P5 intentionally timed the named `TestCreate`/`TestReport` workload
itself. No profiling environment variable was enabled, and no external engine,
network, or external oracle was used. This follows the monotonic-timing and
rum-vs-rum rules in `docs/testing-strategy.md:74-94`.

The iteration/product-budget change noted in the task did not invalidate any
probe: no outer benchmark loop wrapped a generating construct, and the largest
generated input was 200 elements. Collection generation happened once before
timing; the bounded P5 table construction was itself the named workload. There
was no attempt to evade or reinterpret the generated-element limit.

| Probe batch | Shapes | Load before | Load after |
|---|---|---:|---:|
| P1 | identical versus distinct early-exit checks | `1.50 1.25 1.46` | `1.62 1.28 1.47` |
| P2 | expression sorts, de-dup, tally, and sets | `1.80 1.34 1.46` | `1.82 1.35 1.46` |
| P3 | collection predicates and ordering predicates | `1.58 1.33 1.45` | `1.61 1.34 1.45` |
| P4 | bounded Association sort width series | `2.00 1.46 1.49` | `1.92 1.45 1.49` |
| P5 | `TestCreate` / `TestReport` option path | `2.25 1.56 1.53` | `2.25 1.56 1.53` |

The binary callbacks used by the inventory had these rum shapes; each size and
builtin was evaluated in a fresh process, and `AbsoluteTiming[builtinCall]`
contained only the call shown in the inventory:

```text
equal = Function[{a, b}, calls = calls + 1; SameQ[a, b]];
never = Function[{a, b}, calls = calls + 1; False];
before = Function[{a, b}, calls = calls + 1; Less[a, b]];
```

P1 used `equal` with `Table[1,{n}]` and `never` with `Range[n]`. P2/P3 used
`never` for worst-case equivalence scans and `before` for ordering scans. P4
used ascending `AssociationThread[Range[n] -> Range[n]]`, which is the
worst-comparison insertion shape proved by
`associations/resumable/key_sort.rs:98-110,165-190,327-340,399-425`.

The P1 release results directly locate the mistaken “never exits” claim:

| Operation, `n=200` | Decisive/identical input | All-distinct/always-false input | Code conclusion |
|---|---:|---:|---|
| `DeleteDuplicates[..., test]` | `0.840443 ms`, 199 calls | `78.980613 ms`, 19,900 calls | First matching kept representative ends the candidate (`dedup.rs:242-247,257-262`). |
| `Tally[..., test]` | `0.846810 ms`, 199 calls | `60.326 ms`, 19,900 calls in P2 | First matching bin ends the candidate (`collection_stats.rs:128-150`). |
| `Union[..., SameTest -> test]` | `0.876906 ms`, 199 calls | `79.764419 ms`, 19,900 calls | De-dup match ends the candidate (`ordering_sets.rs:594-605,635-645`). |
| `DuplicateFreeQ[..., SameTest -> test]` | `0.088768 ms`, 1 call | `61.818 ms`, 19,900 calls in P3 | First match returns `False` (`predicates/collections.rs:381-398`). |
| `ContainsAny[..., SameTest -> test]` | `153.615696 ms`, 40,000 calls | `124.413 ms`, 40,000 calls in P3 | The answer is already true after cell 1, but the shared matrix still visits all cells (`predicates/collections.rs:422-475,501-503`). |

Brief 339 does not include the exact source of its identical/distinct
`DeleteDuplicates` probe. That builtin's rum surface is a positional second
binary test, not a `SameTest -> ...` option: it dispatches `[expr, test]`
directly (`dedup.rs:43-51,78-106`) and is absent from the registered
`SameTest` option family (`runtime/options/defaults.rs:95-99`). Identical input
only makes the first comparison decisive if that positional callback resolves
to exact System `True`. Thus the director's equal timings remain valid timing
observations, but without the source they do not establish which callback
result/path was measured. P1 used the supported positional form and recorded
the exact-True versus always-false call counts; the source discrepancy is
flagged rather than guessed.

An untimed one-process-per-case trace also showed the actual schedules:

```text
Sort[{3,1,2}, traced]
  result: {1,2,3}
  pairs:  {{3,1},{3,2},{1,3},{1,2},{2,3},{2,1}}

KeySort[<|1->...,2->...,3->...|>, traced]
  pairs:  {{2,1},{3,1},{3,2}}

DeleteDuplicates[{1,1,1}, traced]
  pairs:  {{1,1},{1,1}}

ContainsAny[{1,1},{1,1}, SameTest -> traced]
  result: True
  pairs:  four calls (the complete 2 x 2 matrix)
```

Those traces agree with the row-major directed-pair loop at
`ordering_sets.rs:839-913`, the insertion scan at
`associations/resumable/key_sort.rs:86-190`, the candidate short-circuit at
`dedup.rs:206-263`, and the matrix loop at
`predicates/collections.rs:422-475`.

## 2. Complete affected-builtin inventory

The catalog entries for the expression ordering/equivalence family are at
`crates/rum-stdlib/src/catalog/structure_ordering.rs:13-125,235-289`, the
ordering/collection predicates at
`crates/rum-stdlib/src/catalog/predicates.rs:238-297,317-356`, the Association
sorts at `crates/rum-stdlib/src/catalog/associations.rs:239-268`, and the
complete registered `SameTest` option list at
`crates/rum-runtime/src/runtime/options/defaults.rs:61-73,95-99`.

Unless another size is shown, the timing cells are `n=100` and `n=200` and
have the form `milliseconds / binary-callback calls`. Unary key applications
in `SortBy`/`KeySortBy` are not included in that count. The binary callback
returned `False` for equivalence scans and a normal strict ordering result for
ordering scans, so the table measures the worst callback-count shape without
changing input generation. The factor is time at the larger size divided by
time at the smaller size.

| Builtin | Callback surface | Small | Large | Factor / outcome | Classification and implementation evidence |
|---|---|---:|---:|---:|---|
| `Sort` | positional binary ordering | `42.056 / 9,900` | `175.282 / 39,800` | `4.17x` | **Defect:** full directed-pair table, `O(n^2)` calls and space (`structure_ops/ordering_sets.rs:89-117,839-913`). |
| `SortBy` | unary key plus optional binary key ordering | `42.005 / 9,900` | `179.648 / 39,800` | `4.28x` | **Defect in the binary-ordering arm:** keys are decorated first, then the same full table is filled (`ordering_sets.rs:122-165,647-837`). The ordinary key-only arm uses native sorting (`:647-701`). |
| `ReverseSort` | optional binary ordering | `42.529 / 9,900` | `179.916 / 39,800` | `4.23x` | **Defect:** independent copy of the full directed-pair table, followed by reverse (`structure_ops/collection_stats.rs:210-240,363-454`). |
| `KeySort` | optional binary key ordering | `n=50: 8.844 / 1,225` | `n=100: 42.712 / 4,950` | `4.83x`; `n=200` aborts with stack overflow | **Defect:** recursive insertion plus eager whole-state clones (`associations/transforms.rs:83-129`; `associations/resumable/key_sort.rs:68-190`). The default arm uses native sort (`transforms.rs:89-101`). |
| `KeySortBy` | unary key plus optional binary key ordering | `n=25: 2.183 / 300` | `n=50: 10.104 / 1,225` | `4.63x`; `n=75` is `24.624 / 2,775`; `n=100` aborts with stack overflow | **Defect in the three-argument arm:** recursive insertion and eager clones occur both around key evaluation and comparison (`associations/resumable/key_sort.rs:243-427`). The two-argument arm decorates once and uses native sort (`:193-239`). |
| `OrderedQ` | optional binary ordering | `0.499 / 99` | `0.942 / 199` | `1.89x` | **Healthy:** adjacent pairs only and immediate false exit; already iterative and resumable (`predicates/ordering.rs:36-149`). |
| `LexicographicOrder` | optional binary ordering | `0.869 / 200` | `1.706 / 400` | `1.96x` | **Healthy:** at most forward and reverse calls at each equal position and exits at the first ordered position (`predicates/ordering.rs:151-353`). |
| `DeleteDuplicates` | optional positional binary equivalence test | `15.652 / 4,950` | `61.793 / 19,900` | `3.95x` worst case | **Already short-circuits.** Worst-case quadratic is expected for an arbitrary binary equivalence relation with no hash key; the current reference says so (`docs/reference/builtins.md:1248-1255`; `structure_ops/dedup.rs:206-263`). |
| `Tally` | optional positional binary equivalence test | `15.492 / 4,950` | `60.326 / 19,900` | `3.89x` worst case | **Already short-circuits.** It stops at the first matching bin; the current reference records worst-case `O(n^2)` (`docs/reference/builtins.md:5433-5442`; `collection_stats.rs:92-151`). |
| `Union` | `SameTest` option | `15.272 / 4,950` | `63.228 / 19,900` | `4.14x` worst case | **Already short-circuits per candidate.** An always-false relation forces the triangular de-dup scan; a match finishes the candidate (`ordering_sets.rs:464-504,594-645`). |
| `Intersection` | `SameTest` option | `31.935 / 10,000` | `124.080 / 40,000` | `3.89x` worst case | **Already short-circuits per group/candidate.** A match advances to the next input group; an always-false two-list case scans the full membership product (`ordering_sets.rs:505-538,607-621`). |
| `Complement` | `SameTest` option | `46.319 / 14,950` | `185.405 / 59,900` | `4.00x` worst case | **Already short-circuits per candidate.** The measured calls are triangular de-dup plus rectangular exclusion membership; a membership match rejects immediately (`ordering_sets.rs:539-565,623-645`). |
| `DuplicateFreeQ` | `SameTest` option | `16.960 / 4,950` | `61.818 / 19,900` | `3.65x` worst case | **Already short-circuits globally:** the first true pair returns `False` (`predicates/collections.rs:331-399`). |
| `IntersectingQ` | `SameTest` option | `30.655 / 10,000` | `125.641 / 40,000` | `4.10x` | **Full-matrix family:** computes `any` only after all callback effects (`predicates/collections.rs:401-475,494-503`). |
| `DisjointQ` | `SameTest` option | `31.061 / 10,000` | `123.718 / 40,000` | `3.98x` | **Full-matrix family:** negates the same post-matrix `any` (`predicates/collections.rs:401-475,504-506`). |
| `SubsetQ` | `SameTest` option | `30.794 / 10,000` | `125.570 / 40,000` | `4.08x` | **Full-matrix family:** checks matched RHS columns only after the matrix (`predicates/collections.rs:401-475,507-509,521-523`). |
| `ContainsAll` | `SameTest` option | `32.293 / 10,000` | `125.973 / 40,000` | `3.90x` | **Full-matrix family:** same column-coverage result (`predicates/collections.rs:401-475,507-509`). |
| `ContainsAny` | `SameTest` option | `30.934 / 10,000` | `124.413 / 40,000` | `4.02x` | **Full-matrix family:** even an immediate true result still makes every call, as P1 demonstrated (`predicates/collections.rs:401-475,501-503`). |
| `ContainsExactly` | `SameTest` option | `31.026 / 10,000` | `123.870 / 40,000` | `3.99x` | **Full-matrix family:** then performs perfect bipartite matching; arbitrary relations can require the whole graph (`predicates/collections.rs:511-513,529-669`). |
| `ContainsNone` | `SameTest` option | `31.344 / 10,000` | `126.125 / 40,000` | `4.02x` | **Full-matrix family:** negates post-matrix `any` (`predicates/collections.rs:401-475,504-506`). |
| `ContainsOnly` | `SameTest` option | `31.047 / 10,000` | `125.170 / 40,000` | `4.03x` | **Full-matrix family:** row coverage is checked after all calls (`predicates/collections.rs:401-475,510,525-527`). |
| `TestCreate` | `SameTest` option stored in a test object | `0.874 / 0` | `1.643 / 0` | `1.88x` | **Not an ordering loop.** It parses and stores the option; it never invokes it (`runtime/debug_tools/test_report/create.rs:5-59`). |
| `TestReport` | report-level or per-test `SameTest` | `1.827 / 100` | `3.780 / 200` | `2.07x` | **Healthy linear consumer:** exactly one chosen comparison per evaluated test, suspension-safe through the evaluated-value continuation (`runtime/debug_tools/test_report/evaluation.rs:87-105`; `runtime/debug_tools/test_report/report.rs:123-157`). |

The eight full-matrix rows share one implementation, not eight duplicated
defects (`predicates/collections.rs:212-239,401-475`). Their current time is
quadratic because their current callback schedule is exactly the Cartesian
product. That schedule cannot be shortened under brief 339's observational
rule; this is addressed in section 4 rather than turned into a code step.

### Audited exclusions

These names were inspected because the task called them out, but they do not
accept a binary comparator or `SameTest` in rum's current catalog.

| Surface | Why it is outside the callback-ordering inventory | Evidence |
|---|---|---|
| `GatherBy` and the absent `Gather` | `GatherBy` evaluates unary key functions; there is no registered `Gather` builtin. It neither accepts nor synthesizes a binary equivalence test. | Catalog and implementation at `crates/rum-stdlib/src/catalog/structure_ordering.rs:58-69` and `crates/rum-stdlib/src/structure_ops/collection_stats.rs:276-316,456-536`; the catalog contains only the `GatherBy` registration in this family. |
| `Count`, `MemberQ`, `Position` | They are pattern/level/`Heads` tree walkers, not equality callbacks. | `crates/rum-runtime/src/runtime/tree_walkers/count_like.rs:4-46`; `crates/rum-runtime/src/runtime/tree_walkers/position.rs:4-54`; registrations at `crates/rum-runtime/src/runtime/builtin_catalog/rules.rs:404-433,508-534`; `Heads` defaults at `runtime/options/defaults.rs:22-24,47-49`. |
| `Counts`, `CountsBy`; absent `CountBy` / `CountDistinct*` | `Counts` uses structural keys and `CountsBy` applies one unary transform. The other queried names have no current catalog registration. | `crates/rum-stdlib/src/catalog/associations_transform.rs:255-290`; unary `CountsBy` driver at `crates/rum-stdlib/src/associations/resumable/aggregate.rs:81-115`. |
| `DeleteDuplicatesBy`, `ReverseSortBy`, `MaxBy`/`MinBy` and related `*By` selectors | Their user function computes a unary key. Their ordering is native/canonical and there is no binary user relation. | `crates/rum-stdlib/src/catalog/structure_ordering.rs:25-37,114-205`; `structure_ops/dedup.rs:109-204`; `structure_ops/collection_stats.rs:243-275`. |
| `KeyUnion`, `KeyIntersection`, `KeyComplement` | Association keys use structural `HashMap`/`HashSet` membership. `KeyUnion`'s optional function fills missing values; it is not a comparator. | `crates/rum-stdlib/src/associations/transforms/key_collections.rs:6-33,75-289`; catalog at `catalog/associations_transform.rs:48-105`. |
| `KeyMemberQ`, `KeyFreeQ` | They use structural key equality and expose no `SameTest`. | `crates/rum-stdlib/src/associations/basic/key_predicates.rs:3-67,75-126`; catalog at `crates/rum-stdlib/src/catalog/associations.rs:193-228`. |

## 3. Step-by-step implementation plan

The work is four independently gateable steps. Step 1 is useful regardless of
the semantic decision. Steps 2–4 are blocked until the director ratifies the
callback-schedule contract in Open Question 1. There is no `SameTest`
early-exit step because such a step would fail the task's hard observational
condition.

| Step | Result | Dependency | Mechanical gate |
|---|---|---|---|
| 1 | Checked-in callback-sort target labels and default-path sentinels | None | Script syntax, two complete suite runs, standalone result/call-count validation |
| 2 | Public contract says valid ordering predicates determine results while invocation pair/count/order and interleaving are unspecified | Director approval | Generated-reference drift, focused docs tests, workspace tests |
| 3 | Shared iterative, resumable stable merge engine used by `Sort`, custom `SortBy`, and `ReverseSort` | Steps 1–2 | Correctness, strict call upper bound, every-comparison suspension test, stack width, A/B |
| 4 | `KeySort` and three-argument `KeySortBy` migrated from recursive insertion/eager cloning to the shared engine | Steps 1–3 | Association preservation, key-once/order, suspension, wide stack safety, A/B |

### Step 1 — establish permanent measurements

Modify only `scripts/core-language-microbench.sh`. Add small/large custom
callback pairs for `Sort`, `SortBy`, `ReverseSort`, `KeySort`, and `KeySortBy`,
plus one wide default-path sentinel for each. The exact timed expressions are
listed in draft brief 340. Inputs are prepared outside the timed statement;
the timed statement contains only the builtin call, as required by
`docs/dev/codex/TEMPLATE.md:78-98`.

Use list sizes 100/200, `KeySort` sizes 50/100, and `KeySortBy` sizes 25/50.
Those Association sizes are intentionally below the observed base-build crash
bound. Run default scale only. The result of this step is measurement coverage,
not a performance claim and not a new harness. The suite already emits
per-label `timings.tsv` rows from monotonic `AbsoluteTiming`
(`docs/testing-strategy.md:74-87`).

The script is currently 19,076 characters, already in the organization warning
range (`docs/code-organization-guidelines.md:64-81`). The step must report its
before/after size. This plan does not mix a harness extraction into the
measurement-only diff; if the new labels take it into the split-plan range, the
future boundary is the static generated-rum label catalog versus the shell
runner/parser. A later panel should extract feature-oriented `.rum` fragments
before adding more flat heredoc content.

Gate:

1. `bash -n scripts/core-language-microbench.sh`.
2. Inspect the generated `.rum` source and quote the fifteen exact statements.
3. Run two complete bounded suite repetitions with profiling off, reporting
   load before/after, status, row count, and the fifteen rows.
4. In separate release processes, verify sorted results and current callback
   counts; do not put counters or validation inside timed labels.
5. Confirm no existing statement, TSV schema, repetition policy, or harness
   behavior changed.

### Step 2 — ratify and publish the semantic boundary

Update the catalog summaries for `Sort`, `SortBy`, `ReverseSort`, `KeySort`,
and `KeySortBy`, regenerate `docs/reference/builtins.md`, and update the
decision-gated TODO note. The contract should say:

- A binary ordering callback is expected to define a deterministic strict weak
  ordering over the compared values/keys. For such a callback, the sorted
  value is contractual.
- The implementation may select, omit, or reorder comparison pairs. The number,
  direction, order, and relative interleaving of comparator invocations are not
  contractual. Programs must not use comparator side effects as a sequencing
  API.
- A selected comparison may request input. If it does, rum preserves and
  resumes that one pending invocation exactly; it does not restart the sort.
- `Sort`, custom `SortBy`, `KeySort`, and custom `KeySortBy` are stable for
  comparator ties. `ReverseSort` is the reverse of that ascending arrangement,
  so reversal also reverses a tied run.
- `SortBy` and `KeySortBy` still evaluate the unary key specification exactly
  once per input element in input order; for a `SortBy` list specification,
  each listed key function is applied once in list order. Key/comparator
  interleaving is unspecified (`ordering_sets.rs:961-972`).
- Preserve the existing truth interpretation separately: expression sorts use
  exact System `True` (`structure_ops/mod.rs:65-79`); Association sorts retain
  `key_sort_ordering_decision` including its current integer/truthiness cases
  (`associations/resumable/key_sort.rs:429-444`). This plan does not silently
  normalize those two existing language surfaces.

This is an intentional contract clarification/change because current callback
effects are observable even though the generated reference does not promise a
schedule. Documentation must precede tests that stop pinning the accidental
`n(n-1)` schedule; the testing policy requires intentional changes to update
docs and tests together (`docs/testing-strategy.md:60-72,103-117`).

`catalog/structure_ordering.rs` is 11,945 characters (warning range) and
`catalog/associations.rs` is 9,429 (preferred range). Step 2 changes only five
cohesive registration records; splitting the ordering catalog while changing
its wording would be artificial. Its report must record both sizes and that
the future real boundary, if the catalog grows, is ordering/de-dup registrations
versus set-operation registrations (`docs/code-organization-guidelines.md:64-97`).

Gate: regenerate the reference, run the reference drift test and workspace
tests, and review that no result semantics or default forms changed. If the
director does not approve the unspecified schedule, stop after Step 1: an
asymptotically better comparison sort cannot preserve every currently observed
directed callback call.

### Step 3 — expression sorts on one resumable merge engine

Create `crates/rum-stdlib/src/structure_ops/resumable_merge_sort/mod.rs` with
focused state tests in sibling `tests.rs`, and expose only a narrow
`pub(crate)` internal entry from
`crates/rum-stdlib/src/structure_ops/mod.rs`. Move the three custom-comparator
drivers out of `ordering_sets.rs` and `collection_stats.rs`; leave their
default branches using the current native sort unchanged
(`ordering_sets.rs:89-165`; `collection_stats.rs:210-240`).

The engine should be an iterative stable bottom-up merge, generic over the
owned item and a small completion context. Its owned state contains:

- the ordering expression and result-policy function;
- completed runs for the current pass (or an equivalent `O(n)` run plan);
- the current left and right runs;
- the cursor in each run;
- the partial merged output;
- the next-pass runs and final rendering context.

For each merge decision compare the right candidate against the left
candidate. Move the right item only when the family-specific callback policy
says it precedes the left; otherwise move the left item. Choosing left on a
tie is the stability rule. Drain a remaining run without callback calls. A
loop, not native recursion, advances runs and passes. Total comparisons are at
most `n * ceil(log2(n))`, total live run/scratch elements are `O(n)`, and no
`n x n` Boolean table remains.

`SortBy` must retain its existing input-order, exactly-once key decoration
before entering the merge driver (`ordering_sets.rs:647-701`). `ReverseSort`
uses the same ascending engine and reverses only at completion, matching its
documented “sort, then reverse” shape (`catalog/structure_ordering.rs:103-113`).

The critical evaluation pattern is:

1. Build the pending callback expression from borrowed/current candidates.
2. Call `session.evaluate`.
3. On `Ok(value)`, apply the result directly without constructing or cloning a
   continuation snapshot.
4. Only on `EvalError::InputRequired` move the single owned merge state into a
   `suspend_with_frame` closure.
5. On another error, propagate it unchanged.

This avoids the eager-clone pattern visible in the Association insertion code
and uses the frame exactly as intended by
`suspension/core.rs:109-180`. The current list sort loops already pattern-match
`InputRequired` this way (`ordering_sets.rs:764-810,839-899`); retain that good
part while replacing their algorithm.

Gate tests:

- empty/singleton/ascending/descending/mixed/tied values for all three forms;
- stable tagged ties for `Sort` and `SortBy`, and reversed tied order for
  `ReverseSort`;
- a deterministic strict ordering callback whose output agrees with the
  corresponding default-order result on the covered exact integer subset;
- `calls <= n * ceil(log2(n))` at a non-power-of-two width, replacing—not
  weakening—the tests that currently require `n(n-1)` at
  `tests/structure_ops/ordering_sets.rs:75-94` and
  `tests/structure_ops/collection_stats.rs:109-124`;
- a wide input that proves no native-frame growth;
- a comparator that suspends on every chosen comparison, repeatedly resumes,
  and produces the same final value and pair trace as an uninterrupted run of
  the new engine;
- existing key-suspension coverage for `SortBy`
  (`tests/session_requests/functional_sequence_resume.rs:50-76`);
- A/B on all Step-1 expression-sort target rows and default sentinels, plus the
  script's same-tier core controls. The target call-count quotient must change
  from quadratic to the stated bound; timing must materially improve, and no
  default/same-tier row may regress.

### Step 4 — migrate Association custom ordering

Replace `resume_key_sort_with_ordering_*` and the custom-ordering
`resume_key_sort_by_ordering_*` insertion functions in
`crates/rum-stdlib/src/associations/resumable/key_sort.rs:68-190,243-427` with
adapters to the shared merge engine. Keep default `KeySort` at
`associations/transforms.rs:89-101` and default/key-only `KeySortBy` at
`associations/resumable/key_sort.rs:193-239` unchanged.

The adapter for `KeySort` compares `AssociationEntryForm.key`. The adapter for
three-argument `KeySortBy` first evaluates one key per entry, left-to-right,
using the same “move state only on `InputRequired`” pattern, then sorts decorated
entries. The director-approved Step-2 contract permits comparator/key
interleaving to change, but exactly-once input-order key evaluation remains a
hard invariant. Render with
`render_association_entry_forms_preserving_heads`, so rule versus delayed-rule
heads and Association versus rule-list output remain intact
(`associations/resumable/key_sort.rs:77-83,253-256`; existing behavior tests at
`tests/strings/incidental_runtime/association_key_ops.rs:249-331`).

Retain `key_sort_ordering_decision` byte-for-byte unless the module boundary
requires only a visibility change (`key_sort.rs:429-444`). This step changes
the selection and order of comparator calls, not what a returned comparator
value means.

Gate tests:

- Association and rule-list inputs; immediate and delayed rules; ascending,
  descending, ties, and transformed ties;
- key function called exactly once per entry and in input order;
- the current `1`, `0`, `-1`, System `True`, System `False`, and non-Boolean
  comparator-result interpretation;
- a strict comparison-count upper bound and a width comfortably above 200,
  proving the two observed stack overflows are gone;
- comparator suspension on every chosen comparison and key suspension at every
  key, with uninterrupted/resumed final value and trace identity;
- all existing request tests, including `KeySort` at
  `tests/incidental_session_requests_transforms_resume_associations.rs:91-115`
  and `KeySortBy` at
  `tests/strings/incidental_runtime/session_requests.rs:172-225`;
- A/B on all Step-1 Association target/default rows and same-tier Association
  controls. Call counts must satisfy the merge bound, both former crash probes
  must complete, and no default/same-tier row may regress.

### Suspension invariants shared by Steps 3–4

1. **One pending decision.** At suspension, state identifies exactly one
   comparison by pass/run and left/right cursor. Its callback has begun, but
   its Boolean/ordering decision has not been applied.
2. **Exactly-once application.** Resume consumes the returned value once,
   appends exactly one chosen item, and advances exactly one cursor. It never
   calls the pending comparison again.
3. **Prefix preservation.** Completed runs, the merged-output prefix, decorated
   keys, and untouched suffixes are owned by the frame and are byte-for-byte
   the same after resume.
4. **Nested resuspension.** If a later comparison requests input, the new frame
   is appended ahead of remaining outer frames exactly as
   `suspension/core.rs:222-260` specifies; no source or completed merge work is
   replayed.
5. **Key exactly once.** `SortBy` and `KeySortBy` never re-evaluate a key whose
   value is already in the state, including after repeated input requests.
6. **Uninterrupted equivalence.** For the same sequence of callback return
   values, an uninterrupted run and a run suspending at every callback produce
   the same result and the same new-algorithm pair trace.
7. **Error transparency.** Non-input errors propagate without installing a
   merge continuation or rendering partial output.

These are testable with rum's host-facing request API; existing tests already
demonstrate the basic one-frame pattern for list sorts, set tests, predicates,
and `KeySort` (`tests/session_requests/functional_sequence_resume.rs:79-108`;
`tests/session_requests/set_same_test_resume.rs:11-52`;
`tests/session_requests/predicate_collections_resume.rs:11-58`;
`tests/incidental_session_requests_transforms_resume_associations.rs:91-115`).

### Module-size and fan-out assessment

`ordering_sets.rs` is 33,057 characters and `collection_stats.rs` is 26,506,
both in the mandatory split-plan range. `key_sort.rs` is 13,208, in the warning
range. The thresholds and assess-on-touch requirement are
`docs/code-organization-guidelines.md:64-97,193-209`.

Creating the `structure_ops/resumable_merge_sort/` module is the semantic split:
callback-driven ordering and its continuation state leave two mixed,
split-plan files. It should shrink both rather than add new behavior to them.
`structure_ops` currently has 15 direct children; the new module makes 16,
below the approximately-20 soft ceiling at
`docs/code-organization-guidelines.md:149-160`. Migrating Association sorting
then deletes most of the 13k insertion module rather than growing it. No new
root-level stdlib module and no runtime-crate change is warranted. Keeping the
focused state tests in `resumable_merge_sort/tests.rs` also follows the rule to
measure implementation separately from in-file tests
(`docs/code-organization-guidelines.md:83-87,99-120`).

If either touched implementation remains above 20k after the move, its report
must record the next real boundary: set-operation scanning/options can leave
`ordering_sets.rs`, while grouping/tally can be separated from extremal and
reverse-order operations in `collection_stats.rs`. Do not perform those
unrelated splits inside the algorithm patch. The existing broad Association
test file is 15,186 characters (test warning range), so Step 4 creates the
focused `associations_key_sort.rs` instead of growing it; the Association test
directory remains below the direct-child ceiling
(`docs/code-organization-guidelines.md:99-120,149-160`).

## 4. Answers to the five questions

### 1. Is a resumable merge sort worth building?

**Yes, after the callback-schedule contract is ratified.** The director's
`n=800` result is 955.985 ms for a custom comparator versus 0.019 ms for the
default path, while the code proves `n(n-1)` callback calls plus an `n x n`
table (`ordering_sets.rs:105-116,839-913`). The Association implementations
add actual stack-overflow failures by `n=100`/`200`, backed by the recursive
calls at `key_sort.rs:169-189,403-425`. This is both a performance and a
correctness/robustness problem.

The existing continuation API supports the required shape: a frame can own
arbitrary `Send + 'static` run/cursor/output state and is resumed once
(`suspension/core.rs:24-43,109-180`). Risk is moderate, not architectural: the
hard parts are stable tie handling, moving state without eager clones, and
proving exactly-once progress under repeated suspension. The invariants and
tests above make those risks mechanical. No runtime API or `unsafe` is needed.

### 2. Can `SameTest` exit early without changing results?

**The paths that can preserve today's observable schedule already do. The
full-matrix family cannot drop calls under the rule in brief 339.**

For `DeleteDuplicates`, `Tally`, the set operations, and `DuplicateFreeQ`, a
true result already ends the relevant scan (`dedup.rs:242-262`;
`collection_stats.rs:128-150`; `ordering_sets.rs:594-645`;
`predicates/collections.rs:381-398`). No patch is needed.

For `IntersectingQ`, `DisjointQ`, `SubsetQ`, and the five `Contains*`
relations, every `lhs x rhs` callback call currently occurs in row-major order
before the answer is computed (`predicates/collections.rs:422-475,494-527`). A
user callback can mutate state, print, emit messages, trace, or suspend.
Returning after an already decisive Boolean would make calls disappear. That
violates the explicit requirement that no currently observed call disappear,
even when the final Boolean is unchanged. There is no purity/effect proof in
this evaluator path, so skipping a general user callback is not provably
unobservable.

The exact guarantee in this plan is therefore: all equality/SameTest paths are
left untouched; their existing call order/count and suspension behavior remain
unchanged. A special case for apparent `SameQ` is not proposed because tracing
or changed definitions can make evaluation itself observable, and it would not
help a general user callback.

### 3. What is the full affected inventory?

The complete inventory is section 2: seven binary ordering consumers; fourteen
de-dup/set/collection binary-equivalence consumers; and the two debug option
surfaces, of which only `TestReport` invokes the selected `SameTest`. The
registration proof is the union of
`catalog/structure_ordering.rs:13-125,235-289`,
`catalog/predicates.rs:238-297,317-356`,
`catalog/associations.rs:239-268`, and
`runtime/options/defaults.rs:61-73,95-99`. The audited exclusions explain every
name specifically requested by brief 339.

Only five builtins need algorithm work: `Sort`, the three-argument/custom arm
of `SortBy`, `ReverseSort`, custom `KeySort`, and custom `KeySortBy`. The eight
matrix consumers share quadratic behavior but are not legally optimizable by
early exit under the required observable boundary. The remaining consumers
are already linear or already short-circuit at the earliest result-preserving
point.

### 4. What are the suspension invariants?

They are the seven numbered invariants in section 3: one pending decision,
exactly-once application, owned prefix preservation, correct nested
resuspension, keys evaluated once, uninterrupted/resumed equivalence, and
transparent non-input errors. The definitive test suspends on every comparison
and asserts both final value and callback-pair trace against an uninterrupted
run of the same new engine. This is stronger than only suspending on the first
pair, which is all the current sort request test covers
(`tests/session_requests/functional_sequence_resume.rs:79-108`).

“Same result as uninterrupted” means the same result as an uninterrupted run
of the ratified new algorithm for the same callback returns. It cannot also
mean reproducing the old full directed-call trace: eliminating that trace is
the optimization and requires the Step-2 contract decision.

### 5. What is the step order and why is each step independently gateable?

Measurements land first because every later hot-path change needs stable target
and default-neighbor rows (`docs/testing-strategy.md:74-94`). The semantic
contract lands second because tests must not discard an observable schedule
before the normative reference declares it unspecified
(`docs/testing-strategy.md:60-72`). The generic expression-sort engine lands
third and proves the continuation algorithm in the simpler `Expr` setting.
Association adapters land fourth, reusing that gate while separately proving
entry-head preservation, key evaluation, and removal of native recursion.

There is no lower-risk early-exit step to put first: the relevant de-dup/set
states already early-exit, and changing the matrix family is semantically
forbidden under the task's rule. Each proposed step has a disjoint main scope
and can be reviewed, tested, and rejected without partially landing the next.

### Open questions for the director

1. **Required semantic decision:** approve or reject the proposed ordering
   callback contract—strict weak ordering determines the result; comparator
   pair/count/order/direction and key/comparator interleaving are unspecified.
   Recommendation: approve. Without it, Steps 2–4 must not run; only a
   schedule-preserving iterative cleanup is legal, and expression sorts remain
   `O(n^2)`.
2. **SameTest policy:** should the project deliberately relax brief 339's
   observational rule for the eight matrix predicates in a future design?
   Recommendation: no in this lane. If the answer is yes, it needs a separate
   language-contract brief covering callback effects and each relation's
   logical stopping condition; it is not an automatic optimization.
3. **Association comparator return values:** should the unusual current rule
   that an unrecognized/non-Boolean result means “insert before” become public
   contract, or merely remain compatible while a later semantic brief reviews
   it? `key_sort_ordering_decision` implements that rule at
   `associations/resumable/key_sort.rs:429-444`. Recommendation: preserve it in
   this lane without promoting it; do not mix comparator-value cleanup with
   algorithm replacement.
4. **Director de-dup probe source:** what exact positional callback expression
   produced the all-identical/all-distinct timings in brief 339? Recommendation:
   attach the exact rum statement to the review record. If it used the
   supported positional form and returned exact `True` for identical values,
   reproduce that exact statement on this revision before revisiting the P1/code
   verdict; if it used an option-shaped expression or a callback that never
   returned exact `True`, the equal timing is the expected worst-case scan.

## 5. Draft follow-up implementation briefs

The drafts below follow `docs/dev/codex/TEMPLATE.md`. They are raw material for
director review and renumbering; per the template's plan-first rule they must
not be dispatched unchanged (`TEMPLATE.md:17-37`).

---

### Draft brief 340 — callback-ordering microbench labels

#### Goal

Add a permanent, bounded timing panel for the five custom callback-sort paths
and their five default-path sentinels, without changing Rust or claiming an
optimization. The panel must reproduce the current complexity separation and
provide exact target rows for later rum-vs-rum A/B gates.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`, and
  `docs/dev/codex/plan-339-user-callback-ordering.md`.
- Existing harness and `rumMicroOnce` contract:
  `scripts/core-language-microbench.sh:193-296`.
- Existing label placement and terminal marker:
  `scripts/core-language-microbench.sh:305-534`.
- Performance rules: `docs/testing-strategy.md:74-94`.
- Acceptance is rum's own release CLI and checked-in microbench output, never
  an external engine.

#### Scope

- File you MAY modify:
  - `scripts/core-language-microbench.sh` — only its generated `.rum` source
    setup/labels and the matching `ClearAll` list if one is needed.
- Files/areas you MUST NOT touch: every Rust file, every doc, every other
  script, generated output, Cargo files, `.git`, and git config.
- Do not change existing labels, statements, repetitions, environment, parser,
  or TSV schemas.

#### Deliverable

Prepare dedicated immutable inputs outside timing and add these exact
`rumMicroOnce` statements before `RUM_CORE_MICRO_END`:

| Label | Size/setup | Exact timed statement |
|---|---|---|
| `sort-callback-small` | reversed List, 100 | `Sort[gMicroSortCallbackSmall, Less]` |
| `sort-callback-large` | reversed List, 200 | `Sort[gMicroSortCallbackLarge, Less]` |
| `sortby-callback-small` | reversed List, 100 | `SortBy[gMicroSortByCallbackSmall, Identity, Less]` |
| `sortby-callback-large` | reversed List, 200 | `SortBy[gMicroSortByCallbackLarge, Identity, Less]` |
| `reversesort-callback-small` | ascending List, 100 | `ReverseSort[gMicroReverseSortCallbackSmall, Less]` |
| `reversesort-callback-large` | ascending List, 200 | `ReverseSort[gMicroReverseSortCallbackLarge, Less]` |
| `keysort-callback-small` | ascending Association, 50 | `KeySort[gMicroKeySortCallbackSmall, Less]` |
| `keysort-callback-large` | ascending Association, 100 | `KeySort[gMicroKeySortCallbackLarge, Less]` |
| `keysortby-callback-small` | ascending Association, 25 | `KeySortBy[gMicroKeySortByCallbackSmall, Identity, Less]` |
| `keysortby-callback-large` | ascending Association, 50 | `KeySortBy[gMicroKeySortByCallbackLarge, Identity, Less]` |
| `sort-default-wide` | reversed List, 1600 | `Sort[gMicroSortDefaultWide]` |
| `sortby-default-wide` | reversed List, 1600 | `SortBy[gMicroSortByDefaultWide, Identity]` |
| `reversesort-default-wide` | ascending List, 1600 | `ReverseSort[gMicroReverseSortDefaultWide]` |
| `keysort-default-wide` | descending Association, 1600 | `KeySort[gMicroKeySortDefaultWide]` |
| `keysortby-default-wide` | descending Association, 1600 | `KeySortBy[gMicroKeySortByDefaultWide, Identity]` |

Use `Range`/`Reverse[Range]` and `AssociationThread` only in the setup region,
never inside the timed statement. Use the exact sizes above at default scale;
do not raise Association sizes past the base-build crash boundary. Add no
counter, assignment, `Print`, sink, or validating read to a timed expression.

In standalone, separately processed probes, validate each result and record
current call counts for the ten callback labels. The expected base mechanism
is `n(n-1)` for the three expression-sort pairs and `n(n-1)/2` for ascending
Association insertion; a mismatch is a diagnostic to report, not a label to
massage.

#### rum / Rust pitfalls (read twice)

1. The generated source file must keep the `.rum` suffix; do not introduce a
   Wolfram-language suffix.
2. Data construction is outside the timed expression. `rumMicroOnce` must time
   the exact builtin statement in the table.
3. `KeySortBy` at 100 and `KeySort` at 200 crash on this base. Do not use those
   widths in the checked-in base panel.
4. Profiling/diagnostic variables are off for all timings.
5. The product budget is real. Do not wrap a generating setup in the repeated
   timed loop; create each small input once before timing.
6. One label has one variable. Do not combine sorting with result validation.
7. Persist raw suite output before parsing and report load around every timing
   run. Do not wait for a quieter load.
8. `scripts/core-language-microbench.sh` starts at 19,076 characters. Report
   its before/after size and the plan's generated-rum-label versus shell-runner
   future split boundary if it crosses 20k; do not refactor the harness here.

#### Verification (MANDATORY — run and report verbatim)

Use a timeout derived from one prior unmodified suite run and the bounded
runner for the two final runs:

```text
bash -n scripts/core-language-microbench.sh
bash scripts/core-language-microbench.sh --output-dir target/codex/brief340-run-1
bash scripts/core-language-microbench.sh --output-dir target/codex/brief340-run-2
git diff --check
git status --short
```

Report the generated `.rum` path and quote all fifteen exact statements. Both
suite runs must have `status=ok`, `exit_code=0`, a complete label table, and
exactly fifteen more rows than the pre-change suite. Paste all fifteen
`timings.tsv` rows from both runs. Report before/after load, suite wall time,
and timeout headroom.

Run one standalone release process per callback-count/result assertion with
stdin from `/dev/null`; paste its expression, result, and count. Rust format and
tests are not required because Scope contains no Rust; say so explicitly.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Modify exactly the one Scope file.
- No Rust, docs, generated artifacts, second timing harness, network, or
  external engine.
- Do not change an existing label or harness behavior.
- Do not put setup, assignment, a sink, `Print`, or validation in a timed
  statement.
- Every rum source created by the script or for a probe uses `.rum`.
- Each CLI probe is its own process/script with stdin from `/dev/null`.
- Report load with every timing and observe the product budget; do not work
  around a generated-element limit by changing runtime policy.
- Keep probe and suite outputs under repo-local ignored `target/codex/brief340/`
  paths; touch nothing outside the repo.

#### Report format

1. Per-file change summary and exact generated-source statements.
2. Fifteen verbatim rows from each suite run.
3. Small/large quotient table and current callback counts.
4. Default-path sentinel and existing-neighbor disturbance summary.
5. Load averages, wall time, and timeout headroom.
6. Standalone correctness results.
7. Script before/after size and split assessment.
8. `bash -n`, `git diff --check`, and final `git status --short` output.

---

### Draft brief 341 — define the ordering-callback schedule contract

#### Goal

Publish the semantic boundary required for an asymptotically better custom
sort: deterministic strict-weak-order callbacks determine the sorted result,
while comparison selection, direction, count, order, and key/comparator
interleaving are unspecified. Preserve exactly-once input-order key-specification
evaluation (including each member of a `SortBy` function list) and the existing
family-specific interpretation of callback return values. Make no algorithm
change.

**Dispatch precondition:** the director must explicitly approve this contract.
If approval is absent or the chosen wording still guarantees today's complete
directed call schedule, stop without editing.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/dev/codex/plan-339-user-callback-ordering.md`, and the brief-340 report.
- Normative catalog entries:
  `crates/rum-stdlib/src/catalog/structure_ordering.rs:70-125` and
  `crates/rum-stdlib/src/catalog/associations.rs:239-268`.
- Generated references:
  `docs/reference/builtins.md:2590-2608,4439-4457,4790-4808`.
- Current truth policies:
  `crates/rum-stdlib/src/structure_ops/mod.rs:65-79` and
  `crates/rum-stdlib/src/associations/resumable/key_sort.rs:429-444`.
- Current decision gate: `TODO.md:60-64`.
- Documentation is the test contract: `docs/testing-strategy.md:60-72`.

#### Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/catalog/structure_ordering.rs` — only `Sort`,
    `SortBy`, and `ReverseSort` summaries;
  - `crates/rum-stdlib/src/catalog/associations.rs` — only `KeySort` and
    `KeySortBy` summaries;
  - `docs/reference/builtins.md` — regenerated output only, never hand-edited;
  - `TODO.md` — only the custom `Sort`/`KeySort` decision-gate paragraph, to
    record the approved decision and pending implementation.
- Files/areas you MUST NOT touch: algorithm implementations, tests, scripts,
  other docs/catalog entries, runtime/core, Cargo files, `.git`, and git config.
- Do not broaden or standardize comparator return-value semantics in this
  brief.

#### Deliverable

1. Each of the five reference entries states the approved strict-weak-order
   result precondition and that comparator call selection/direction/count/order
   are unspecified.
2. `SortBy` and `KeySortBy` state that the key specification is evaluated once
   per input element in input order; each member of a `SortBy` function list is
   applied once in list order. Relative interleaving with comparator calls is
   unspecified.
3. Stable ties are explicit for `Sort`, custom `SortBy`, `KeySort`, and custom
   `KeySortBy`; `ReverseSort` explicitly reverses the ascending arrangement.
4. Input suspension is supported for a comparison chosen by the
   implementation and resumes without restarting the operation.
5. `KeySort`'s catalog text acknowledges its existing optional binary ordering
   form; do not invent a new form.
6. `TODO.md` records that the semantic decision is settled and the measured
   implementation work remains pending; do not mark it complete.
7. Regenerated reference exactly matches the catalog.

#### rum / Rust pitfalls (read twice)

1. This is a language-contract change even though today's reference did not
   promise a count. User callback effects are observable.
2. Do not promise `O(n log n)` here; this brief changes no implementation.
3. “Strict weak ordering” governs guaranteed sorted results. It does not make
   impure callbacks impossible; it states that their scheduling effects are
   not a sequencing contract.
4. Preserve exact System-`True` behavior for expression sorts and the current
   Association return-value policy. Those are separate semantics.
5. `ReverseSort` reverses the complete ascending result; do not accidentally
   promise stable input order inside reverse ties.
6. The reference is generated. Edit catalog source, then regenerate.
7. Report before/after sizes for both touched catalogs. Keep the cohesive data
   modules intact in this wording-only change; record the ordering-versus-set
   registration boundary if `structure_ordering.rs` remains above 10k.

#### Verification (MANDATORY — run and report verbatim)

Use `scripts/run-bounded.py` with bounds derived from prior same-shape runs:

```text
cargo fmt --check
cargo run -p rum-stdlib --bin generate-builtin-reference
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
git diff --check
git status --short
```

Paste the five generated reference paragraphs and the TODO paragraph. Confirm
that the generator is clean on a second run. No timing A/B is required because
no executable path changes; say so.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Do NOT proceed without the director's explicit contract approval.
- Modify only Scope files; `docs/reference/builtins.md` is generator output.
- No algorithm, tests, scripts, attributes, arity, option, or callback-return
  changes.
- Testing Policy is a hard line: rum's own documented contract only, no
  external oracle, differential harness, compatibility matrix, or network.
- No `unsafe`; no runtime/core changes.
- Touch nothing outside the repo. Keep any command output under the repo-local,
  ignored `target/codex/brief341/` directory.

#### Report format

1. The approved contract wording, quoted verbatim.
2. Per-file change summary and catalog size/split assessment.
3. Five generated-reference excerpts and TODO excerpt.
4. Generator, test, format, and diff-check output.
5. Confirmation that no executable behavior or benchmark changed.
6. Final `git status --short` limited to Scope.
7. Stop-loss/open questions.

---

### Draft brief 342 — resumable merge sort for expression ordering

#### Goal

Replace the all-directed-pairs custom comparator tables used by `Sort`,
three-argument `SortBy`, and `ReverseSort` with one safe-Rust, stable,
iterative, resumable bottom-up merge engine. Achieve `O(n log n)` callback
calls and `O(n)` auxiliary elements while preserving documented results,
exactly-once key evaluation, messages/errors, and input suspension. Leave every
default sort path unchanged.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`,
  `docs/dev/codex/plan-339-user-callback-ordering.md`, and final briefs
  340–341 reports.
- Current custom sort/table paths:
  `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:89-165,702-913` and
  `crates/rum-stdlib/src/structure_ops/collection_stats.rs:210-240,363-454`.
- Shared structure helpers:
  `crates/rum-stdlib/src/structure_ops/mod.rs:1-79`.
- Suspension contract:
  `crates/rum-runtime/src/runtime/suspension/core.rs:5-43,109-180,202-260`.
- Current behavior/pair-count tests:
  `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs:1-110` and
  `crates/rum-stdlib/src/tests/structure_ops/collection_stats.rs:75-124`.
- Current request coverage:
  `crates/rum-stdlib/src/tests/session_requests/functional_sequence_resume.rs:50-108`.
- Acceptance is the ratified rum reference, focused tests, and checked-in
  brief-340 A/B labels.

#### Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/structure_ops/mod.rs` — module declaration and
    minimal internal visibility only;
  - `crates/rum-stdlib/src/structure_ops/ordering_sets.rs` — remove/adapt only
    custom `Sort`/`SortBy` table machinery;
  - `crates/rum-stdlib/src/structure_ops/collection_stats.rs` — remove/adapt
    only custom `ReverseSort` table machinery;
  - `crates/rum-stdlib/src/tests/structure_ops/ordering_sets.rs` — result,
    stability, complexity, and width tests;
  - `crates/rum-stdlib/src/tests/structure_ops/collection_stats.rs` — matching
    `ReverseSort` tests;
  - `crates/rum-stdlib/src/tests/session_requests/functional_sequence_resume.rs`
    — repeated comparator-suspension coverage.
- Files you MAY create:
  - `crates/rum-stdlib/src/structure_ops/resumable_merge_sort/mod.rs` — generic
    state/driver;
  - `crates/rum-stdlib/src/structure_ops/resumable_merge_sort/tests.rs` —
    focused pure-state unit tests.
- Files/areas you MUST NOT touch: Association code, SameTest/equivalence code,
  runtime/core, catalogs/reference/TODO, scripts/labels, Cargo files, other
  tests/docs, `.git`, and git config.
- If the existing suspension API proves insufficient, stop and report the
  exact missing capability; do not widen into `rum-runtime`.

#### Deliverable

1. One generic owned merge state with `O(n)` live elements: current/next runs,
   left/right cursors, partial output, ordering expression, return-value policy,
   and completion context.
2. An iterative bottom-up merge driver. Compare right against left; take right
   only when it precedes left; take left on ties; drain suffixes without calls.
3. `Sort` adapter preserving the original expression head and exact-System-
   `True` callback interpretation.
4. `SortBy` adapter preserving input-order exactly-once key evaluation and
   tagged stable ties.
5. `ReverseSort` adapter sorting ascending through the same driver, then
   reversing exactly once.
6. `InputRequired` is the only branch that installs a frame and moves state.
   An `Ok` comparison allocates/clones no full continuation snapshot; another
   error propagates unchanged.
7. Delete the three `n x n` tables and their full-pair drivers. Do not retain a
   fallback that silently selects them by width.
8. Replace exact `n(n-1)` test assertions with result/stability assertions and
   a mechanical `calls <= n * ceil(log2(n))` upper bound.
9. Repeated-suspension tests proving every invariant in plan section 3,
   including identical uninterrupted/resumed pair traces for the new engine.
10. A split assessment with before/after character counts for
    `ordering_sets.rs`, `collection_stats.rs`, and the new module.

#### rum / Rust pitfalls (read twice)

1. The Step-341 contract is a precondition. Do not implement first and infer
   semantic permission afterward.
2. A stable merge compares the right front to the left front and chooses left
   unless right is strictly before. Reversing argument order or choosing right
   on a tie changes results.
3. Do not use recursion per item, pair, run, or pass.
4. Do not clone all runs/state before an `Ok` comparison. Pattern-match the
   `session.evaluate` result and construct the frame only for `InputRequired`.
5. A resumed return value applies the already-started comparison; never invoke
   that callback again.
6. `SortBy` keys are evaluated once and survive suspension. Do not interleave a
   second key pass with merging.
7. Preserve non-List expression heads, invalid-normal messages, empty/singleton
   behavior, and exact callback truth semantics.
8. No unconditional counter/instrumentation and no default-path branch or
   allocation beyond the existing custom-argument match.
9. Safe Rust only. Do not use `unsafe` to move items out of run storage; use a
   safe owned representation.

#### Verification (MANDATORY — run and report verbatim)

Before editing, run two bounded base suites into
`target/codex/brief342-base-1` and `-base-2`. After the change:

```text
cargo fmt --check
cargo test -p rum-stdlib structure_ops
cargo test -p rum-stdlib session_requests_functional_sequence_resume
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
bash scripts/core-language-microbench.sh --output-dir target/codex/brief342-new-1
bash scripts/core-language-microbench.sh --output-dir target/codex/brief342-new-2
git diff --check
git status --short
```

Use `scripts/run-bounded.py` for every long/spawning command, with bounds
derived from the base or a prior same-shape focused run. The timing cap is two
base plus two candidate suite runs; compare best-of-2 per side. Profiling is
off for timings. Paste all base/new rows for the six expression callback
labels, three expression default sentinels, and same-tier controls
`do-iterator-null`, `sameq-integer`, `sameq-list`, and `list-literal-small`.
Report load averages for all four suite runs.

Run one release callback-count probe at a non-power-of-two width and paste the
count/bound for each builtin. Run the suspend-at-every-comparison request test
and paste its test name/result. The target counts must meet the merge bound;
all target timing rows must materially improve and no sentinel/control row may
regress. Apply the template stop-loss after two failures of the same mechanism
or no movement in count/timing.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Modify only Scope files.
- No `unsafe`; no runtime/core/API/representation change.
- Do not touch default ordering algorithms or SameTest paths.
- The ratified callback-schedule contract is the sole permitted observable
  schedule change; preserve all result, error, key, and suspension contracts.
- No eager full-state clone on the nonsuspending comparison path.
- Hot-path A/B and same-tier/default sentinels are mandatory.
- Tests use rum's own documented behavior only; no external engine, oracle,
  compatibility matrix, network, or third-party output.
- Every temporary rum probe stays under `target/codex/brief342/`, uses `.rum`,
  one script/process, stdin from `/dev/null`, and reports load with timings.
- Touch nothing outside the repo.

#### Report format

1. Per-file change summary and before/after module sizes.
2. Merge-state shape, comparison rule, and proof of the call/space bound.
3. Old/new callback counts and base/new target timing rows.
4. Default/same-tier sentinel table and load averages.
5. Stability, edge, wide-stack, and error test list.
6. Suspend-at-every-comparison invariant results and pair-trace comparison.
7. Format/test/reference/diff-check output.
8. Final `git status --short` limited to Scope.
9. Stop-loss/open questions.

---

### Draft brief 343 — migrate Association callback sorts

#### Goal

Move custom `KeySort` and three-argument `KeySortBy` from recursive insertion
and eager continuation clones onto the shared resumable merge engine from
brief 342. Remove the observed native stack overflows, reduce comparisons to
`O(n log n)`, preserve Association/rule-list and delayed-rule forms, preserve
exactly-once input-order key evaluation, and leave both default paths unchanged.

#### Context pointers

- Read `AGENTS.md`, `docs/testing-strategy.md`,
  `docs/code-organization-guidelines.md`,
  `docs/dev/codex/plan-339-user-callback-ordering.md`, and final briefs
  340–342 reports.
- Custom `KeySort` route:
  `crates/rum-stdlib/src/associations/transforms.rs:83-129`.
- Recursive/eager-clone implementations and return policy:
  `crates/rum-stdlib/src/associations/resumable/key_sort.rs:68-190,243-444`.
- Module facades:
  `crates/rum-stdlib/src/associations/mod.rs:12-46` and
  `crates/rum-stdlib/src/associations/resumable.rs:1-13`.
- Existing result tests:
  `crates/rum-stdlib/src/tests/strings/incidental_runtime/association_key_ops.rs:249-331,472-475`.
- Existing request tests:
  `crates/rum-stdlib/src/tests/incidental_session_requests_transforms_resume_associations.rs:91-115`
  and
  `crates/rum-stdlib/src/tests/strings/incidental_runtime/session_requests.rs:172-225`.
- Acceptance is the ratified rum reference, focused tests, former-crash probes,
  and the checked-in Association A/B rows.

#### Scope

- Files you MAY modify:
  - `crates/rum-stdlib/src/structure_ops/resumable_merge_sort/mod.rs` — only
    minimal generic visibility/adaptation required by Association items;
  - `crates/rum-stdlib/src/structure_ops/mod.rs` — only the matching internal
    re-export if needed;
  - `crates/rum-stdlib/src/associations/transforms.rs` — custom `KeySort` route
    only;
  - `crates/rum-stdlib/src/associations/resumable/key_sort.rs` — replace custom
    insertion drivers; preserve default `KeySortBy` decoration/sort;
  - `crates/rum-stdlib/src/associations/resumable.rs` and
    `crates/rum-stdlib/src/associations/mod.rs` — minimal facade cleanup;
  - `crates/rum-stdlib/src/tests/associations/mod.rs` — one test module line;
  - `crates/rum-stdlib/src/tests/incidental_session_requests_transforms_resume_associations.rs`
    — `KeySort` repeated-suspension coverage;
  - `crates/rum-stdlib/src/tests/strings/incidental_runtime/session_requests.rs`
    — `KeySortBy` key/comparator repeated-suspension coverage.
- File you MAY create:
  - `crates/rum-stdlib/src/tests/associations/associations_key_sort.rs` — focused
    result, return-policy, count, and wide-stack tests.
- Files/areas you MUST NOT touch: default sort algorithms, other Association
  transforms, SameTest paths, runtime/core, catalogs/reference/TODO,
  scripts/labels, Cargo files, other docs/tests, `.git`, and git config.
- If the shared engine cannot accept `AssociationEntryForm` without a runtime
  or representation change, stop and report; do not widen Scope.

#### Deliverable

1. `KeySort` adapter compares entry keys and renders through
   `render_association_entry_forms_preserving_heads`.
2. Three-argument `KeySortBy` decorates each entry exactly once, left-to-right,
   suspension-safely, then merges decorated entries by stored key.
3. Both adapters reuse the brief-342 iterative merge state and its
   `InputRequired`-only frame construction. No recursive per-entry/per-pair
   continuation remains.
4. Preserve `key_sort_ordering_decision` behavior for integer `1`, `0`, `-1`,
   System Booleans, and other values; move it only if visibility requires.
5. Preserve input order for ties, Association versus rule-list result head,
   immediate versus delayed rules, and no evaluation of rule values.
6. Leave one-argument `KeySort` and two-argument `KeySortBy` native ordering
   paths source-equivalent except mechanical imports/facades.
7. Delete eager clones of `head`, call args, function, ordering, entries,
   sorted prefix, and candidate on each successful comparison/key evaluation.
8. A non-power-of-two callback count satisfies
   `calls <= n * ceil(log2(n))`; widths 200 and 1,000 complete without abort or
   native-stack growth.
9. Repeated key and comparator suspension proves the seven plan invariants and
   exactly-once key calls.
10. Report before/after `key_sort.rs` size and its remaining responsibility.

#### rum / Rust pitfalls (read twice)

1. Do not change the default arms at `transforms.rs:89-101` and
   `resumable/key_sort.rs:193-239` while removing custom insertion.
2. `AssociationEntryForm` carries key, value, and delayedness. Never rebuild it
   as an unconditional immediate `Rule` or evaluate its value.
3. Key functions receive keys, not values, and run exactly once in input order.
4. The Step-341 contract permits comparator schedule/interleaving changes, but
   not key duplication, reordering, or omitted keys.
5. Preserve the current non-Boolean comparator-result policy. Semantic cleanup
   is a separate director decision.
6. Construct a continuation frame only after `InputRequired`; do not clone the
   complete Association state on `Ok`.
7. No native recursion in key decoration, merging, or completion.
8. Safe Rust only; no `unsafe`, representation, or runtime API changes.

#### Verification (MANDATORY — run and report verbatim)

Before editing, run two bounded base suites into
`target/codex/brief343-base-1` and `-base-2`, plus the two known former-crash
release probes (`KeySort` at 200; `KeySortBy` at 100), each as its own `.rum`
process with stdin `/dev/null`. Record the expected base failures verbatim.
After editing:

```text
cargo fmt --check
cargo test -p rum-stdlib associations_key_sort
cargo test -p rum-stdlib incidental_session_requests_transforms_resume_associations
cargo test -p rum-stdlib session_requests
cargo test -p rum-stdlib builtin_reference_docgen
cargo test --workspace
bash scripts/core-language-microbench.sh --output-dir target/codex/brief343-new-1
bash scripts/core-language-microbench.sh --output-dir target/codex/brief343-new-2
git diff --check
git status --short
```

Use the bounded runner with timeouts derived from prior same-shape runs. The
timing cap is two base plus two candidate suite runs; compare best-of-2 per
side. Profiling is off for timing. Paste all base/new rows for all four
Association callback labels, both Association default sentinels, and same-tier
controls `association-small-literal-repeat`,
`association-set-key-incremental-small`,
`association-set-key-incremental-large`, `sameq-integer`, and `do-iterator-null`.
Report load around suite timing.

Rerun the two former-crash processes and add a width-1,000 result/count probe.
They must complete, and the count must meet the merge bound. Paste key-call
order/count and suspend-at-every-comparison outputs. Target rows must materially
improve; no default/control row may regress. Stop after two failures of the
same mechanism or if callback counts do not move.

#### Hard rules

- Do NOT commit, branch, push, or touch git config.
- Modify only Scope files.
- No `unsafe`; no runtime/core/API/representation change.
- Do not change default `KeySort`/`KeySortBy`, other Association operations, or
  any SameTest path.
- Preserve rule heads, values, delayedness, key-once/input order, return-value
  interpretation, messages, errors, and suspension.
- No eager whole-state cloning and no native recursive driver.
- Hot-path A/B, default sentinels, same-tier controls, and former-crash probes
  are mandatory.
- Testing Policy is a hard line: rum-only documented behavior, no external
  engine/oracle/harness/matrix, no network.
- Each release probe is one `.rum` script/process with stdin `/dev/null`;
  report load with timing and respect product/generated-element budgets.
- Keep probe and gate outputs under `target/codex/brief343/`; touch nothing
  outside the repo.

#### Report format

1. Per-file change summary and `key_sort.rs` before/after size.
2. Adapter/state shape and confirmation that custom recursion/eager clones are
   gone.
3. Former-crash before/after outputs and width-1,000 count/bound.
4. Key exactly-once/order, entry-form preservation, and return-policy tests.
5. Repeated key/comparator suspension invariant results.
6. Base/new target, default, and same-tier timing table with load averages.
7. Format/test/reference/diff-check output.
8. Final `git status --short` limited to Scope.
9. Stop-loss/open questions.
