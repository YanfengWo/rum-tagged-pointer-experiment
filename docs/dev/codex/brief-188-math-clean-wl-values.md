# Brief 188 — Math clean WL-value fixes (unit-5 fix 2)

Implementation brief. Executes unit-5 triage fix-batch 2 (tracker
`docs/dev/usability-pass.md`, unit-5 section; evidence E3/E7 in
`docs/dev/usability-pass/math-findings.md`). Read the tracker's unit-5
triage + the USER singularity decision FIRST. This batch is the
folk-conservative CORRECTNESS complement to brief-187: where WL DEFINES
a plain value that rum can represent but rum currently returns a silent
inert call, return that value. It is DELIBERATELY tiny — only the
cases whose WL value is certain. Everything else STAYS INERT (deferred);
do not guess a WL value for an uncertain case.

Two changes, no messages, no special values, no new algorithms.

## Change 1 — `Mod[m, 0] -> m` (integer_ops.rs)

WL defines `Mod[m, 0] == m`. rum currently returns a silent inert
`Mod[5, 0]` (`mod_integer` returns `None` on a zero modulus →
`unwrap_or_else` inert). In
`crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs`, `eval_mod`:
after both args are parsed as exact integers, if the modulus (`rhs`) is
zero, return `Ok(Expr::integer(lhs))` (the first argument), instead of
falling through to the inert call. Everything else — valid nonzero
modulus, non-integer/symbolic args (stay inert) — is UNCHANGED.

Do NOT touch `Quotient[m, 0]`, `PowerMod[_, _, 0]`, `Divide`, `Round`,
`Power`, or `Factorial`: those are the singularities that STAY INERT per
the user decision (WL returns ComplexInfinity/Indeterminate there, which
rum does not represent). Only `Mod[m, 0] = m` has a clean plain value.

## Change 2 — `PartitionsP[-1] -> 0`, `PartitionsQ[-1] -> 0` (combinatorics.rs)

WL defines `PartitionsP[n] == 0` and `PartitionsQ[n] == 0` for every
`n < 0` (no partitions of a negative integer). rum currently returns a
silent inert call: `eval_partitions_p` / `eval_partitions_q`
(`crates/rum-stdlib/src/combinatorics.rs:42` / `:64`) route the index
through the shared `nonnegative_combinatorics_index`, which returns
`None` (silent) for a negative value.

Fix ONLY these two functions: BEFORE the
`nonnegative_combinatorics_index` gate, add — when the parsed `index`
is negative — `return Ok(Expr::integer(BigInt::from(0_u8)))`.

Do NOT change the shared `nonnegative_combinatorics_index` helper, and
do NOT touch the other combinatorics functions that use it
(`BernoulliB`, `Subfactorial`, `StirlingS1`, `StirlingS2`,
`CatalanNumber`, `Multinomial`, `Pochhammer`): their WL value for
negative arguments is NOT confidently known, so they STAY INERT
(deferred). Positive/zero indices are UNCHANGED (still go through the
helper and the existing recurrence).

## Tests

Add tests (co-locate with the existing math tests):
- `crates/rum-runtime/src/runtime/tests/**` (or the arithmetic test
  module — confirm the path): `Mod[5, 0]` → `5`, `Mod[-3, 0]` → `-3`;
  `Mod[7, 3]` → `1` unchanged; a symbolic `Mod[x, 0]` stays inert
  (`Mod[x, 0]`), unchanged.
- `crates/rum-stdlib/src/tests/combinatorics.rs`: `PartitionsP[-1]` →
  `0`, `PartitionsP[-5]` → `0`, `PartitionsQ[-1]` → `0`; the
  positive/zero cases (`PartitionsP[4]` → `5`, `PartitionsP[0]` → `1`,
  `PartitionsQ[0]` → `1`) unchanged; a still-deferred sibling stays
  inert (e.g. `CatalanNumber[-1]` → `CatalanNumber[-1]`, no change).
- Keep all existing math tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
# the clean WL values now returned; singularities + deferred siblings unchanged:
printf '' | cargo run -q -p rum-cli -- --eval '{Mod[5,0], Mod[-3,0], Mod[7,3], Mod[x,0], PartitionsP[-1], PartitionsQ[-1], PartitionsP[4], PartitionsP[0], CatalanNumber[-1], Quotient[5,0]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
`Mod[5,0]`→`5`, `Mod[-3,0]`→`-3`, `Mod[7,3]`→`1`, `Mod[x,0]`→`Mod[x,0]`
(inert), `PartitionsP[-1]`→`0`, `PartitionsQ[-1]`→`0`, `PartitionsP[4]`→
`5`, `PartitionsP[0]`→`1`; the deferred `CatalanNumber[-1]` and the
singularity `Quotient[5,0]` STILL inert and message-free.

## Scope

- MAY modify:
  `crates/rum-runtime/src/runtime/arithmetic/integer_ops.rs` (`eval_mod`
  zero-modulus),
  `crates/rum-stdlib/src/combinatorics.rs` (PartitionsP/PartitionsQ
  negative index),
  and the corresponding test files under
  `crates/rum-runtime/src/runtime/tests/**` and
  `crates/rum-stdlib/src/tests/**`.
- MUST NOT touch: `nonnegative_combinatorics_index` (shared helper); any
  other combinatorics/number-theory function; `Quotient`/`PowerMod`/
  `Divide`/`Round`/`Power`/`Factorial` (singularities stay inert); the
  guide, `docs/reference/builtins.md`, `TODO.md`, the tracker, findings
  files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve any suspend/resume shape you touch.
- Folk-conservative: return a WL value ONLY for the two certain cases
  here (`Mod[m,0]=m`, `PartitionsP/Q[n<0]=0`). Do NOT extend to any
  other function — uncertain WL surfaces STAY INERT.
- Valid and symbolic inputs are byte-identical to today.
- No A/B gate needed (not an eval/dispatch/matcher hot path; these are
  small guards on already-parsed integer paths).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file/function, line ranges, the guard added.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
