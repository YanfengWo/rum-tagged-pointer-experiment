# Long-Tail Gap Audit

Date: 2026-07-02.

This is the authoritative diagnostic reconciliation for common core-language Wolfram System-context names that are present in WolframKernel but not implemented, or implemented divergently, in `rum`. It reconciles the brief-082 96-row manual pass with the saved WolframKernel `FunctionalityAreas` grouping data. It is a report only; it does not change runtime code or the support matrix.

## Method

Inputs used:

- WK grouping artifact: `WLBuiltin/wolfram-language-groups.json` (used as-is; not regenerated).
- rum symbols: ``target/debug/rum-cli --eval 'Names["System`*"]'`` returned 504 unique names for this run.
- brief-082 baseline: the on-disk 96-row `docs/long-tail-gap-audit.md` input was parsed and merged, not discarded.
- WK examples: resolved by the documented local-oracle policy (`RUM_DIFFTEST_ORACLE_KERNEL`, then PATH/common local kernel paths) and saved raw to `/tmp/wk-long-tail-gap-083-examples.tsv` before parsing.
- rum check for grouping-added rows: `/tmp/rum-long-tail-gap-083-grouping-added.raw`.

The grouping candidate universe is the union of the CL-01..CL-09-adjacent `FunctionalityAreas`, plus the coarse areas needed because WK places in-scope names there: `MathFunctions` for exact integer/rational helpers, `PolynomialSymbols` for `Variables`/`Cancel`/`Collect`, `ClusteringSymbols` for the `Sequence*` walkers, `RecurrenceSymbols` for `Differences`, `FormSymbols` for `HoldForm`, and `BitSymbols` as an explicit uncertainty family. The candidate gaps are that universe minus rum's System-context names.

Provenance values in the table:

- `82-listed`: present in the on-disk 96-row brief-082 input.
- `grouping-added`: added by this grouping reconciliation.

The brief text also named `IntegerDigits`, `FromDigits`, `PadLeft`, and `PadRight` as grouping catches. They were already present in the on-disk 96-row input, so the table keeps them as `82-listed`; they are grouping-confirmed rather than counted in the +35 delta below.

## Exclusion Rules

The grouping areas are intentionally coarse. A grouping candidate was dropped when it belonged to one of these out-of-scope buckets:

- float, transcendental, special numeric, numeric-solver, calculus, or real/complex-domain functionality outside rum's rationals-only target;
- graphics, plotting, geometry, display layout, colors, images, or front-end-facing forms;
- linear algebra, sparse arrays, GPU arrays, distance metrics, clustering, Fourier/signal, or arrays-heavy functionality beyond simple list-shape helpers;
- NLP, dictionary, language-data, translation, text analytics, or linguistic services;
- notation, typeset operators, boxes, and display-only comparison/operator families;
- random, date/time, IO/import/export, external/system state, or environment surfaces beyond the matrix-listed subset;
- other broad CAS, solver, compiler, debug-internal, test-internal, or metadata surfaces not currently part of the kernel-first contract.

Exact integer/rational helpers explicitly promoted by the brief are kept. Borderline exact helpers are listed under "Uncertain Or Needs Decision" instead of being silently counted as gaps.

## Totals

- Candidate grouping universe: 1474 unique names across 45 selected/coarse FunctionalityAreas.
- rum System-context symbols in this run: 504.
- Candidate grouping gaps after subtracting rum: 1026.
- Grouping candidates promoted after exclusions: 127.
- Grouping candidates left uncertain / needs-decision: 129.
- Final authoritative gaps after merging brief-082 manual-only rows: 131.
- Old brief-082 report count: 96; authoritative delta: +35.

Drop counts:

| Reason bucket | Dropped count |
|---|---:|
| float-math / special numeric | 412 |
| graphics / display / geometry | 27 |
| linear-algebra / arrays-heavy | 62 |
| NLP / language-data / text analytics | 27 |
| notation / boxes / typeset forms | 126 |
| random / dates / IO / system state | 24 |
| other scoped-out breadth | 92 |

Counts by CL category:

| Category | Count |
|---|---:|
| CL-01 expression, truth, and structural foundation | 52 |
| CL-02 definitions, attributes, assignments, and scoping | 2 |
| CL-03 rules, patterns, and tree walkers | 9 |
| CL-04 functional and list combinators | 41 |
| CL-05 associations and keyed data | 0 |
| CL-06 procedural control, exceptions, and failure flow | 2 |
| CL-07 string language | 20 |
| CL-08 package, namespace, options, and protection tooling | 3 |
| CL-09 tuning, debugging, and verification surface | 2 |

