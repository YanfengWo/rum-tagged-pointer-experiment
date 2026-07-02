# brief-094 — support-matrix accuracy pass (record the 52 new builtins + folk limits)

## Goal

Reconcile `docs/core-language-support-matrix.md` with the ACTUAL current implementation so
the v0.1 contract is accurate. The long-tail line (briefs 084-093) added 52 builtins that
were previously `planned`/missing/unlisted; `Factor` is implemented but was never listed;
several folk limits need recording. DOC-ONLY: NO product-code change, no `git commit`. Verify
every claim against the running kernel (`target/debug/rum-cli --eval`) + WK where a limit is
behavior-specific (documented oracle resolution, not a hardcoded path).

## What changed since the matrix was last accurate

### The 52 newly-implemented builtins (move to `implemented`, or add if unlisted)
- **scalar math (084):** `Sign` `Floor` `Ceiling` `IntegerPart` `FractionalPart` `Minus` `MinMax`
- **strings (085):** `StringReverse` `ToLowerCase` `LowerCaseQ` `UpperCaseQ` `Capitalize` `Decapitalize`
- **introspection (086):** `SymbolName` `NameQ` `LeafCount` `HoldForm`
- **integer digits (087):** `IntegerLength` `IntegerDigits` `IntegerReverse` `FromDigits` `DigitCount`
- **list plumbing (088):** `Differences` `PadLeft` `PadRight` `ConstantArray`
- **set/combinatorial ops (089):** `Intersection` `Subsets` `Tuples` `Partition`
- **iterators + combinatorics (090):** `Sum` `Product` `Binomial` `Fibonacci`
- **factorization group (091):** `Divisors` `EulerPhi` `MoebiusMu` `DivisorSigma` `PrimeNu` `PrimeOmega` `NextPrime`
- **recurrence combinatorics (092):** `PartitionsP` `PartitionsQ` `BernoulliB` `Subfactorial` `StirlingS1` `StirlingS2` `CatalanNumber` `Multinomial` `Pochhammer`
- **prime (093):** `Prime` `PrimePi`

Place each in the CL section matching its neighbors (put them where similar existing
functions live; the number-theory/combinatorics functions go with the arithmetic/foundation
material). Every one is verifiable: `rum-cli --eval` it before listing it as `implemented`.

### Folk limits to record (as `partial` notes, not silent `implemented`)
- **`Factor`**: implemented but STRUCTURAL/linear only — it extracts LINEAR factors (rational
  roots); it does NOT find quadratic-or-higher irreducible factors (`x^4+4y^4`, `x^4+4` stay
  unfactored). General polynomial factorization (Berlekamp–Zassenhaus) is a documented
  NON-GOAL. Add a `Factor` entry with this exact criterion (it was previously unlisted).
- **Factorization-derived** (`Divisors`/`EulerPhi`/`MoebiusMu`/`DivisorSigma`/`PrimeNu`/
  `PrimeOmega`/`NextPrime`): exact via trial + Pollard's rho + deterministic Miller-Rabin;
  handle factors up to ~1e9 at the default integer-work-limit; larger →
  `TerminatedEvaluation["IntegerWorkLimit"]` (raisable). Note the work-limit.
- **`Sum`/`Product`**: NUMERIC iterators only (concrete bounds, reuse Table/Do forms); a
  SYMBOLIC-bound `Sum[i,{i,1,n}]` stays UNEVALUATED (no closed-form/Gosper — folk).
- **`Binomial`/`Fibonacci`/`Subfactorial`/`Catalan`/`Stirling`/`Bernoulli`/`PartitionsP`…**:
  exact for integer args; SYMBOLIC args → passthrough (except `Pochhammer`, which computes the
  finite product for symbolic first arg). `BernoulliB` is exact RATIONAL.
- **`Prime`/`PrimePi`**: on-demand sieve, work-limited (large n → TerminatedEvaluation);
  `PrimePi` accepts rational x (counts ≤ Floor[x]).
- **`PrimeQ`** (pre-existing): trial division — exact but O(√n), slow for very large n
  (record as a note if not already).
- Reference the folk-implementation policy once (a short "implementation-fidelity" note):
  simple/exact-on-common + message/fallback on heavy cases; rationals-only.

## Method

1. Read the matrix's structure (CL-01..CL-09 + the status legend + any math/arith material).
2. For EACH CL section: verify its existing `implemented`/`partial`/`planned` entries still
   match reality (spot-probe with rum-cli); MOVE anything now-implemented out of
   `planned`/`partial`; ADD the 52 (+ Factor) to the right section's `implemented`/`partial`.
3. Record the folk limits above as `partial` notes with the exact criteria.
4. Keep the matrix's existing tone/format; update the "Current Roadmap State" summary if it
   references counts/lanes that moved.
5. Do NOT overclaim: if a function is `implemented` but has a folk limit, mark it `partial`
   with the limit; only list `implemented` for the fully-WK-matching ones.

## Verification

- No product-code change: `git status` shows only `docs/core-language-support-matrix.md` (+
  this brief's Outcome). `git diff` touches no `crates/`.
- Every function newly listed as `implemented`/`partial` was probed with `rum-cli --eval`
  (paste a handful of representative probes confirming presence + the claimed behavior/limit).
- `cargo fmt --check` (should be a no-op; no code changed) — optional.

## Hard rules

- DOC-ONLY. No product-code change. No `git commit`. Never edit any `mma-hol` file.
- Verify against the running kernel before listing anything `implemented`; verify limits
  against WK where behavior-specific (documented oracle resolution).
- Be accurate, not aspirational: `partial` with a documented limit beats a false `implemented`.

## Report format

1. What changed — which CL sections got the 52 (+ Factor) additions; what MOVED from
   planned/partial → implemented; the folk-limit notes added.
2. Verification — the rum-cli probes confirming presence/limits; `git status` proving
   doc-only.
3. Any entry you were unsure how to classify (implemented vs partial) — list for the director.
