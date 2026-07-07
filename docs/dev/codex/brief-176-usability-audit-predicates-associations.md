# Brief 176 — Usability audit: predicates + associations families (83 builtins)

Audit brief — sweep unit 3 of the Usability Pass. Read
`docs/dev/usability-pass.md` FIRST: axes, compat rule, coverage
discipline, and the unit-1/unit-2 triage + closed-ledger sections (they
show the judgment standard and the corrections method). This is an
AUDIT: you change no behavior and no code; the only file you create is
the findings document.

## Objective

Sweep every builtin in the predicates and associations catalog families
against the three usability axes and write
`docs/dev/usability-pass/predicates-associations-findings.md`.

## Inputs

- `crates/rum-stdlib/src/catalog/predicates.rs` (50 `BuiltinSpec`) and
  `crates/rum-stdlib/src/catalog/associations.rs` (33) — the
  authoritative name lists; 83 total. Findings table has AT LEAST one
  row per entry, grouped by family, catalog order.
- Implementations: `crates/rum-stdlib/src/predicates/` and
  `crates/rum-stdlib/src/associations/`.
- Contracts: entries in `docs/reference/builtins.md`; guide chapters
  (`docs/guide/structured-data.md` for associations — check which
  apply).
- WL comparison: documented WL surfaces from your knowledge (options,
  operator forms, `SameTest`, level specs, `Missing[…]` handling,
  default-value forms). Design reference only — never run an external
  engine.

## Pre-seeded findings to verify (unit-3 WFR candidates)

The WFR survey (`docs/dev/wfr-core-survey.md`) flagged three
association utilities as high-value scripting demand. Check whether rum
has an equivalent; if not, record as a `gap`/addition proposal (these
would be NEW builtins, folk, idea-level — like brief-174):

- `KeyGroupBy[assoc, f]` — GroupBy over KEYS (with optional reduction).
- `AssociationKeyDeflatten[flatAssoc]` — flat list-keyed association →
  nested association; check whether a flatten twin
  (`AssociationKeyFlatten`) is wanted too.
- `MergeNested[{assoc…}, f]` — recursive `Merge` for nested
  associations (config layering; pairs with the codec family +
  `ConformSchema`).

Also survey the predicate family for WL-documented operator forms rum
lacks (the operator-form mechanism now exists from brief-164/175; a
predicate operator form like `GreaterThan[n]` / `EqualTo[n]` /
`MatchQ[patt]` is common WL) — RECORD every gap; scheduling of
call-planning fixes is a triage concern, not an audit one.

## Findings file format

Same as units 1-2 (the two existing findings files are the template):
the seven-column table (Builtin | Origin | Verdict | Axis | Finding |
Proposal | Size), `OK` rows required, one row per axis-gap; then an
**Evidence appendix** (verbatim probe transcripts, stdin fed:
`printf '' | cargo run -q -p rum-cli -- --eval '…'`; batch related
checks; `file:line` citations acceptable for code-visible gaps) and a
**Triage sketch** (2-5 candidate fix batches ranked by script-user
value).

## Rubric (per builtin)

- Flexibility: WL-documented options/operator forms/argument shapes rum
  lacks; escape hatches for the non-default case; for associations,
  `Missing[…]` handling and default-value forms
  (`Lookup[assoc, key, default]`).
- Convenience: sensible defaults; the common scripting case short;
  missing obvious sibling/inverse (record, folk-scope).
- Safety: malformed input fails closed or messages (never a silent
  wrong answer — units 1-2 found several of these); surprising
  evaluation/threading; `Options[...]` registration matches actually
  accepted options (units 1-2 found stale/missing/lying ones — check
  each association/predicate that accepts options).

## Probe budget

Cap ~65 probe evals total (83 builtins, many trivially OK); anything
needing more becomes an open question. Expected wall time 25-45 min.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/predicates.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/associations.rs
# one-off shell check: every catalog name from BOTH files appears as a
# table row; must print COVERAGE-OK 83/83 (paste check + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create:
  `docs/dev/usability-pass/predicates-associations-findings.md` — the
  ONLY file in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`,
  the tracker, the other findings files, or anything else.

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
