# Predicates + Associations Usability Audit Findings

Coverage statement: findings-table rows cover 83 distinct catalog names: 50 from
`predicates.rs` and 33 from `associations.rs`. The coverage check printed
`COVERAGE-OK 83/83`. One extra spillover row records the non-catalog
`MatchQ` operator-form gap named by the brief.

## Top 10 Gaps By Script-User Value

1. Collection predicates lack `SameTest`, so approximate or transformed membership checks require manual normalization.
2. Containment predicates, `Between`, and `AllTrue`/`AnyTrue`/`NoneTrue` lack common unary operator forms for pipelines.
3. `VectorQ`, `MatrixQ`, and `ArrayQ` lack element-test forms; `ArrayQ` also lacks shape-pattern forms.
4. `GroupBy` groups association input by values, and there is no `KeyGroupBy` equivalent for grouping by keys.
5. There is no recursive merge helper for nested associations such as layered configuration data.
6. There is no key-path flatten or deflatten helper for flat list-keyed associations and nested associations.
7. `Lookup[key, default][assoc]` is missing even though direct `Lookup[assoc, key, default]` works.
8. `KeySortBy` lacks a custom ordering argument even though `KeySort` accepts one.
9. `OrderedQ` and `LexicographicOrder` lack custom ordering/comparator forms.
10. The non-catalog `MatchQ[patt]` operator form is still missing, while related relation operators such as `EqualTo` and `GreaterThan` already work.

## Findings Table

