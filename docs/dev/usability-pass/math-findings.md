# Math Usability Audit Findings

Coverage statement: findings-table rows cover 62 distinct catalog names: 31
from `arithmetic.rs`, 9 from `number_theory.rs`, 11 from `combinatorics.rs`,
5 from `integer_digits.rs`, 5 from `polynomials.rs`, and 1 from
`calculus.rs`. The coverage check printed `COVERAGE-OK 62/62`.

## Top 10 Gaps By Script-User Value

1. **Safety:** `FromDigits[{2}, 2]` returns `2` instead of rejecting a digit outside the base.
2. **Safety:** zero divisors/moduli and zero rounding multiples stay inert without messages in `Divide`, `Mod`, `Quotient`, `PowerMod`, and `Round`.
3. **Safety:** `Coefficient` and `CoefficientList` treat unsupported target-dependent terms such as `x^-1` as coefficients.
4. **Safety:** nonnegative-index combinatorics functions return inert calls without diagnostics on negative exact integers.
5. **Safety:** zero-domain number-theory calls such as `Divisors[0]`, `DivisorSigma[_, 0]`, `PrimeNu[0]`, `PrimeOmega[0]`, and `NextPrime[_, 0]` return inert calls without diagnostics.
6. **Safety:** invalid digit bases and negative digit lengths in `IntegerLength`, `IntegerDigits`, `IntegerReverse`, and `DigitCount` return inert calls without diagnostics.
7. `Total` lacks the cheap documented level forms (`Total[list, n]` and `Total[list, {level}]`).
8. Attribute discovery for `GCD`, `LCM`, `Max`, and `Min` lacks the grouping attributes users expect (`Orderless`/`Flat`/`OneIdentity`, plus listability discovery for extrema).
9. `Commonest[list, n]` and list-valued `Quantile[list, qs]` are missing cheap aggregation forms.
10. **Safety:** malformed `D` variable/order specs return inert calls without diagnostics, even though supported higher-order and multi-variable derivative forms already work.

## Findings Table

