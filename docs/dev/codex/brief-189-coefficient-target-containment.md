# Brief 189 — Coefficient/CoefficientList target-containment guard (unit-5 fix 3)

Implementation brief. Executes unit-5 triage fix-batch 3 (tracker
`docs/dev/usability-pass.md`, unit-5 section; evidence E9/E10 in
`docs/dev/usability-pass/math-findings.md`). Read the tracker's unit-5
triage FIRST. This fixes a SILENT-WRONG structural bug: a negative power
of the target variable is mis-absorbed as a coefficient, so
`CoefficientList[x + 1/x, x]` returns `{x^-1, 1}` — a coefficient that
still contains the target variable.

## The bug (precise)

`Polynomial::from_expr`
(`crates/rum-stdlib/src/polynomials/core.rs:85`) sends `Power[base, exp]`
to `pow(exp)` ONLY when `exp` is a non-negative usize; a NEGATIVE
exponent (`Power[x, -1]` = `1/x`) falls through to
`Self::indeterminate(expr.clone())` (line 92), i.e. `1/x` becomes an
OPAQUE indeterminate whose base textually contains `x`. Coefficient
extraction then drops that term into the `x^0` slot, so the coefficient
contains the target variable.

Confirmed current behavior (verbatim, this is the acceptance spec):
```
Coefficient[x + 1/x, x, 1]        -> 1                     (CORRECT — keep)
Coefficient[x + 1/x, x, 0]        -> x^-1                  (WRONG — fix)
CoefficientList[x + 1/x, x]       -> {x^-1, 1}            (WRONG — fix)
Coefficient[Sin[x] + x, x, 0]     -> Sin[x]               (WL-correct — keep)
Coefficient[Sin[x] + x, x, 1]     -> 1                     (keep)
Coefficient[1/(1+x), x, 0]        -> 1/(1+x)              (WL-consistent — keep)
CoefficientList[Sin[x] + x^2, x]  -> {Sin[x], 0, 1}       (WL-consistent — keep)
Coefficient[x^2 + 3 x + 1, x, 2]  -> 1                     (keep)
```

## Why a naive "coefficient contains the target" guard is WRONG

A transcendental term like `Sin[x]` is a LEGITIMATE `x^0` coefficient in
WL (`Coefficient[Sin[x]+x, x, 0]` = `Sin[x]`, `CoefficientList[Sin[x]+
x^2, x]` = `{Sin[x], 0, 1}`), and it contains `x`. So "coefficient
contains the target" would wrongly fail those. The ONLY defect is a
NEGATIVE INTEGER POWER of a target VARIABLE (a Laurent term) leaking into
a coefficient. `1/(1+x)` = `Power[Plus[1,x], -1]` is fine too — its base
is `1+x`, not the bare variable, so it acts as an opaque constant (WL
agrees).

## Change — precise guard (in `crates/rum-stdlib/src/polynomials/mod.rs`)

Add a detection helper: `output_has_negative_target_power(expr, targets)`
— a recursive walk that returns true iff `expr` contains a subexpression
`Power[b, k]` where `b` is EXACTLY one of the `targets` (a target
variable) and `k` is a NEGATIVE integer. Mirror the existing recursive
walk shape of `expr_contains_exact_subexpr`
(`crates/rum-stdlib/src/calculus.rs:247` / `predicates.rs:351`; a small
local copy in `polynomials/mod.rs` is consistent with those).

- In `eval_coefficient` (`mod.rs:36`): after
  `polynomial.coefficient_for_monomial_power(&target, power).to_expr(...)`
  produces the result expr, if it
  `output_has_negative_target_power(result, target_vars)`, return the
  inert `Expr::call(head.clone(), args.to_vec())` instead. `target_vars`
  = the factor bases of the parsed `target` monomial.
- In `eval_coefficient_list` (`mod.rs:74`): after
  `polynomial.coefficient_list_expr(&variables, session)` produces the
  list expr, if it `output_has_negative_target_power(list, target_vars)`,
  return the inert call instead. `target_vars` = the factor bases across
  the parsed `variables` monomials.

Because a correct coefficient is FREE of negative powers of the target,
this triggers ONLY on the leaked-Laurent case. Every correct query above
(the "keep" rows, incl. `Sin[x]` and `1/(1+x)`) is UNCHANGED; only
`Coefficient[x+1/x, x, 0]` and `CoefficientList[x+1/x, x]` newly fail
closed to their inert call.

Do NOT change `Polynomial::from_expr`, the extraction routines, `Expand`,
`Together`, or `Factor` — the guard is a post-extraction output check
only. No new CAS / Laurent support.

## Tests

Add tests in `crates/rum-stdlib/src/tests/polynomials.rs`:
- FAIL CLOSED (newly inert): `Coefficient[x + 1/x, x, 0]` →
  `Coefficient[x + 1/x, x, 0]` (inert), `CoefficientList[x + 1/x, x]` →
  inert call. `Coefficient[x + 1/x^2, x, 0]` likewise.
- KEEP (unchanged): `Coefficient[x + 1/x, x, 1]` → `1`;
  `Coefficient[Sin[x] + x, x, 0]` → `Sin[x]`;
  `CoefficientList[Sin[x] + x^2, x]` → `{Sin[x], 0, 1}`;
  `Coefficient[1/(1+x), x, 0]` → `1/(1+x)`;
  `Coefficient[x^2 + 3 x + 1, x, 2]` → `1`;
  `CoefficientList[1 + x + y + x y, x]` and the genuine multivariate
  cases from findings E9 unchanged.
- Keep all existing polynomial tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Coefficient[x + 1/x, x, 1], Coefficient[x + 1/x, x, 0], CoefficientList[x + 1/x, x], Coefficient[Sin[x] + x, x, 0], CoefficientList[Sin[x] + x^2, x], Coefficient[1/(1+x), x, 0], Coefficient[x^2 + 3 x + 1, x, 2], CoefficientList[1 + x + y + x y, x]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
the probe returns `{1, Coefficient[x + 1/x, x, 0], CoefficientList[x +
1/x, x], Sin[x], {Sin[x], 0, 1}, (1+x)^-1, 1, …}` — i.e. only the two
Laurent-leak queries newly inert, every "keep" row unchanged.

## Scope

- MAY modify: `crates/rum-stdlib/src/polynomials/mod.rs` (the two eval
  functions + the new detection helper) and
  `crates/rum-stdlib/src/tests/polynomials.rs` (new tests).
- MUST NOT touch: `Polynomial::from_expr` / `polynomials/core.rs`, the
  extraction routines, `Expand`/`Together`/`Factor`, any other builtin;
  the guide, `docs/reference/builtins.md`, `TODO.md`, the tracker,
  findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- The guard is a POST-extraction output check keyed on a negative integer
  power of a target VARIABLE only — NOT a "contains the target" check
  (that would wrongly fail `Sin[x]`). Every "keep" acceptance row must be
  byte-identical to today.
- No A/B gate needed (not a hot path; a bounded walk of the produced
  coefficient expr on the Coefficient/CoefficientList paths only).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — the helper + the two guard sites, line ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
