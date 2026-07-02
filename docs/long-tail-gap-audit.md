# Long-Tail Gap Audit

Date: 2026-07-02.

This is a diagnostic audit for common core-language Wolfram System-context names
that are present in WolframKernel but not implemented, or implemented
divergently, in `rum`. It is intentionally a report only; it does not change
runtime code or the support matrix.

## Method

Raw symbol enumeration:

- WolframKernel oracle:
  `/Applications/Wolfram.app/Contents/MacOS/WolframKernel`
- WK raw symbol dump: `/tmp/wk-system-symbols.raw`
- WK normalized symbol list: `/tmp/wk-system-symbols.txt`
- rum raw symbol dump: `/tmp/rum-system-symbols.raw`
- rum normalized symbol list: `/tmp/rum-system-symbols.txt`
- normalized WK-minus-rum diff: `/tmp/wk-minus-rum-system-symbols.txt`

The WK ``Names["System`*"]`` result was normalized from short names. The rum
``Names["System`*"]`` result was normalized by stripping the printed
`System` context. The raw diff was then filtered to functions and symbols that belong to
the CL-01..CL-09 core-language categories and are plausibly common in ordinary
kernel code.

Each retained candidate was checked with a small exact WK example and the same
rum input. Consolidated example artifacts:

- WK examples: `/tmp/wk-long-tail-gap-examples.tsv`
- rum examples: `/tmp/rum-long-tail-gap-examples.tsv`
- final checked candidate list: `/tmp/long-tail-gap-final-cases.tsv`

Not counted: `LetterCharacter`, `DigitCharacter`, `WhitespaceCharacter`, and
`WordCharacter` appear in WK-minus-rum `Names`, but the basic checked
`StringMatchQ` examples already match WK in rum. Those look like symbol
registration / `Names` omissions, not missing behavior for this audit.

Out-of-scope domains were excluded rather than enumerated member-by-member:

- notebook, front-end, box, dynamic UI, cloud, deployment, and service APIs;
- graphics, visualization, geometry, geographic/entity/knowledge APIs;
- machine learning, neural-net, data-resource, language-data, and semantic
  services;
- file, network, process, import/export, paclet, and system APIs beyond the
  small matrix-listed surface;
- floating-point, machine-number, transcendental, numeric-solver, calculus,
  linear-algebra-heavy, and special-function domains;
- broad date/calendar, unit/quantity, audio/video/image, graph, and statistics
  domains.

Rationals-only exact scalar helpers were kept when they are common
core-language utilities. Machine-real and transcendental numerics such as `N`,
`Sin`, `Cos`, `Exp`, and `Log` were excluded by design.

## Totals

- WK System-context symbols: 7800 unique names.
- rum System-context symbols: 504 unique names.
- Raw WK-minus-rum diff: 7301 names.
- rum-only extension names: 5
  (`MessageDefinitions`, `MessageTags`, `NormalizeRules`, `RumProfileReset`,
  `RumProfileSnapshot`).
- Confirmed in-scope common gaps after filtering and example checks: 96.

Counts by category:

| Category | Count |
|---|---:|
| CL-01 expression, truth, and structural foundation | 26 |
| CL-02 definitions, attributes, assignments, and scoping | 2 |
| CL-03 rules, patterns, and tree walkers | 6 |
| CL-04 functional and list combinators | 39 |
| CL-05 associations and keyed data | 0 |
| CL-06 procedural control, exceptions, and failure flow | 2 |
| CL-07 string language | 16 |
| CL-08 package, namespace, options, and protection tooling | 3 |
| CL-09 tuning, debugging, and verification surface | 2 |

Commonness and ease buckets:

| Commonness | Ease | Count |
|---|---|---:|
| high | trivial | 12 |
| high | moderate | 4 |
| high | involved | 1 |
| medium | trivial | 21 |
| medium | moderate | 48 |
| medium | involved | 7 |
| low | moderate | 3 |

## Gap List

