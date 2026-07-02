# brief-084 — long-tail batch 1a: exact scalar-math builtins (folk, WK-calibrated)

## Goal

Implement 7 missing exact scalar-math builtins from the long-tail audit's first (trivial ×
high-commonness) batch: `Sign`, `Floor`, `Ceiling`, `IntegerPart`, `FractionalPart`, `Minus`,
`MinMax`. rum currently returns each UNEVALUATED. Folk/good-enough, RATIONALS-ONLY, exact —
symbolic passthrough for non-numeric args, WK-calibrated on every case
([[rum-folk-implementation-scope]] policy). This IS a product-code change (new builtins);
the no-regression bar includes mma-hol byte-identical.

## Behavior (calibrate every case against WolframKernel)

All operate on rum's exact numbers (Integer, Rational); NO Real type exists. For a
non-numeric argument, return the symbolic form unevaluated (e.g. `Sign[x]` stays `Sign[x]`).

- **`Sign[x]`**: exact number → `-1` | `0` | `1` (`Sign[-3/2]`→`-1`, `Sign[0]`→`0`,
  `Sign[3]`→`1`). Symbolic otherwise.
- **`Floor[x]`**: greatest integer ≤ x, toward −∞ (`Floor[7/2]`→`3`, `Floor[-7/2]`→`-4`,
  `Floor[3]`→`3`).
- **`Ceiling[x]`**: least integer ≥ x, toward +∞ (`Ceiling[7/2]`→`4`, `Ceiling[-7/2]`→`-3`).
- **`IntegerPart[x]`**: truncate toward zero (`IntegerPart[7/2]`→`3`, `IntegerPart[-7/2]`→`-3`).
- **`FractionalPart[x]`**: `x - IntegerPart[x]` (`FractionalPart[7/2]`→`1/2`,
  `FractionalPart[-7/2]`→`-1/2`; sign follows x).
- **`Minus[x]`**: unary negation → `-x` i.e. `Times[-1, x]` (`Minus[3]`→`-3`, `Minus[x]`→`-x`).
- **`MinMax[list]`**: → `{Min[list], Max[list]}` (`MinMax[{3,1,2}]`→`{1,3}`). Empty/degenerate:
  match WK.

Do NOT implement the 2-argument "round to a multiple" forms (`Floor[x,a]`, `Round[x,a]`,
`MinMax[list,d]`) unless trivial — folk scope is the common 1-arg form; if you skip them, note
it. `Round` already exists (banker's rounding, matches WK) — do not touch it.

**Attributes:** calibrate each against WK (`Attributes[Sign]` etc.) and set them to MATCH
(e.g. `Sign`/`Floor`/`Ceiling`/`IntegerPart`/`FractionalPart` are `Listable`+`NumericFunction`
+`Protected` in WK, so `Floor[{7/2, -1/2}]` threads; verify and replicate). `Minus`/`MinMax`:
use WK's actual attributes. Getting `Listable` right is part of the spec (test the threading).

## Where / how

- Register in `crates/rum-runtime/src/runtime/builtin_catalog/` (see how `Abs`/`Round`/`Min`/
  `Max` are registered — mirror their pattern, module + `BuiltinHandler` variant + handler).
  Handlers likely near the existing numeric predicates/reducers
  (`crates/rum-runtime/src/runtime/predicates/numeric.rs` or the arithmetic eval modules —
  follow where `Abs`/`Round` live).
- Reuse existing exact-rational helpers (BigInt/Rational) rather than new float paths.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add rum-runtime unit tests (WK-calibrated) for each function: the exact cases above +
Listable threading + symbolic passthrough + a negative/zero/edge case each. Also spot-check a
few directly against WK via the documented oracle resolution (`RUM_DIFFTEST_ORACLE_KERNEL` /
`wolframscript`, NOT a hardcoded path); save raw WK output to a file, then parse (exact
rationals).

## Hard rules

- Product code allowed ONLY for these 7 builtins (catalog + handlers + attributes) + their
  tests. Do NOT touch `Round`/`Abs`/`Min`/`Max`. Do NOT change `Expr`/serde. No new `unsafe`.
- RATIONALS-ONLY: no Real/float paths. Symbolic passthrough for non-numeric args.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (a separate
  consolidated matrix pass will record these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 7 builtins (catalog entries + handlers + attributes), files touched;
   any 2-arg form skipped (with why).
2. WK calibration — the attribute sets + a few in→out examples confirmed against WK.
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Anything that diverged from WK you couldn't match (should be none for these).

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. 7 exact scalar-math builtins, WK-conformant.

Committed. Implemented `Sign`, `Floor`, `Ceiling`, `IntegerPart`, `FractionalPart`, `Minus`,
`MinMax` (arithmetic.rs handlers + builtin_catalog/numeric.rs registration + metadata.rs
attributes/variants + eval_dispatch wiring). Folk/rationals-only exact, symbolic passthrough,
WK-calibrated attributes (Listable threading verified). Gate (Claude reproduced): behavioral
spot-check all match WK (Sign[-3/2]=-1, Floor[-7/2]=-4, Ceiling[-7/2]=-3, IntegerPart[-7/2]=-3,
FractionalPart[-7/2]=-1/2, Minus[x]=-x, MinMax[{3,1,2}]={1,3}, Floor[{7/2,-1/2}]={3,-1},
Sign[x] symbolic); `cargo test -p rum-runtime` 748/0/2 (+3 test fns); `cargo test --workspace`
no failures; `mma_hol_cold_lane` 4/0 byte-identical; fmt clean. No WK divergence in the 1-arg
scope; 2-arg round-to-multiple forms intentionally skipped (folk scope). Round/Abs/Min/Max
untouched. Matrix update deferred to the consolidated accuracy pass.

Next: batch-1b (string helpers: StringReverse/ToLowerCase/LowerCaseQ/UpperCaseQ/Capitalize/
Decapitalize).
