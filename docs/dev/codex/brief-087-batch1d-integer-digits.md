# brief-087 — long-tail batch 1d: integer-digit builtins (folk, WK-calibrated)

## Goal

Implement 5 missing exact-integer builtins: `IntegerLength`, `IntegerDigits`, `IntegerReverse`,
`FromDigits`, `DigitCount`. All currently unevaluated. Exact/rationals-only, folk, WK-calibrated
([[rum-folk-implementation-scope]]). Product-code; no-regression bar includes mma-hol
byte-identical. These have base + multi-arg forms with specific conventions — calibrate each
against WK precisely (anchors below, all verified against WolframKernel).

## Behavior (WK anchors — match exactly)

Integers only; negatives use the ABSOLUTE value; non-integer/symbolic arg → symbolic
passthrough. Attributes in parentheses — calibrate + set to match.

- **`IntegerLength[n]`** (`{Listable, Protected}`): digit count. `IntegerLength[12345]`→`5`,
  `IntegerLength[0]`→`0` (special — zero has length 0), `IntegerLength[-12]`→`2` (abs).
  2-arg base: `IntegerLength[255, 16]`→`2` (FF).
- **`IntegerDigits[n]`** (`{Listable, Protected}`): digit list, most-significant first.
  `IntegerDigits[12345]`→`{1,2,3,4,5}`, `IntegerDigits[-12]`→`{1,2}` (abs).
  2-arg base: `IntegerDigits[255,16]`→`{15,15}`.
  3-arg length (left-pad with 0 / truncate to `len`): `IntegerDigits[5,2,8]`→
  `{0,0,0,0,0,1,0,1}`.
- **`IntegerReverse[n]`** (`{Listable, Protected, ReadProtected}`): reverse the digits, dropping
  leading zeros of the RESULT. `IntegerReverse[1230]`→`321`. 2-arg base:
  `IntegerReverse[100,2]`→`19` (100=1100100₂, reversed 0010011₂=19).
- **`FromDigits[list]`** (`{Protected}`): inverse of IntegerDigits. `FromDigits[{1,2,3}]`→`123`,
  2-arg base `FromDigits[{1,0,1},2]`→`5`. STRING form: `FromDigits["123"]`→`123` (parse a digit
  string; support base too: `FromDigits["ff",16]` if trivial, else note).
- **`DigitCount[n]`** (`{Protected, ReadProtected}`): 1-arg → a length-`base` vector where
  position `i` = count of digit `i` for `i=1..base-1` and the LAST position = count of digit `0`
  (base 10 → 10-vector): `DigitCount[121]`→`{2,1,0,0,0,0,0,0,0,0}`. 2-arg base:
  `DigitCount[255,16]`→ the 16-vector `{0,...,0,2,0}`. 3-arg specific digit: `DigitCount[121,10,1]`
  →`2` (how many `1` digits). Calibrate the vector convention EXACTLY against WK.

If any multi-arg form is genuinely too fiddly, ship the common forms + note what you skipped —
but the anchors above should all be reachable.

## Where / how

Register in `builtin_catalog/` + `metadata.rs` + dispatch like the recent batches; these are
exact-integer ops (BigInt) — put handlers near the other integer/number builtins (e.g. where
`IntegerExponent`/`Mod`/`Quotient` live, or a new `integer_digits` module). Reuse BigInt
radix conversion; no float paths.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests per function: the exact anchors above + a base form + a negative +
Listable threading (`IntegerLength[{12,345}]`) + symbolic passthrough + the `IntegerLength[0]`
and 3-arg-padding edges + the `DigitCount` vector convention + `FromDigits` string form. Spot-
check against WK via the documented oracle resolution (`RUM_DIFFTEST_ORACLE_KERNEL`/
`wolframscript`, not a hardcoded path); save raw to a file, parse.

## Hard rules

- Product code ONLY for these 5 builtins (+ tests). Exact-integer only; abs for negatives;
  symbolic passthrough for non-integer. No `Expr`/serde change. No new `unsafe`.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes + the vector/padding conventions.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 5 builtins (files), any multi-arg form skipped (with why).
2. WK calibration — attributes + the anchors (esp. DigitCount vector convention, 3-arg padding).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match, with analysis.

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. 5 integer-digit builtins, WK-conformant.

Committed. `IntegerLength`, `IntegerDigits`, `IntegerReverse`, `FromDigits`, `DigitCount`
(new `integer_digits.rs` + builtin_catalog/numeric.rs + metadata + dispatch + mod.rs). Exact
BigInt, abs for negatives, base/multi-arg forms, symbolic passthrough. Gate (Claude reproduced):
all anchors match WK — IntegerLength[0]=0/[255,16]=2, IntegerDigits[5,2,8]={0,0,0,0,0,1,0,1}/
[-12]={1,2}, IntegerReverse[1230]=321/[100,2]=19, FromDigits[{1,0,1},2]=5/["123"]=123,
DigitCount[121]={2,1,0,...}/[121,10,1]=2. `cargo test -p rum-runtime` 754/0/2; `--workspace`
no failures; `mma_hol_cold_lane` 4/0 byte-identical; fmt clean.

Attribute discrepancy settled in Codex's favor: WK `Attributes[IntegerReverse]` =
`{Protected, ReadProtected}` (NO Listable — the brief was wrong; I had misread the earlier
nested-list probe). WK's IntegerReverse threads over lists anyway (built-in, not via the
attribute) — `IntegerReverse[{12,340}]`={21,43} in both. Good WK-over-brief calibration.

Long-tail batch 1 DONE: 1a(7) + 1b(6) + 1c(4) + 1d(5) = 22 new builtins. Next: batch-2
(list plumbing: Differences/PadLeft/PadRight/ConstantArray) + moderate (Sum/Product, set ops).