### Predicates

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| TrueQ | WL | OK | | | | |
| BooleanQ | WL | OK | | | | |
| Boole | WL | OK | | | | |
| AtomQ | WL | OK | | | | |
| ListQ | WL | OK | | | | |
| StringQ | WL | OK | | | | |
| IntegerQ | WL | OK | | | | |
| NumberQ | WL | OK | | | | |
| NumericQ | WL | OK | | | | |
| EvenQ | WL | OK | | | | |
| OddQ | WL | OK | | | | |
| Positive | WL | OK | | | | |
| Negative | WL | OK | | | | |
| NonPositive | WL | OK | | | | |
| NonNegative | WL | OK | | | | |
| PossibleZeroQ | WL | OK | | | | |
| MachineNumberQ | WL | OK | | | | |
| RealValuedNumberQ | WL | OK | | | | |
| RealValuedNumericQ | WL | OK | | | | |
| PrimeQ | WL | OK | | | | |
| AlgebraicIntegerQ | WL | OK | | | | |
| PolynomialQ | WL | OK | | | | |
| PolynomialExpressionQ | WL | OK | | | | |
| RationalExpressionQ | WL | OK | | | | |
| DateObjectQ | WL | OK | | | | |
| TimeObjectQ | WL | OK | | | | |
| DuplicateFreeQ | WL | gap | flexibility | `SameTest` is unsupported and `Options[DuplicateFreeQ]` is empty; the implementation accepts exactly one argument (Evidence E3; `crates/rum-stdlib/src/predicates/collections.rs:3`). | Add `SameTest -> SameQ` option support, keeping structural equality as the default. | S |
| IntersectingQ | WL | gap | flexibility | `SameTest` is unsupported and `Options[IntersectingQ]` is empty; the implementation accepts exactly two list arguments (Evidence E3; `crates/rum-stdlib/src/predicates/collections.rs:26`). | Add `SameTest -> SameQ` for custom intersection tests. | S |
| DisjointQ | WL | gap | flexibility | `SameTest` is unsupported and `Options[DisjointQ]` is empty; the implementation accepts exactly two list arguments (Evidence E3; `crates/rum-stdlib/src/predicates/collections.rs:45`). | Add `SameTest -> SameQ`; default exact disjointness unchanged. | S |
| SubsetQ | WL | gap | flexibility | `SameTest` is unsupported and `Options[SubsetQ]` is empty; the implementation accepts exactly two list arguments (Evidence E3; `crates/rum-stdlib/src/predicates/collections.rs:64`). | Add `SameTest -> SameQ` for custom subset membership. | S |
| Order | WL | OK | | | | |
| OrderedQ | WL | gap | flexibility | The custom ordering predicate form is rejected by exact arity checking, and the unary attempted operator form messages on an atomic criterion (Evidence E5; `crates/rum-stdlib/src/predicates/ordering.rs:15`). | Add the optional ordering predicate form; consider a compatible operator form only if kept with the documented surface. | S |
| LexicographicOrder | WL | gap | flexibility | A three-argument comparator form stays inert with no diagnostic even though the evaluator special-cases non-two arities (Evidence E5; `crates/rum-stdlib/src/predicates/ordering.rs:44`). | Implement the comparator form or fail closed with a targeted unsupported-form message. | S |
| NumericalOrder | WL | OK | | | | |
| AlphabeticOrder | WL | OK | | | | |
| ContainsAll | WL | gap | flexibility | The unary operator form is rejected, and `SameTest` is unsupported with empty options (Evidence E2, E3; `crates/rum-stdlib/src/predicates/collections.rs:85`). | Add `ContainsAll[needles]` as an operator and `SameTest -> SameQ`. | S |
| ContainsAny | WL | gap | flexibility | The unary operator form is rejected, and `SameTest` is unsupported with empty options (Evidence E2, E3; `crates/rum-stdlib/src/predicates/collections.rs:98`). | Add `ContainsAny[needles]` as an operator and `SameTest -> SameQ`. | S |
| ContainsExactly | WL | gap | flexibility | The unary operator form is rejected, and `SameTest` is unsupported with empty options (Evidence E2, E3; `crates/rum-stdlib/src/predicates/collections.rs:111`). | Add `ContainsExactly[needles]` as an operator and `SameTest -> SameQ`. | S |
| ContainsNone | WL | gap | flexibility | The unary operator form is rejected, and `SameTest` discovery is absent with empty options (Evidence E2, E3; `crates/rum-stdlib/src/predicates/collections.rs:121`). | Add `ContainsNone[needles]` as an operator and `SameTest -> SameQ`. | S |
| ContainsOnly | WL | gap | flexibility | The unary operator form is rejected, and `SameTest` discovery is absent with empty options (Evidence E2, E3; `crates/rum-stdlib/src/predicates/collections.rs:134`). | Add `ContainsOnly[allowed]` as an operator and `SameTest -> SameQ`. | S |
| Between | WL | gap | flexibility | `Between[range][value]` emits an arity message instead of forming the interval predicate operator (Evidence E2; `crates/rum-stdlib/src/predicates/ordering.rs:103`). | Add the unary operator form while keeping `Between[value, range]` unchanged. | S |
| VectorQ | WL | gap | flexibility | The element-test form is rejected by exact arity checking (Evidence E4; `crates/rum-stdlib/src/predicates/arrays.rs:3`). | Add `VectorQ[expr, test]` with the current shape test as the default. | S |
| MatrixQ | WL | gap | flexibility | The element-test form is rejected by exact arity checking (Evidence E4; `crates/rum-stdlib/src/predicates/arrays.rs:17`). | Add `MatrixQ[expr, test]` with current rectangularity semantics unchanged. | S |
| ArrayQ | WL | gap | flexibility | Only an integer rank is accepted; shape-list and element-test forms stay inert or arity-message (Evidence E4; `crates/rum-stdlib/src/predicates/arrays.rs:31`). | Add documented shape-pattern and element-test forms, preserving the current one- and two-argument rank behavior. | M |
| AllTrue | WL | gap | flexibility | `AllTrue[crit][expr]` emits an arity message instead of forming the predicate operator (Evidence E2; `crates/rum-stdlib/src/predicates/truth.rs:77`). | Add the unary operator form; keep direct level-spec behavior unchanged. | S |
| AnyTrue | WL | gap | flexibility | `AnyTrue[crit][expr]` emits an arity message instead of forming the predicate operator (Evidence E2; `crates/rum-stdlib/src/predicates/truth.rs:77`). | Add the unary operator form; keep direct level-spec behavior unchanged. | S |
| NoneTrue | WL | gap | flexibility | `NoneTrue[crit][expr]` emits an arity message instead of forming the predicate operator (Evidence E2; `crates/rum-stdlib/src/predicates/truth.rs:77`). | Add the unary operator form; keep direct level-spec behavior unchanged. | S |
| CoprimeQ | WL | OK | | | | |
| SameQ | WL | OK | | | | |
| UnsameQ | WL | OK | | | | |

