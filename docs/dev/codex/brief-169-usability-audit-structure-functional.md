# Brief 169 — Usability audit: structure + functional families (89 builtins)

Audit brief — sweep unit 2 of the Usability Pass. Read
`docs/dev/usability-pass.md` FIRST: axes, compat rule, coverage
discipline, and the unit-1 triage section (its corrections show the
judgment standard). This is an AUDIT: you change no behavior and no
code; the only file you create is the findings document.

## Objective

Sweep every builtin in the structure and functional catalog families
against the three usability axes and write
`docs/dev/usability-pass/structure-functional-findings.md`.

## Inputs

- `crates/rum-stdlib/src/catalog/structure.rs` (47 `BuiltinSpec`
  entries) and `crates/rum-stdlib/src/catalog/functional.rs` (42) —
  the authoritative name lists; 89 total. The findings table has AT
  LEAST one row per entry, grouped by family, catalog order.
- Implementations: `crates/rum-stdlib/src/structure_ops/` and
  `crates/rum-stdlib/src/functional_ops/` (+ their `.rs` roots).
- Contracts: entries in `docs/reference/builtins.md`; relevant guide
  chapters (`docs/guide/structured-data.md`, `docs/guide/evaluation.md`
  — check which apply).
- WL comparison: documented WL surfaces from your knowledge (options,
  operator forms, listability, negative/`All`/`Span` argument forms).
  Design reference only — never run an external engine.

## Pre-seeded findings to verify (from unit-1 spillover)

- `Cases[patt][expr]` operator form is missing (recon probe showed an
  arity message). Check the whole family for WL-documented operator
  forms rum lacks (`Select`/`Map`/`SortBy` already work; verify which
  others do or don't — e.g. `Append`/`Prepend`/`Delete`/`Extract`/
  `Position`/`Count`/`DeleteCases`/`FirstCase`/`GroupBy`/`KeyMap`…
  whichever of these exist in THESE two catalogs).
- IMPORTANT judgment note: operator-form FIXES that touch call
  planning are currently frozen (perf-measurement moratorium) — still
  RECORD every gap; the freeze affects fix scheduling, not the audit.

## Findings file format

Same as unit 1 (`strings-findings.md` is the template): the seven-column
table (Builtin | Origin | Verdict | Axis | Finding | Proposal | Size),
`OK` rows required, one row per axis-gap; then an **Evidence appendix**
(verbatim probe transcripts, stdin fed:
`printf '' | cargo run -q -p rum-cli -- --eval '…'`; batch related
checks; `file:line` citations acceptable for code-visible gaps) and a
**Triage sketch** (2-5 candidate fix batches ranked by script-user
value).

## Rubric (per builtin)

- Flexibility: WL-documented options/operator forms/argument shapes
  rum lacks (negative indices, `All`, `Span`, level specs, default
  arguments); escape hatches for the non-default case.
- Convenience: sensible defaults; magic literals needing names; the
  common scripting case short; missing obvious inverse/sibling
  (record, folk-scope).
- Safety: malformed input fails closed or messages (never a silent
  wrong answer); surprising evaluation/threading; `Protected` where
  constant-like; check `Options[...]` registration matches actual
  accepted options (unit-1 found stale/missing registrations).

## Probe budget

Cap ~70 probe evals total (89 builtins, many trivially OK); anything
needing more becomes an open question. Expected wall time 25-45 min.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/structure.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/functional.rs
# one-off shell check: every catalog name from BOTH files appears as a
# table row; must print COVERAGE-OK 89/89 (paste check + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/structure-functional-findings.md`
  — the ONLY file in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`,
  the tracker, the strings findings file, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new
  dependencies, no code changes.
- Every `gap` row is evidence-backed: probe transcript or `file:line`
  citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines.
- Stop-loss: if the probe surface misbehaves, STOP after two attempts
  on the same probe and report it.

## Report format

1. Coverage statement: rows vs names, check output verbatim.
2. Top 10 gaps by script-user value, one line each.
3. The triage sketch.
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