### Arithmetic

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Identity | WL | OK | | | | |
| Plus | WL | OK | | Core grouping attributes are present; exact-rational arithmetic is intended folk scope (Evidence E1; `docs/guide/design-boundaries.md:13`). | | |
| Times | WL | OK | | Core grouping attributes are present; exact-rational arithmetic is intended folk scope (Evidence E1; `docs/guide/design-boundaries.md:13`). | | |
| Power | WL | gap | safety | `Power[0, 0]` stays inert without a domain message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:28`). | Emit a targeted indeterminate-form message and return a fail-closed inert or `$Failed` result; keep exact integer-power semantics unchanged. | S |
| Subtract | WL | OK | | Existing canonical arithmetic makes ordinary list subtraction work even though the direct catalog attributes are sparse (Evidence E2). | | |
| Divide | WL | gap | safety | `Divide[1, 0]` stays inert without a zero-divisor message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/canonical.rs:107`). | Emit a zero-divisor diagnostic and fail closed; exact rational division otherwise unchanged. | S |
| Abs | WL | gap | convenience | `Abs` is `Listable` but lacks the numeric-function/protection attributes used by the neighboring numeric surface (Evidence E1; `crates/rum-stdlib/src/catalog/arithmetic.rs:47`). | Add `NumericFunction` and `Protected`; current listable absolute-value behavior unchanged. | S |
| Sign | WL | OK | | | | |
| Floor | WL | OK | | Exact rational floor is intended; no floating-point behavior is expected (Evidence E5; `docs/guide/design-boundaries.md:13`). | | |
| Ceiling | WL | OK | | Exact rational ceiling is intended; no floating-point behavior is expected (`docs/guide/design-boundaries.md:13`). | | |
| IntegerPart | WL | OK | | Exact rational truncation is intended; no floating-point behavior is expected (`docs/guide/design-boundaries.md:13`). | | |
| FractionalPart | WL | OK | | Exact rational fractional parts are intended; no floating-point behavior is expected (`docs/guide/design-boundaries.md:13`). | | |
| Minus | WL | OK | | | | |
| Round | WL | gap | safety | `Round[3/2, 0]` stays inert without a zero-multiple diagnostic; the useful `Round[x, a]` form itself already works (Evidence E3, E5; `crates/rum-runtime/src/runtime/arithmetic/rounding.rs:106`). | Emit a message for zero multiples; keep exact rational round-to-multiple behavior unchanged. | S |
| Factorial | WL | gap | safety | Negative exact integers return inert calls without a domain message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:13`). | Emit a nonnegative-integer diagnostic for negative numeric input; keep symbolic inert fallback unchanged. | S |
| Factorial2 | WL | gap | safety | Negative exact integers return inert calls without a domain message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:49`). | Emit a nonnegative-integer diagnostic for negative numeric input; keep symbolic inert fallback unchanged. | S |
| Total | WL | gap | flexibility | `Total[list, n]` and `Total[list, {level}]` return inert calls; the implementation accepts exactly one list argument (Evidence E5; `crates/rum-runtime/src/runtime/arithmetic/parts.rs:4`). | Add cheap level forms over existing list traversal and `Plus`; default one-argument behavior unchanged. | M |
| Mean | WL | OK | | Empty lists fail closed with a message; exact rational means are intended (Evidence E8; `docs/guide/design-boundaries.md:13`). | | |
| Median | WL | OK | | Empty lists fail closed with a message; exact rational averaging is intended (Evidence E8; `docs/guide/design-boundaries.md:13`). | | |
| Commonest | WL | gap | flexibility | `Commonest[list, n]` emits an arity message and stays inert (Evidence E5; `crates/rum-stdlib/src/arithmetic/stats.rs:32`). | Add the documented count form, returning the top `n` frequency classes with current tie ordering preserved. | S |
| Quantile | WL | gap | flexibility | A list of requested quantiles stays inert even though each exact scalar quantile is supported (Evidence E5; `crates/rum-stdlib/src/arithmetic/stats.rs:60`). | Map exact scalar quantile selection over a list of `q` values; keep the current nearest-rank scalar method unchanged. | S |
| Denominator | WL | OK | | Exact-rational denominator behavior is intended; no floating-point behavior is expected (`docs/guide/design-boundaries.md:13`). | | |
| Numerator | WL | OK | | Exact-rational numerator behavior is intended; no floating-point behavior is expected (`docs/guide/design-boundaries.md:13`). | | |
| Mod | WL | gap | safety | `Mod[5, 0]` stays inert without a zero-modulus message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:5`, `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:133`). | Emit a zero-modulus diagnostic and fail closed; valid integer modulus behavior unchanged. | S |
| Quotient | WL | gap | safety | `Quotient[5, 0]` stays inert without a zero-divisor message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:22`, `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:142`). | Emit a zero-divisor diagnostic and fail closed; valid floor quotient behavior unchanged. | S |
| PowerMod | WL | gap | safety | `PowerMod[2, 3, 0]` stays inert without a zero-modulus message (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:90`, `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:159`). | Emit a zero-modulus diagnostic and fail closed. | S |
| PowerMod | WL | gap | flexibility | Negative exponent modular-inverse forms return inert even for invertible bases, and non-invertible inverse cases get no targeted failure (Evidence E3; `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:103`). | Add a small extended-GCD inverse path for negative exponents; message on non-invertible bases. | M |
| GCD | WL | gap | convenience | `GCD` is only `Listable`; `Attributes[GCD]` lacks the documented grouping/discovery attributes called out for this family (Evidence E1, E2; `crates/rum-stdlib/src/catalog/arithmetic.rs:180`). | Add `Orderless`/`Flat`/`OneIdentity` and protection/numeric discovery as appropriate; measure because this touches attribute-driven planning. | S |
| LCM | WL | gap | convenience | `LCM` is only `Listable`; `Attributes[LCM]` lacks the documented grouping/discovery attributes called out for this family (Evidence E1, E2; `crates/rum-stdlib/src/catalog/arithmetic.rs:186`). | Add `Orderless`/`Flat`/`OneIdentity` and protection/numeric discovery as appropriate; measure because this touches attribute-driven planning. | S |
| Max | WL | gap | convenience | `Max` has no attributes, so symbolic ordering and attribute discovery differ from the expected orderless/listable extrema surface (Evidence E1, E2; `crates/rum-stdlib/src/catalog/arithmetic.rs:192`). | Add the documented extrema attributes and preserve current exact extrema reductions; measure attribute-driven planning. | S |
| Min | WL | gap | convenience | `Min` has no attributes, so symbolic ordering and attribute discovery differ from the expected orderless/listable extrema surface (Evidence E1, E2; `crates/rum-stdlib/src/catalog/arithmetic.rs:197`). | Add the documented extrema attributes and preserve current exact extrema reductions; measure attribute-driven planning. | S |
| MinMax | WL | OK | | Current scalar/list exact extrema behavior is documented; `MinMax[list, n]` was probed as inert but not filed because it is outside the cheap confirmed surface for this pass (Evidence E5). | | |

### Number Theory

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Divisors | WL | gap | safety | `Divisors[0]` returns inert with no diagnostic even though the positive-divisor set is not finite (Evidence E6; `crates/rum-stdlib/src/number_theory.rs:27`). | Emit a zero-domain diagnostic and fail closed; keep positive-divisor behavior for nonzero integers. | S |
| EulerPhi | WL | OK | | `EulerPhi[0] -> 0` and negative inputs normalize by absolute value; folk exact-integer scope is intended (Evidence E6). | | |
| MoebiusMu | WL | OK | | `MoebiusMu[0] -> 0` and negative inputs normalize by absolute value; folk exact-integer scope is intended (Evidence E6). | | |
| DivisorSigma | WL | gap | safety | `DivisorSigma[k, 0]` returns inert with no diagnostic (Evidence E6; `crates/rum-stdlib/src/number_theory.rs:113`). | Emit a zero-domain diagnostic and fail closed; keep exact divisor sums for nonzero integers. | S |
| Prime | WL | OK | | Non-positive and rational indices already emit `Prime::intpp`; bounded prime generation is folk scope (Evidence E3, E6; `docs/guide/design-boundaries.md:31`). | | |
| PrimePi | WL | OK | | Exact rational flooring and bounded prime counting are intended folk scope (Evidence E6; `docs/guide/design-boundaries.md:31`). | | |
| PrimeNu | WL | gap | safety | `PrimeNu[0]` returns inert with no diagnostic (Evidence E6; `crates/rum-stdlib/src/number_theory.rs:193`). | Emit a zero-domain diagnostic or document/return the chosen zero value; avoid silent inert fallback for exact numeric input. | S |
| PrimeOmega | WL | gap | safety | `PrimeOmega[0]` returns inert with no diagnostic (Evidence E6; `crates/rum-stdlib/src/number_theory.rs:218`). | Emit a zero-domain diagnostic or document/return the chosen zero value; avoid silent inert fallback for exact numeric input. | S |
| NextPrime | WL | gap | safety | `NextPrime[5, 0]` returns inert with no diagnostic (Evidence E6; `crates/rum-stdlib/src/number_theory.rs:253`). | Emit a nonzero-step diagnostic or define the zero-step result explicitly; keep positive/negative step behavior unchanged. | S |

### Combinatorics

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Binomial | WL | OK | | Bad arity already messages; negative lower index returns `0`, and bounded exact integer work is intended folk scope (Evidence E7). | | |
| Fibonacci | WL | gap | safety | Wrong arities such as `Fibonacci[]` and `Fibonacci[1, 2]` return inert calls without an arity message (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:31`). | Use the standard exact-arity diagnostic; valid integer-index Fibonacci behavior unchanged. | S |
| PartitionsP | WL | gap | safety | Negative exact integer indices return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:54`). | Emit a nonnegative-integer diagnostic and fail closed; bounded recurrence unchanged. | S |
| PartitionsQ | WL | gap | safety | Negative exact integer indices return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:76`). | Emit a nonnegative-integer diagnostic and fail closed; bounded recurrence unchanged. | S |
| BernoulliB | WL | gap | safety | Negative exact integer indices return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:98`). | Emit a nonnegative-integer diagnostic and fail closed; keep Bernoulli-number recurrence scope unchanged. | S |
| Subfactorial | WL | gap | safety | Negative exact integer indices return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:120`). | Emit a nonnegative-integer diagnostic and fail closed; bounded derangement computation unchanged. | S |
| StirlingS1 | WL | gap | safety | Negative exact integer arguments return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:145`). | Emit a nonnegative-integer diagnostic and fail closed; current signed Stirling recurrence unchanged. | S |
| StirlingS2 | WL | gap | safety | Negative exact integer arguments return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:171`). | Emit a nonnegative-integer diagnostic and fail closed; current Stirling recurrence unchanged. | S |
| CatalanNumber | WL | gap | safety | Negative exact integer indices return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:197`). | Emit a nonnegative-integer diagnostic and fail closed; current binomial-derived computation unchanged. | S |
| Multinomial | WL | gap | safety | Negative exact integer arguments return inert calls without a nonnegative-index diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:219`). | Emit a nonnegative-integer diagnostic and fail closed; valid nonnegative multinomial behavior unchanged. | S |
| Pochhammer | WL | gap | safety | Negative count arguments return inert calls without a nonnegative-count diagnostic (Evidence E7; `crates/rum-stdlib/src/combinatorics.rs:243`). | Emit a nonnegative-integer diagnostic for the count; finite product behavior unchanged. | S |

