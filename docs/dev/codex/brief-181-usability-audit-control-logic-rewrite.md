# Brief 181 — Usability audit: control + logic + rewrite families (63 builtins)

Audit brief — sweep unit 4 of the Usability Pass. Read
`docs/dev/usability-pass.md` FIRST: axes, compat rule, coverage
discipline, and the closed-unit ledgers (units 1-3) — they show the
judgment standard, the corrections method, and the disambiguation rule
for operator forms. This is an AUDIT: you change no behavior and no
code; the only file you create is the findings document.

## Objective

Sweep every builtin in the control, logic, and rewrite catalog families
against the three usability axes and write
`docs/dev/usability-pass/control-logic-rewrite-findings.md`.

## Inputs

- `crates/rum-stdlib/src/catalog/control.rs` (35 `BuiltinSpec`),
  `crates/rum-stdlib/src/catalog/logic.rs` (21), and
  `crates/rum-stdlib/src/catalog/rewrite.rs` (7) — the authoritative
  name lists; 63 total. Findings table has AT LEAST one row per entry,
  grouped by family, catalog order.
- Implementations: `crates/rum-stdlib/src/control_flow/`,
  `crates/rum-stdlib/src/logic/`, `crates/rum-stdlib/src/rewrite_builtins.rs`
  (+ their roots — confirm the paths).
- Contracts: entries in `docs/reference/builtins.md`; guide chapters
  (`docs/guide/evaluation.md`, `docs/guide/patterns-and-rules.md` —
  check which apply).
- WL comparison: documented WL surfaces from your knowledge (operator
  forms, options, level specs, held-argument behavior, the return
  contract of control constructs). Design reference only — never run
  an external engine.

## Special attention for THIS unit

- **Held arguments / evaluation control**: many of these builtins carry
  `HoldAll`/`HoldFirst`/`HoldRest` (control flow) — a usability gap
  here can be a SAFETY issue (surprising evaluation). Probe whether
  hold behavior matches the documented contract; a builtin that
  evaluates an argument it should hold (or vice versa) is a `gap`.
- **Rewrite family** (`Replace`/`ReplaceAll`/`ReplaceRepeated`/rules):
  the unit-4 spillover noted from earlier recon — `ReplaceAll[rules]`
  lacks its WL-documented operator form. Check the whole rewrite family
  for missing operator forms and level-spec / `MaxIterations`-style
  options. RECORD every gap; the operator-form mechanism exists
  (brief-164/175/179) and the perf moratorium is lifted, so these are
  schedulable — but disambiguation (no type-sniffing; only arities that
  cannot be a data form) still governs which are addable.
- **Logic family**: check `And`/`Or`/`Xor`/`Nand`/`Nor`/`Implies`/
  `BooleanQ`-style for short-circuit correctness and operator/threaded
  forms where WL documents them.

## Findings file format

Same as units 1-3 (the three existing findings files are the
template): the seven-column table (Builtin | Origin | Verdict | Axis |
Finding | Proposal | Size), `OK` rows required, one row per axis-gap;
then an **Evidence appendix** (verbatim probe transcripts, stdin fed:
`printf '' | cargo run -q -p rum-cli -- --eval '…'`; batch related
checks; `file:line` citations acceptable for code-visible gaps) and a
**Triage sketch** (2-5 candidate fix batches ranked by script-user
value, noting which touch call planning / need the A/B gate).

## Probe budget

Cap ~55 probe evals total (63 builtins); anything needing more becomes
an open question. Expected wall time 25-40 min. For held-argument
probes, remember the CLI needs stdin fed (`printf '' | …`).

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/control.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/logic.rs
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/rewrite.rs
# one-off shell check: every catalog name from the THREE files appears
# as a table row; must print COVERAGE-OK 63/63 (paste check + output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create:
  `docs/dev/usability-pass/control-logic-rewrite-findings.md` — the
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
3. The triage sketch (note call-planning batches).
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
