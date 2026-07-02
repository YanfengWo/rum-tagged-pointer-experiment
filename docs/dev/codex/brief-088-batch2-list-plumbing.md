# brief-088 — long-tail batch 2: list plumbing (Differences/PadLeft/PadRight/ConstantArray)

## Goal

Implement 4 missing list-shape builtins: `Differences`, `PadLeft`, `PadRight`, `ConstantArray`.
All currently unevaluated. Folk, WK-calibrated ([[rum-folk-implementation-scope]]).
Product-code; no-regression bar includes mma-hol byte-identical. Anchors below verified against
WolframKernel.

## Behavior (WK anchors — match exactly)

Non-list / bad args → symbolic passthrough. Attributes in parentheses; calibrate + match.

- **`Differences[list]`** (`{NonThreadable, Protected}`): successive differences.
  `Differences[{1,4,9,16}]`→`{3,5,7}`. 2-arg order: `Differences[{1,4,9,16},2]`→`{2,2}`
  (n-th order). (WK also has a 3-arg `{n,...}` step form — implement 1-arg + order-n; note if
  you skip the fancier step form.)
- **`ConstantArray[c, n]`** (`{Protected}`): `ConstantArray[0,3]`→`{0,0,0}`. Multi-dim list of
  dims: `ConstantArray[x,{2,2}]`→`{{x,x},{x,x}}`. `c` is held structurally (not evaluated per
  cell — it's just replicated).
- **`PadLeft[list, n]`** (`{Protected}`): pad on the LEFT to length `n` with `0`:
  `PadLeft[{1,2},5]`→`{0,0,0,1,2}`. 3-arg pad element: `PadLeft[{1,2},5,x]`→`{x,x,x,1,2}`.
  If the list is LONGER than `n`, TRUNCATE keeping the LAST `n`: `PadLeft[{1,2,3,4,5},3]`→
  `{3,4,5}`. Cyclic pad from a list (3-arg with a list padding): `PadLeft[{1,2},5,{7,8}]`→
  `{8,7,8,1,2}` (the padding cycles, right-aligned to the original). Calibrate the cyclic + the
  truncation exactly against WK.
- **`PadRight[list, n]`** (`{Protected}`): symmetric on the RIGHT: `PadRight[{1,2},5]`→
  `{1,2,0,0,0}`, `PadRight[{1,2},5,x]`→`{1,2,x,x,x}`, truncation keeps the FIRST `n`, cyclic
  pad left-aligned. Calibrate against WK.

Folk scope: the 1-arg / length / pad-element / truncation / basic cyclic forms above +
multi-dim ConstantArray. The fancier PadLeft/PadRight margin/negative-`n`/nested-depth forms and
Differences' step-list form may be skipped — NOTE what you skip.

## Where / how

Register in `builtin_catalog/` + `metadata.rs` + dispatch like the recent batches; these are
list-structure ops — put handlers near the other list/structure builtins (`structure_ops/` /
where `Take`/`Drop`/`Partition`-family live, or a new module). No float paths.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests per function: the exact anchors + a pad-element + a truncation + a
cyclic-pad case + multi-dim ConstantArray + Differences order-2 + symbolic passthrough. Spot-
check against WK via the documented oracle resolution (`RUM_DIFFTEST_ORACLE_KERNEL`/
`wolframscript`, not a hardcoded path); save raw to a file, parse.

## Hard rules

- Product code ONLY for these 4 builtins (+ tests). No `Expr`/serde change. No new `unsafe`.
  Don't touch unrelated builtins.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes + the pad truncation/cyclic conventions.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 4 builtins (files), any fancier form skipped (with why).
2. WK calibration — attributes + the anchors (esp. PadLeft/PadRight truncation + cyclic).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match, with analysis.