### Integer Digits

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| IntegerLength | WL | gap | safety | Invalid bases such as `1` return inert without a diagnostic (Evidence E5; `crates/rum-stdlib/src/integer_digits.rs:23`, `crates/rum-stdlib/src/integer_digits.rs:238`). | Emit a base-domain message for bases below 2; valid exact-integer length behavior unchanged. | S |
| IntegerDigits | WL | gap | safety | Invalid bases and negative length arguments return inert without diagnostics; the documented length form itself already works (Evidence E5; `crates/rum-stdlib/src/integer_digits.rs:53`, `crates/rum-stdlib/src/integer_digits.rs:62`). | Emit base and nonnegative-length diagnostics; preserve `IntegerDigits[n, base, len]`. | S |
| IntegerReverse | WL | gap | safety | Invalid bases such as `1` return inert without a diagnostic (Evidence E5; `crates/rum-stdlib/src/integer_digits.rs:107`). | Emit a base-domain message for bases below 2; valid reversal behavior unchanged. | S |
| FromDigits | WL | gap | safety | List digits are not range-checked: `FromDigits[{2}, 2]` returns `2` while the string path rejects out-of-base digits (Evidence E5; `crates/rum-stdlib/src/integer_digits.rs:144`, `crates/rum-stdlib/src/integer_digits.rs:302`). | Validate every list digit against the base and message/fail closed on negative or out-of-range digits; keep valid list/string forms unchanged. | S |
| DigitCount | WL | gap | safety | Invalid bases such as `1` return inert without a diagnostic; digit and digit-list query forms already work (Evidence E5; `crates/rum-stdlib/src/integer_digits.rs:180`). | Emit a base-domain message for bases below 2; preserve current digit-query behavior. | S |

