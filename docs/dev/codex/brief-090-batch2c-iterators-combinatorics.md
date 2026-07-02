# brief-090 — long-tail batch 2c: iterators (Sum/Product) + cheap combinatorics (Binomial/Fibonacci)

## Goal

Implement 4 builtins: `Sum`, `Product` (numeric iterators), `Binomial`, `Fibonacci`. All
currently unevaluated. Folk, WK-calibrated ([[rum-folk-implementation-scope]]). Product-code;
no-regression bar includes mma-hol byte-identical. Anchors verified against WolframKernel.

## Behavior (WK anchors — match exactly)

### Sum / Product — `{HoldAll, Protected, ReadProtected}` (HoldAll is essential)
NUMERIC iteration only, over concrete bounds. **REUSE the existing iterator machinery that
`Table`/`Do` already use** (same iterator-spec parsing + variable localization) — `Sum` is
`Total`/`Plus @@` over the iterated values, `Product` is `Times @@`. Cover the iterator forms
`Table`/`Do` support:
- single `Sum[i,{i,1,10}]`→`55`; `Product[i,{i,1,5}]`→`120`;
- step `Sum[i,{i,0,10,2}]`→`30`; `Product[k,{k,2,6,2}]`→`48`;
- multi-iterator `Sum[i j,{i,1,2},{j,1,3}]`→`18`;
- list iterator `Sum[i,{i,{2,3,5}}]`→`10`.
An empty sum → `0`, empty product → `1` (match WK).
**OUT OF FOLK SCOPE — do NOT attempt:** symbolic/closed-form summation. WK gives
`Sum[i,{i,1,n}]`→`(n(1+n))/2`; rum should leave a SYMBOLIC-bound Sum/Product UNEVALUATED
(return it as-is). No Gosper / closed-form. (Note this divergence in the report.)

### Binomial[n,k] — `{Listable, NumericFunction, Protected, ReadProtected}`
Exact for integer args: `Binomial[5,2]`→`10`, `Binomial[4,6]`→`0` (k>n≥0 → 0), `Binomial[5,0]`
→`1`. Match WK's integer edge/negative conventions (calibrate `Binomial[-1,2]` etc.). Symbolic
args → passthrough (do NOT compute the polynomial `Binomial[n,2]=n(n-1)/2` — folk).

### Fibonacci[n] — `{Listable, NumericFunction, Protected, ReadProtected}`
Exact. **Use the FAST DOUBLING algorithm** (director's decision — NOT naive iteration/
recursion, NOT Binet which needs reals):
`F(2k) = F(k)·(2·F(k+1) − F(k))`, `F(2k+1) = F(k)² + F(k+1)²`, walking n's bits →
O(log n) BigInt multiplications (handles large n instantly).
`Fibonacci[10]`→`55`, `Fibonacci[0]`→`0`, `Fibonacci[1]`→`1`. Negative index (negafibonacci):
`F(-n) = (-1)^(n+1)·F(n)`, so `Fibonacci[-6]`→`-8`. Symbolic arg → passthrough.
Add a test at a large index (e.g. `Fibonacci[100]`=`354224848179261915075`) to exercise the
doubling path.

## Where / how

- `Sum`/`Product`: register with HoldAll; find where `Table`/`Do` parse iterator specs +
  localize the variable and REUSE that (do not re-derive iteration). Put handlers near the
  iteration builtins.
- `Binomial`/`Fibonacci`: exact-integer (BigInt) — near the other integer/number builtins
  (batch-1d `integer_digits.rs` neighborhood or the numeric handlers). Fast-doubling Fibonacci
  is a compact (~15-line) BigInt function.
- No float paths anywhere.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests: Sum/Product single+step+multi+list+empty; symbolic-bound Sum stays
unevaluated; Binomial integer + k>n=0 + edge/negative; Fibonacci 10/0/1 + negative + a LARGE
index (Fibonacci[100]) + Listable threading; symbolic passthrough for all. Spot-check against WK
via the documented oracle resolution (`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not a
hardcoded path); save raw to a file, parse.

## Hard rules

- Product code ONLY for these 4 builtins (+ tests). Reuse Table/Do iteration for Sum/Product
  (don't fork iteration logic). Fibonacci MUST be fast doubling. No `Expr`/serde change. No new
  `unsafe`.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes. Symbolic-bound Sum/Product + symbolic Binomial/Fibonacci → UNEVALUATED (folk).
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 4 builtins (files); confirm Sum/Product reuse Table/Do iteration and
   Fibonacci uses fast doubling.
2. WK calibration — attributes + anchors (Sum forms, Binomial edges, Fibonacci large + negative).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests (incl. the large-Fibonacci one).
4. Divergences — the symbolic-summation gap (expected) + anything else.
