# brief-092 — long-tail batch 3b: recurrence + closed-form combinatorial number theory

## Goal

Implement 9 folk number-theory builtins that need NO factorization — elegant recurrences +
trivial closed forms: `PartitionsP`, `PartitionsQ`, `BernoulliB`, `Subfactorial`,
`StirlingS1`, `StirlingS2`, `CatalanNumber`, `Multinomial`, `Pochhammer`. All currently
unevaluated. Folk, exact, WK-calibrated ([[rum-folk-implementation-scope]]). Product-code;
no-regression bar includes mma-hol byte-identical.

## Behavior (WK anchors — match exactly)

Non-integer / out-of-domain args → symbolic passthrough (except `Pochhammer`, which is
symbolic-capable, below). Use a work-limit guard on the O(n)/O(n^1.5)/O(nk) recurrences.

- **`PartitionsP[n]`** — # integer partitions, via **Euler's pentagonal-number recurrence**
  (elegant, O(n^1.5)): `PartitionsP[5]`→`7`, `[10]`→`42`, `[0]`→`1`, `[100]`→`190569292`
  (large test to exercise the recurrence).
- **`PartitionsQ[n]`** — # partitions into DISTINCT parts (DP / recurrence):
  `PartitionsQ[5]`→`3`, `[10]`→`10`.
- **`BernoulliB[n]`** — Bernoulli numbers, EXACT RATIONAL (fits rum): via the recurrence
  `Σ_{k=0}^{n} C(n+1,k) B_k = 0`. `Table[BernoulliB[i],{i,0,8}]` →
  `{1, -1/2, 1/6, 0, -1/30, 0, 1/42, 0, -1/30}` (note WK's `B[1] = -1/2` convention; odd n>1
  → 0).
- **`Subfactorial[n]`** — derangements, recurrence `!n = n·!(n-1) + (-1)^n`:
  `Table[Subfactorial[i],{i,0,6}]` → `{1, 0, 1, 2, 9, 44, 265}`.
- **`StirlingS1[n,k]`** — SIGNED Stirling first kind (WK's convention is signed), recurrence
  `s(n,k) = s(n-1,k-1) − (n-1)·s(n-1,k)`: `StirlingS1[4,2]`→`11`, `StirlingS1[5,2]`→`-50`
  (note the sign).
- **`StirlingS2[n,k]`** — Stirling second kind, recurrence `S(n,k) = k·S(n-1,k) + S(n-1,k-1)`:
  `StirlingS2[4,2]`→`7`, `StirlingS2[5,3]`→`25`.
- **`CatalanNumber[n]`** — `Binomial[2n,n]/(n+1)` (REUSE the existing Binomial):
  `Table[CatalanNumber[i],{i,0,6}]` → `{1,1,2,5,14,42,132}`.
- **`Multinomial[n1,n2,...]`** — `(Σ nᵢ)! / ∏(nᵢ!)`; **Orderless** (WK attribute):
  `Multinomial[1,2,3]`→`60`, `Multinomial[2,2]`→`6`.
- **`Pochhammer[a,n]`** — rising factorial `∏_{i=0}^{n-1}(a+i)` for integer n≥0; works for
  SYMBOLIC `a` too (finite product → a polynomial): `Pochhammer[2,3]`→`24`,
  `Pochhammer[3,0]`→`1`, `Pochhammer[a,3]`→`a(1+a)(2+a)` (i.e. rum's canonical form of that
  product). This is the one function that DOES compute with a symbolic first arg.

## Attributes — CALIBRATE DIRECTLY (do not trust this brief's list)

For EACH function, probe WK with a SINGLE `Attributes[f]` call and set rum to match exactly
(prior briefs' attribute lists were misread from a nested probe — always single-probe). Known
notables to verify: `Multinomial` is `Orderless`; `BernoulliB`/`Subfactorial`/`CatalanNumber`
carry `NumericFunction`(+`ReadProtected`); most are `Listable`.

## Where / how

Put these in the number-theory / combinatorics modules (`combinatorics.rs` from batch-2c and
`number_theory.rs` from batch-3a are the natural homes). Reuse `Binomial` for `CatalanNumber`,
factorials for `Multinomial`, exact `Rational` for `BernoulliB`. No float paths.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests per function: the anchors + the large `PartitionsP[100]` + a signed
`StirlingS1` case + the rational `BernoulliB` table + `Pochhammer` symbolic-a + `Multinomial`
Orderless + symbolic passthrough. Spot-check against WK via the documented oracle resolution
(`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not hardcoded); save raw to a file, parse (rum
returns exact rationals — parse BernoulliB with Fraction).

## Hard rules

- Product code ONLY for these 9 builtins (+ tests). Reuse existing Binomial/factorial/Rational.
  No `Expr`/serde change. No new `unsafe`. Work-limit guard on the recurrences.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes (single-probe each). BernoulliB stays EXACT rational.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 9 builtins (files); the recurrence used for each; attributes (single-
   probe results).
2. WK calibration — the anchors matched (incl. PartitionsP[100], signed StirlingS1, rational
   BernoulliB, symbolic Pochhammer, Orderless Multinomial).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match, with analysis.