Commonness and ease buckets:

| Commonness | Ease | Count |
|---|---|---:|
| high | trivial | 12 |
| high | moderate | 8 |
| high | involved | 1 |
| medium | trivial | 22 |
| medium | moderate | 62 |
| medium | involved | 8 |
| low | trivial | 2 |
| low | moderate | 14 |
| low | involved | 2 |

Grouping-added gaps over the on-disk 96-row input:

`Binomial`, `Multinomial`, `Fibonacci`, `LucasL`, `Prime`, `PrimePi`, `NextPrime`, `Divisors`, `DivisorSigma`, `EulerPhi`, `MoebiusMu`, `Subfactorial`, `CatalanNumber`, `BernoulliB`, `Pochhammer`, `FactorialPower`, `StirlingS1`, `StirlingS2`, `PartitionsP`, `PartitionsQ`, `Cancel`, `Collect`, `UpTo`, `CountDistinct`, `CountDistinctBy`, `BooleanVariables`, `SequenceSplit`, `PositionLargest`, `PositionSmallest`, `Subsequences`, `SymmetricDifference`, `StringExtract`, `PalindromeQ`, `PrintableASCIIQ`, `LetterCounts`.

Grouping-confirmed brief-named rows already present in the on-disk input:

`IntegerDigits`, `FromDigits`, `PadLeft`, `PadRight`.

## Gap List

