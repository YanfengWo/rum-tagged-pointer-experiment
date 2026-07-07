# Brief 186 — Usability audit: math families (62 builtins)

Audit brief — sweep unit 5 of the Usability Pass (arithmetic +
number_theory + combinatorics + integer_digits + polynomials +
calculus). Read `docs/dev/usability-pass.md` FIRST: the three axes, the
compat rule, the coverage discipline, and the closed-unit ledgers
(units 1-4) — they show the judgment standard, the corrections method,
and the disambiguation rule for operator forms. This is an AUDIT: you
change NO behavior and NO code; the only file you create is the findings
document.

## Objective

Sweep every builtin in the six math catalog families against the three
usability axes and write `docs/dev/usability-pass/math-findings.md`.

## The folk-scope guardrail (READ TWICE — this governs the whole unit)

This unit is the numeric / CAS-adjacent surface, so the single biggest
risk is filing false gaps for missing algorithmic depth. rum is a
FOLK implementation, not a CAS, and it is RATIONALS-ONLY by design:

- **No floating point. Decimals lower to exact rationals BY DESIGN.**
  `Round`/`Floor`/`Quantile`/`Mean` returning exact rationals, `1.5`
  becoming `3/2` — these are the intended contract, NOT gaps. See
  `docs/guide/design-boundaries.md`.
- **Folk / good-enough algorithms are the contract.** `Factor` is
  structural, `Prime`/`PrimePi` are trial-division-and-small-table,
  number theory is small tables + naive iteration, `D` is a folk
  differentiator. A heavy input getting a message or an inert fallback
  is CORRECT folk behavior, NOT a gap. Do NOT propose "add real
  factorization / symbolic summation / a full polynomial GCD / arbitrary
  special functions" — those are out of scope by design and filing them
  wastes the sweep.

What IS a legitimate finding here (the three axes, folk-compatible):
- **Safety (highest value this unit):** out-of-domain / malformed input
  that returns a SILENT wrong or inert result instead of failing closed
  with a message. E.g. `Prime[0]` / `Prime[-1]`, `Binomial`/`Multinomial`
  on bad arity, `D` of an unsupported form, `Factor` of a non-polynomial,
  `Quantile`/`Median` on an empty list, `Mod[x, 0]`, `PowerMod` with a
  non-invertible base, `FromDigits`/`IntegerDigits` on a bad base or
  negative length. Fail-closed (message + `$Failed`/inert) beats a silent
  wrong answer.
- **Flexibility:** WL-documented OPTIONS or extra call forms that are
  cheap and folk-compatible and currently missing — e.g. `Round[x, a]`
  (round to a multiple), `IntegerDigits[n, base, len]`,
  `FromDigits[list, base]`, `Total[list, n]` / `Total[list, {level}]`,
  `D[f, {x, n}]` (n-th derivative) and `D[f, x, y]`,
  `CoefficientList` multivariate, `DigitCount[n, base, d]`. Only forms
  rum can fold cheaply; NOTE-but-do-not-propose anything needing a new
  algorithm. Operator/curried forms are mostly absent in WL for this
  family — apply the disambiguation rule (only arities that cannot be a
  data form), and most math builtins will NOT qualify.
- **Convenience:** attribute parity that affects real use — `Listable`
  on the elementwise functions (`Abs`/`Sign`/`Floor`/`Ceiling`/`Round`/
  `IntegerPart`/`FractionalPart`/`Mod`/`Quotient`/…), `NumericFunction`,
  `Orderless`/`Flat`/`OneIdentity` on `Plus`/`Times`/`GCD`/`LCM`/`Max`/
  `Min`, `Protected` on constant-like heads. Cite the current
  `Attributes[...]` and the WL-documented set; an attribute mismatch
  that changes threading/ordering behavior is a real finding.

## Inputs

- Catalog name lists (authoritative, 62 total — findings table has AT
  LEAST one row per entry, grouped by family, catalog order):
  `crates/rum-stdlib/src/catalog/arithmetic.rs` (31),
  `number_theory.rs` (9), `combinatorics.rs` (11),
  `integer_digits.rs` (5), `polynomials.rs` (5), `calculus.rs` (1).
- Implementations (confirm the paths):
  `crates/rum-stdlib/src/arithmetic/` + `arithmetic.rs`,
  `crates/rum-stdlib/src/number_theory.rs`,
  `crates/rum-stdlib/src/combinatorics.rs`,
  `crates/rum-stdlib/src/integer_digits.rs`,
  `crates/rum-stdlib/src/polynomials/`,
  `crates/rum-stdlib/src/calculus.rs`.
- Contracts: entries in `docs/reference/builtins.md`;
  `docs/guide/design-boundaries.md` (the folk-scope / rationals-only
  boundary — the authority for what is intentionally out of scope).
- WL comparison: documented WL surfaces from your knowledge (options,
  extra argument forms, attributes, the `Listable`/`NumericFunction`
  sets, domain-error messages). Design reference ONLY — never run an
  external engine.

## Findings file format

Same as units 1-4 (the four existing findings files are the template):
the seven-column table (`Builtin | Origin | Verdict | Axis | Finding |
Proposal | Size`), `OK` rows REQUIRED (the all-rows table IS the
coverage proof), one row per axis-gap; then an **Evidence appendix**
(verbatim probe transcripts, stdin fed: `printf '' | cargo run -q -p
rum-cli -- --eval '…'`; batch related checks; `file:line` citations
acceptable for code-visible gaps) and a **Triage sketch** (2-5 candidate
fix batches ranked by script-user value, noting which touch call
planning / attribute-driven planning and therefore need the A/B gate).

For every row, mark clearly whether an apparent gap is a REAL usability
finding or an intentional folk-scope boundary (the latter is an `OK`
row with a one-line "folk scope / rationals-only — intended" note, NOT a
gap). Getting this classification right is the point of the unit.

## Probe budget

Cap ~55 probe evals total (62 builtins); anything needing more becomes
an open question. Expected wall time 25-40 min. Batch related domain
checks into one `--eval` list where possible; feed stdin (`printf '' |
…`) so the CLI does not block.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/arithmetic.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/number_theory.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/combinatorics.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/integer_digits.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/polynomials.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/calculus.rs
# one-off shell check: every catalog name from the SIX files appears as a
# table row; must print COVERAGE-OK 62/62 (paste the check script + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/math-findings.md` — the ONLY file
  in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`,
  the tracker, the other findings files, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new
  dependencies, no code changes.
- Every `gap` row is evidence-backed: a probe transcript or a
  `file:line` citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines.
- Folk scope is a hard line: do NOT propose new algorithms / CAS depth /
  floating point. A row that would require any of those is an `OK`
  boundary row, not a gap.
- Stop-loss: if the probe surface misbehaves, STOP after two attempts on
  the same probe and report it.

## Report format

1. Coverage statement: rows vs names, the check output verbatim
   (62/62).
2. Top 10 gaps by script-user value, one line each (mark the safety
   ones — silent-wrong / not-fail-closed — first).
3. The triage sketch (note which batches touch call planning /
   attribute-driven planning and need the A/B gate).
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
