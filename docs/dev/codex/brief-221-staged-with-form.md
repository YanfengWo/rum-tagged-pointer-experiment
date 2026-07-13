# Brief 221 — Staged `With[{…}, {…}, body]` declaration form (USER-RULED; unit-9 decision fix 2)

Fix brief — the unit-9 audit's second decision item, ruled by the user
2026-07-13: build the WL-documented (12.2+) staged multi-declaration form,
as its own batch. Source: `docs/dev/usability-pass/kernel-b-findings.md`
(the `With` flexibility `gap` row + E3) and the tracker's unit-9 triage
section. Read both FIRST.

## Today (audit E3)

```
With[{x = 1}, {y = x}, HoldComplete[x, y]]
→ inert With[List[Set[x, 1]], List[Set[y, x]], HoldComplete[x, y]]
! With::argx: expected exactly 2 argument(s), got 3   (× fixed-point repeats)
```

Arity ≥ 3 is rejected at `scoping/lexical/with.rs:4-16` and the call stays
inert with a repeating `argx` message.

## Required behavior (WL semantics, documented knowledge)

`With[decls1, decls2, …, body]` where every argument except the last is a
`List` of assignments = the NESTED equivalent
`With[decls1, With[decls2, With[…, body]]]`:

- Later declaration lists SEE earlier variables:
  `With[{x = 1}, {y = x + 1}, x + y]` → `3`;
  `With[{x = 1}, {y = x}, HoldComplete[x, y]]` → `HoldComplete[1, 1]`.
- Substitution stays textual and reaches held positions, exactly as the
  2-arg form does today (`with.rs:26-28`).
- **The 2-arg form is BYTE-IDENTICAL**: same code path, same semantics —
  including rum's pinned same-list sequential contract
  (`With[{x = 1, y = x}, …]`, `with.rs:31-46`,
  `holding_and_control_part3.rs:32-49`). This brief adds arities ≥ 3 only;
  it does NOT touch same-list ordering. Implement the staged form as a
  structural desugar to nested `With` (build the inner `With` expression
  and evaluate it, or loop the existing single-stage substitution outermost-
  first — either way the observable contract is the nested equivalent).
- Fail-closed: if any non-final argument is not a list of assignments
  (`Set`/`SetDelayed` entries — whatever the 2-arg form accepts today),
  emit the SAME diagnostic class the 2-arg form emits for a malformed
  declaration list, and return inert. `With[body]` (arity 1) and
  `With[]` keep today's `argx` behavior — WL's 1-arg `With` operator form
  is NOT in scope.
- `Block`/`Module` are NOT staged in WL — do not touch them.

## Method

1. Survey tests first: find every test asserting the old arity-3 `argx`
   behavior (the audit pinned it somewhere in the holding/control family);
   list each hit with old→new in the report.
2. Keep the arity-2 dispatch structurally intact (an explicit early branch
   is fine); the ≥3 path is new code. State in the report why the 2-arg
   path is unchanged (this decides whether the director must re-run the
   2-hour mma-hol gate — a clean structural separation means workspace +
   A/B suffice).
3. New tests: the two probes above; a 3-stage form
   (`With[{a = 1}, {b = a + 1}, {c = b + 1}, {a, b, c}]` → `{1, 2, 3}`);
   held-position substitution through stages
   (`With[{x = 1}, {y = x}, Hold[x + y]]` → `Hold[1 + 1]`); a malformed
   middle list fails closed with the 2-arg diagnostic; shadowing across
   stages (`With[{x = 1}, {x = x + 1}, x]` → `2` — inner stage rebinding,
   per the nested-With equivalence); the same-list sequential pin stays
   green untouched.
4. Update the `With` registration description (`builtin_catalog/
   scoping.rs`) to mention the staged form; docgen Limits MERGE if an
   entry exists; regenerate the reference.

## Perf gate

`scripts/core-language-microbench.sh`, saved binaries, alternating runs,
profiling env OFF. Watch label: `with-local` (the 2-arg hot path — must
sit within the control-label noise floor). The ≥3 path is new and has no
baseline.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes: the audit E3 probe (now staged), the new-test panel above,
#   and the 2-arg regression panel (same-list sequential, held substitution)
git status --short            # only MAY-listed files
```

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/scoping/lexical/with.rs`
- `crates/rum-runtime/src/runtime/builtin_catalog/scoping.rs` (description
  text only)
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only)
- Tests: `crates/rum-stdlib/src/tests/holding_and_control_part3.rs`, any
  test file the survey names (list each in the report), one new test file
  allowed.

MUST NOT touch: `module.rs`, `block.rs`, `unique.rs`, the evaluator core,
the matcher, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- The 2-arg form's behavior and code path stay byte-identical; the
  same-list sequential contract is pinned and must not move.
- WL comparison from documented knowledge only; no external engines.
- Stop-loss: if the staged desugar cannot reuse the existing single-stage
  substitution without changing 2-arg behavior, STOP and report.

## Report format

1. The change: desugar mechanism, why the 2-arg path is structurally
   unchanged.
2. Test survey (old arity-3 argx assertions), old→new.
3. Probe panel before/after, verbatim.
4. A/B table + noise-floor verdict (with-local called out) + raw paths.
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