### Polynomials

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| Expand | WL | OK | | Structural polynomial expansion is intentionally bounded; non-polynomial-looking expressions can stay unchanged under folk scope (Evidence E9; `docs/guide/design-boundaries.md:45`). | | |
| Coefficient | WL | gap | safety | `Coefficient[x + 1/x, x, 1]` returns `1`, treating the `x^-1` term as coefficient-side residue instead of rejecting a non-polynomial in the target (Evidence E9; `crates/rum-stdlib/src/polynomials/core.rs:85`, `crates/rum-stdlib/src/polynomials/mod.rs:36`). | When a requested target is present, detect unsupported target-dependent residues and fail closed with a message. | M |
| CoefficientList | WL | gap | safety | `CoefficientList[x + 1/x, x]` returns `{x^-1, 1}`, so a coefficient still contains the target variable (Evidence E9; `crates/rum-stdlib/src/polynomials/core.rs:85`, `crates/rum-stdlib/src/polynomials/mod.rs:74`). | Apply the same target-containment guard as `Coefficient`; keep true polynomial and multivariate coefficient arrays unchanged. | M |
| Together | WL | OK | | Folk rational-function combination works for simple rational expressions; no broader CAS simplification is proposed (Evidence E9; `docs/guide/design-boundaries.md:45`). | | |
| Factor | WL | OK | | Folk structural factorization is intentionally bounded; unchanged fallback for expressions outside the handled polynomial subset is not a gap (Evidence E9; `docs/guide/design-boundaries.md:45`). | | |

