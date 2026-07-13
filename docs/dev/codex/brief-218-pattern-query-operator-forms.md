# Brief 218 — Pattern-query operator forms: Position / Count / MemberQ / FreeQ

Fix brief — unit-9 batch 4 (flexibility, M). Source:
`docs/dev/usability-pass/kernel-b-findings.md` (the four operator-form
`gap` rows, Evidence E13) — read it FIRST. The most routine batch of the
unit: four WL-documented operator forms join the EXISTING callable-
wrapper machinery. Touches call planning → microbench A/B gate.

## Today (E13, baseline)

```
{Extract[{1}], Position[_Integer], Count[_Integer], MemberQ[_Integer], FreeQ[_Integer]}
→ List[Function[Extract[Slot[1], List[1]]], Position[Blank[Integer]], …]
! Position::argtu: expected between 2 and 5 argument(s), got 1
! Count::argtu:    expected between 2 and 4 argument(s), got 1
! MemberQ::argtu:  expected between 2 and 4 argument(s), got 1
! FreeQ::argtu:    expected between 2 and 4 argument(s), got 1
```

`Extract[pos]` already produces its operator wrapper — the in-tree
precedent. The other four are WL-documented operator forms
(`Position[patt][expr]` ≡ `Position[expr, patt]`, likewise Count /
MemberQ / FreeQ) and their unary calls are unambiguous: every data form
is ≥ 2-arg (the units-1/2/3 disambiguation rule holds — no type
sniffing needed).

## Required behavior

- `Position[patt][expr]` → `Position[expr, patt]`; same for `Count`,
  `MemberQ`, `FreeQ`. The unary call materializes the SAME wrapper shape
  the existing families use (E13 shows `Extract[{1}]` →
  `Function[Extract[Slot[1], {1}]]` — match whatever the current
  mechanism produces; do not invent a new wrapper kind).
- The wrapper applied to N≠1 args behaves like the existing wrappers do
  in that case (probe `Extract[{1}][a, b]` and mirror; consistency over
  invention).
- 2+-arg data forms byte-identical, including all levelspec/Heads/n
  forms and the `::argtu` diagnostics for genuinely wrong arities
  (0 args stays an error; 1 arg stops being one).
- Origin: all four operator forms are WL-documented — mark them so in
  docs. No rum-native invention here.

## Method

1. Read the landed precedents FIRST: brief-179 (ten predicate operator
   forms incl. the kernel `MatchQ[patt]` arity exemption) and brief-182
   (five rewrite operator forms incl. `Cases[patt]`) — the mechanism is
   the family tables in
   `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
   (`*_operator_symbol(head, arity)` + `is_*_operator_wrapper` +
   `apply_*_operator_wrapper_to_args`). Add the four names to the
   RIGHT family arm(s) (MemberQ/FreeQ smell like the predicate family,
   Position/Count like rewrite/structure — follow where their siblings
   live and say which you chose and why).
2. The 1-arg arity gate: the wrapper arm must win BEFORE the `argtu`
   check fires (mirror how Cases/MatchQ handle it — their gates live
   with the implementations, `tree_walkers/position.rs:4-6` /
   `count_like.rs` for these four).
3. Tests: one per head — unary wrapper shape, wrapper application
   equivalence (`Position[_Integer][{1, a, 2}]` ==
   `Position[{1, a, 2}, _Integer]`), a levelspec/Heads data form
   regression, and the 0-arg error preserved. Baseline-verify every
   preservation pin (standing lesson).

## Perf gate

One A/B (`scripts/core-language-microbench.sh`, saved binaries built IN
THE REPO PATH — checksums must be reproducible by the director — B C B C,
profiling env OFF, idle machine; load appears → STOP and report). No
label hits these heads directly; nearest neighbors `matchq-integer-*`
and `replaceall-simple` are the watch panel; the real check is that the
table lookup adds nothing measurable anywhere.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes: the E13 panel after (wrappers + application equivalence),
#   0-arg errors, data-form regressions — batched, before/after
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
- `crates/rum-runtime/src/runtime/kernel_data/rewrite_data/tree_walkers/`
  (`position.rs`, `count_like.rs` — arity-gate integration only)
- Tests: the operator-form / tree-walker test families (list each file
  in the report), one new test file allowed
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only) if the operator forms warrant notes

MUST NOT touch: the evaluator core, the matcher, `Extract`'s existing
wrapper, scoping, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- Consistency over invention: every choice (wrapper shape, N≠1
  application, family arm) mirrors an existing landed sibling — name the
  sibling in the report.
- Baseline-verify preservation pins; WL comparison from documented
  knowledge only.
- Stop-loss: if any of the four cannot use the existing wrapper chain
  without touching the evaluator core, STOP and report.

## Report format

1. Per-head summary: family arm chosen + the mirrored sibling.
2. Test survey old→new (the 1-arg argtu assertions).
3. Probe panel before/after verbatim.
4. A/B table + noise-floor verdict + raw paths + binary checksums.
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
