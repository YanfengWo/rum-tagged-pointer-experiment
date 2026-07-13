# Brief 219 — Presentation truthfulness: terminal HoldForm transparency + form-summary honesty

Fix brief — unit-9 batch 5, the unit's CLOSING batch (convenience, S/M).
Source: `docs/dev/usability-pass/kernel-b-findings.md` (the `HoldForm`
convenience `gap` row, the `InputForm`/`StandardForm` summary rows,
Evidence E15) — read it FIRST. Renderer + docs only; the evaluator is
untouched and the microbench A/B is WAIVED (justify by showing no
eval-path file in the diff — the standing honesty rule: say plainly what
was not run and why).

## (a) Terminal `HoldForm` transparency

Today (E15): the output-like `ToString` arms hide the wrapper
(`ToString[HoldForm[1 + 1], OutputForm]` → `"1 + 1"`), but CLI/REPL
value rendering uses canonical `Expr::Display` and prints
`HoldForm[Plus[1, 1]]` literally (`kernel_data/text_rendering.rs:98-103`;
`rum-terminal/src/lib.rs:38-47`). WL's documented `HoldForm` contract is
"prints as expr" — the wrapper is invisible in OUTPUT while keeping its
content held.

Required:

- Terminal value presentation (CLI `--eval` results, REPL results — and
  the jupyter result path if it shares the same seam; report which
  surfaces share it) renders `HoldForm` wrappers TRANSPARENTLY while
  keeping rum's canonical style for everything else:
  `HoldForm[1 + 1]` displays as `Plus[1, 1]`; nested occurrences are
  transparent too (`{HoldForm[1 + 1]}` → `List[Plus[1, 1]]`) — reuse the
  EXISTING transparency logic the output-like `ToString` arm already
  has; do NOT switch the terminal to infix/output formatting (canonical
  display stays; ONLY `HoldForm` becomes invisible).
- Every output not containing `HoldForm` is BYTE-IDENTICAL. This is the
  hard pin: the change is invisible except where a `HoldForm` wrapper
  used to print.
- `InputForm`-style rendering (`ToString[…, InputForm]`) keeps the
  wrapper (round-tripping; already correct, E15) — untouched.
- Session-level `Expr::to_string`/`FullForm` semantics are NOT the
  renderer and must not change (tests assert on them everywhere).
- The stop-loss from the audit's folk-scope guardrail applies: NO
  typesetting expansion, no new form implementations.

## (b) Form-summary honesty

Today: `InputForm`'s and `StandardForm`'s reference summaries mention
only the `ToExpression` consumer, though `ToString[…, InputForm]` /
`ToString[…, StandardForm]` are implemented and observably distinct for
`HoldForm` (E15; `rum-stdlib/src/strings/text/builtins.rs:585-595`).

Required: update the two catalog registration descriptions
(`builtin_catalog/strings.rs`) to name both consumers; regenerate the
reference. If (a) changes what the `HoldForm` summary should say (its
description may currently describe the literal-wrapper terminal
behavior), align it too — check `Information[HoldForm]`/the reference
entry and report.

## Method

1. Survey FIRST: every test asserting terminal-rendered output (rum-cli
   / rum-repl / rum-terminal / rum-jupyter test files) that contains
   `HoldForm` — and, separately, confirm no SESSION-level test asserts
   terminal strings. List hits old→new.
2. Implement the transparency at the terminal seam by REUSING the
   existing output-arm helper (name the helper; if it needs visibility
   widening, say so). No new rendering logic.
3. Tests: terminal-level — `HoldForm[1 + 1]` displays transparently;
   nested case; a no-HoldForm output byte-identity spot check; the
   ToString arms unchanged (E15 panel re-asserted).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
# probes: CLI --eval renders for HoldForm[1+1], {HoldForm[1+1]},
#   Hold[1+1] (unchanged), ToString E15 panel (unchanged), before/after
git status --short            # only MAY-listed files
# the A/B waiver justification: paste git diff --stat and note that no
#   evaluator/matcher/planner file appears in it
```

## Scope

MAY modify:
- `crates/rum-terminal/src/lib.rs` (or the actual terminal render seam —
  name it)
- `crates/rum-runtime/src/runtime/kernel_data/text_rendering.rs`
  (helper exposure only)
- `crates/rum-runtime/src/runtime/builtin_catalog/strings.rs` +
  `crates/rum-runtime/src/runtime/builtin_catalog/functional.rs`
  (description text only — InputForm/StandardForm live in strings.rs,
  HoldForm in functional.rs)
- `crates/rum-stdlib/src/docgen.rs` + `docs/reference/builtins.md`
  (regenerated; Limits MERGE only)
- Tests: rum-cli / rum-repl / rum-jupyter / rum-terminal test files the
  survey names (list each), one new test file allowed

MUST NOT touch: the evaluator, the matcher, `Expr::Display` /
`Expr::to_string` (session-level canonical form), the ToString arms'
behavior, hold semantics, anything else.

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- The byte-identity pin for HoldForm-free output is absolute —
  baseline-verify it with at least three representative outputs (a list,
  a message-bearing eval, a string result).
- WL comparison from documented knowledge only.
- Stop-loss: if terminal transparency cannot reuse the existing
  output-arm logic without dragging in infix formatting, STOP and report
  the coupling.

## Report format

1. The seam: which surfaces share it, which helper was reused.
2. Test survey old→new.
3. Probe panel before/after verbatim (transparency + byte-identity +
   ToString unchanged).
4. The A/B waiver justification (diff stat, no eval-path files).
5. Verification outputs verbatim.
6. Deviations / open questions (empty if none).
