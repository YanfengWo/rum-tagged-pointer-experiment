# Brief 161 — Usability audit: strings family (99 builtins)

Audit brief — sweep unit 1 of the Usability Pass. Read
`docs/dev/usability-pass.md` FIRST: its axes, compat rule, and coverage
discipline all bind this brief. This is an AUDIT: you change no behavior
and no code; the only file you create is the findings document.

## Objective

Sweep every builtin in the strings catalog family against the three
usability axes (flexibility / convenience / safety) and write
`docs/dev/usability-pass/strings-findings.md`.

## Inputs

- `crates/rum-stdlib/src/catalog/strings.rs` — the authoritative name
  list (99 `BuiltinSpec` entries). The findings table has AT LEAST one
  row per entry, in catalog order.
- Implementations: `crates/rum-stdlib/src/strings.rs` and
  `crates/rum-stdlib/src/strings/`.
- Contracts as documented: the entries in `docs/reference/builtins.md`;
  guide chapters `docs/guide/text-utilities.md` and
  `docs/guide/string-patterns.md`.
- WL comparison: your knowledge of the DOCUMENTED Wolfram Language
  surface (options such as `IgnoreCase`/`Overlaps`, operator forms,
  listability). Design reference only — do NOT run or compare against
  any external engine (testing policy, `DISCLAIMER.md`).

## Findings file format

One table, catalog order, one row per builtin minimum (a builtin with
gaps on two axes gets two rows):

| Builtin | Origin | Verdict | Axis | Finding | Proposal | Size |

- Origin: `WL` (the name exists in Wolfram Language) or `rum`
  (rum-native addition).
- Verdict: `OK` or `gap`. `OK` rows leave Axis/Finding/Proposal empty —
  they are still REQUIRED rows (the all-rows table is the coverage
  proof).
- Axis: `flexibility` / `convenience` / `safety`.
- Finding: one concrete sentence.
- Proposal: the compatible change. For WL names: a new option or a
  rum-native sibling, and state "default unchanged" explicitly. For
  rum-native names: freer, but small and folk-scope.
- Size: S / M / L implementation estimate.

After the table, two sections:

1. **Evidence appendix** — for every `gap` row whose finding is
   behavioral (not a missing option/form visible from code, which takes
   a `file:line` citation in the Finding cell instead), a verbatim probe
   transcript. Feed stdin:
   `printf '' | cargo run -q -p rum-cli -- --eval '…'`. Batch related
   checks into one eval where possible.
2. **Triage sketch** — group the `gap` rows into 2-5 candidate fix-brief
   batches ranked by script-user value, one-line rationale each.

## Rubric (what to check per builtin)

- Flexibility: WL-documented options rum lacks; WL operator forms rum
  lacks; is the non-default case reachable at all?
- Convenience: are defaults sensible for scripting? magic literals that
  should be named constants? is the common case short, or does it force
  a wrapper the user must write?
- Safety: malformed/edge input fails closed with a message (never a
  silent wrong answer); no surprising evaluation/listability behavior;
  constant-like symbols are `Protected`.
- The guide chapters' own workaround patterns (anywhere the prose
  teaches a multi-step idiom for a one-intent task) are automatic gap
  material.

## Probe budget

Probes are single fast evals — batch them. Cap ~60 probe evals total;
anything needing more becomes an open question in the report. Expected
wall time 20-40 min.

## Verification (MANDATORY — run all, paste verbatim)

```
grep -c 'BuiltinSpec::' crates/rum-stdlib/src/catalog/strings.rs
# write a one-off shell check that every catalog name appears as a
# table row in the findings file; it must print COVERAGE-OK 99/99
# (paste the check and its output)
git status --short          # ONLY the new findings file
cargo fmt --check           # clean — you touched no code
```

## Scope

- MAY create: `docs/dev/usability-pass/strings-findings.md` — the ONLY
  file in the diff.
- MUST NOT touch: any crate code, tests, guide, reference, `TODO.md`,
  `docs/dev/usability-pass.md`, or anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new
  dependencies, no code changes.
- Every `gap` row is evidence-backed: probe transcript or `file:line`
  citation. Never claim a behavior you did not run.
- WL comparison from documented knowledge only; no external engines.
- Stop-loss: if the probe surface misbehaves (CLI hangs, unexpected
  crashes), STOP after two attempts on the same probe and report it.

## Report format

1. Coverage statement: rows vs names, with the check output verbatim.
2. Top 10 gaps by script-user value, one line each.
3. The triage sketch (candidate fix-brief batches).
4. Verification outputs verbatim.
5. Stop-loss / open questions (empty if none).
