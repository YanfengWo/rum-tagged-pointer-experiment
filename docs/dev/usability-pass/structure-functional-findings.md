# Structure + Functional Usability Audit Findings

Coverage statement: findings-table rows cover 89 distinct catalog names: 47 from `structure.rs` and 42 from `functional.rs`. The coverage check printed `COVERAGE-OK 89/89`.

## Top 10 Gaps By Script-User Value

1. `Map`/`MapIndexed`/`MapAll`/`Scan`/`Apply`/`MapApply`/`Comap`/`ComapApply` accept `Heads`, but `Options[...]` is empty, so option discovery and `SetOptions` do not match the real surface.
2. Common structural operators such as `Delete`, `Insert`, `Take`, `Drop`, `TakeList`, `Partition`, `Riffle`, `GatherBy`, `MaxBy`, and `TakeLargestBy` lack unary operator forms, unlike nearby `Append`/`Prepend`/`SortBy`.
3. `MapAt` silently leaves out-of-range targets unchanged, and `SubsetMap` silently falls back on duplicate, overlapping, or missing exact paths.
4. `Subsets` rejects `All`, `Infinity`, and stepped size ranges even though its own diagnostic advertises them.
5. `Partition` has only fixed block size and positive offset forms; padding, alignment, and operator forms are missing.
6. `PadLeft`/`PadRight` only accept scalar target lengths, so rectangular or per-axis padding needs manual wrappers.
7. Collection equivalence/order controls are thin: `DeleteDuplicates`, `Tally`, set operations, `SortBy`, `ReverseSort`, and `GatherBy` miss common custom-comparison forms.
8. `MaxBy`/`MinBy` return only one element and `TakeLargestBy`/`TakeSmallestBy` lack operator forms, making top-k pipelines verbose or unavailable.
9. `Transpose` only swaps the first two dimensions and `Reverse` rejects `All` as a level spec.
10. `Inner` and `Dot` return inert dimension-mismatch calls with no message, which is hard to distinguish from a deliberately symbolic expression.

## Findings Table

