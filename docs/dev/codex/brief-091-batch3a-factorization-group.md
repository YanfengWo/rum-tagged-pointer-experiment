# brief-091 — long-tail batch 3a: number theory via integer factorization (Pollard's rho)

## Goal

(A) Add `num-integer` + `num-traits` as DIRECT deps (they are already transitive via
`num-bigint` — FREE, no new crates), so BigInt gets `.gcd()`/`.lcm()`/`.sqrt()`/`.nth_root()`/
`.extended_gcd()` (+ `modpow` is already inherent on `num-bigint`). (B) Implement 7 number-theory
builtins that all derive from the PRIME FACTORIZATION of an integer: `Divisors`, `EulerPhi`,
`MoebiusMu`, `DivisorSigma`, `PrimeNu`, `PrimeOmega`, `NextPrime`. Folk, WK-calibrated
([[rum-folk-implementation-scope]]). Product-code; no-regression bar includes mma-hol
byte-identical.

## Part A — dependencies (free)

Add to `[workspace.dependencies]` in the root `Cargo.toml`: `num-integer = "0.1"`,
`num-traits = "0.2"` (versions already pinned in Cargo.lock: 0.1.46 / 0.2.19). Reference them
(`.workspace = true`) in `crates/rum-runtime/Cargo.toml` (and rum-core if a helper lands there).
Bring the traits into scope where needed (`use num_integer::{Integer, Roots};`
`use num_traits::{Zero, One, Signed};`). Confirm the lock is unchanged (no new crates).

## Part B — the shared factorization helper (exact, elegant/fast)

One internal helper `factorize(n: &BigInt) -> Vec<(BigInt, u32)>` (prime, exponent), on
`|n|`. Method (all EXACT-in-result):
1. small trial division (primes up to ~10^5-10^6) to peel small factors;
2. for a remaining composite cofactor, **Pollard's rho** (Brent's variant is fine) using
   `.gcd()` + `modpow`/mul-mod to split it;
3. a **deterministic Miller-Rabin** primality test (via `modpow`) to decide when a factor is
   prime: the witness set `{2,3,5,7,11,13,17,19,23,29,31,37}` is PROVEN deterministic for
   n < 3.317×10^24 — exact for all realistic folk inputs; note that beyond that bound it is
   heuristic (acceptable, or guard with the integer-work-limit).
Put an integer-work-limit guard on the factorization so a pathological input terminates
(`TerminatedEvaluation`/limit) rather than hanging. Reuse `.gcd()`/`.sqrt()`/`modpow` — do NOT
hand-roll them.

## Part C — the 7 builtins (WK anchors — match exactly)

Non-integer / symbolic arg → symbolic passthrough. Integers use `|n|` where noted.

- **`Divisors[n]`** (`{Listable, Protected}`): all positive divisors, sorted, on `|n|`.
  `Divisors[12]`→`{1,2,3,4,6,12}`, `Divisors[1]`→`{1}`, `Divisors[7]`→`{1,7}`,
  `Divisors[-12]`→`{1,2,3,4,6,12}`.
- **`EulerPhi[n]`** (`{Listable, Protected}`): `n·∏(1-1/p)`. `EulerPhi[12]`→`4`, `[1]`→`1`,
  `[7]`→`6`.
- **`MoebiusMu[n]`** (`{Listable, Protected}`): `0` if not squarefree, else `(-1)^ω`.
  `MoebiusMu[1]`→`1`, `[12]`→`0`, `[30]`→`-1`, `[6]`→`1`.
- **`DivisorSigma[k, n]`** (`{Listable, NHoldAll, Protected}`): `∏ (p^((e+1)k)-1)/(p^k-1)`;
  `k=0` → number of divisors. `DivisorSigma[1,12]`→`28`, `DivisorSigma[0,12]`→`6`,
  `DivisorSigma[2,10]`→`130`.
- **`PrimeNu[n]`** (`{Listable, Protected, ReadProtected}`): # DISTINCT primes.
  `PrimeNu[12]`→`2`, `PrimeNu[1]`→`0`.
- **`PrimeOmega[n]`** (`{Listable, Protected, ReadProtected}`): # primes WITH multiplicity.
  `PrimeOmega[12]`→`3`, `PrimeOmega[1]`→`0`.