### Calculus

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |
|---|---|---|---|---|---|---|
| D | WL | gap | safety | Malformed variable/order specs such as `D[x^2, 1]`, `D[x^2, {x, -1}]`, and `D[x^2, {x, a}]` return inert calls without diagnostics; supported `{x, n}` and repeated-variable forms already work, and abstract derivative heads are intended folk behavior (Evidence E9; `crates/rum-stdlib/src/calculus.rs:27`, `crates/rum-stdlib/src/calculus.rs:180`). | Emit targeted messages for invalid derivative variables and nonnegative orders; keep existing folk derivative/operator fallback unchanged. | S |

## Evidence Appendix

Commands were run with the repository bounded wrapper while feeding empty stdin
to `cargo run`.

### E1: Attribute Survey

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Attributes /@ {Identity, Plus, Times, Power, Subtract, Divide, Abs, Sign, Floor, Ceiling, IntegerPart, FractionalPart, Minus, Round, Factorial, Factorial2, Total, Mean, Median, Commonest, Quantile, Denominator, Numerator, Mod, Quotient, PowerMod, GCD, LCM, Max, Min, MinMax}, Attributes /@ {Divisors, EulerPhi, MoebiusMu, DivisorSigma, Prime, PrimePi, PrimeNu, PrimeOmega, NextPrime}, Attributes /@ {Binomial, Fibonacci, PartitionsP, PartitionsQ, BernoulliB, Subfactorial, StirlingS1, StirlingS2, CatalanNumber, Multinomial, Pochhammer}, Attributes /@ {IntegerLength, IntegerDigits, IntegerReverse, FromDigits, DigitCount}, Attributes /@ {Expand, Coefficient, CoefficientList, Together, Factor, D}}'\'''
List[List[List[], List[Listable, Orderless, Flat, OneIdentity], List[Listable, Orderless, Flat, OneIdentity], List[Listable], List[], List[], List[Listable], List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected], List[Listable], List[Listable], List[Listable], List[], List[], List[], List[], List[], List[Listable], List[Listable], List[Listable], List[Listable], List[Listable], List[Listable], List[Listable], List[], List[], List[Protected]], List[List[Listable, Protected], List[Listable, Protected], List[Listable, Protected], List[Listable, NHoldAll, Protected], List[Listable, Protected], List[Listable, Protected], List[Protected, ReadProtected], List[Protected, ReadProtected], List[Protected, ReadProtected]], List[List[Listable, NumericFunction, Protected], List[Listable, NumericFunction, Protected, ReadProtected], List[Listable, Protected], List[Listable, Protected], List[Listable, Protected], List[Listable, NumericFunction, Protected, ReadProtected], List[Listable, Protected], List[Listable, Protected], List[Listable, NumericFunction, Protected, ReadProtected], List[Listable, NumericFunction, Orderless, Protected], List[Listable, NumericFunction, Protected]], List[List[Listable, Protected], List[Listable, Protected], List[Protected, ReadProtected], List[Protected], List[Protected, ReadProtected]], List[List[], List[], List[], List[], List[], List[HoldRest]]]
```

### E2: Threading And Grouping Spot Checks

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Subtract[{3,4},1], Divide[{2,4},2], Max[{1,3},2], Min[{1,3},2], GCD[6, 9, 15], LCM[2, 3, 4], GCD[{6,9}, 3], LCM[{2,3}, 6], Max[{1,3,2}], Min[{1,3,2}], Plus[{1,2},10], Times[{1,2},10]}'\'''
List[List[2, 3], List[1, 2], 3, 1, 3, 12, List[3, 3], List[6, 6], 3, 1, List[11, 12], List[10, 20]]
```

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Max[x, 1], Max[1, x], Max[x, y], Max[y, x], Min[x, 1], Min[1, x], GCD[a, b, c], GCD[c, a, b], LCM[a, b, c], LCM[c, a, b], Attributes /@ {Plus, Times, GCD, LCM, Max, Min}}'\'''
List[Max[x, 1], Max[1, x], Max[x, y], Max[y, x], Min[x, 1], Min[1, x], GCD[a, b, c], GCD[c, a, b], LCM[a, b, c], LCM[c, a, b], List[List[Listable, Orderless, Flat, OneIdentity], List[Listable, Orderless, Flat, OneIdentity], List[Listable], List[Listable], List[], List[]]]
```

### E3: Arithmetic Domain Failures

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Mod[5, 0], Quotient[5, 0], PowerMod[2, 3, 0], PowerMod[2, -1, 5], PowerMod[2, -1, 4], Round[3/2, 0], Divide[1, 0], Power[0, 0], Factorial[-1], Factorial2[-1], Prime[0], Prime[-1], Prime[3/2], Divisors[0], DivisorSigma[1, 0], PrimeNu[0], PrimeOmega[0], NextPrime[5, 0]}'\'''
! Prime::intpp: Positive integer argument expected in Prime[0].
! Prime::intpp: Positive integer argument expected in Prime[-1].
! Prime::intpp: Positive integer argument expected in Prime[3/2].
List[Mod[5, 0], Quotient[5, 0], PowerMod[2, 3, 0], PowerMod[2, -1, 5], PowerMod[2, -1, 4], Round[3/2, 0], Divide[1, 0], Power[0, 0], Factorial[-1], Factorial2[-1], Prime[0], Prime[-1], Prime[3/2], Divisors[0], DivisorSigma[1, 0], PrimeNu[0], PrimeOmega[0], NextPrime[5, 0]]
```