### CL-01 (52)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| Minus | 82-listed | `Minus[3] -> -3` | missing: rum returns unevaluated | high | trivial | no |
| Sign | 82-listed | `Sign[-3] -> -1` | missing: rum returns unevaluated | high | trivial | no |
| Floor | 82-listed | `Floor[7/3] -> 2` | missing: rum returns unevaluated | high | trivial | no |
| Ceiling | 82-listed | `Ceiling[7/3] -> 3` | missing: rum returns unevaluated | high | trivial | no |
| IntegerPart | 82-listed | `IntegerPart[-7/3] -> -2` | missing: rum returns unevaluated | medium | trivial | no |
| FractionalPart | 82-listed | `FractionalPart[7/3] -> 1/3` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerDigits | 82-listed | `IntegerDigits[1230] -> {1, 2, 3, 0}` | missing: rum returns unevaluated | medium | moderate | no |
| FromDigits | 82-listed | `FromDigits[{1, 2, 3}] -> 123` | missing: rum returns unevaluated | medium | moderate | no |
| IntegerLength | 82-listed | `IntegerLength[12345] -> 5` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerExponent | 82-listed | `IntegerExponent[48, 2] -> 4` | missing: rum returns unevaluated | medium | moderate | no |
| IntegerReverse | 82-listed | `IntegerReverse[1230] -> 321` | missing: rum returns unevaluated | medium | trivial | no |
| IntegerString | 82-listed | `IntegerString[10, 2] -> "1010"` | missing: rum returns unevaluated | medium | moderate | no |
| DigitCount | 82-listed | `DigitCount[112233, 10, 2] -> 2` | missing: rum returns unevaluated | medium | moderate | no |
| DigitSum | 82-listed | `DigitSum[12345] -> 15` | missing: rum returns unevaluated | medium | trivial | no |
| QuotientRemainder | 82-listed | `QuotientRemainder[17, 5] -> {3, 2}` | missing: rum returns unevaluated | high | moderate | no |
| NumeratorDenominator | 82-listed | `NumeratorDenominator[7/3] -> {7, 3}` | missing: rum returns unevaluated | medium | trivial | no |
| MinMax | 82-listed | `MinMax[{3, 1, 2}] -> {1, 3}` | missing: rum returns unevaluated | medium | trivial | no |
| Subdivide | 82-listed | `Subdivide[0, 1, 4] -> {0, 1/4, 1/2, 3/4, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| Dimensions | 82-listed | `Dimensions[{{1, 2}, {3, 4}}] -> {2, 2}` | missing: rum returns unevaluated | high | moderate | no |
| ArrayDepth | 82-listed | `ArrayDepth[{{1, 2}, {3, 4}}] -> 2` | missing: rum returns unevaluated | medium | trivial | no |
| LeafCount | 82-listed | `LeafCount[f[a, b + c]] -> 5` | missing: rum returns unevaluated | medium | trivial | no |
| FlattenAt | 82-listed | `FlattenAt[{{1, 2}, {3, 4}}, 1] -> {1, 2, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| SymbolName | 82-listed | `SymbolName[Plus] -> "Plus"` | missing: rum returns unevaluated | high | trivial | no |
| Xnor | 82-listed | `Xnor[True, False] -> False` | missing: rum returns unevaluated | medium | trivial | no |
| Majority | 82-listed | `Majority[True, False, True] -> True` | missing: rum returns unevaluated | medium | trivial | no |
| Variables | 82-listed | `Variables[x + y^2] -> {x, y}` | missing: rum returns unevaluated | medium | moderate | no |
| Binomial | grouping-added | `Binomial[5, 2] -> 10` | missing: rum returns unevaluated | high | moderate | no |
| Multinomial | grouping-added | `Multinomial[2, 3, 1] -> 60` | missing: rum returns unevaluated | medium | moderate | no |
| Fibonacci | grouping-added | `Fibonacci[7] -> 13` | missing: rum returns unevaluated | medium | moderate | no |
| LucasL | grouping-added | `LucasL[5] -> 11` | missing: rum returns unevaluated | low | moderate | no |
| Prime | grouping-added | `Prime[5] -> 11` | missing: rum returns unevaluated | medium | moderate | no |
| PrimePi | grouping-added | `PrimePi[10] -> 4` | missing: rum returns unevaluated | medium | moderate | no |
| NextPrime | grouping-added | `NextPrime[10] -> 11` | missing: rum returns unevaluated | medium | moderate | no |
| Divisors | grouping-added | `Divisors[12] -> {1, 2, 3, 4, 6, 12}` | missing: rum returns unevaluated | medium | moderate | no |
| DivisorSigma | grouping-added | `DivisorSigma[1, 12] -> 28` | missing: rum returns unevaluated | low | moderate | no |
| EulerPhi | grouping-added | `EulerPhi[9] -> 6` | missing: rum returns unevaluated | low | moderate | no |
| MoebiusMu | grouping-added | `MoebiusMu[12] -> 0` | missing: rum returns unevaluated | low | moderate | no |
| Subfactorial | grouping-added | `Subfactorial[5] -> 44` | missing: rum returns unevaluated | low | moderate | no |
| CatalanNumber | grouping-added | `CatalanNumber[4] -> 14` | missing: rum returns unevaluated | low | moderate | no |
| BernoulliB | grouping-added | `BernoulliB[4] -> -1/30` | missing: rum returns unevaluated | low | moderate | no |
| Pochhammer | grouping-added | `Pochhammer[3, 4] -> 360` | missing: rum returns unevaluated | low | moderate | no |
| FactorialPower | grouping-added | `FactorialPower[5, 3] -> 60` | missing: rum returns unevaluated | low | moderate | no |
| StirlingS1 | grouping-added | `StirlingS1[4, 2] -> 11` | missing: rum returns unevaluated | low | moderate | no |
| StirlingS2 | grouping-added | `StirlingS2[4, 2] -> 7` | missing: rum returns unevaluated | low | moderate | no |
| PartitionsP | grouping-added | `PartitionsP[5] -> 7` | missing: rum returns unevaluated | low | involved | no |
| PartitionsQ | grouping-added | `PartitionsQ[5] -> 3` | missing: rum returns unevaluated | low | involved | no |
| Cancel | grouping-added | `Cancel[(x^2 - 1)/(x - 1)] -> 1 + x` | missing: rum returns unevaluated | high | moderate | no |
| Collect | grouping-added | `Collect[a*x + b*x + c, x] -> c + (a + b)*x` | missing: rum returns unevaluated | high | moderate | no |
| UpTo | grouping-added | `Take[{1, 2}, UpTo[3]] -> {1, 2}` | missing/partial: `Take[..., UpTo[n]]` returns unevaluated | high | moderate | yes (partial selector mention) |
| CountDistinct | grouping-added | `CountDistinct[{a, b, a}] -> 2` | missing: rum returns unevaluated | medium | trivial | no |
| CountDistinctBy | grouping-added | `CountDistinctBy[{-1, 1, 2}, Abs] -> 2` | missing: rum returns unevaluated | medium | moderate | no |
| BooleanVariables | grouping-added | `BooleanVariables[p && q] -> {p, q}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-02 (2)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| HoldForm | 82-listed | `HoldForm[1 + 1] -> HoldForm[1 + 1]` | divergent: rum evaluates held argument (`HoldForm[2]`) | high | trivial | no (incidental mention only) |
| NValues | 82-listed | `NValues[f] -> {}` | missing: rum returns unevaluated | low | moderate | no |

### CL-03 (9)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| FirstPosition | 82-listed | `FirstPosition[{a, b, a}, a] -> {1}` | missing: rum returns unevaluated | medium | moderate | no |
| FirstCase | 82-listed | `FirstCase[{x, 2, 3}, _Integer] -> 2` | missing: rum returns unevaluated | medium | moderate | no |
| SequenceCases | 82-listed | `SequenceCases[{1, 2, 1, 2, 3}, {1, 2}] -> {{1, 2}, {1, 2}}` | missing: rum returns unevaluated | medium | involved | no |
| SequencePosition | 82-listed | `SequencePosition[{1, 2, 1, 2, 3}, {1, 2}] -> {{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | involved | no |
| SequenceReplace | 82-listed | `SequenceReplace[{1, 2, 1, 2, 3}, {1, 2} -> x] -> {x, x, 3}` | missing: rum returns unevaluated | medium | involved | no |
| SequenceCount | 82-listed | `SequenceCount[{1, 2, 1, 2, 3}, {1, 2}] -> 2` | missing: rum returns unevaluated | medium | involved | no |
| SequenceSplit | grouping-added | `SequenceSplit[{1, 2, 0, 3}, {0}] -> {{1, 2}, {3}}` | missing: rum returns unevaluated | medium | involved | no |
| PositionLargest | grouping-added | `PositionLargest[{1, 3, 2}] -> {2}` | missing: rum returns unevaluated | medium | moderate | no |
| PositionSmallest | grouping-added | `PositionSmallest[{1, 3, 2}] -> {1}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-04 (41)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| Mean | 82-listed | `Mean[{1, 2, 4}] -> 7/3` | missing: rum returns unevaluated | medium | moderate | no |
| ConstantArray | 82-listed | `ConstantArray[x, 3] -> {x, x, x}` | missing: rum returns unevaluated | high | moderate | no |
| Intersection | 82-listed | `Intersection[{1, 2, 3}, {2, 3, 4}] -> {2, 3}` | missing: rum returns unevaluated | medium | moderate | no |
| Partition | 82-listed | `Partition[{1, 2, 3, 4}, 2] -> {{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| Riffle | 82-listed | `Riffle[{a, b, c}, x] -> {a, x, b, x, c}` | missing: rum returns unevaluated | medium | moderate | no |
| RotateLeft | 82-listed | `RotateLeft[{1, 2, 3}, 1] -> {2, 3, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| RotateRight | 82-listed | `RotateRight[{1, 2, 3}, 1] -> {3, 1, 2}` | missing: rum returns unevaluated | medium | trivial | no |
| PadLeft | 82-listed | `PadLeft[{1, 2}, 4] -> {0, 0, 1, 2}` | missing: rum returns unevaluated | medium | moderate | no |
| PadRight | 82-listed | `PadRight[{1, 2}, 4] -> {1, 2, 0, 0}` | missing: rum returns unevaluated | medium | moderate | no |
| Tally | 82-listed | `Tally[{a, b, a}] -> {{a, 2}, {b, 1}}` | missing: rum returns unevaluated | medium | moderate | no |
| Gather | 82-listed | `Gather[{1, 2, 1, 3}] -> {{1, 1}, {2}, {3}}` | missing: rum returns unevaluated | medium | moderate | no |
| GatherBy | 82-listed | `GatherBy[{"a", "bb", "c"}, StringLength] -> {{"a", "c"}, {"bb"}}` | missing: rum returns unevaluated | medium | moderate | no |
| Split | 82-listed | `Split[{1, 1, 2, 3, 3}] -> {{1, 1}, {2}, {3, 3}}` | missing: rum returns unevaluated | medium | moderate | no |
| SplitBy | 82-listed | `SplitBy[{"a", "bb", "c"}, StringLength] -> {{"a"}, {"bb"}, {"c"}}` | missing: rum returns unevaluated | medium | moderate | no |
| Subsets | 82-listed | `Subsets[{a, b}] -> {{}, {a}, {b}, {a, b}}` | missing: rum returns unevaluated | medium | moderate | no |
| Tuples | 82-listed | `Tuples[{0, 1}, 2] -> {{0, 0}, {0, 1}, {1, 0}, {1, 1}}` | missing: rum returns unevaluated | medium | moderate | no |
| Permutations | 82-listed | `Permutations[{a, b, c}, {2}] -> {{a, b}, {a, c}, {b, a}, {b, c}, {c, a}, {c, b}}` | missing: rum returns unevaluated | medium | moderate | no |
| Ordering | 82-listed | `Ordering[{3, 1, 2}] -> {2, 3, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| OrderingBy | 82-listed | `OrderingBy[{"aa", "b"}, StringLength] -> {2, 1}` | missing: rum returns unevaluated | medium | moderate | no |
| ReverseSort | 82-listed | `ReverseSort[{2, 3, 1}] -> {3, 2, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| ReverseSortBy | 82-listed | `ReverseSortBy[{"aaa", "b", "cc"}, StringLength] -> {"aaa", "cc", "b"}` | missing: rum returns unevaluated | medium | moderate | no |
| MaximalBy | 82-listed | `MaximalBy[{"aa", "b", "cc"}, StringLength] -> {"aa", "cc"}` | missing: rum returns unevaluated | medium | moderate | no |
| MinimalBy | 82-listed | `MinimalBy[{"aa", "b", "cc"}, StringLength] -> {"b"}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeLargest | 82-listed | `TakeLargest[{5, 1, 4}, 2] -> {5, 4}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeSmallest | 82-listed | `TakeSmallest[{5, 1, 4}, 2] -> {1, 4}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeLargestBy | 82-listed | `TakeLargestBy[{"aaa", "b", "cc"}, StringLength, 1] -> {"aaa"}` | missing: rum returns unevaluated | medium | moderate | no |
| TakeSmallestBy | 82-listed | `TakeSmallestBy[{"aaa", "b", "cc"}, StringLength, 1] -> {"b"}` | missing: rum returns unevaluated | medium | moderate | no |
| Commonest | 82-listed | `Commonest[{a, b, a}] -> {a}` | missing: rum returns unevaluated | medium | moderate | no |
| DeleteAdjacentDuplicates | 82-listed | `DeleteAdjacentDuplicates[{1, 1, 2, 2, 1}] -> {1, 2, 1}` | missing: rum returns unevaluated | medium | trivial | no |
| ArrayFlatten | 82-listed | `ArrayFlatten[{{{{1}}, {{2}}}, {{{3}}, {{4}}}}] -> {{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| ArrayPad | 82-listed | `ArrayPad[{1, 2}, 1] -> {0, 1, 2, 0}` | missing: rum returns unevaluated | medium | involved | yes (planned ArrayPad-style) |
| ArrayReshape | 82-listed | `ArrayReshape[{1, 2, 3, 4}, {2, 2}] -> {{1, 2}, {3, 4}}` | missing: rum returns unevaluated | medium | moderate | no |
| Pick | 82-listed | `Pick[{a, b, c}, {1, 0, 1}, 1] -> {a, c}` | missing: rum returns unevaluated | medium | moderate | no |
| Differences | 82-listed | `Differences[{1, 4, 9}] -> {3, 5}` | missing: rum returns unevaluated | high | trivial | no |
| Median | 82-listed | `Median[{3, 1, 2}] -> 2` | missing: rum returns unevaluated | medium | moderate | no |
| AllSameBy | 82-listed | `AllSameBy[{"a", "b"}, StringLength] -> True` | missing: rum returns unevaluated | medium | moderate | no |
| Curry | 82-listed | `Curry[f, 2][a][b] -> f[a, b]` | missing: rum returns unevaluated | medium | moderate | no (CurryApplied only) |
| FoldPairList | 82-listed | `FoldPairList[{#1 + #2, #2} &, 0, {1, 2}] -> {1, 3}` | missing: rum returns unevaluated | low | moderate | no |
| FoldWhileList | 82-listed | `FoldWhileList[#1 + #2 &, 0, {1, 2, 3}, LessThan[4]] -> {0, 1, 3, 6}` | missing: rum returns unevaluated | low | moderate | no |
| Subsequences | grouping-added | `Subsequences[{a, b, c}, {2}] -> {{a, b}, {b, c}}` | missing: rum returns unevaluated | medium | moderate | no |
| SymmetricDifference | grouping-added | `SymmetricDifference[{1, 2}, {2, 3}] -> {1, 3}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-05 (0)

No confirmed missing common functions in this audit.

### CL-06 (2)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| Sum | 82-listed | `Sum[i, {i, 3}] -> 6` | missing: rum returns unevaluated | high | moderate | yes (CL-X excluded; brief promotes) |
| Product | 82-listed | `Product[i, {i, 3}] -> 6` | missing: rum returns unevaluated | medium | moderate | no |

### CL-07 (20)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| StringReverse | 82-listed | `StringReverse["abc"] -> "cba"` | missing: rum returns unevaluated | high | trivial | no |
| ToLowerCase | 82-listed | `ToLowerCase["ABC"] -> "abc"` | missing: rum returns unevaluated | high | trivial | no |
| StringInsert | 82-listed | `StringInsert["abcd", "X", 3] -> "abXcd"` | missing: rum returns unevaluated | medium | moderate | no |
| StringReplacePart | 82-listed | `StringReplacePart["abcd", "X", {2, 3}] -> "aXd"` | missing: rum returns unevaluated | medium | moderate | no |
| StringReplaceList | 82-listed | `StringReplaceList["abc", "a" -> "x"] -> {"xbc"}` | missing: rum returns unevaluated | medium | involved | no |
| StringPartition | 82-listed | `StringPartition["abcdef", 2] -> {"ab", "cd", "ef"}` | missing: rum returns unevaluated | medium | moderate | no |
| StringPadRight | 82-listed | `StringPadRight["ab", 4, "x"] -> "abxx"` | missing: rum returns unevaluated | high | trivial | no |
| StringRotateLeft | 82-listed | `StringRotateLeft["abcd", 1] -> "bcda"` | missing: rum returns unevaluated | medium | trivial | no |
| StringRotateRight | 82-listed | `StringRotateRight["abcd", 1] -> "dabc"` | missing: rum returns unevaluated | medium | trivial | no |
| FromCharacterCode | 82-listed | `FromCharacterCode[{65, 66}] -> "AB"` | missing: rum returns unevaluated | medium | moderate | no |
| LetterNumber | 82-listed | `LetterNumber["b"] -> 2` | missing: rum returns unevaluated | medium | moderate | no |
| LowerCaseQ | 82-listed | `LowerCaseQ["abc"] -> True` | missing: rum returns unevaluated | medium | trivial | no |
| UpperCaseQ | 82-listed | `UpperCaseQ["ABC"] -> True` | missing: rum returns unevaluated | medium | trivial | no |
| Capitalize | 82-listed | `Capitalize["abc"] -> "Abc"` | missing: rum returns unevaluated | medium | trivial | no |
| Decapitalize | 82-listed | `Decapitalize["ABC"] -> "aBC"` | missing: rum returns unevaluated | medium | trivial | no |
| FromLetterNumber | 82-listed | `FromLetterNumber[2] -> "b"` | missing: rum returns unevaluated | medium | moderate | no |
| StringExtract | grouping-added | `StringExtract["alpha beta", 2] -> "beta"` | missing: rum returns unevaluated | medium | moderate | no |
| PalindromeQ | grouping-added | `PalindromeQ["level"] -> True` | missing: rum returns unevaluated | low | trivial | no |
| PrintableASCIIQ | grouping-added | `PrintableASCIIQ["ABC"] -> True` | missing: rum returns unevaluated | low | trivial | no |
| LetterCounts | grouping-added | `LetterCounts["abca"] -> <\|"a" -> 2, "c" -> 1, "b" -> 1\|>` | missing: rum returns unevaluated | medium | moderate | no |

### CL-08 (3)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| NameQ | 82-listed | ``NameQ["System`Plus"] -> True`` | missing: rum returns unevaluated | high | trivial | no |
| OptionQ | 82-listed | `OptionQ[Heads -> True] -> True` | missing: rum returns unevaluated | high | trivial | no |
| FilterRules | 82-listed | `FilterRules[{a -> 1, b -> 2}, {b}] -> {b -> 2}` | missing: rum returns unevaluated | medium | moderate | no |

### CL-09 (2)

| Function | Provenance | WK example | rum status | Commonness | Ease | Matrix-listed? |
|---|---|---|---|---|---|---|
| VerificationTest | 82-listed | `Head[VerificationTest[1 + 1, 2]] -> TestObject` | missing: rum returns unevaluated | high | involved | no (TestCreate/TestReport only) |
| TraceScan | 82-listed | `TraceScan[Function[Null], 1 + 1] -> 2` | missing: rum returns unevaluated | medium | involved | no (Trace/TracePrint only) |

## Recommended Implementation Batches

1. Trivial high-commonness first wave: `Minus`, `Sign`, `Floor`, `Ceiling`, `HoldForm`, `StringReverse`, `ToLowerCase`, `StringPadRight`, `LowerCaseQ`, `UpperCaseQ`, `Capitalize`, `Decapitalize`, `NameQ`, `OptionQ`, `SymbolName`, `IntegerLength`, `IntegerReverse`, `DigitSum`, `NumeratorDenominator`, `MinMax`, `LeafCount`, `Xnor`, `Majority`, and `Differences`.

2. Exact scalar and rational-expression helpers: `IntegerPart`, `FractionalPart`, `QuotientRemainder`, `IntegerDigits`, `FromDigits`, `IntegerExponent`, `IntegerString`, `DigitCount`, `Binomial`, `Multinomial`, `Cancel`, `Collect`, and `BooleanVariables`.

3. List shape, counting, and set plumbing: `Dimensions`, `ArrayDepth`, `FlattenAt`, `ConstantArray`, `PadLeft`, `PadRight`, `ArrayFlatten`, `ArrayPad`, `ArrayReshape`, `UpTo`, `CountDistinct`, `CountDistinctBy`, `SymmetricDifference`, `Subsequences`, `Pick`, `Intersection`, `Partition`, `Riffle`, rotations, `Tally`, `Gather`, `GatherBy`, `Split`, `SplitBy`, `Subsets`, `Tuples`, and `Permutations`.

4. Ordering, ranking, aggregation, and position helpers: `Ordering`, `OrderingBy`, `ReverseSort`, `ReverseSortBy`, `MaximalBy`, `MinimalBy`, `TakeLargest`, `TakeSmallest`, `TakeLargestBy`, `TakeSmallestBy`, `Commonest`, `DeleteAdjacentDuplicates`, `Mean`, `Median`, `AllSameBy`, `PositionLargest`, and `PositionSmallest`.

5. String helpers: `StringInsert`, `StringReplacePart`, `StringReplaceList`, `StringPartition`, `StringRotateLeft`, `StringRotateRight`, `FromCharacterCode`, `LetterNumber`, `FromLetterNumber`, `StringExtract`, `PalindromeQ`, `PrintableASCIIQ`, and `LetterCounts`.

6. Tree, sequence, functional, and iterator surface: `FirstPosition`, `FirstCase`, `SequenceCases`, `SequencePosition`, `SequenceReplace`, `SequenceCount`, `SequenceSplit`, `Curry`, `FoldPairList`, `FoldWhileList`, `Sum`, and `Product`.

7. Exact number-theory long tail: `Prime`, `PrimePi`, `NextPrime`, `Divisors`, `DivisorSigma`, `EulerPhi`, `MoebiusMu`, `Fibonacci`, `LucasL`, `Subfactorial`, `CatalanNumber`, `BernoulliB`, `Pochhammer`, `FactorialPower`, `StirlingS1`, `StirlingS2`, `PartitionsP`, and `PartitionsQ`.

8. Package/options/testing/debugging: `NValues`, `FilterRules`, `VerificationTest`, and `TraceScan`.

## Uncertain Or Needs Decision

These 129 grouping candidates are not counted in the 131 authoritative gaps because scope is genuinely unclear or they sit next to an explicitly excluded domain:

- Exact bitwise integer operations: `BitAnd, BitOr, BitXor, BitNot, BitGet, BitSet, BitClear, BitFlip, BitShiftLeft, BitShiftRight, BitLength`.
- Exact number-theory breadth beyond the promoted subset: `FactorInteger, ChineseRemainder, ExtendedGCD, ModularInverse, Divisible, CompositeQ, PrimePowerQ, Primes, PowerModList, IntegerPartitions, DivisorSum, PerfectNumber, PerfectNumberQ, MixedRadix, NumberCompose, NumberDecompose, NumberDigit, NumberExpand`.
- Boolean/SAT and logic transformers: `BooleanConvert, BooleanTable, BooleanMinimize, BooleanFunction, BooleanCountingFunction, BooleanMinterms, BooleanMaxterms, LogicalExpand, TautologyQ, SatisfiableQ, SatisfiabilityCount, SatisfiabilityInstances, FindInstance, Resolve, Exists, ForAll`.
- Polynomial/CAS breadth beyond Cancel/Collect/Variables: `CoefficientRules, FromCoefficientRules, Exponent, FactorList, FactorTerms, HornerForm, MonomialList, PolynomialGCD, PolynomialLCM, PolynomialQuotient, PolynomialRemainder, PolynomialReduce, Resultant, SquareFreeQ, IrreduciblePolynomialQ`.
- String pattern, Unicode, and import/export boundary: `StartOfLine, EndOfLine, WordBoundary, StringSkeleton, HexadecimalCharacter, PunctuationCharacter, CharacterName, CharacterNormalize, ExportString, ImportString, and the Names-only character-class omissions LetterCharacter/DigitCharacter/WhitespaceCharacter/WordCharacter`.
- Definition, attribute, inactive, and rule-dispatch metadata: `NHoldAll, NHoldFirst, NHoldRest, ReadProtected, Locked, Constant, NumericFunction, FullDefinition, ExcludedContexts, IncludedContexts, Defer, Activate, Inactivate, Inactive, Dispatch, TwoWayRule`.
- Functional/list boundary helpers: `AllMatch, AnyMatch, NoneMatch, ComposeList, InverseFunction, DeleteElements, Discard, UniqueElements, RankedMax, RankedMin, LexicographicSort, NumericalSort, SortedBy, Groupings`.

## Matrix Reconciliation Notes

- `ArrayPad` is covered indirectly by the CL-04 planned ArrayPad-style note.
- `UpTo` is mentioned by current CL-01 selector parity notes, but `Take[..., UpTo[n]]` still returns unevaluated in rum.
- `Sum` is explicitly listed in CL-X as outside the current non-math core target, but this audit retains the brief-082 promotion for finite exact iterator support.
- `Curry` is not listed directly; only `CurryApplied` is listed.
- `VerificationTest` is not listed directly; only `TestCreate` and `TestReport` are listed.
- `TraceScan` is not listed directly; only `Trace` and `TracePrint` are listed.
- `HoldForm` is only incidentally mentioned in message formatting, not tracked as a same-name holding function.
- Most confirmed gaps remain matrix omissions. If any implementation batch is accepted, update `docs/core-language-support-matrix.md` alongside the code.

---

## Scope decisions (user, 2026-07-02) — "土气版" implementation policy

The implementation philosophy is **good-enough / folk implementations ("土气版")**, matching
the existing `Factor`, `PrimeQ` (trial division), etc.: simple, exact-on-common-inputs, with a
message / unevaluated / `TerminatedEvaluation` fallback on the heavy cases. NOT full CAS
algorithms.

**Do vs don't for the number-theory family (part of the +35 grouping-added):**
- **DO (small-table + naive feasible)** — implement folk-style, LOW priority (implement after
  the trivial-common batches): `Divisors`, `EulerPhi`, `MoebiusMu`, `DivisorSigma`,
  `PrimeNu`, `PrimeOmega`, `NextPrime` (trial from factorization / trial search); `Fibonacci`,
  `LucasL`, `Binomial`, `Multinomial`, `CatalanNumber`, `Subfactorial`, `Pochhammer`,
  `FactorialPower`, `StirlingS1`, `StirlingS2` (cheap recurrences, exact); `PartitionsP`/
  `PartitionsQ` (Euler pentagonal recurrence); `BernoulliB` (exact rational recurrence — fits
  rationals-only). `Prime` / `PrimePi`: a SMALL precomputed table (up to ~1e6–1e7, a few
  hundred KB–few MB — NOT the "first 1e9 primes" ≈ 1–2 GB, too big to embed) + a message and a
  crude sieve/trial fallback beyond the table (or `TerminatedEvaluation` for huge inputs).
- **DON'T (too heavy — documented non-goal, not a gap):** general polynomial factorization for
  `Factor` (Berlekamp–Zassenhaus: mod-p reduction + Hensel lifting — needs too much number
  theory). `Factor` stays the current structural/pattern version: it factors e.g.
  `x^3+y^3+z^3-3xyz` and `x^2-y^2` but NOT `x^4+4y^4` / `x^4+4` (Sophie Germain) or general
  polynomials. Record this as a `partial` limit in the support matrix, not a gap.

**Batch order = commonness × ease, folk-style throughout:**
1. Trivial × high-commonness (FIRST): `Minus`, `Sign`, `Floor`, `Ceiling`, `HoldForm` (HoldAll
   fix), `StringReverse`, `ToLowerCase`, `LowerCaseQ`, `UpperCaseQ`, `Capitalize`,
   `Decapitalize`, `IntegerLength`, `IntegerReverse`, `SymbolName`, `NameQ`, `MinMax`,
   `Differences`, `LeafCount`, …
2. Moderate: `Sum`/`Product` (iterator specs), set ops (`Intersection`/`Union`/`Complement`/
   `Partition`), list plumbing (`PadLeft`/`PadRight`/`ConstantArray`), `Binomial`/`Fibonacci`.
3. LAST / on-demand: the folk number-theory family above.
The 131 authoritative gaps all stay in scope (folk-doable); only their PRIORITY differs.
