# brief-080 — fix the Flat/Orderless+Optional matcher divergences (brief-077's 4 queued bugs)

## Goal

Fix rum's matcher so the 4 WK divergences found by brief-077 go away and the quarantined
tests can be un-`#[ignore]`'d. The `#[ignore]`'d tests in
`crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs` ARE the spec
(their asserted values are WK-calibrated). This is a PRODUCT-CODE (matcher) change — the
matcher is a GLOBAL hot path used by ALL pattern matching, so the no-regression bar is
absolute: **byte-identical behavior on everything else, including the mma-hol acceptance
suite.** Partial progress is acceptable and EXPECTED: fix what can be made WK-byte-identical
WITHOUT regressing anything; leave anything you cannot fix safely still `#[ignore]`'d with a
clear written reason. Do NOT force a fix that breaks other matcher cases.

## The 4 divergences (from brief-077; all in the Flat/Orderless+Optional + sequence corner)

1. **Flat+Orderless+Optional binding ORDER:** `g[1,2] /. g[a_,b_:0] :> {a,b}` → WK `{1,2}`,
   rum `{2,1}`. Root clue: for Flat, WK binds `a` to the flat-wrapped singleton `g[1]` first
   (`g[1] /. g[a_,b_:0] :> {a,b}` → WK `{g[1], 0}`), rum binds `a→1`. This is an
   enumeration-ORDER issue (which candidate binding is tried first).
2. **Flat+Orderless ReplaceList under-collects grouped Flat matches:** WK returns the grouped
   `g[...]` variants PLUS the atomic variants; rum returns fewer. Completeness of the
   Flat-grouping enumeration.
3. **Orderless+Optional with Blank/BlankSequence neighbor ReplaceList under-collects** the
   default/omission branches (e.g. `f[a_, b___, c_ : 0]`): rum drops some omission
   enumerations WK produces.
4. **Flat+Orderless+Optional with sequence neighbors:** WK 10 results, rum 3 — both a
   different first binding AND heavy under-collection (a compound of 1+2+3).

Very likely 1 is the most tractable (ordering) and 4 the least (compound). Tackle in order
of tractability; each fixed one un-`#[ignore]`s its test.

## Method (diagnose before editing)

1. **Re-derive WK's exact behavior** for each failing case with `wolframscript` (the
   `#[ignore]`'d tests already encode it, but re-confirm the ORDER/completeness — save raw
   output to a file, parse carefully; rum returns exact rationals). Also probe a few
   NEIGHBORING cases (Flat-only, Orderless-only, Optional-only, and the 5 already-passing
   combined cases) so you know the invariants a fix must NOT break.
2. **Locate the responsible arm.** The code is `pattern_matching/sequence/orderless/`
   (`entry.rs`, `match_choices.rs`, `candidates.rs`, `collect.rs`, `selected.rs`,
   `optional.rs`) and `pattern_matching/sequence/optional_sequence/*`. Find where the
   candidate-binding ORDER (bug 1) and the match COLLECTION/enumeration (bugs 2-4) are
   decided. Explain the divergence in the report BEFORE the fix.
3. **Fix minimally**, preserving the ordering/output of every non-divergent case. Prefer a
   change localized to the Flat+Orderless+Optional arm; if a fix must touch a shared
   ordering/enumeration routine, prove (by the gate) it changed nothing else.
4. **Un-`#[ignore]`** each test you fixed; leave the rest `#[ignore]`'d with an updated
   reason (what's still wrong + why you didn't fix it).

## No-regression gate (ABSOLUTE — this is a hot-path matcher change)

```
cargo fmt --check
cargo test -p rum-runtime            # ALL green incl. newly un-ignored; note ignored count drop
cargo test --workspace               # 1251+ passed / 0 failed; paste totals
```
Then the mma-hol byte-identical gate (a matcher change is always-on, so it MUST NOT alter
the acceptance suite) — run the existing real-tower / heavy-prover lane exactly as the repo
does it and confirm byte-identical:
```
cargo test -p rum-runtime mma_hol            # the cold-lane / real-tower runtime tests
```
If a broader mma-hol run is the established gate (e.g. a drule/meson module via the snapshot
path), run a representative one and confirm PASS byte-identical to before. Paste evidence.
Spot-check the matcher hot path is not slowed (a quick before/after on a matching-heavy
microbench if you touched a shared routine).

## Hard rules

- No `git commit`/branch/push. Never edit any `mma-hol` file (acceptance standard is
  read-only). No new `unsafe`. No `Expr`/serde shape change.
- **Absolute no-regression:** every currently-passing test + the mma-hol suite stays
  byte-identical. A fix that regresses ANYTHING is rejected — revert it and leave that bug
  `#[ignore]`'d instead.
- **Stop-loss:** partial progress is fine. Do NOT keep forcing a fix that breaks other
  cases. Report what you fixed, what you couldn't, and why.
- WK is the oracle (real `wolframscript`); save raw output to a file then parse; rum timing/
  values are exact rationals (parse with Fraction, not a decimal regex).
- Paste REAL verbatim output; never fake a green.

## Report format

1. Diagnosis — per bug: WK's exact behavior, the responsible code arm, why rum diverged.
2. What changed — the matcher edit per fixed bug (files + the localized change); which tests
   un-`#[ignore]`'d.
3. Gate — `cargo test -p rum-runtime` + `--workspace` + the mma-hol byte-identical evidence,
   all verbatim; the ignored-count before→after.
4. Not fixed — each bug left `#[ignore]`'d, with why it couldn't be fixed without regressing.
5. Open questions.