### E4: Source Citations For Arithmetic Safety

- `Mod`, `Quotient`, and `PowerMod` return the original call when helper functions return `None`; zero modulus/divisor produces `None` at `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:133`, `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:142`, and `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:159`.
- `PowerMod` rejects every negative exponent by returning the original call at `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs:103`.
- `Round` rejects a zero multiple by returning the original call at `crates/rum-runtime/src/runtime/arithmetic/rounding.rs:122`.
- `Factorial` and `Factorial2` reject negative exact integers by returning the original call at `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:13` and `crates/rum-runtime/src/runtime/arithmetic/factorial.rs:49`.

### E5: Aggregation And Integer-Digit Forms

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Total[{{1, 2}, {3, 4}}, 2], Total[{{1, 2}, {3, 4}}, {2}], Total[{1, 2, 3}, 1], Commonest[{a, b, a, c, b, a}, 2], Quantile[{1, 2, 3}, {0, 1/2, 1}], MinMax[{3, 1, 2}, 2], Round[7/3, 1/2], IntegerDigits[5, 2, 4], IntegerDigits[5, 2, -1], FromDigits[{1, 0, 1}, 2], FromDigits[{2}, 2], DigitCount[12321, 10, 2], DigitCount[12321, 10, {1, 2}], DigitCount[12321, 1], IntegerLength[10, 1], IntegerReverse[10, 1]}'\'''
! Commonest::argx: expected exactly 1 argument(s), got 2
! Commonest::argx: expected exactly 1 argument(s), got 2
List[Total[List[List[1, 2], List[3, 4]], 2], Total[List[List[1, 2], List[3, 4]], List[2]], Total[List[1, 2, 3], 1], Commonest[List[a, b, a, c, b, a], 2], Quantile[List[1, 2, 3], List[0, 1/2, 1]], MinMax[List[3, 1, 2], 2], 5/2, List[0, 1, 0, 1], IntegerDigits[5, 2, -1], 5, 2, 2, List[2, 2], DigitCount[12321, 1], IntegerLength[10, 1], IntegerReverse[10, 1]]
```

### E6: Number-Theory Domain Probe

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Divisors[-6], Divisors[0], EulerPhi[-9], EulerPhi[0], MoebiusMu[-12], MoebiusMu[0], DivisorSigma[1, -6], DivisorSigma[1, 0], Prime[5], Prime[0], PrimePi[10], PrimePi[-1], PrimeNu[-12], PrimeNu[0], PrimeOmega[-12], PrimeOmega[0], NextPrime[5], NextPrime[5, -1], NextPrime[5, 0]}'\'''
! Prime::intpp: Positive integer argument expected in Prime[0].
! Prime::intpp: Positive integer argument expected in Prime[0].
List[List[1, 2, 3, 6], Divisors[0], 6, 0, 0, 0, 12, DivisorSigma[1, 0], 11, Prime[0], 4, 0, 2, PrimeNu[0], 3, PrimeOmega[0], 7, 3, NextPrime[5, 0]]
```