### Predicate Spillover

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| MatchQ (kernel) | WL | gap | flexibility | The brief-named `MatchQ[patt]` operator form is missing; the unary call emits an arity message. `EqualTo[1][1]` and `GreaterThan[3][4]` already work, so the gap is not the general operator mechanism (Evidence E1). | Add `MatchQ[patt]` in the unit that owns `MatchQ`, using the existing operator-form machinery. | S |

### Associations

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Association | WL | gap | convenience | The WFR-requested flat key-path helpers are absent: `AssociationKeyDeflatten[...]` and its likely inverse `AssociationKeyFlatten[...]` remain inert (Evidence E10; catalog has no such builtins at `crates/rum-stdlib/src/catalog/associations.rs:5`). | Add `AssociationKeyDeflatten` for list-keyed flat associations and consider the sibling `AssociationKeyFlatten`. | M |
| AssociationQ | WL | OK | | | | |
| Lookup | WL | gap | flexibility | Direct default and wrapper forms work, but `Lookup[key, default][assoc]` is parsed as an invalid two-argument direct lookup and stays inert after messages (Evidence E6; `crates/rum-stdlib/src/associations/basic.rs:81`). | Add the two-argument default operator form alongside existing `Lookup[key]`. | S |
| Key | WL | OK | | | | |
| Keys | WL | OK | | | | |
| Values | WL | OK | | | | |
| KeyExistsQ | WL | OK | | | | |
| KeyTake | WL | OK | | | | |
| KeyDrop | WL | OK | | | | |
| KeyMap | WL | OK | | | | |
| KeyValueMap | WL | OK | | | | |
| KeySelect | WL | OK | | | | |
| KeyMemberQ | WL | OK | | | | |
| KeyFreeQ | WL | OK | | | | |
| KeySort | WL | OK | | | | |
| KeySortBy | WL | gap | flexibility | `KeySortBy[assoc, f, p]` is rejected even though `KeySort[assoc, p]` accepts a custom ordering predicate (Evidence E7; `crates/rum-stdlib/src/associations/resumable/key_sort.rs:9`). | Add a third ordering argument for transformed keys, matching the `KeySort` comparison path. | S |
| AssociationMap | WL | OK | | | | |
| AssociationThread | WL | OK | | | | |
| KeyUnion | WL | OK | | | | |
| KeyIntersection | WL | OK | | | | |
| KeyComplement | WL | OK | | | | |
| Normal | WL | OK | | | | |
| AssociateTo | WL | OK | | | | |
| KeyDropFrom | WL | OK | | | | |
| KeyValuePattern | WL | OK | | | | |
| Merge | WL | gap | convenience | `Merge` handles flat keyed collections and has an operator form, but the WFR-requested recursive nested merge helper is absent; `MergeNested[...]` remains inert (Evidence E8, E10). | Add `MergeNested[{assoc...}, f]` for recursive config-style association merges. | M |
| Catenate | WL | OK | | | | |
| Counts | WL | OK | | | | |
| CountsBy | WL | OK | | | | |
| GroupBy | WL | gap | convenience | Association input is grouped by values, not keys, and the WFR-requested `KeyGroupBy[assoc, f]` remains inert (Evidence E10; association grouping applies the key function to `entry.value` at `crates/rum-stdlib/src/associations/grouping/association.rs:20`). | Add `KeyGroupBy[assoc, f]`, with an optional reduction form if it follows `GroupBy`. | M |
| PositionIndex | WL | OK | | | | |
| Missing | WL | OK | | | | |
| DeleteMissing | WL | OK | | | | |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin
to `cargo run`.

