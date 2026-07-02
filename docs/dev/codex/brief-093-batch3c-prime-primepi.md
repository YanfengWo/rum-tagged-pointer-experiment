# brief-093 — long-tail batch 3c: Prime / PrimePi (folk sieve, WK-calibrated)

## Goal

Implement the last two folk number-theory builtins: `Prime` (n-th prime) and `PrimePi`
(prime-counting π). Currently unevaluated. No elegant closed form (the advanced ones —
Meissel-Lehmer — are heavy, out of scope like general `Factor`); folk = an on-demand sieve of
Eratosthenes bounded by the integer-work-limit (reuse batch-3a's primality/sieve helpers in
`number_theory.rs`). Exact, WK-calibrated ([[rum-folk-implementation-scope]]). Product-code;
no-regression bar includes mma-hol byte-identical. This completes the number-theory line.

## Behavior (WK anchors — match exactly)

- **`Prime[n]`** (`{Listable, Protected}`) — the n-th prime for a POSITIVE integer n:
  `Prime[1]`→`2`, `Prime[10]`→`29`, `Prime[100]`→`541`, `Prime[1000]`→`7919`,
  `Prime[10000]`→`104729`. For n ≤ 0 or non-integer: emit WK's message
  `Prime::intpp` ("Positive integer argument expected in Prime[..].") if feasible, and return
  UNEVALUATED (`Prime[0]` stays `Prime[0]`); symbolic arg → unevaluated.
  Implementation: sieve up to an upper bound for the n-th prime (for n ≥ 6,
  `p_n < n·(ln n + ln ln n)`; small n from a tiny hardcoded list), take the n-th; or a
  segmented sieve. Guard with the integer-work-limit (large n → `TerminatedEvaluation
  ["IntegerWorkLimit"]`, raisable).
- **`PrimePi[x]`** (`{Listable, Protected}`) — number of primes ≤ x, for integer OR RATIONAL
  x (rum is rationals-only): count primes ≤ `Floor[x]`. `PrimePi[10]`→`4`, `PrimePi[100]`→`25`,
  `PrimePi[1000]`→`168`, `PrimePi[2]`→`1`, `PrimePi[1]`→`0`, `PrimePi[13/2]`→`3` (primes ≤ 6:
  {2,3,5}). Sieve to `Floor[x]` + count; work-limit guarded. Symbolic arg → unevaluated.

## Where / how

Add to `number_theory.rs` (batch-3a) — reuse its sieve / primality machinery + the
integer-work-limit budget pattern. Register in the catalog + metadata + dispatch like the
recent batches. No float paths (x is exact Integer/Rational; use Floor for PrimePi). Do NOT
embed a large static prime table (an on-demand bounded sieve is simpler and doesn't bloat the
binary).

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests: the Prime anchors (incl. `Prime[10000]=104729` to exercise the
sieve), the PrimePi anchors incl. the RATIONAL `PrimePi[13/2]=3`, `Prime[0]` unevaluated
(+ message if implemented), Listable threading, symbolic passthrough, and a work-limit case.
Spot-check against WK via the documented oracle resolution
(`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not hardcoded); save raw to a file, parse.

## Hard rules

- Product code ONLY for these 2 builtins (+ tests). Reuse batch-3a sieve/primality; no static
  table. No `Expr`/serde change. No new `unsafe`. Work-limit guard (large inputs → graceful
  TerminatedEvaluation).
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these). Do NOT touch `PrimeQ`.
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes (single-probe: both are `{Listable, Protected}`).
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — Prime + PrimePi (files); the sieve bound/approach; the work-limit behavior.
2. WK calibration — the anchors matched (incl. Prime[10000], PrimePi[13/2] rational);
   attributes single-probed.
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match (e.g. the Prime::intpp message), with analysis.

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. Prime/PrimePi (folk sieve); number-theory line COMPLETE.

Committed (number_theory.rs + catalog + metadata + dispatch + tests). Gate (Claude reproduced):
Prime[1]=2/[100]=541/[1000]=7919/[10000]=104729 (on-demand sieve), PrimePi[10]=4/[100]=25/
[1000]=168/[13/2]=3 (rational floor)/[1]=0, Prime[0]→unevaluated, Prime[{5,6}]={11,13}
(Listable), symbolic passthrough. `cargo test -p rum-runtime` 781/0/2; `--workspace` no
failures; `mma_hol_cold_lane` 4/0 byte-identical; fmt clean. Both {Listable,Protected}
(single-probed). Work-limit guarded; no static table. PrimeQ untouched.

Long-tail line COMPLETE: 50 + 2 = **52 new builtins** across b1(scalar/string/introspection/
integer-digits) + b2(list-plumbing/set-ops) + b2c(iterators/Binomial/Fibonacci) + b3a(Pollard
factorization group) + b3b(recurrence combinatorics) + b3c(Prime/PrimePi). Next (pre-ship
finish): the consolidated support-matrix accuracy pass — record all 52 + Factor linear-only +
factorization work-limit + the partial limits.
