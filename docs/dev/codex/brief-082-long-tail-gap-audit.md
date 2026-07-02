# brief-082 — long-tail gap audit: enumerate the in-scope common functions rum lacks

## Goal

Produce a COMPLETE, categorized, ranked list of the common core-language Wolfram functions
that `rum` does NOT yet implement (or implements divergently), so the follow-up
implementation line has an accurate scope. This is a DIAGNOSTIC/AUDIT brief: it writes a
report doc only — NO product-code change. The differential lanes already proved rum's
IMPLEMENTED surface is WK-conformant (Lane A curated + Lane B = 0 rum-owned divergence), so
the gap is MISSING functions, not divergent ones. Enumerate them mechanically, then filter
to rum's scope.

## Method

1. **Enumerate both symbol sets** (use the RECORDED oracle method — do NOT hardcode a kernel
   path). The documented local-kernel policy resolves the WK oracle via
   `RUM_DIFFTEST_ORACLE_KERNEL`, then PATH `WolframKernel`/`MathKernel`, then common install
   paths (see `rum-difftest` `resolve_oracle_kernel` in `crates/rum-difftest/src/config.rs`
   and `docs/testing-strategy.md`). For this one-off enumeration (there is no difftest
   "list symbols" subcommand), run `Names["System`*"]` through that resolved kernel — e.g.
   `wolframscript -code 'Export["/tmp/wk-system-symbols.txt", Names["System`*"], "Lines"]'`
   (wolframscript auto-resolves the kernel), or the kernel from
   `RUM_DIFFTEST_ORACLE_KERNEL`. Save raw to a file.
   - rum: `target/debug/rum-cli --eval 'Names["System`*"]'` (rum has ~504). (A pre-dumped
     rum list may be at `/tmp/rum-system-symbols.txt`; regenerate to be safe.)
2. **Diff:** WK-minus-rum = every System symbol rum lacks (~5000+, mostly out of scope).
3. **Filter to IN-SCOPE common core-language functions.** Keep only functions that belong to
   the matrix's scoped categories (see `docs/core-language-support-matrix.md`, CL-01..CL-09)
   AND are commonly used. EXCLUDE the out-of-scope domains (these are NOT rum's target — list
   them in the report but do not enumerate their members):
   - graphics / visualization / geometry / front-end / notebook / cloud;
   - machine learning, neural nets, entities, knowledge, units, quantities, dates/calendars
     beyond what the matrix already lists;
   - **floating-point / machine-number / transcendental numerics** — rum is RATIONALS-ONLY
     by design (no Real type): exclude `N`, `Sin`/`Cos`/`Exp`/`Log`/etc. that require reals,
     machine-precision, and numeric solvers. KEEP exact-arithmetic scalar functions
     (`Sign`, `Floor`, `Ceiling`, `Round`, `Mod`, `Quotient`, `GCD`, `LCM`, `Min`, `Max`,
     `Abs` (have), `IntegerDigits`, `FromDigits`, etc.);
   - file / network / process / system / import-export beyond the small surface the matrix
     already lists;
   - special-function / calculus-heavy / linear-algebra-heavy domains.
4. **Cross-check each in-scope candidate against WK + rum** (a basic input each): confirm rum
   returns it UNEVALUATED (missing) or DIVERGES, and record WK's result. Save raw WK output
   to a file, then parse (rum returns exact rationals).
5. **Organize by CL category** (CL-01..CL-09) and for each function record: WK behavior (one
   example in→out), status in rum (missing / divergent), a **commonness** tag (high / medium
   / low — how central to general core-language use), and an **ease** estimate (trivial /
   moderate / involved — e.g. `Sign`/`Floor` trivial; `Sum`/`Product` with iterator specs
   moderate; anything needing new evaluator machinery involved).
6. **Reconcile with the matrix:** for each gap, note whether the matrix already lists it
   (`planned`/`partial`) or MISSES it entirely (matrix inaccuracy to fix later).
7. **Recommend an implementation batch order** for the follow-up line: group the gaps into
   coherent batches (e.g. "scalar exact-math: Sign/Floor/Ceiling/Round/Mod/Quotient",
   "set ops: Intersection/Union/Complement/Partition/Riffle", "string: StringReverse/
   ToLowerCase/StringInsert", "iterators: Sum/Product", "small fixes: HoldForm HoldAll"),
   ranked by commonness×ease.

## Known starting signal (from the seed lane-a run — include + expand)

Confirmed MISSING (return unevaluated): `Sign`, `Floor`, `Ceiling`, `Sum`, `Product`,
`Intersection`, `Partition`, `StringReverse`, `ToLowerCase`. Confirmed DIVERGENT:
`HoldForm` (evaluates its arg — missing `HoldAll`; `HoldForm[1+1]` → rum `HoldForm[2]`, WK
keeps `HoldForm[1+1]`). These MUST appear in the report; the audit's job is to find the
REST systematically.

## Deliverable

Write `docs/long-tail-gap-audit.md`:
- a short method note + the out-of-scope exclusion list;
- the in-scope gap list, organized by CL-01..CL-09, each row: function, WK example, rum
  status (missing/divergent), commonness, ease, matrix-listed? (yes/no);
- a totals summary (count per category, count per commonness×ease bucket);
- the recommended batched implementation order for the follow-up.

## Hard rules

- NO product-code change. Report/doc only (`docs/long-tail-gap-audit.md` + optional raw
  dumps under `/tmp`). No `git commit`. Never edit any `mma-hol` file. No new `unsafe`.
- WK is the oracle (real WolframKernel/`wolframscript`); save raw output to a file, then
  parse; rum returns exact rationals (parse with Fraction, not a decimal regex).
- Respect the RATIONALS-ONLY scope — do NOT list float/transcendental numerics as gaps.
- Be COMPLETE within scope (the point is to not miss long-tail) but do NOT pad with
  out-of-scope domains. When unsure whether something is in scope, list it in a separate
  "uncertain / needs-decision" section rather than dropping it silently.

## Report format (your chat reply)

1. Totals — how many System symbols WK has, rum has, the raw diff size, and the in-scope
   gap count after filtering.
2. The gap list summary per CL category (counts + the high-commonness/trivial-ease ones
   called out).
3. The recommended batch order for the implementation line.
4. Uncertain/needs-decision items (scope calls you want confirmed).
5. Confirm `docs/long-tail-gap-audit.md` written and NO product-code change.