### E1: Predicate Operator Survey Spillover

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{EvenQ[2][3], Positive[0][1], PrimeQ[5][7], Between[{1, 3}][2], DuplicateFreeQ[{1, 1}][x], ContainsAny[{a, b}][{c, a}], AllTrue[EvenQ][{2, 4}], MatchQ[_Integer][1], EqualTo[1][1], GreaterThan[3][4]}'
! Between::argx: expected exactly 2 argument(s), got 1
! ContainsAny::argx: expected exactly 2 argument(s), got 1
! AllTrue::argtu: expected between 2 and 3 argument(s), got 1
! MatchQ::argx: expected exactly 2 argument(s), got 1
! Between::argx: expected exactly 2 argument(s), got 1
! ContainsAny::argx: expected exactly 2 argument(s), got 1
! AllTrue::argtu: expected between 2 and 3 argument(s), got 1
! MatchQ::argx: expected exactly 2 argument(s), got 1
List[True[3], False[1], True[7], Between[List[1, 3]][2], False[x], ContainsAny[List[a, b]][List[c, a]], AllTrue[EvenQ][List[2, 4]], MatchQ[Blank[Integer]][1], True, True]
```

### E2: Predicate Operator Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{ContainsAll[{a}][{a, b}], ContainsAny[{a}][{b, a}], ContainsExactly[{a, b}][{b, a}], ContainsNone[{z}][{a, b}], ContainsOnly[{a, b}][{a}], Between[{1, 3}][2], AllTrue[EvenQ][{2, 4}], AnyTrue[OddQ][{2, 3}], NoneTrue[OddQ][{2, 4}]}'
! ContainsAll::argx: expected exactly 2 argument(s), got 1
! ContainsAny::argx: expected exactly 2 argument(s), got 1
! ContainsExactly::argx: expected exactly 2 argument(s), got 1
! ContainsNone::argx: expected exactly 2 argument(s), got 1
! ContainsOnly::argx: expected exactly 2 argument(s), got 1
! Between::argx: expected exactly 2 argument(s), got 1
! AllTrue::argtu: expected between 2 and 3 argument(s), got 1
! AnyTrue::argtu: expected between 2 and 3 argument(s), got 1
! NoneTrue::argtu: expected between 2 and 3 argument(s), got 1
List[ContainsAll[List[a]][List[a, b]], ContainsAny[List[a]][List[b, a]], ContainsExactly[List[a, b]][List[b, a]], ContainsNone[List[z]][List[a, b]], ContainsOnly[List[a, b]][List[a]], Between[List[1, 3]][2], AllTrue[EvenQ][List[2, 4]], AnyTrue[OddQ][List[2, 3]], NoneTrue[OddQ][List[2, 4]]]
```