### CL-01 (26)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| Minus | `Minus[3]` -> `-3` | missing: rum returns unevaluated | high | trivial | no |
| Sign | `Sign[-3]` -> `-1` | missing: rum returns unevaluated | high | trivial | no |
| Floor | `Floor[7/3]` -> `2` | missing: rum returns unevaluated | high | trivial | no |
| Ceiling | `Ceiling[7/3]` -> `3` | missing: rum returns unevaluated | high | trivial | no |
| IntegerPart | `IntegerPart[-7/3]` -> `-2` | missing: rum returns unevaluated | medium | trivial | no |
| FractionalPart | `FractionalPart[7/3]` -> `1/3` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerDigits | `IntegerDigits[1230]` -> `{1, 2, 3, 0}` | missing: rum returns unevaluated | medium | moderate | no |
| FromDigits | `FromDigits[{1, 2, 3}]` -> `123` | missing: rum returns unevaluated | medium | moderate | no |
| IntegerLength | `IntegerLength[12345]` -> `5` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerExponent | `IntegerExponent[48, 2]` -> `4` | missing: rum returns unevaluated | medium | moderate | no |
| IntegerReverse | `IntegerReverse[1230]` -> `321` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerString | `IntegerString[10, 2]` -> `"1010"` | missing: rum returns unevaluated | medium | moderate | no |
| DigitCount | `DigitCount[112233, 10, 2]` -> `2` | missing: rum returns unevaluated | medium | moderate | no |
| DigitSum | `DigitSum[12345]` -> `15` | missing: rum returns unevaluated | medium | trivial | no |
| QuotientRemainder | `QuotientRemainder[17, 5]` -> `{3, 2}` | missing: rum returns unevaluated | high | moderate | no |
| NumeratorDenominator | `NumeratorDenominator[7/3]` -> `{7, 3}` | missing: rum returns unevaluated | medium | trivial | no |
| MinMax | `MinMax[{3, 1, 2}]` -> `{1, 3}` | missing: rum returns unevaluated | medium | trivial | no |
| Subdivide | `Subdivide[0, 1, 4]` -> `{0, 1/4, 1/2, 3/4, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| Dimensions | `Dimensions[{{1, 2}, {3, 4}}]` -> `{2, 2}` | missing: rum returns unevaluated | high | moderate | no |
| ArrayDepth | `ArrayDepth[{{1, 2}, {3, 4}}]` -> `2` | missing: rum returns unevaluated | medium | trivial | no |
| LeafCount | `LeafCount[f[a, b + c]]` -> `5` | missing: rum returns unevaluated | medium | trivial | no |
| FlattenAt | `FlattenAt[{{1, 2}, {3, 4}}, 1]` -> `{1, 2, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| SymbolName | `SymbolName[Plus]` -> `"Plus"` | missing: rum returns unevaluated | high | trivial | no |
| Xnor | `Xnor[True, False]` -> `False` | missing: rum returns unevaluated | medium | trivial | no |
| Majority | `Majority[True, False, True]` -> `True` | missing: rum returns unevaluated | medium | trivial | no |
| Variables | `Variables[x + y^2]` -> `{x, y}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-02 (2)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| HoldForm | `HoldForm[1 + 1]` -> `HoldForm[1 + 1]` | divergent: rum evaluates held argument (`HoldForm[2]`) | high | trivial | no (incidental mention only) |
| NValues | `NValues[f]` -> `{}` | missing: rum returns unevaluated | low | moderate | no |

### CL-03 (6)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| FirstPosition | `FirstPosition[{a, b, a}, a]` -> `{1}` | missing: rum returns unevaluated | medium | moderate | no |
| FirstCase | `FirstCase[{x, 2, 3}, _Integer]` -> `2` | missing: rum returns unevaluated | medium | moderate | no |
| SequenceCases | `SequenceCases[{1, 2, 1, 2, 3}, {1, 2}]` -> `{{1, 2}, {1, 2}}` | missing: rum returns unevaluated | medium | involved | no |
| SequencePosition | `SequencePosition[{1, 2, 1, 2, 3}, {1, 2}]` -> `{{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | involved | no |
| SequenceReplace | `SequenceReplace[{1, 2, 1, 2, 3}, {1, 2} -> x]` -> `{x, x, 3}` | missing: rum returns unevaluated | medium | involved | no |
| SequenceCount | `SequenceCount[{1, 2, 1, 2, 3}, {1, 2}]` -> `2` | missing: rum returns unevaluated | medium | involved | no |

### CL-04 (39)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| Mean | `Mean[{1, 2, 4}]` -> `7/3` | missing: rum returns unevaluated | medium | moderate | no |
| ConstantArray | `ConstantArray[x, 3]` -> `{x, x, x}` | missing: rum returns unevaluated | high | moderate | no |
| Intersection | `Intersection[{1, 2, 3}, {2, 3, 4}]` -> `{2, 3}` | missing: rum returns unevaluated | medium | moderate | no |
| Partition | `Partition[{1, 2, 3, 4}, 2]` -> `{{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| Riffle | `Riffle[{a, b, c}, x]` -> `{a, x, b, x, c}` | missing: rum returns unevaluated | medium | moderate | no |
| RotateLeft | `RotateLeft[{1, 2, 3}, 1]` -> `{2, 3, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| RotateRight | `RotateRight[{1, 2, 3}, 1]` -> `{3, 1, 2}` | missing: rum returns unevaluated | medium | trivial | no |
| PadLeft | `PadLeft[{1, 2}, 4]` -> `{0, 0, 1, 2}` | missing: rum returns unevaluated | medium | moderate | no |
| PadRight | `PadRight[{1, 2}, 4]` -> `{1, 2, 0, 0}` | missing: rum returns unevaluated | medium | moderate | no |
| Tally | `Tally[{a, b, a}]` -> `{{a, 2}, {b, 1}}` | missing: rum returns unevaluated | medium | moderate | no |
| Gather | `Gather[{1, 2, 1, 3}]` -> `{{1, 1}, {2}, {3}}` | missing: rum returns unevaluated | medium | moderate | no |
| GatherBy | `GatherBy[{"a", "bb", "c"}, StringLength]` -> `{{"a", "c"}, {"bb"}}` | missing: rum returns unevaluated | medium | moderate | no |
| Split | `Split[{1, 1, 2, 3, 3}]` -> `{{1, 1}, {2}, {3, 3}}` | missing: rum returns unevaluated | medium | moderate | no |
| SplitBy | `SplitBy[{"a", "bb", "c"}, StringLength]` -> `{{"a"}, {"bb"}, {"c"}}` | missing: rum returns unevaluated | medium | moderate | no |
| Subsets | `Subsets[{a, b}]` -> `{{}, {a}, {b}, {a, b}}` | missing: rum returns unevaluated | medium | moderate | no |
| Tuples | `Tuples[{0, 1}, 2]` -> `{{0, 0}, {0, 1}, {1, 0}, {1, 1}}` | missing: rum returns unevaluated | medium | moderate | no |
| Permutations | `Permutations[{a, b, c}, {2}]` -> `{{a, b}, {a, c}, {b, a}, {b, c}, {c, a}, {c, b}}` | missing: rum returns unevaluated | medium | moderate | no |
| Ordering | `Ordering[{3, 1, 2}]` -> `{2, 3, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| OrderingBy | `OrderingBy[{"aa", "b"}, StringLength]` -> `{2, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| ReverseSort | `ReverseSort[{2, 3, 1}]` -> `{3, 2, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| ReverseSortBy | `ReverseSortBy[{"aaa", "b", "cc"}, StringLength]` -> `{"aaa", "cc", "b"}` | missing: rum returns unevaluated | medium | moderate | no |
| MaximalBy | `MaximalBy[{"aa", "b", "cc"}, StringLength]` -> `{"aa", "cc"}` | missing: rum returns unevaluated | medium | moderate | no |
| MinimalBy | `MinimalBy[{"aa", "b", "cc"}, StringLength]` -> `{"b"}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeLargest | `TakeLargest[{5, 1, 4}, 2]` -> `{5, 4}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeSmallest | `TakeSmallest[{5, 1, 4}, 2]` -> `{1, 4}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeLargestBy | `TakeLargestBy[{"aaa", "b", "cc"}, StringLength, 1]` -> `{"aaa"}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeSmallestBy | `TakeSmallestBy[{"aaa", "b", "cc"}, StringLength, 1]` -> `{"b"}` | missing: rum returns unevaluated | medium | moderate | no |
| Commonest | `Commonest[{a, b, a}]` -> `{a}` | missing: rum returns unevaluated | medium | moderate | no |
| DeleteAdjacentDuplicates | `DeleteAdjacentDuplicates[{1, 1, 2, 2, 1}]` -> `{1, 2, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| ArrayFlatten | `ArrayFlatten[{{{{1}}, {{2}}}, {{{3}}, {{4}}}}]` -> `{{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| ArrayPad | `ArrayPad[{1, 2}, 1]` -> `{0, 1, 2, 0}` | missing: rum returns unevaluated | medium | involved | yes (planned ArrayPad-style) |
| ArrayReshape | `ArrayReshape[{1, 2, 3, 4}, {2, 2}]` -> `{{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| Pick | `Pick[{a, b, c}, {1, 0, 1}, 1]` -> `{a, c}` | missing: rum returns unevaluated | medium | moderate | no |
| Differences | `Differences[{1, 4, 9}]` -> `{3, 5}` | missing: rum returns unevaluated | high | trivial | no |
| Median | `Median[{3, 1, 2}]` -> `2` | missing: rum returns unevaluated | medium | moderate | no |
| AllSameBy | `AllSameBy[{"a", "b"}, StringLength]` -> `True` | missing: rum returns unevaluated | medium | moderate | no |
| Curry | `Curry[f, 2][a][b]` -> `f[a, b]` | missing: rum returns unevaluated | medium | moderate | no (CurryApplied only) |
| FoldPairList | `FoldPairList[{#1 + #2, #2} &, 0, {1, 2}]` -> `{1, 3}` | missing: rum returns unevaluated | low | moderate | no |
| FoldWhileList | `FoldWhileList[#1 + #2 &, 0, {1, 2, 3}, LessThan[4]]` -> `{0, 1, 3, 6}` | missing: rum returns unevaluated | low | moderate | no |

### CL-05 (0)

No confirmed missing common functions in this audit.

### CL-06 (2)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| Sum | `Sum[i, {i, 3}]` -> `6` | missing: rum returns unevaluated | high | moderate | yes (CL-X excluded; brief promotes) |
| Product | `Product[i, {i, 3}]` -> `6` | missing: rum returns unevaluated | medium | moderate | no |

### CL-07 (16)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| StringReverse | `StringReverse["abc"]` -> `"cba"` | missing: rum returns unevaluated | high | trivial | no |
| ToLowerCase | `ToLowerCase["ABC"]` -> `"abc"` | missing: rum returns unevaluated | high | trivial | no |
| StringInsert | `StringInsert["abcd", "X", 3]` -> `"abXcd"` | missing: rum returns unevaluated | medium | moderate | no |
| StringReplacePart | `StringReplacePart["abcd", "X", {2, 3}]` -> `"aXd"` | missing: rum returns unevaluated | medium | moderate | no |
| StringReplaceList | `StringReplaceList["abc", "a" -> "x"]` -> `{"xbc"}` | missing: rum returns unevaluated | medium | involved | no |
| StringPartition | `StringPartition["abcdef", 2]` -> `{"ab", "cd", "ef"}` | missing: rum returns unevaluated | medium | moderate | no |
| StringPadRight | `StringPadRight["ab", 4, "x"]` -> `"abxx"` | missing: rum returns unevaluated | high | trivial | no |
| StringRotateLeft | `StringRotateLeft["abcd", 1]` -> `"bcda"` | missing: rum returns unevaluated | medium | trivial | no |
| StringRotateRight | `StringRotateRight["abcd", 1]` -> `"dabc"` | missing: rum returns unevaluated | medium | trivial | no |
| FromCharacterCode | `FromCharacterCode[{65, 66}]` -> `"AB"` | missing: rum returns unevaluated | medium | moderate | no |
| LetterNumber | `LetterNumber["b"]` -> `2` | missing: rum returns unevaluated | medium | moderate | no |
| LowerCaseQ | `LowerCaseQ["abc"]` -> `True` | missing: rum returns unevaluated | medium | trivial | no |
| UpperCaseQ | `UpperCaseQ["ABC"]` -> `True` | missing: rum returns unevaluated | medium | trivial | no |
| Capitalize | `Capitalize["abc"]` -> `"Abc"` | missing: rum returns unevaluated | medium | trivial | no |
| Decapitalize | `Decapitalize["ABC"]` -> `"aBC"` | missing: rum returns unevaluated | medium | trivial | no |
| FromLetterNumber | `FromLetterNumber[2]` -> `"b"` | missing: rum returns unevaluated | medium | moderate | no |

### CL-08 (3)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| NameQ | ``NameQ["System`Plus"]`` -> `True` | missing: rum returns unevaluated | high | trivial | no |
| OptionQ | `OptionQ[Heads -> True]` -> `True` | missing: rum returns unevaluated | high | trivial | no |
| FilterRules | `FilterRules[{a -> 1, b -> 2}, {b}]` -> `{b -> 2}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-09 (2)

| Function | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|
| VerificationTest | `Head[VerificationTest[1 + 1, 2]]` -> `TestObject` | missing: rum returns unevaluated | high | involved | no (TestCreate/TestReport only) |
| TraceScan | `TraceScan[Function[Null], 1 + 1]` -> `2` | missing: rum returns unevaluated | medium | involved | no (Trace/TracePrint only) |

## Recommended Implementation Batches

1. Scalar exact math and structural introspection:
   `Minus`, `Sign`, `Floor`, `Ceiling`, `IntegerPart`, `FractionalPart`,
   `QuotientRemainder`, `NumeratorDenominator`, integer digit helpers,
   `MinMax`, `Subdivide`, `Dimensions`, `ArrayDepth`, `LeafCount`,
   `FlattenAt`, `SymbolName`, `Xnor`, `Majority`, and `Variables`.

2. Small string helpers:
   `StringReverse`, `ToLowerCase`, `LowerCaseQ`, `UpperCaseQ`, `Capitalize`,
   `Decapitalize`, `StringPadRight`, `StringRotateLeft`,
   `StringRotateRight`, `StringInsert`, `StringReplacePart`,
   `StringPartition`, `FromCharacterCode`, `LetterNumber`, and
   `FromLetterNumber`.

3. List shape plumbing:
   `ConstantArray`, `Differences`, `Pick`, `Intersection`, `Partition`,
   `Riffle`, rotations, padding, `Tally`, `Gather`, `GatherBy`, `Split`,
   `SplitBy`, `Subsets`, `Tuples`, `Permutations`, `ArrayFlatten`,
   `ArrayReshape`, and then `ArrayPad`.

4. Ordering, ranking, and small aggregation:
   `Mean`, `Median`, `Ordering`, `OrderingBy`, `ReverseSort`,
   `ReverseSortBy`, `MaximalBy`, `MinimalBy`, `TakeLargest`,
   `TakeSmallest`, `TakeLargestBy`, `TakeSmallestBy`, `Commonest`, and
   `AllSameBy`.

5. Definitions, holding, options, and package helpers:
   fix `HoldForm` by installing the right holding behavior, then add
   `NValues`, `NameQ`, `OptionQ`, and `FilterRules`.

6. Functional and iterator convenience:
   `Curry`, `FoldPairList`, `FoldWhileList`, and a finite exact-iterator
   subset for `Sum` / `Product`.

7. Tree and sequence search:
   first `FirstPosition` / `FirstCase` on top of existing `Position` /
   `Cases`, then `SequenceCases`, `SequencePosition`, `SequenceReplace`, and
   `SequenceCount`.

8. Verification and debugging:
   `VerificationTest` should likely be layered over the existing
   `TestCreate` / `TestReport` machinery. `TraceScan` can follow the current
   `Trace` / `TracePrint` traversal model.

## Uncertain Or Needs Decision

These names are not counted in the 96 confirmed gaps because they either need a
scope decision or fall close to an excluded domain:

- Exact but math-breadth helpers: bitwise integer functions (`BitAnd`,
  `BitOr`, `BitXor`, `BitNot`, `BitGet`, `BitSet`, `BitClear`, `BitFlip`,
  `BitShiftLeft`, `BitShiftRight`, `BitLength`), number-theory helpers
  (`Divisors`, `FactorInteger`, `ChineseRemainder`, `ExtendedGCD`, and
  related prime / divisor functions), and exact numeric shape functions such
  as `Clip`, `UnitStep`, and `Unitize`.
- Boolean algebra solvers and enumerators: `BooleanTable`, `BooleanConvert`,
  `TautologyQ`, `SatisfiableQ`, `BooleanMinimize`, and related SAT-style
  functions.
- Broader random surface: `SeedRandom`, `RandomInteger`, `RandomChoice`,
  `RandomSample`, and related random APIs. `BlockRandom` exists, but this
  audit did not infer broad random support from that.
- Attribute constants and definition metadata: `NHoldAll`, `NHoldFirst`,
  `NHoldRest`, `ReadProtected`, `Locked`, `Constant`, `NumericFunction`,
  `Format`, and `FormatValues`. These are core-adjacent, but some imply
  broader formatting, protection, or numeric-function semantics.
- Held / inactive / box-formatting layer: `Defer`, `Activate`, `Inactivate`,
  `Inactive`, `MakeBoxes`, `MakeExpression`, box constructs, and rich
  formatting forms.
- Functional boundary names needing semantic calibration: `Compose`,
  `ComposeList`, `AllMatch`, `AnyMatch`, and `NoneMatch`.
- Sparse-array and linear-algebra arrays: `SparseArray`, `IdentityMatrix`,
  `DiagonalMatrix`, `MatrixRank`, and related matrix helpers. The matrix
  mentions sparse-array parity as future work, but the brief excludes
  linear-algebra-heavy domains.
- Legacy procedural and dialog helpers: `Goto`, `Label`, `DialogInput`,
  `DialogReturn`, and `WaitUntil`.
- System variables and environment state: `$IterationLimit`,
  `$RecursionLimit`, `$Line`, `$HistoryLength`, `$Version`, `$VersionNumber`,
  `$TimedOut`, and similar names. They are common symbols, but this brief is
  about function gaps.
- Unicode, language-data, and text-analysis breadth: `CharacterName`,
  `CharacterNormalize`, `WordCount`, `WordCounts`, `TextCases`, and related
  linguistic-data helpers.

## Matrix Reconciliation

Only a few confirmed gaps are already visible in the support matrix:

- `ArrayPad` is covered indirectly by the CL-04 planned ArrayPad-style note.
- `Sum` is explicitly listed in CL-X as outside the current non-math core
  target, but this brief promoted it for the iterator audit.
- `Curry` is not listed directly; only `CurryApplied` is listed.
- `VerificationTest` is not listed directly; only `TestCreate` and
  `TestReport` are listed.
- `TraceScan` is not listed directly; only `Trace` and `TracePrint` are
  listed.
- `HoldForm` is only incidentally mentioned in failure-message formatting, not
  tracked as a same-name holding function.

Most confirmed gaps are therefore matrix omissions rather than already-tracked
planned rows. If the follow-up implementation line accepts these batches, the
matrix should be updated alongside the code.
