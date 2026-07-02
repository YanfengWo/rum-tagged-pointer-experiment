# brief-083 — reconcile the long-tail gap list against the WK grouping data (authoritative)

## Goal

Produce the AUTHORITATIVE in-scope common-function gap list by reconciling brief-082's manual
pass with the WK `FunctionalityAreas` grouping data (the "radical route"). brief-082's manual
filter was a good first cut but PROVABLY incomplete: cross-checking the grouping caught common
in-scope gaps it missed (`Binomial`, `PadLeft`, `PadRight`, `IntegerDigits`, `FromDigits`,
`Prime`, `PrimePi`, `Cancel`, `Collect`). Use the grouping to find ALL of them. DIAGNOSTIC
only — update `docs/long-tail-gap-audit.md`, NO product-code change, no `git commit`.

## Inputs (already on disk — do NOT regenerate; the extractor needs network)

- `WLBuiltin/wolfram-language-groups.json` — `FunctionalityAreas` → symbol-name lists (208
  areas, generated via `scripts/export-wl-builtins.wls`; gitignored local artifact). USE THIS.
- rum's symbols: `target/debug/rum-cli --eval 'Names["System`*"]'` (503).
- brief-082's first cut: `docs/long-tail-gap-audit.md` (96) — merge, don't discard.
- WK oracle for examples: resolve via the documented policy (`RUM_DIFFTEST_ORACLE_KERNEL` /
  `wolframscript`), NOT a hardcoded path.

## Method

1. **Collect the candidate universe** = union of symbols in the IN-SCOPE FunctionalityAreas
   (below), then subtract rum's 503 → candidate gaps.
2. **Apply the EXCLUSION rules** (these are why the grouping alone over-counts — areas are
   coarse). Drop a candidate if it is:
   - **float/transcendental/special-math** (rum is RATIONALS-ONLY): `Sin`/`Cos`/`Tan`/`Exp`/
     `Log`/`Sqrt`/`N`/`Surd`/`ArcSin`.../`Erf`/`Gamma`/`Zeta`/`Bessel*`/`Elliptic*`/etc., and
     anything in `MathFunctions` EXCEPT the exact-integer/rational subset (KEEP: `Sign`,
     `Floor`, `Ceiling`, `Round`(have), `IntegerPart`, `FractionalPart`, `Mod`(have),
     `Quotient`(have), `QuotientRemainder`, `GCD`(have), `LCM`(have), `Binomial`, `Multinomial`,
     `Factorial`(have), `Fibonacci`, `LucasL`, `Prime`, `PrimePi`, `PrimeQ`(have), `NextPrime`,
     `Divisors`, `DivisorSigma`, `EulerPhi`, `MoebiusMu`, `PowerMod`(have), `IntegerExponent`,
     `IntegerDigits`, `FromDigits`, `IntegerLength`, `IntegerReverse`, `CoprimeQ`(have),
     `Subfactorial`, `CatalanNumber`, `BernoulliB`(rational), `Pochhammer`, `FactorialPower`,
     `StirlingS1/S2`, `PartitionsP/Q` — exact-arithmetic-compatible; when unsure, list under
     "uncertain");
   - **graphics / plotting / regions / colors / images** (`Graphics`, `Plot`, `Disk`,
     `Point`, `Polygon`, `Line`, `Framed`, `Grid`, `Column`(display), `Panel`, ...);
   - **linear algebra / arrays-heavy** (`Det`, `Inverse`, `Cross`, `Norm`, `IdentityMatrix`,
     `SparseArray`, `GPUArray`, `Band`, `Tr`, `Fourier`, ...) — KEEP simple list-shape ones
     (`ConstantArray`, `PadLeft`, `PadRight`, `ArrayReshape`, `ArrayPad`, `ArrayDepth`,
     `Dimensions`, `ArrayFlatten`, `Permutations`, `Tuples`, `Subsets`, `Subsequences`);
   - **NLP / language-data / dictionary / text-analytics** (`WordList`, `DictionaryWordQ`,
     `PartOfSpeech`, `TextTranslation`, `LanguageIdentify`, `WordStem`, `Synonyms`,
     `TextCases`/`TextStructure`/`TextContents`, `Transliterate`, `RemoveDiacritics`(have),
     ...);
   - **notation / typeset / box / display operators** (`CircleDot`, `CenterDot`, `Wedge`,
     `Vee`, `Cup`, `Cap`, `Congruent`, `Precedes`/`Succeeds`(+ Not*/*SlantEqual/*Tilde
     families), `LeftTriangle*`, `Square*`, `AngleBracket`, `BracketingBar`, `Subscript`,
     `Superscript`, `OverBar`/`UnderBar`/`OverDot`/`OverHat`, `SpanFromLeft`, ... — these are
     front-end/typesetting, out of a rationals-only core-language runtime);
   - **random / dates / files / external / cloud / devices / crypto** beyond the tiny surface
     the matrix already lists (`RandomInteger`, `RandomReal`, `RandomWord`, `DateList`,
     `ExportString`(maybe keep?), `ImportString`, `WriteString`, ...);
   - **interval/complex constructs that require reals** (`Interval` arithmetic on reals,
     `Complexes`/`Reals` domain objects) — keep the exact ones only.
   Everything you drop, record with its reason bucket (for the totals).
3. **Merge** the surviving candidates with brief-082's 96 (union). Flag each as
   "82-listed" / "grouping-added" (like `Binomial`, `PadLeft`, `Prime`, `Cancel`, `Collect`).
4. **Organize by the matrix CL-01..CL-09 category** (map each gap to its CL section) and for
   each record: WK example (in→out, oracle-verified), commonness (high/med/low), ease
   (trivial/moderate/involved), matrix-listed (yes/no).
5. **Batches:** finalize the ranked implementation batch order (commonness × ease), calling
   out the trivial high-commonness first wave.

## Deliverable

Rewrite `docs/long-tail-gap-audit.md` as the AUTHORITATIVE version:
- method note (grouping + manual reconciliation) + the exclusion rules + per-reason drop
  counts (float-math / graphics / linalg / nlp / notation / random-dates-io / other);
- the authoritative in-scope gap list by CL category (with the 82-vs-grouping provenance);
- totals (candidate universe, after-exclusion, final gap count, vs the old 96);
- the ranked batch plan for the implementation line.
Keep the old 96 numbers noted for provenance.

## Hard rules

- NO product-code change; doc only. No `git commit`. Never edit any `mma-hol` file. No unsafe.
- USE the on-disk `WLBuiltin/wolfram-language-groups.json`; do NOT regenerate (no network).
- WK oracle via the documented resolution for examples; save raw output to a file, then parse
  (rum returns exact rationals).
- Respect RATIONALS-ONLY: do not list float/transcendental numerics as gaps. When a scope
  call is genuinely unclear, put it in an "uncertain / needs-decision" section, don't guess.

## Report format (chat reply)

1. Totals: candidate universe, per-reason drop counts, final authoritative gap count, and
   the delta over brief-082's 96 (what the grouping added).
2. The grouping-added gaps brief-082 missed (the value of this pass).
3. Final per-CL counts + the first implementation batch (trivial × high-commonness).
4. Uncertain/needs-decision scope calls.
5. Confirm `docs/long-tail-gap-audit.md` rewritten, no product-code change.