### Structure

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| FullForm | WL | OK | | | | |
| Head | WL | OK | | | | |
| Length | WL | OK | | | | |
| Depth | WL | OK | | | | |
| LeafCount | WL | OK | | | | |
| First | WL | OK | | | | |
| Last | WL | OK | | | | |
| Rest | WL | OK | | | | |
| Most | WL | OK | | | | |
| Take | WL | gap | flexibility | `Take[spec][expr]` emits an arity message instead of forming a pipeline operator (Evidence E1). | Add a compatible unary operator form for the WL-inherited sequence surface; schedule behind the call-planning freeze. | S |
| Drop | WL | gap | flexibility | `Drop[spec][expr]` emits an arity message instead of forming a pipeline operator (Evidence E1). | Add a compatible unary operator form; default direct-call behavior unchanged. | S |
| TakeDrop | WL | OK | | | | |
| Insert | WL | gap | flexibility | `Insert[item, pos][expr]` emits an arity message instead of forming an operator (Evidence E1). | Add `Insert[item, pos]` as an operator form. | S |
| Delete | WL | gap | flexibility | `Delete[pos][expr]` emits an arity message instead of forming an operator (Evidence E1). | Add `Delete[pos]` as an operator form. | S |
| TakeList | WL | gap | flexibility | `TakeList[spec][expr]` emits an arity message instead of forming an operator (Evidence E1). | Add `TakeList[spec]` as an operator form. | S |
| Range | WL | OK | | | | |
| Differences | WL | OK | | | | |
| ConstantArray | WL | OK | | | | |
| PadLeft | WL | gap | flexibility | The implementation parses only a scalar non-negative target length, so rectangular target dimensions are rejected (Evidence E3; `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:97`). | Add list-dimension and margin-aware padding forms while keeping scalar padding unchanged. | M |
| PadRight | WL | gap | flexibility | The implementation parses only a scalar non-negative target length, so rectangular target dimensions are rejected (Evidence E3; `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:97`). | Add list-dimension and margin-aware padding forms while keeping scalar padding unchanged. | M |
| Subsets | WL | gap | safety | `All`, `Infinity`, and stepped size ranges are rejected even though `Subsets::nninfseq` advertises those forms (Evidence E3; `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:211`). | Implement the advertised size-spec forms or narrow the diagnostic; the compatible fix is to accept them. | S |
| Tuples | WL | OK | | | | |
| Partition | WL | gap | flexibility | Only 2- and 3-argument fixed-size partitioning is accepted; operator, padding, and alignment forms are rejected (Evidence E1, E3; `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:61`). | Add operator and padded/aligned partition forms with the current fixed-block behavior unchanged. | M |
| Flatten | WL | OK | | | | |
| Join | WL | OK | | | | |
| Reverse | WL | gap | flexibility | `Reverse[expr, All]` is rejected because level parsing accepts only positive integers or lists of them (Evidence E3; `crates/rum-stdlib/src/structure_ops/reordering.rs:107`). | Accept `All` as a level spec for all structural levels. | S |
| Transpose | WL | gap | flexibility | Only identity and first-two-axis swap permutations are parsed, so higher-rank permutations are rejected (Evidence E3; `crates/rum-stdlib/src/structure_ops/reordering.rs:101`). | Add arbitrary positive permutation lists for rectangular list tensors. | M |
| DeleteDuplicates | WL | gap | flexibility | The custom equality predicate form is unsupported and a two-argument call stays inert (Evidence E4; `crates/rum-stdlib/src/structure_ops/dedup.rs:3`). | Add the documented test argument or a `SameTest` option; default structural equality unchanged. | M |
| DeleteDuplicatesBy | WL | gap | flexibility | `DeleteDuplicatesBy[f][expr]` stays inert instead of forming the unary operator (Evidence E4; `crates/rum-stdlib/src/structure_ops/dedup.rs:30`). | Add the unary operator form. | S |
| Tally | WL | gap | flexibility | The custom equality predicate form is rejected by exact arity checking (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:29`). | Add the optional test argument; default structural tally unchanged. | S |
| GatherBy | WL | gap | safety | A function-list specification is treated as an ordinary callable key, producing singleton groups rather than hierarchical grouping or a clear unsupported-form message (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:139`). | Implement hierarchical function-list grouping or fail closed with a specific message. | M |
| GatherBy | WL | gap | flexibility | `GatherBy[f][expr]` emits an arity message instead of forming the unary operator (Evidence E1). | Add `GatherBy[f]` as an operator form. | S |
| Sort | WL | OK | | | | |
| SortBy | WL | gap | flexibility | The three-argument ordering form stays inert for nonatomic input (Evidence E4; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:45`). | Implement `SortBy[expr, f, p]` or message clearly if deferred. | M |
| ReverseSort | WL | gap | flexibility | A custom ordering predicate is rejected by exact arity checking (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:91`). | Add the optional comparator form; default reverse structural ordering unchanged. | S |
| ReverseSortBy | WL | gap | flexibility | `ReverseSortBy[f][expr]` emits an arity message instead of forming the unary operator (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:109`). | Add the unary operator form. | S |
| MaxBy | WL | gap | flexibility | Count and operator forms are missing: `MaxBy[expr, f, n]` and `MaxBy[f][expr]` are rejected (Evidence E1, E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:196`). | Add top-k and unary operator forms; keep single-result default unchanged. | M |
| MinBy | WL | gap | flexibility | Count and operator forms are missing: `MinBy[expr, f, n]` is rejected, and the same exact two-argument implementation is shared with `MaxBy` (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:196`). | Add top-k and unary operator forms; keep single-result default unchanged. | M |
| TakeLargestBy | WL | gap | flexibility | `TakeLargestBy[f, n][expr]` emits an arity message instead of forming the unary operator (Evidence E1, E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:226`). | Add the unary operator form. | S |
| TakeSmallestBy | WL | gap | flexibility | `TakeSmallestBy[f, n][expr]` emits an arity message instead of forming the unary operator (Evidence E4; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:226`). | Add the unary operator form. | S |
| Riffle | WL | gap | flexibility | `Riffle[x][expr]` emits an arity message instead of forming the unary operator; only one separator between adjacent top-level elements is implemented (Evidence E1; `crates/rum-stdlib/src/structure_ops/collection_stats.rs:165`). | Add the operator form and later the documented positional forms if kept in scope. | S |
| Union | WL | gap | flexibility | `SameTest`-style custom equivalence is not parsed; a rule argument is treated as a same-head input and produces a heads message (Evidence E4; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:82`). | Add `SameTest` option support for set operations. | M |
| Intersection | WL | gap | flexibility | The shared set-operation path accepts only same-head normal expressions, leaving no custom equivalence option (Evidence E4; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:99`). | Add `SameTest` option support for set operations. | M |
| Complement | WL | gap | flexibility | The shared set-operation path accepts only same-head normal expressions, leaving no custom equivalence option (Evidence E4; `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:129`). | Add `SameTest` option support for set operations. | M |
| Append | WL | OK | | | | |
| Prepend | WL | OK | | | | |
| Symbol | WL | OK | | | | |
| Construct | WL | OK | | | | |

### Functional

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| BlockMap | WL | OK | | | | |
| SubsetMap | WL | gap | safety | Duplicate, overlapping, or missing exact paths fall back to an inert call or the original expression without a message (Evidence E5; `crates/rum-stdlib/src/functional_ops/targeted_map.rs:42`). | Emit a targeted message for incompatible or missing selected paths; keep successful replacement behavior unchanged. | S |
| Comap | WL | gap | safety | `Comap` accepts `Heads -> True/False`, but `Options[Comap]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:310`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` in `Options[Comap]` and honor `SetOptions` through the same default path. | S |
| ComapApply | WL | gap | safety | `ComapApply` accepts `Heads -> True/False`, but `Options[ComapApply]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:388`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` in `Options[ComapApply]` and honor `SetOptions`. | S |
| Accumulate | WL | OK | | | | |
| MapIndexed | WL | gap | safety | `MapIndexed` accepts `Heads -> True/False`, but `Options[MapIndexed]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:180`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` and use the registered default when no explicit option is supplied. | S |
| Map | WL | gap | safety | `Map` accepts `Heads -> True/False`, but `Options[Map]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:180`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` and use the registered default when no explicit option is supplied. | S |
| MapAll | WL | gap | safety | `MapAll` accepts a `Heads` option tail, but `Options[MapAll]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:252`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register the accepted option and wire the default through options lookup. | S |
| MapAt | WL | gap | safety | Missing exact targets are skipped silently, so `MapAt[f, {a, b}, 3]` returns `{a, b}` with no message (Evidence E5; `crates/rum-stdlib/src/functional_ops/targeted_map.rs:172`). | Emit a missing-part message when an explicit target path cannot be resolved. | S |
| Scan | WL | gap | safety | `Scan` accepts `Heads -> True/False`, but `Options[Scan]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:180`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` and use the registered default when no explicit option is supplied. | S |
| MapThread | WL | OK | | | | |
| TakeWhile | WL | OK | | | | |
| LengthWhile | WL | OK | | | | |
| Inner | WL | gap | safety | Dimension or head mismatch returns the inert `Inner[...]` call with no diagnostic (Evidence E5; `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:11`). | Emit a clear dimension/head mismatch message while preserving symbolic inertness only for genuinely nonnumeric symbolic cases. | S |
| Dot | WL | gap | safety | Dimension mismatch returns the inert `Dot[...]` call with no diagnostic (Evidence E5; `crates/rum-stdlib/src/functional_ops/arrays/mod.rs:25`). | Emit a clear dimension/head mismatch message. | S |
| Fold | WL | OK | | | | |
| FoldList | WL | OK | | | | |
| FoldPair | WL | OK | | | | |
| FoldWhile | WL | OK | | | | |
| SequenceFold | WL | OK | | | | |
| SequenceFoldList | WL | OK | | | | |
| Nest | WL | OK | | | | |
| NestWhile | WL | OK | | | | |
| NestList | WL | OK | | | | |
| NestWhileList | WL | OK | | | | |
| FixedPoint | WL | OK | | | | |
| FixedPointList | WL | OK | | | | |
| Apply | WL | gap | safety | `Apply` accepts `Heads -> True/False`, but `Options[Apply]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:97`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` and use the registered default when no explicit option is supplied. | S |
| MapApply | WL | gap | safety | `MapApply` accepts `Heads -> True/False`, but `Options[MapApply]` is empty (Evidence E2; `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:97`; `crates/rum-runtime/src/runtime/options/defaults.rs:20`). | Register `Heads -> False` and use the registered default when no explicit option is supplied. | S |
| OperatorApplied | WL | OK | | | | |
| CurryApplied | WL | OK | | | | |
| ReverseApplied | WL | OK | | | | |
| Composition | WL | OK | | | | |
| RightComposition | WL | OK | | | | |
| Through | WL | OK | | | | |
| Operate | WL | OK | | | | |
| Array | WL | OK | | | | |
| ArrayFilter | WL | OK | | | | |
| ArrayReduce | WL | OK | | | | |
| Outer | WL | OK | | | | |
| Select | WL | OK | | | | |
| SelectFirst | WL | OK | | | | |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin to `cargo run`.

### E1: Structural Operator Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Append[x][{a, b}], Prepend[x][{a, b}], Delete[1][{a, b}], Insert[x, 2][{a, b}], Take[2][{a, b, c}], Drop[1][{a, b, c}], TakeList[{2, 1}][{a, b, c, d}], Partition[2][{a, b, c}], Riffle[x][{a, b}], GatherBy[f][{a, b}], MaxBy[f][{a, b}], TakeLargestBy[f, 1][{a, b}]}'
! Delete::argx: expected exactly 2 argument(s), got 1
! Insert::argx: expected exactly 3 argument(s), got 2
! Take::argx: expected at least 2 arguments
! Drop::argx: expected at least 2 arguments
! TakeList::argx: expected exactly 2 argument(s), got 1
! Partition::argtu: expected between 2 and 3 argument(s), got 1
! Riffle::argx: expected exactly 2 argument(s), got 1
! GatherBy::argx: expected exactly 2 argument(s), got 1
! MaxBy::argx: expected exactly 2 argument(s), got 1
! TakeLargestBy::argx: expected exactly 3 argument(s), got 2
! Delete::argx: expected exactly 2 argument(s), got 1
! Insert::argx: expected exactly 3 argument(s), got 2
! Take::argx: expected at least 2 arguments
! Drop::argx: expected at least 2 arguments
! TakeList::argx: expected exactly 2 argument(s), got 1
! Partition::argtu: expected between 2 and 3 argument(s), got 1
! Riffle::argx: expected exactly 2 argument(s), got 1
! GatherBy::argx: expected exactly 2 argument(s), got 1
! MaxBy::argx: expected exactly 2 argument(s), got 1
! TakeLargestBy::argx: expected exactly 3 argument(s), got 2
List[List[a, b, x], List[x, a, b], Delete[1][List[a, b]], Insert[x, 2][List[a, b]], Take[2][List[a, b, c]], Drop[1][List[a, b, c]], TakeList[List[2, 1]][List[a, b, c, d]], Partition[2][List[a, b, c]], Riffle[x][List[a, b]], GatherBy[f][List[a, b]], MaxBy[f][List[a, b]], TakeLargestBy[f, 1][List[a, b]]]
```

### E2: Hidden `Heads` Options

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Options /@ {Depth, ArrayFilter, Map, MapIndexed, MapAll, Scan, Apply, MapApply, Comap, ComapApply}, Map[f, p[q][x], 1, Heads -> True], Apply[f, p[q][x], 1, Heads -> True], Comap[{f, g}, p[q][x], 1, Heads -> True], ComapApply[{f, g}, p[q][x], Heads -> True]}'
List[List[List[Rule[Heads, False]], List[Rule[Padding, "Fixed"]], List[], List[], List[], List[], List[], List[], List[], List[]], f[p[q]][f[x]], f[q][x], List[p[q][x]][f[p[q][x]], g[p[q][x]]], List[x][f[x], g[x]]]
```

### E3: Structural Spec Limits

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Subsets[{a, b, c}, All], Subsets[{a, b, c}, Infinity], Subsets[{a, b, c}, {1, 3, 2}], PadLeft[{{1}, {2}}, {2, 3}], PadRight[{{1}, {2}}, {2, 3}, x], Partition[{1, 2, 3, 4}, 2, 1, {1, 1}, x], Transpose[{{{a, b}, {c, d}}, {{e, f}, {g, h}}}, {3, 2, 1}], Reverse[{a, b}, All]}'
! Subsets::nninfseq: Position 2 of Subsets[List[a, b, c], All] must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.
! Subsets::nninfseq: Position 2 of Subsets[List[a, b, c], Infinity] must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.
! Subsets::nninfseq: Position 2 of Subsets[List[a, b, c], List[1, 3, 2]] must be All, Infinity, nmax, {nmin}, {nmin, nmax} or {nmin, nmax, dn}, where nmin is a non-negative integer, nmax is non-negative integer or Infinity and dn is a nonzero integer.
! PadLeft::ilsm: List of machine-sized integers expected at position 2 in PadLeft[List[List[1], List[2]], List[2, 3]].
! PadRight::ilsm: List of machine-sized integers expected at position 2 in PadRight[List[List[1], List[2]], List[2, 3], x].
! Partition::argtu: expected between 2 and 3 argument(s), got 5
! Transpose::permspec: Invalid permutation specification found at position 2 in Transpose[List[List[List[a, b], List[c, d]], List[List[e, f], List[g, h]]], List[3, 2, 1]].
! Reverse::ilsmp: Single or list of positive machine-sized integers expected at position 2 of Reverse[List[a, b], All].
List[Subsets[List[a, b, c], All], Subsets[List[a, b, c], Infinity], Subsets[List[a, b, c], List[1, 3, 2]], PadLeft[List[List[1], List[2]], List[2, 3]], PadRight[List[List[1], List[2]], List[2, 3], x], Partition[List[1, 2, 3, 4], 2, 1, List[1, 1], x], Transpose[List[List[List[a, b], List[c, d]], List[List[e, f], List[g, h]]], List[3, 2, 1]], Reverse[List[a, b], All]]
```