### E7: Combinatorics Domain Probe

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Fibonacci[], Fibonacci[1, 2], PartitionsP[-1], PartitionsQ[-1], BernoulliB[-1], Subfactorial[-1], StirlingS1[-1, 1], StirlingS2[1, -1], CatalanNumber[-1], Multinomial[], Multinomial[2, -1], Pochhammer[x, -1], Binomial[5], Binomial[5, 2, 1], Binomial[5, -1]}'\'''
! Binomial::argx: expected exactly 2 argument(s), got 1
! Binomial::argx: expected exactly 2 argument(s), got 3
! Binomial::argx: expected exactly 2 argument(s), got 1
! Binomial::argx: expected exactly 2 argument(s), got 3
List[Fibonacci[], Fibonacci[1, 2], PartitionsP[-1], PartitionsQ[-1], BernoulliB[-1], Subfactorial[-1], StirlingS1[-1, 1], StirlingS2[1, -1], CatalanNumber[-1], 1, Multinomial[-1, 2], Pochhammer[x, -1], Binomial[5], Binomial[5, 2, 1], 0]
```

### E8: Empty Aggregates Fail Closed

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Mean[{}], Median[{}], Quantile[{}, 1/2], Commonest[{}]}'\'''
! Mean::empty: Mean cannot aggregate an empty list.
! Median::empty: Median cannot aggregate an empty list.
! Quantile::empty: Quantile cannot aggregate an empty list.
! Mean::empty: Mean cannot aggregate an empty list.
! Median::empty: Median cannot aggregate an empty list.
! Quantile::empty: Quantile cannot aggregate an empty list.
List[Mean[List[]], Median[List[]], Quantile[List[], 1/2], List[]]
```