- **`NextPrime[n]`** / **`NextPrime[n,k]`** (`{Protected, ReadProtected}`): next prime > n;
  k-th next; negative k = previous prime(s). `NextPrime[10]`→`11`, `NextPrime[10,3]`→`17`,
  `NextPrime[-1]`→`2`. WK's negative-prime convention (`NextPrime[2,-1]`→`-2`) is quirky —
  match it if straightforward, else implement k≥1 + note the negative-k/negative-prime gap.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests per function (the anchors + a prime + a prime-power + squarefree/not +
symbolic passthrough) + a LARGE-input factorization test that exercises Pollard's rho (e.g. a
product of two ~10-digit primes: `PrimeNu`/`Divisors` on it, confirm vs WK) + confirm the deps
are direct and Cargo.lock unchanged. Spot-check against WK via the documented oracle resolution
(`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not hardcoded); save raw to a file, parse.

## Hard rules

- Product code for: the 2 Cargo.toml dep additions + the factorization helper + these 7
  builtins + tests. Use `.gcd()`/`.sqrt()`/`.extended_gcd()`/`modpow` — do NOT hand-roll them
  (and do NOT need to touch the existing hand-rolled `bigint_gcd`/`bigint_abs` — a separate
  cleanup will de-dupe those). No `Expr`/serde change. No new `unsafe`.
- Exact results (Pollard rho + deterministic MR below the proven bound); work-limit guard for
  pathological inputs. Symbolic/non-integer passthrough.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these). Do NOT touch `PrimeQ` (its trial-division stays; this batch's MR
  is internal to factorization).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes.
- Paste REAL verbatim output; never fake a green.

## Report format

1. Part A — the dep additions; confirm Cargo.lock unchanged (no new crates).
2. Part B — the factorization helper (trial + Pollard rho + det-MR); the MR bound + work-limit.
3. Part C — the 7 builtins (files) + attributes + anchors matched; NextPrime negative-k status.
4. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical +
   the large-factorization test, verbatim.
5. Any WK divergence you couldn't match, with analysis.

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. Factorization group (Pollard's rho) + free deps, WK-conformant.

Committed. (A) num-integer + num-traits are now DIRECT deps of rum-runtime — Cargo.lock diff is
ONLY the two dependency edges, NO new [[package]] (free, as predicted). (B) `number_theory.rs`:
trial division to 100k + Brent's Pollard rho + deterministic Miller-Rabin ({2..37} witnesses,
exact < 3.317e24), all under the integer-work-limit guard. (C) 7 builtins: Divisors/EulerPhi/
MoebiusMu/DivisorSigma(NHoldAll)/PrimeNu/PrimeOmega/NextPrime.

Gate (Claude reproduced): anchors match WK (Divisors[12]/[-12], EulerPhi[12]=4, MoebiusMu
[30]=-1/[12]=0, DivisorSigma[1,12]=28/[0,12]=6, PrimeNu/PrimeOmega[12]=2/3, NextPrime[10,3]=17/
[2,-1]=-2, PrimeNu[{12,30}]={2,3} threading). Pollard's rho earns its keep: factors up to ~1e9
(9-digit-factor 1e18 semiprimes work: PrimeOmega[999999937*999999929]=2); 10-digit-factor 1e20
semiprimes → TerminatedEvaluation["IntegerWorkLimit"] (graceful, raisable, folk-acceptable).
`cargo test -p rum-runtime` 778/0/2; `--workspace` no failures; `mma_hol_cold_lane` 4/0
byte-identical; fmt clean. PrimeQ untouched. Attribute settled vs live WK: PrimeNu/PrimeOmega=
{Protected,ReadProtected} (no Listable — Codex matched WK + manual threading; the brief anchor
was wrong again — I keep misreading the nested-list probe).

Long-tail progress: 34 (b1/2/2b/2c) + 7 = 41 new builtins. Remaining folk number-theory:
recurrence group (PartitionsP/Q, BernoulliB, Subfactorial, StirlingS1/S2) + trivial closed forms
(CatalanNumber/Multinomial/Pochhammer) + Prime/PrimePi (small table). Doc note for the matrix:
factorization-derived fns handle factors to ~1e9 at the default work-limit (raisable).