### E4: Collection Controls

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{DeleteDuplicates[{1, -1, 2}, SameQ[Abs[#1], Abs[#2]] &], DeleteDuplicatesBy[Abs][{-1, 1, 2}], Tally[{1, -1, 2}, SameQ[Abs[#1], Abs[#2]] &], GatherBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}], SortBy[{3, 1, 2}, Identity, Greater], ReverseSort[{3, 1, 2}, Less], ReverseSortBy[Identity][{3, 1, 2}], MaxBy[{1, -3, 2}, Abs, 2], MinBy[{1, -3, 2}, Abs, 2], TakeLargestBy[Abs, 2][{1, -3, 2}], TakeSmallestBy[Abs, 2][{1, -3, 2}], Union[{1, -1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)]}'
! Tally::argx: expected exactly 1 argument(s), got 2
! ReverseSort::argx: expected exactly 1 argument(s), got 2
! ReverseSortBy::argx: expected exactly 2 argument(s), got 1
! MaxBy::argx: expected exactly 2 argument(s), got 3
! MinBy::argx: expected exactly 2 argument(s), got 3
! TakeLargestBy::argx: expected exactly 3 argument(s), got 2
! TakeSmallestBy::argx: expected exactly 3 argument(s), got 2
! Union::heads: Heads Rule and List at positions 2 and 1 are expected to be the same.
! Tally::argx: expected exactly 1 argument(s), got 2
! ReverseSort::argx: expected exactly 1 argument(s), got 2
! ReverseSortBy::argx: expected exactly 2 argument(s), got 1
! MaxBy::argx: expected exactly 2 argument(s), got 3
! MinBy::argx: expected exactly 2 argument(s), got 3
! TakeLargestBy::argx: expected exactly 3 argument(s), got 2
! TakeSmallestBy::argx: expected exactly 3 argument(s), got 2
! Union::heads: Heads Rule and List at positions 2 and 1 are expected to be the same.
List[DeleteDuplicates[List[1, -1, 2], Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]], DeleteDuplicatesBy[Abs][List[-1, 1, 2]], Tally[List[1, -1, 2], Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]], List[List[1], List[2], List[3], List[4]], SortBy[List[3, 1, 2], Identity, Greater], ReverseSort[List[3, 1, 2], Less], ReverseSortBy[Identity][List[3, 1, 2]], MaxBy[List[1, -3, 2], Abs, 2], MinBy[List[1, -3, 2], Abs, 2], TakeLargestBy[Abs, 2][List[1, -3, 2]], TakeSmallestBy[Abs, 2][List[1, -3, 2]], Union[List[1, -1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]]
```

### E5: Silent Target And Tensor Failures

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{MapAt[f, {a, b}, 3], MapAt[f, {a, b}, {All, 3}], SubsetMap[Reverse, {a, b}, {{1}, {1}}], SubsetMap[Reverse, {a, b}, {{1}, {1, 1}}], MapThread[f, {{a, b}, {c}}, 1], Inner[f, {a, b}, {{c, d}}, g], Dot[{a, b}, {c}]}'
! Thread::tdlen: cannot thread `f` over list arguments of lengths 2, 1
! Thread::tdlen: cannot thread `f` over list arguments of lengths 2, 1
! Thread::tdlen: cannot thread `f` over list arguments of lengths 2, 1
List[List[a, b], List[a, b], SubsetMap[Reverse, List[a, b], List[List[1], List[1]]], List[a, b], MapThread[f, List[List[a, b], List[c]]], Inner[f, List[a, b], List[List[c, d]], g], Dot[List[a, b], List[c]]]
```

### E6: Source Citations

- Hidden options: normalization accepts direct `Heads` for `Apply`/`MapApply` at `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:97`, `Map`/`Scan`/`MapIndexed` at `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:180`, `MapAll` at `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:252`, `Comap` at `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:310`, and `ComapApply` at `crates/rum-stdlib/src/functional_ops/apply_map/normalize.rs:388`; default options register only `ArrayFilter`, `Cases`/`DeleteCases`/`Count`/`MemberQ`/`Level`/`Depth`, `Trace`/`TracePrint`, and `FreeQ`/`Position` for these option names at `crates/rum-runtime/src/runtime/options/defaults.rs:20`.
- Structural subset limits: scalar-only padding is parsed at `crates/rum-stdlib/src/structure_ops/list_plumbing.rs:97`; `Partition` accepts only 2 or 3 arguments at `crates/rum-stdlib/src/structure_ops/combinatorial_lists.rs:61`; `Reverse` level parsing accepts only positive integers/lists at `crates/rum-stdlib/src/structure_ops/reordering.rs:107`; `Transpose` only defines identity or swap-first-two permutations at `crates/rum-stdlib/src/structure_ops/reordering.rs:101`.
- Collection controls: `DeleteDuplicatesBy` returns a one-argument inert call at `crates/rum-stdlib/src/structure_ops/dedup.rs:35`; `Tally` exact-arity handling is at `crates/rum-stdlib/src/structure_ops/collection_stats.rs:29`; `GatherBy` exact two-argument handling is at `crates/rum-stdlib/src/structure_ops/collection_stats.rs:139`; `SortBy` leaves three-argument nonatomic calls inert at `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:71`; set operations share same-head grouping at `crates/rum-stdlib/src/structure_ops/ordering_sets.rs:175`.
- Targeted mapping safety: `MapAt` skips missing selected paths at `crates/rum-stdlib/src/functional_ops/targeted_map.rs:172`; `SubsetMap` returns inert or fallback for incompatible/missing paths at `crates/rum-stdlib/src/functional_ops/targeted_map.rs:42` and `crates/rum-stdlib/src/functional_ops/targeted_map.rs:360`.

## Triage Sketch

1. **Hidden option registration and defaults**: register and honor `Heads -> False` for `Map`, `MapIndexed`, `MapAll`, `Scan`, `Apply`, `MapApply`, `Comap`, and `ComapApply`. This is the smallest high-value safety batch.
2. **Operator-form parity for structural pipelines**: add unary forms for `Delete`, `Insert`, `Take`, `Drop`, `TakeList`, `Partition`, `Riffle`, `GatherBy`, `MaxBy`, `MinBy`, `TakeLargestBy`, and `TakeSmallestBy`. This is valuable but touches call planning, so schedule after the current freeze.
3. **Sequence and tensor flexibility**: implement the advertised `Subsets` size specs, multidimensional `PadLeft`/`PadRight`, padded/aligned `Partition`, general `Transpose` permutations, and `Reverse[..., All]`.
4. **Collection comparison controls**: add custom equivalence/order forms for `DeleteDuplicates`, `Tally`, `Union`/`Intersection`/`Complement`, `ReverseSort`, `SortBy`, and hierarchical `GatherBy`.
5. **Fail-closed diagnostics**: message on missing `MapAt` targets, incompatible `SubsetMap` path sets, non-integer `TakeLargestBy`/`TakeSmallestBy` counts, and `Inner`/`Dot` tensor mismatches.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/structure.rs
47
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/functional.rs
42
```

```text
$ names=$(rg -o '"[A-Za-z][A-Za-z0-9]*"' crates/rum-stdlib/src/catalog/structure.rs crates/rum-stdlib/src/catalog/functional.rs | sed 's/.*"\([^"]*\)"/\1/' | sort -u); rows=$(awk -F'|' '/^\| [A-Za-z][A-Za-z0-9]* \|/ {gsub(/ /,"",$2); if ($2 != "Builtin") print $2}' docs/dev/usability-pass/structure-functional-findings.md | sort -u); missing=$(comm -23 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); extra=$(comm -13 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); count=$(printf '%s\n' "$rows" | sed '/^$/d' | wc -l | tr -d ' '); if [ -z "$missing$extra" ] && [ "$count" = "89" ]; then echo "COVERAGE-OK 89/89"; else echo "COVERAGE-FAIL $count/89"; [ -z "$missing" ] || printf 'missing:\n%s\n' "$missing"; [ -z "$extra" ] || printf 'extra:\n%s\n' "$extra"; fi
COVERAGE-OK 89/89
```

```text
$ git status --short
?? docs/dev/usability-pass/structure-functional-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

- `Cases`, `Count`, `Position`, `DeleteCases`, `FirstCase`, `Extract`, association `Key*` functions, and `GroupBy` were mentioned in the spillover prompt but are not entries in the two audited catalogs, so they are intentionally excluded from the 89-name coverage count.