### E9: Polynomial And Calculus Probe

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Expand[(x + 1)^2], Expand[Sin[x]], Coefficient[x^2 + 3 x + 1, x, 2], Coefficient[x^2 + y, {x, y}], CoefficientList[1 + x + y + x y, x], CoefficientList[1 + x + y + x y, {x, y}], CoefficientList[Sin[x], x], Together[1/x + 1/y], Together[Sin[x]], Factor[x^3 - x], Factor[Sin[x]], D[x^3, x], D[x^3, {x, 2}], D[x^3, x, x], D[x^2 y, x, y], D[x^2, 1], D[Sin[x], x]}'\'''
List[Plus[1, Power[x, 2], Times[2, x]], Sin[x], 1, 0, List[Plus[1, y], Plus[1, y]], List[List[1, 1], List[1, 1]], List[Sin[x]], Times[Power[x, -1], Power[y, -1], Plus[x, y]], Sin[x], Times[x, Plus[-1, x], Plus[1, x]], Sin[x], Times[3, Power[x, 2]], Times[6, x], Times[6, x], Times[2, x], D[Power[x, 2], 1], Derivative[1][Sin][x]]
```

```text
$ scripts/run-bounded.py --timeout 60 -- bash -lc 'printf "" | cargo run -q -p rum-cli -- --eval '\''{Factor[x + 1/x], Factor[1/(1 + x)], Expand[1/(1 + x)], Coefficient[x + 1/x, x, 1], CoefficientList[x + 1/x, x], Together[x + 1/x], D[x^2, {x, -1}], D[x^2, {1, 2}], D[x^2, {x, a}], D[x^2, {x, 0}], D[f[x], x], D[f[x], {x, 2}]}'\'''
List[Plus[Power[x, -1], x], Power[Plus[1, x], -1], Power[Plus[1, x], -1], 1, List[Power[x, -1], 1], Times[Power[x, -1], Plus[1, Power[x, 2]]], D[Power[x, 2], List[x, -1]], D[Power[x, 2], List[1, 2]], D[Power[x, 2], List[x, a]], Power[x, 2], Derivative[1][f][x], Derivative[2][f][x]]
```

### E10: Source Citations For Parser-Level Gaps

- `Total` accepts exactly one argument and otherwise returns the original call at `crates/rum-runtime/src/runtime/arithmetic/parts.rs:4`.
- `FromDigits` accepts list digits through `expr_as_integer_digit_list` and immediately folds them at `crates/rum-stdlib/src/integer_digits.rs:144`; the list parser only checks exact-integer shape at `crates/rum-stdlib/src/integer_digits.rs:302`, while the string parser range-checks digits at `crates/rum-stdlib/src/integer_digits.rs:313`.
- `Polynomial::from_expr` turns unsupported powers into independent indeterminates at `crates/rum-stdlib/src/polynomials/core.rs:85`; `Coefficient` and `CoefficientList` consume that polynomial model at `crates/rum-stdlib/src/polynomials/mod.rs:36` and `crates/rum-stdlib/src/polynomials/mod.rs:74`.
- `D` returns the original call when `parse_derivative_spec` fails at `crates/rum-stdlib/src/calculus.rs:27`; derivative specs are parsed at `crates/rum-stdlib/src/calculus.rs:180`.

## Triage Sketch

1. **Digit and zero-arithmetic fail-closed batch**: validate `FromDigits` list digits; message on invalid bases/lengths; message on zero divisors/moduli/multiples for `Divide`, `Mod`, `Quotient`, `PowerMod`, `Round`, and `Power[0, 0]`. Pure diagnostics/domain checks; no call planning.
2. **Integer-domain diagnostics batch**: add nonnegative/zero-domain messages for `Factorial*`, number-theory zero cases, and combinatorics negative-index/count cases. Pure diagnostics around existing folk algorithms.
3. **Polynomial/calculus safety batch**: add target-containment guards for `Coefficient`/`CoefficientList` residues and targeted messages for malformed `D` specs. No new CAS algorithms.
4. **Cheap flexibility batch**: implement `Total` level forms, `Commonest[list, n]`, list-valued `Quantile`, and negative-exponent `PowerMod` via modular inverse with a non-invertible-base message.
5. **Attribute hygiene batch**: align `GCD`/`LCM`/`Max`/`Min` grouping attributes and the small numeric-function/protection omissions such as `Abs`. This touches attribute-driven planning and needs the rum-vs-rum A/B gate.

## Verification Outputs

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/arithmetic.rs
31
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/number_theory.rs
9
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/combinatorics.rs
11
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/integer_digits.rs
5
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/polynomials.rs
5
```

```text
$ grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/calculus.rs
1
```

```text
$ names=$(rg -o '"[A-Za-z][A-Za-z0-9]*"' crates/rum-stdlib/src/catalog/arithmetic.rs crates/rum-stdlib/src/catalog/number_theory.rs crates/rum-stdlib/src/catalog/combinatorics.rs crates/rum-stdlib/src/catalog/integer_digits.rs crates/rum-stdlib/src/catalog/polynomials.rs crates/rum-stdlib/src/catalog/calculus.rs | sed 's/.*"\([^"]*\)"/\1/' | sort -u); rows=$(awk -F'|' '/^\| [A-Za-z][A-Za-z0-9]* \|/ {gsub(/ /,"",$2); if ($2 != "Builtin") print $2}' docs/dev/usability-pass/math-findings.md | sort -u); missing=$(comm -23 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); extra=$(comm -13 <(printf '%s\n' "$names") <(printf '%s\n' "$rows")); count=$(printf '%s\n' "$rows" | sed '/^$/d' | wc -l | tr -d ' '); if [ -z "$missing$extra" ] && [ "$count" = "62" ]; then echo "COVERAGE-OK 62/62"; else echo "COVERAGE-FAIL $count/62"; [ -z "$missing" ] || printf 'missing:\n%s\n' "$missing"; [ -z "$extra" ] || printf 'extra:\n%s\n' "$extra"; fi
COVERAGE-OK 62/62
```

```text
$ git status --short
?? docs/dev/usability-pass/math-findings.md
```

```text
$ scripts/run-bounded.py --timeout 120 -- cargo fmt --check
```

## Stop-Loss / Open Questions

- None.
