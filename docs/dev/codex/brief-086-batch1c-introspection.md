# brief-086 — long-tail batch 1c: symbol/expr introspection + HoldForm (folk, WK-calibrated)

## Goal

Implement 3 missing introspection builtins — `SymbolName`, `NameQ`, `LeafCount` — and add
`HoldForm` (currently NOT registered: `HoldForm[1+1]` wrongly → `HoldForm[2]`). Folk,
WK-calibrated ([[rum-folk-implementation-scope]]). Product-code change; no-regression bar
includes mma-hol byte-identical. `HoldForm`'s output-suppression is the tricky part (below).

## Behavior (WK-calibrated; anchors verified against WolframKernel)

- **`SymbolName[sym]`** → the short name as a String, context stripped:
  `SymbolName[Global`foo]` → `"foo"`, `SymbolName[x]` → `"x"`. Attributes `{Protected}` (NOT
  HoldAll — it takes the symbol; a non-symbol arg → unevaluated/symbolic passthrough).
- **`NameQ["str"]`** → `True` if the string is the name of an existing symbol, else `False`:
  `NameQ["Sin"]`→... (whatever rum has; use an existing rum symbol like `NameQ["Plus"]`→True),
  `NameQ["ZZQ12"]`→`False`. Folk = EXACT name existence (WK also accepts `*` patterns — you MAY
  support the pattern form if `Names` already does the matching; otherwise exact is fine, note
  it). Attributes `{Protected}`. Non-string arg → symbolic passthrough.
- **`LeafCount[expr]`** → number of atomic subexpressions in the FullForm tree (heads count):
  `LeafCount[f[x,y]]`→`3`, `LeafCount[1+2 x]` (=`Plus[1,Times[2,x]]`)→`5`, `LeafCount[5]`→`1`.
  Attributes `{Protected}` (NOT Listable). Match WK's counting exactly.
- **`HoldForm[expr]`** — Attributes `{HoldAll, Protected}`:
  1. HOLD the argument: `HoldForm[1+1]` stays `HoldForm[1+1]` internally (NOT `HoldForm[2]`);
     `FullForm[HoldForm[1+1]]` → `HoldForm[Plus[1,1]]`.
  2. INVISIBLE WRAPPER in output: OutputForm / InputForm / StandardForm render `HoldForm[expr]`
     as just `expr` — e.g. `ToString[HoldForm[1+1]]` → `"1 + 1"` (no `HoldForm[...]` around it),
     exactly like WK. Only `FullForm` (and internal structure) shows the `HoldForm` head.
     This needs a formatter rule that treats `HoldForm` as transparent in the display forms
     (find where rum formats expressions / `Hold` is shown, and special-case `HoldForm`). If the
     wrapper-suppression is genuinely too invasive to do cleanly, at MINIMUM ship the correct
     `HoldAll` holding + report the display gap — but prefer full WK behavior.

## Where / how

- Register in `builtin_catalog/` + `metadata.rs` (BuiltinHandler variants + attributes) +
  dispatch, like the recent batches. `SymbolName`/`NameQ` are namespace introspection — see
  `namespace_tools/names.rs` (where `Names` lives) to reuse symbol-table lookup. `LeafCount` is
  a pure expression traversal (count FullForm atoms) — put it near other expr-size/structure
  builtins (`ByteCount`/`Depth`/`LeafCount` neighborhood). `HoldForm`: register with `HoldAll`
  + add the formatter transparency rule.

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated tests: `SymbolName` (context strip + symbolic), `NameQ` (exists/not + non-
string passthrough), `LeafCount` (the 3/5/1 anchors + a nested case), `HoldForm` (holds: FullForm
shows HoldForm[Plus[1,1]] and it is NOT 2; transparent output: ToString/OutputForm == the inner
expr's string). Spot-check against WK via the documented oracle resolution
(`RUM_DIFFTEST_ORACLE_KERNEL`/`wolframscript`, not a hardcoded path); save raw to a file, parse.

## Hard rules

- Product code ONLY for these 4 builtins (+ the HoldForm formatter rule) + tests. No
  `Expr`/serde change. No new `unsafe`. Don't touch unrelated builtins/formatting.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass records these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes. (Watch the HoldForm formatter change for byte-identical mma-hol — it must not
  alter output of any non-HoldForm expression.)
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 4 builtins + the HoldForm formatter transparency rule (files).
2. WK calibration — attributes + the anchors (LeafCount 3/5, HoldForm FullForm vs OutputForm).
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any divergence you couldn't match (esp. HoldForm display) with analysis.

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. 3 introspection builtins + HoldForm, WK-conformant.

Committed. `SymbolName` (context-strip → "foo"), `NameQ` (exact existence: Plus→True, ZZQ12→
False; WK pattern form skipped/folk), `LeafCount` (FullForm atoms: 3/5/1), `HoldForm`
({HoldAll,Protected} + formatter transparency in strings/text/render.rs). Gate (Claude
reproduced): all behaviors match WK; **HoldForm verified against WK directly** — it now HOLDS
(`FullForm[HoldForm[1+1]]`=`HoldForm[Plus[1,1]]`, not 2) and the wrapper is transparent in
OutputForm (`ToString[HoldForm[1+1]]`="1 + 1" in both rum and WK) but KEPT in InputForm
(`ToString[...,InputForm]`="HoldForm[1 + 1]" in both). `cargo test -p rum-runtime` 751/0/2;
`cargo test --workspace` no failures; `mma_hol_cold_lane` 4/0 byte-identical; fmt clean.

Codex correctly followed the real WK probe over the brief's mistaken claim that InputForm is
transparent (WK keeps the wrapper in InputForm) — good calibration discipline. NameQ pattern-
string form is the only skipped WK behavior (folk scope). The `a b`→`"a * b"` Times rendering
is a PRE-EXISTING rum OutputForm choice (`*` vs space), unrelated to this batch.

Long-tail batch 1 progress: 1a (7 scalar math) + 1b (6 string) + 1c (4 introspection) done.
Next: batch-1d (integer digits: IntegerLength/IntegerReverse/IntegerDigits/FromDigits/DigitCount).
