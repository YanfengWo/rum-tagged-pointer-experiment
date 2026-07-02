# brief-089 — long-tail batch 2b: set / combinatorial list ops (folk, WK-calibrated)

## Goal

Implement 4 missing list ops: `Intersection`, `Subsets`, `Tuples`, `Partition`. (`Union` and
`Complement` ALREADY exist — do NOT touch them; mirror `Union` for `Intersection`.) Folk,
WK-calibrated ([[rum-folk-implementation-scope]]). Product-code; no-regression bar includes
mma-hol byte-identical. Anchors below verified against WolframKernel.

## Behavior (WK anchors — match exactly)

Non-list / bad args → symbolic passthrough. Attributes in parentheses; calibrate + match.

- **`Intersection[a, b, ...]`** (`{Flat, OneIdentity, Protected, ReadProtected}` — SAME as
  `Union`; match it): sorted + deduplicated common elements.
  `Intersection[{1,2,3},{2,3,4}]`→`{2,3}`; n-arg
  `Intersection[{1,2,3,4},{2,3,4},{3,4,5}]`→`{3,4}`; 1-arg = sort+dedup of the single list.
- **`Subsets[list]`** (`{Protected}`): all subsets, ordered by size then lexicographic:
  `Subsets[{1,2,3}]`→`{{},{1},{2},{3},{1,2},{1,3},{2,3},{1,2,3}}`.
  `Subsets[{1,2,3},2]`→ size ≤ 2 (`{{},{1},{2},{3},{1,2},{1,3},{2,3}}`);
  `Subsets[{1,2,3},{2}]`→ EXACTLY size 2 (`{{1,2},{1,3},{2,3}}`);
  `Subsets[list,{min,max}]`→ sizes in range. Match WK's ordering exactly.
- **`Tuples[list, n]`** (`{Protected}`): all length-n tuples (with repetition):
  `Tuples[{1,2},2]`→`{{1,1},{1,2},{2,1},{2,2}}`. LIST-of-lists form (cartesian product):
  `Tuples[{{1,2},{3,4}}]`→`{{1,3},{1,4},{2,3},{2,4}}`. Match the ordering.
- **`Partition[list, n]`** (`{Protected}`): non-overlapping blocks of `n`, dropping an
  incomplete final block: `Partition[{1,2,3,4,5},2]`→`{{1,2},{3,4}}`. 3-arg offset (sliding
  window): `Partition[{1,2,3,4,5},2,1]`→`{{1,2},{2,3},{3,4},{4,5}}`. The fancier padding /
  cyclic / `{n}` forms may be skipped — NOTE what you skip.

## Where / how

Register in `builtin_catalog/` + `metadata.rs` + dispatch. `Intersection` is a set op — put it
NEXT TO `Union`/`Complement` and reuse their sort/dedup machinery + copy `Union`'s attribute set.
`Subsets`/`Tuples`/`Partition` are combinatorial list ops — `structure_ops/` (near the list
builtins / the batch-2 `list_plumbing.rs`). No float paths.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests per function: the anchors + n-arg Intersection + the three Subsets
forms + both Tuples forms + Partition 2-arg and 3-arg-offset + symbolic passthrough + confirm
`Intersection`'s attributes equal `Union`'s. Spot-check against WK via the documented oracle
resolution (`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not a hardcoded path); save raw to a
file, parse.

## Hard rules

- Product code ONLY for these 4 builtins (+ tests). Do NOT touch `Union`/`Complement` or other
  builtins. No `Expr`/serde change. No new `unsafe`.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes + the Subsets/Tuples/Partition orderings.
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 4 builtins (files), any fancier form skipped (with why); confirm
   Union/Complement untouched.
2. WK calibration — attributes (Intersection == Union) + the anchors (Subsets forms, Tuples
   two forms, Partition offset).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match, with analysis.
