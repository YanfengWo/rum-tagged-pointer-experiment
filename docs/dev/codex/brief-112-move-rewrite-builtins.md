# Brief 112 — Stage-3 batch 11: rewrite_builtins → rum-stdlib (the pattern engine stays)

## Goal

Eleventh Stage-3 move (pipeline + rules per briefs 106-111, incl. the
brief-111 amendments: mechanism tests MAY stay runtime-side with test-local
registrations, listed per batch; the control_data wrapper shape is an
approved solution when reference analysis shows the implementation is
evaluator machinery). The `rewrite_builtins` family (Replace/ReplaceAll/
ReplaceRepeated/ReplaceList/Cases/DeleteCases/… — whatever its module
actually owns) moves to `rum-stdlib` in whichever of the two proven shapes
fits: full implementation move (briefs 108-110 shape) or spec-ownership move
with kernel-side implementation + thin wrappers (brief-111 shape). The
PATTERN ENGINE (`pattern_matching/**`, `definition_application`,
`binding_substitution`, tree walkers) does not move under any outcome. After
this brief: `cargo test --workspace` green; CLI smoke
`--eval "Length[Cases[{1, a, 2, b}, _Integer]] + First[{x} /. x -> 5]"` → `7`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror briefs 108-111.
- Facts (verified 2026-07-04):
  - Implementation: `runtime/rewrite_builtins.rs` (149-line facade) +
    `runtime/rewrite_builtins/` (10 files: `cases.rs`+`cases/`,
    `replace.rs`+`replace/`), 7 `eval_*` fns.
  - Specs: `builtin_catalog/rules.rs` (37 references) is a MIXED file:
    `Set`/`SetDelayed`/`UpSet`/`Rule`/`RuleDelayed`-style definition
    machinery specs and the Part family (brief-109) STAY; only entries whose
    handlers live in the rewrite_builtins module move (expected ~7-10;
    ownership by definition site; report the split).
  - Audited coupling: 26 kernel imports, the highest ratio of the line — the
    family is a thin interface over the matcher. Choose the solution shape by
    reference analysis: if the bodies are mostly matcher-internal calls that
    would demand >10 promotions, take the brief-111 wrapper shape instead
    (facade enum + one promoted dispatch method); if they compile against
    already-public kernel_api with ≤10 new promotions, do the full move.
    State the decision and its evidence FIRST in the report.
  - Suspension: ReplaceAll/ReplaceRepeated drive `drive_internal_value`
    (already pub); resumable shapes verbatim.
  - Tests: dedicated `definitions_and_rules_cases`-style files are already
    partially migrated; ratio rule for the rest; mechanism tests may stay
    with test-local registrations (list each).
  - Promotion stop line: ≤10 genuinely new items under the full-move shape
    (STOP AT 11 and switch to the wrapper shape or report); ≤3 under the
    wrapper shape.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/rewrite_builtins*` (DELETE or rename to a
  kernel-side `rewrite_data`-style home per the chosen shape);
  `runtime/mod.rs`; `builtin_catalog/rules.rs` (family-entry removals only —
  the definition-machinery and Part entries stay); `runtime/tests/**` per
  migration rules + `runtime/tests/mod.rs`; `kernel_api.rs` retargets +
  promotion sites/facades; test-local registrations; `Cargo.lock` NOT
  expected to change.
- MUST NOT touch: `pattern_matching/**`, `definition_application`,
  `binding_substitution`, `tree_walkers` beyond compiler-forced import
  retargets (list each; matcher LOGIC changes = the split is wrong — STOP),
  other families, `eval_dispatch/`, other crates, any `mma-hol` clone,
  `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **The matcher is untouchable.** Replace*/Cases semantics (rule
   normalization, level specs, hold behavior) byte-stable; message
   text/order identical.
2. **Bodies verbatim** whichever shape; wrapper shape delegates, never
   reimplements.
3. **Spec identity** modulo handler path; install order core-then-stdlib.
4. **Migration discipline**: ratio rule, nothing deleted/weakened, totals
   conserved (currently 1300/0), full table, test-local registrations listed.
5. No microbench UNLESS you choose the wrapper shape AND the wrapped
   handlers include hot-path ones — ReplaceAll IS hot (mma-hol rewriting);
   if wrapper shape is chosen, SAY SO in the report and Claude will run the
   A/B at the gate (you do not need to run it; keep the probe budget).
6. Stop-loss: promotion lines above, matcher-logic changes, unattributable
   non-target failures, same compile error twice → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod rewrite_builtins\|rewrite_builtins::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                           # report (311 + moved)
cargo run -p rum-cli --release -- --eval "Length[Cases[{1, a, 2, b}, _Integer]] + First[{x} /. x -> 5]" 2>/dev/null | tail -1   # expect 7
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the catalog split + the shape decision with evidence.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Clean-check probes: `git status --short -- crates/`.
- Minimal diff; paste real verbatim outputs; never fake a green.

## Report format (your final message)

1. The shape decision (full move vs wrapper) with the reference-analysis
   evidence, FIRST.
2. What changed — stay-list/moved map, catalog split, promotion inventory,
   FULL migration table, test-local registrations.
3. Verification — each command with exact verbatim output.
4. Verdict — batch health; whether the gate needs a microbench (wrapper +
   hot handlers?); advice for arithmetic (the finale).
5. Stop-loss / open questions (empty if none).