### E3: Predicate `SameTest` And Options

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{DuplicateFreeQ[{1, -1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], IntersectingQ[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], DisjointQ[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], SubsetQ[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], ContainsAll[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], ContainsExactly[{1}, {-1}, SameTest -> (SameQ[Abs[#1], Abs[#2]] &)], Options /@ {DuplicateFreeQ, IntersectingQ, DisjointQ, SubsetQ, ContainsAll, ContainsAny, ContainsExactly, ContainsNone, ContainsOnly}}'
! DuplicateFreeQ::argx: expected exactly 1 argument(s), got 2
! IntersectingQ::argx: expected exactly 2 argument(s), got 3
! DisjointQ::argx: expected exactly 2 argument(s), got 3
! SubsetQ::argx: expected exactly 2 argument(s), got 3
! ContainsAll::argx: expected exactly 2 argument(s), got 3
! ContainsExactly::argx: expected exactly 2 argument(s), got 3
! DuplicateFreeQ::argx: expected exactly 1 argument(s), got 2
! IntersectingQ::argx: expected exactly 2 argument(s), got 3
! DisjointQ::argx: expected exactly 2 argument(s), got 3
! SubsetQ::argx: expected exactly 2 argument(s), got 3
! ContainsAll::argx: expected exactly 2 argument(s), got 3
! ContainsExactly::argx: expected exactly 2 argument(s), got 3
List[DuplicateFreeQ[List[1, -1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], IntersectingQ[List[1], List[-1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], DisjointQ[List[1], List[-1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], SubsetQ[List[1], List[-1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], ContainsAll[List[1], List[-1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], ContainsExactly[List[1], List[-1], Rule[SameTest, Function[SameQ[Abs[Slot[1]], Abs[Slot[2]]]]]], List[List[], List[], List[], List[], List[], List[], List[], List[], List[]]]
```

### E4: Array Predicate Test Forms

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{VectorQ[{1, 2}, IntegerQ], VectorQ[{1, a}, IntegerQ], MatrixQ[{{1, 2}, {3, 4}}, IntegerQ], MatrixQ[{{1, 2}, {3, a}}, IntegerQ], ArrayQ[{{1, 2}, {3, 4}}, {2, 2}], ArrayQ[{{1, 2}, {3, a}}, 2, IntegerQ]}'
! VectorQ::argx: expected exactly 1 argument(s), got 2
! VectorQ::argx: expected exactly 1 argument(s), got 2
! MatrixQ::argx: expected exactly 1 argument(s), got 2
! MatrixQ::argx: expected exactly 1 argument(s), got 2
! ArrayQ::argtu: expected between 1 and 2 argument(s), got 3
List[VectorQ[List[1, 2], IntegerQ], VectorQ[List[1, a], IntegerQ], MatrixQ[List[List[1, 2], List[3, 4]], IntegerQ], MatrixQ[List[List[1, 2], List[3, a]], IntegerQ], ArrayQ[List[List[1, 2], List[3, 4]], List[2, 2]], ArrayQ[List[List[1, 2], List[3, a]], 2, IntegerQ]]
```

### E5: Predicate Ordering Controls

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{OrderedQ[{3, 2, 1}, Greater], OrderedQ[Greater][{3, 2, 1}], LexicographicOrder[{a}, {b}, Greater], AlphabeticOrder["b", "a", IgnoreCase -> True]}'
! OrderedQ::argx: expected exactly 1 argument(s), got 2
! OrderedQ::normal: Nonatomic expression expected at position 1 in OrderedQ[Greater].
List[OrderedQ[List[3, 2, 1], Greater], OrderedQ[Greater][List[3, 2, 1]], LexicographicOrder[List[a], List[b], Greater], AlphabeticOrder["b", "a", Rule[IgnoreCase, True]]]
```

### E6: Lookup Defaults And Option Discovery

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Lookup[<|"a" -> 1|>, "b"], Lookup[<|"a" -> 1|>, "b", 0], Lookup[<|"a" -> 1|>, {"a", "b"}, 0], Lookup["b", 0][<|"a" -> 1|>], Lookup[<|"a" -> 1|>, "a", 0, f], Lookup[<|"a" -> 1|>, "b", 0, f], Options /@ {Lookup, KeySort, KeyUnion, DeleteMissing}}'
! Lookup::invrl: The argument "b" is not a valid Association or a list of rules.
! Lookup::invrl: The argument "b" is not a valid Association or a list of rules.
List[Missing["KeyAbsent", "b"], 0, List[1, 0], Lookup["b", 0][Association[Rule["a", 1]]], f[1], 0, List[List[], List[], List[], List[]]]
```

### E7: Keyed Operators And `KeySortBy`

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{KeyTake[{"a"}][<|"a" -> 1, "b" -> 2|>], KeyDrop[{"a"}][<|"a" -> 1, "b" -> 2|>], KeyMap[f][<|"a" -> 1|>], KeyValueMap[f][<|"a" -> 1|>], KeySelect[StringQ][<|"a" -> 1, 2 -> 3|>], KeySortBy[StringLength][<|"bb" -> 1, "a" -> 2|>], KeyExistsQ["a"][<|"a" -> 1|>], KeyMemberQ["a"][<|"a" -> 1|>], KeyFreeQ["z"][<|"a" -> 1|>]}'
List[Association[Rule["a", 1]], Association[Rule["b", 2]], Association[Rule[f["a"], 1]], List[f["a", 1]], Association[Rule["a", 1]], Association[Rule["a", 2], Rule["bb", 1]], True, True, True]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{KeySort[<|"b" -> 1, "a" -> 2|>, Greater], KeySortBy[<|"bb" -> 1, "a" -> 2|>, StringLength, Greater], AssociationMap[f][{a, b}], AssociationMap[f][a], AssociationThread[{"a", "b"} -> {1}], KeyUnion[{<|"a" -> 1|>, <|"b" -> 2|>}], KeyUnion[{<|"a" -> 1|>, <|"b" -> 2|>}, Missing], KeyIntersection[{<|"a" -> 1, "b" -> 2|>, <|"b" -> 3|>}], KeyComplement[{<|"a" -> 1, "b" -> 2|>, <|"b" -> 3|>}]}'
! KeySortBy::argt: KeySortBy called with 3 arguments; 1 or 2 arguments are expected.
! AssociationMap::invrp: The argument a is not a valid Association or a list.
! AssociationMap::invrp: The argument a is not a valid Association or a list.
! AssociationThread::idim: List["a", "b"] and List[1] must have the same length.
! KeySortBy::argt: KeySortBy called with 3 arguments; 1 or 2 arguments are expected.
! AssociationMap::invrp: The argument a is not a valid Association or a list.
! AssociationThread::idim: List["a", "b"] and List[1] must have the same length.
List[Association[Rule["a", 2], Rule["b", 1]], KeySortBy[Association[Rule["bb", 1], Rule["a", 2]], StringLength, Greater], Association[Rule[a, f[a]], Rule[b, f[b]]], AssociationMap[f, a], AssociationThread[Rule[List["a", "b"], List[1]]], List[Association[Rule["a", 1], Rule["b", Missing["KeyAbsent", "b"]]], Association[Rule["a", Missing["KeyAbsent", "a"]], Rule["b", 2]]], List[Association[Rule["a", 1], Rule["b", Missing["b"]]], Association[Rule["a", Missing["a"]], Rule["b", 2]]], List[Association[Rule["b", 2]], Association[Rule["b", 3]]], Association[Rule["a", 1]]]
```

### E8: Aggregation Forms Already Present

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Counts[{a, b, a}], Counts[{a, b, a}, {a, c}], CountsBy[{1, 2, 3}, OddQ], CountsBy[OddQ][{1, 2, 3}], GroupBy[{1, 2, 3, 4}, OddQ], GroupBy[{1, 2, 3, 4}, OddQ, Total], GroupBy[OddQ][{1, 2, 3}], GroupBy[{1, 2, 3, 4}, {OddQ, Mod[#, 3] &}], Merge[{<|"a" -> 1|>, <|"a" -> 2|>}, Total], Merge[Total][{<|"a" -> 1|>, <|"a" -> 2|>}]}'
List[Association[Rule[a, 2], Rule[b, 1]], Association[Rule[a, 2], Rule[c, 0]], Association[Rule[True, 2], Rule[False, 1]], Association[Rule[True, 2], Rule[False, 1]], Association[Rule[True, List[1, 3]], Rule[False, List[2, 4]]], Association[Rule[True, 4], Rule[False, 6]], Association[Rule[True, List[1, 3]], Rule[False, List[2]]], Association[Rule[True, Association[Rule[1, List[1]], Rule[0, List[3]]]], Rule[False, Association[Rule[2, List[2]], Rule[1, List[4]]]]], Association[Rule["a", 3]], Association[Rule["a", 3]]]
```

### E9: Missing Data And Association Safety Spot Checks

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{DeleteMissing[{1, Missing["x"], 2}], DeleteMissing[<|"a" -> Missing["x"], "b" -> 2|>], DeleteMissing[{<|"a" -> Missing["x"], "b" -> 2|>}], DeleteMissing[{1, Missing["x"], {Missing["y"], 2}}, 2], DeleteMissing[{1, Missing["x"], {Missing["y"], 2}}, Infinity], MissingQ[Missing["x"]]}'
List[List[1, 2], Association[Rule["b", 2]], List[Association[Rule["a", Missing["x"]], Rule["b", 2]]], List[1, List[2]], List[1, List[2]], True]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Association["a" -> 1, "a" -> 2], Lookup[<|"a" -> 1, "a" -> 2|>, "a"], Keys[<|"a" -> 1, "a" -> 2|>], AssociationQ[Association[Rule["a", 1], a]], Association[Rule["a", 1], a], Association[{"a" -> 1, "b" -> 2}], Association[<|"a" -> 1|>]}'
List[Association[Rule["a", 2]], 2, List["a"], False, Association[Rule["a", 1], a], Association[Rule["a", 1], Rule["b", 2]], Association[Rule["a", 1]]]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{Keys[<|"a" -> 1|>, f], Values[<|"a" -> 1|>, f], Keys[1], Values[1], KeyTake[<|"a" -> 1|>, 1], KeyDrop[<|"a" -> 1|>, 1], KeyExistsQ[1, "a"], KeyMemberQ[1, "a"], KeyFreeQ[1, "a"]}'
! Keys::invrl: The argument 1 is not a valid Association or a list of rules.
! Values::invrl: The argument 1 is not a valid Association or a list of rules.
! KeyExistsQ::invrl: The argument 1 is not a valid Association or a list of rules.
! KeyMemberQ::invas: The argument 1 is not a valid Association or rule.
! KeyFreeQ::invas: The argument 1 is not a valid Association or rule.
! Keys::invrl: The argument 1 is not a valid Association or a list of rules.
! Values::invrl: The argument 1 is not a valid Association or a list of rules.
! KeyMemberQ::invas: The argument 1 is not a valid Association or rule.
! KeyFreeQ::invas: The argument 1 is not a valid Association or rule.
List[List[f["a"]], List[f[1]], Keys[1], Values[1], Association[], Association[Rule["a", 1]], False, KeyMemberQ[1, "a"], KeyFreeQ[1, "a"]]
```

### E10: WFR Association Addition Candidates

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{KeyGroupBy[<|"aa" -> 1, "b" -> 2|>, StringLength], AssociationKeyDeflatten[<|{"a", "b"} -> 1|>], AssociationKeyFlatten[<|"a" -> <|"b" -> 1|>|>], MergeNested[{<|"db" -> <|"host" -> "a"|>|>, <|"db" -> <|"port" -> 5432|>|>} , Last]}'
List[KeyGroupBy[Association[Rule["aa", 1], Rule["b", 2]], StringLength], AssociationKeyDeflatten[Association[Rule[List["a", "b"], 1]]], AssociationKeyFlatten[Association[Rule["a", Association[Rule["b", 1]]]]], MergeNested[List[Association[Rule["db", Association[Rule["host", "a"]]]], Association[Rule["db", Association[Rule["port", 5432]]]]], Last]]
```

```text
$ printf '' | scripts/run-bounded.py --timeout 60 -- cargo run -q -p rum-cli -- --eval '{GroupBy[<|"aa" -> 1, "b" -> 2|>, StringLength], GroupBy[<|"aa" -> 1, "b" -> 22|>, OddQ], GroupBy[<|"aa" -> 1, "b" -> 22|>, Key[StringLength]]}'
! Key::invak: The argument 1 is not a valid Association.
! Key::invak: The argument 22 is not a valid Association.
! Key::invak: The argument 1 is not a valid Association.
! Key::invak: The argument 22 is not a valid Association.
! Key::invak: The argument 1 is not a valid Association.
! Key::invak: The argument 22 is not a valid Association.
! Key::invak: The argument 1 is not a valid Association.
! Key::invak: The argument 22 is not a valid Association.
! Key::invak: The argument 1 is not a valid Association.
! Key::invak: The argument 22 is not a valid Association.
List[Association[Rule[StringLength[1], Association[Rule["aa", 1]]], Rule[StringLength[2], Association[Rule["b", 2]]]], Association[Rule[True, Association[Rule["aa", 1]]], Rule[False, Association[Rule["b", 22]]]], Association[Rule[Key[StringLength][1], Association[Rule["aa", 1]]], Rule[Key[StringLength][22], Association[Rule["b", 22]]]]]
```

### E11: Source Citations

- Predicate collection exact-arity and exact-equality paths are in `crates/rum-stdlib/src/predicates/collections.rs:3`, `:26`, `:45`, `:64`, and `:145`.
- Truth aggregate arity rejects unary operator forms at `crates/rum-stdlib/src/predicates/truth.rs:77`.
- `Between` accepts exactly two arguments at `crates/rum-stdlib/src/predicates/ordering.rs:103`.
- Array predicate arities are fixed at `crates/rum-stdlib/src/predicates/arrays.rs:3`, `:17`, and `:31`.
- `Lookup` only creates an operator form for the one-argument key case at `crates/rum-stdlib/src/associations/basic.rs:95`.
- `KeySortBy` accepts only one or two arguments at `crates/rum-stdlib/src/associations/resumable/key_sort.rs:14`.
- Association grouping applies the grouping function to `entry.value` at `crates/rum-stdlib/src/associations/grouping/association.rs:20`, leaving key grouping as a sibling-addition candidate.
- The association catalog has no `KeyGroupBy`, `AssociationKeyDeflatten`, `AssociationKeyFlatten`, or `MergeNested` entries in `crates/rum-stdlib/src/catalog/associations.rs:5`.

## Triage Sketch

1. **Predicate comparison controls**: add `SameTest -> SameQ` for `DuplicateFreeQ`, `IntersectingQ`, `DisjointQ`, `SubsetQ`, and `Contains*`; add custom ordering for `OrderedQ` and `LexicographicOrder`.
2. **Predicate operator forms**: add unary operators for `ContainsAll`/`Any`/`Exactly`/`None`/`Only`, `Between`, `AllTrue`, `AnyTrue`, and `NoneTrue`; route non-catalog `MatchQ[patt]` to its owning unit.
3. **Array predicate shape/test forms**: add `VectorQ`/`MatrixQ` element tests and `ArrayQ` shape-pattern plus element-test forms.
4. **Association scripting additions**: add `Lookup[key, default]` operator form and `KeySortBy[assoc, f, p]`; then add `KeyGroupBy`, `AssociationKeyDeflatten`/`AssociationKeyFlatten`, and `MergeNested`.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/predicates.rs
50
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/associations.rs
33
```

```text
$ names=$(rg -o '"[A-Za-z][A-Za-z0-9]*"' crates/rum-stdlib/src/catalog/predicates.rs crates/rum-stdlib/src/catalog/associations.rs | sed 's/.*"\([^"]*\)"/\1/' | sort -u); rows=$(awk -F'|' '/^\| [^|]+ \|/ {gsub(/^ +| +$/,"",$2); if ($2 != "Builtin") print $2}' docs/dev/usability-pass/predicates-associations-findings.md | sort -u); covered=$(comm -12 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); missing=$(comm -23 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); count=$(printf '%s\n' "$covered" | sed '/^$/d' | wc -l | tr -d ' '); if [ -z "$missing" ] && [ "$count" = "83" ]; then echo "COVERAGE-OK 83/83"; else echo "COVERAGE-FAIL $count/83"; [ -z "$missing" ] || printf 'missing:\n%s\n' "$missing"; fi
COVERAGE-OK 83/83
```

```text
$ git status --short
?? docs/dev/usability-pass/predicates-associations-findings.md
```

```text
$ cargo fmt --check
```

## Stop-Loss / Open Questions

- None.
