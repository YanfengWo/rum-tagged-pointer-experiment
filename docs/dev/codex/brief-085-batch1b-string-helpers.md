# brief-085 — long-tail batch 1b: string helpers (folk, WK-calibrated)

## Goal

Implement 6 missing string builtins from the long-tail audit: `StringReverse`, `ToLowerCase`,
`LowerCaseQ`, `UpperCaseQ`, `Capitalize`, `Decapitalize`. rum currently returns each
UNEVALUATED (but `ToUpperCase` IS implemented — mirror it). Folk/good-enough, WK-calibrated on
every case ([[rum-folk-implementation-scope]]). Product-code change; no-regression bar includes
mma-hol byte-identical.

## Behavior (calibrate every case against WolframKernel)

Operate on rum `String` values, Unicode-aware exactly like the existing `ToUpperCase`. For a
non-string argument, return the symbolic form unevaluated (e.g. `StringReverse[x]` stays).

- **`ToLowerCase["HELLO"]`** → `"hello"` (the exact mirror of `ToUpperCase`; use the same
  case-mapping path).
- **`StringReverse["hello"]`** → `"olleh"` (reverse the characters — match how rum's
  `Characters`/string layer defines a character; match WK).
- **`Capitalize["hello world"]`** → `"Hello world"` (upcase the FIRST character only, rest
  unchanged). Do NOT implement the 2-arg mode form (`Capitalize[s, "AllWords"]`) unless
  trivial — folk scope is the default; note if skipped.
- **`Decapitalize["Hello"]`** → `"hello"` (downcase the first character only).
- **`LowerCaseQ[s]`** / **`UpperCaseQ[s]`**: predicates. **Calibrate the edge semantics against
  WK precisely** — these are where divergence hides:
  - `LowerCaseQ["abc"]`→True, `LowerCaseQ["aBc"]`→False, `UpperCaseQ["ABC"]`→True;
  - non-letters and empty: check WK for `LowerCaseQ["a1"]`, `LowerCaseQ["123"]`,
    `LowerCaseQ[""]`, `LowerCaseQ["ß"]` etc. and MATCH exactly (WK's rule is roughly "all
    letters present are lowercase"; confirm the empty-string and no-letter cases against WK
    rather than guessing). These also accept a single character.

## Where / how

Mirror `ToUpperCase`: `metadata.rs` (BuiltinHandler variant), `builtin_catalog/strings.rs`
(registration), `strings/text/builtins.rs` (handler), `eval_dispatch/surface.rs` (dispatch),
tests in `tests/strings/surface.rs`. Reuse the existing case-mapping / character helpers;
do NOT touch `ToUpperCase`.

**Attributes:** calibrate against WK (`Attributes[ToLowerCase]` etc.) and match (`ToUpperCase`
is `Listable`+`Protected` — the case ops thread over lists; verify + replicate for all 6,
including the `*Q` predicates).

## Verification (run exactly; paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # incl. new tests; paste totals
cargo test --workspace               # no failures; paste totals
cargo test -p rum-runtime mma_hol    # byte-identical
```
Add WK-calibrated unit tests for each: the exact cases above + the `*Q` edge cases (non-letter,
empty, mixed) + Listable threading (`ToLowerCase[{"A","B"}]`) + symbolic passthrough. Spot-check
a few directly against WK via the documented oracle resolution (`RUM_DIFFTEST_ORACLE_KERNEL` /
`wolframscript`, NOT a hardcoded path); save raw WK output to a file, then parse.

## Hard rules

- Product code ONLY for these 6 builtins (catalog + handlers + attributes) + tests. Do NOT
  touch `ToUpperCase` or other string builtins. No `Expr`/serde change. No new `unsafe`.
- Non-string args → symbolic passthrough.
- No `git commit`. Never edit any `mma-hol` file. Do NOT edit the support matrix (consolidated
  matrix pass will record these).
- Absolute no-regression: workspace + mma-hol byte-identical. WK is the oracle for behavior +
  attributes (especially the `*Q` edge cases).
- Paste REAL verbatim output; never fake a green.

## Report format

1. What changed — the 6 builtins (files), any 2-arg form skipped.
2. WK calibration — attributes + the `*Q` edge-case results confirmed against WK.
3. Verification — `cargo test -p rum-runtime` + `--workspace` + `mma_hol` byte-identical,
   verbatim; the new tests.
4. Any WK divergence you couldn't match (with analysis).

---

## Outcome (Claude gate, 2026-07-02) — DONE + KEEP. 6 string helpers, WK-conformant (incl. *Q edges).

Committed. Implemented `StringReverse`, `ToLowerCase`, `LowerCaseQ`, `UpperCaseQ`, `Capitalize`,
`Decapitalize` (new `strings/text/case.rs` + builtin_catalog/strings.rs + metadata + dispatch +
tests). Folk, Unicode-aware (mirrors ToUpperCase), symbolic passthrough. Gate (Claude
reproduced): behavior matches WK (StringReverse "olleh", ToLowerCase, Capitalize/Decapitalize
first-char, Listable `ToLowerCase[{"A","B"}]={"a","b"}`); **the *Q edge semantics verified
directly against WK: `{LowerCaseQ["a1"],["123"],[""],["aBc"]}` = `{False,False,True,False}` in
BOTH** (the subtle non-letter/empty cases match). `cargo test -p rum-runtime` 749/0/2; `cargo
test --workspace` no failures; `mma_hol_cold_lane` 4/0 byte-identical; fmt clean. Only skipped:
`Capitalize[s,"AllWords"]` (folk scope, allowed). ToUpperCase untouched.

Next: batch-1c (symbol/expr introspection: SymbolName/NameQ/LeafCount + HoldForm HoldAll fix).
