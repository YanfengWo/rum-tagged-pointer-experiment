# Brief 110 — Stage-3 batch 9: functional_ops → rum-stdlib

## Goal

Ninth Stage-3 move (pipeline + rules per briefs 106-109, commits
d876645/de49b62/f01e1b1/1447894). The `functional_ops` BUILTIN family
(Map/Fold/Nest/Select/Apply/Outer/…) moves to `rum-stdlib`; anything
referenced by kernel-side code stays kernel-side per reference analysis (the
brief-108/109 split precedent). After this brief: runtime greps clean of the
family module; `cargo test --workspace` green; CLI smoke
`--eval "Fold[Plus, 0, {1, 2, 3, 4}] + Length[Select[{1, 2, 3, 4}, EvenQ]]"` → `12`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror briefs 108/109.
  All standing rules apply (verbatim bodies + in-transit free-fn conversion,
  spec identity, path-precise gates, Option-1 test migration with ratio rule,
  probes never rewritten, test-local registrations listed,
  behavior-equivalence adaptations reported individually, stays are verbatim
  relocations with the stay-list pasted).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/functional_ops/` — 27 files, ~6.1k lines, 42
    `eval_*` fns. Specs: `builtin_catalog/functional.rs` (43 references;
    ownership by definition site).
  - No kernel-side PATH references to `functional_ops::` exist outside the
    glob — but glob-world bare-name uses don't grep; reference analysis
    decides the stay-list as always. KNOWN boundary: the pure-function apply
    machinery (`try_apply_function`, `parse_function`, `parse_composition`,
    slot substitution, callable-operator wrappers) is EVALUATOR semantics
    living in kernel modules (`function_and_subst_helpers.rs`,
    `slot_substitution.rs`) — it is NOT part of this family and MUST NOT
    move. If any functional_ops file turns out to be part of that machinery,
    it stays (new `runtime/functional_data.rs` only if needed).
  - `targeted_map.rs` depends on kernel exact-path helpers — already promoted
    via kernel_api in brief-109; plain imports now.
  - This family is the most SUSPENSION-DENSE (resumable Map/Fold/Nest/Scan
    continuations). Byte-stable continuation shapes are the top verbatim
    concern.
  - Promotion stop line: ≤8 genuinely new items (audited 15 kernel imports).
    STOP AT 9, even mid-edit (brief-108 lesson).
  - Tests: `runtime/tests/functional_ops/` (partially migrated already —
    coordinate with the existing stdlib modules incl.
    `incidental_apply_map`/`incidental_callable`), plus
    `session_requests/functional_*` resume files already partially in stdlib.
    Expect a large migration (Map/Select are common probes). Ratio rule;
    persist raw logs first.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/functional_ops/**` (DELETE after the split)
  + NEW `runtime/functional_data.rs` (only if reference analysis demands);
  `runtime/mod.rs`; `builtin_catalog/functional.rs` (family-entry removals);
  `runtime/tests/**` per migration rules + `runtime/tests/mod.rs`;
  `kernel_api.rs` retargets + promotion sites/facades (visibility + one-line
  doc only, list each); `Cargo.lock` NOT expected to change.
- MUST NOT touch: other families, `eval_dispatch/`,
  `function_and_subst_helpers.rs`/`slot_substitution.rs` beyond
  compiler-forced import retargets (list each; if apply-machinery LOGIC needs
  changes the split is wrong — STOP), other crates, any `mma-hol` clone,
  `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Suspension shapes are the crown jewels here**: every
   `suspend_with_frame`/`drive_*` continuation in resumable
   Map/Fold/Nest/Scan/Select moves VERBATIM. Any resume-shape change is a
   defect even if tests stay green.
2. **The apply machinery stays** — Function/&-slots/Composition application
   is evaluator semantics, not a builtin family.
3. **Bodies verbatim**; message text/order identical.
4. **Spec identity** modulo handler path; install order core-then-stdlib.
5. **Mass migration discipline**: ratio rule, nothing deleted/weakened,
   totals conserved (currently 1300/0), full table, test-local registrations
   listed.
6. No microbench (ownership move only — state it).
7. Stop-loss: >8 genuinely new promotions (STOP AT 9), apply-machinery logic
   changes, unattributable non-target failures, same compile error twice →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod functional_ops\|functional_ops::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                       # report (234 + moved)
cargo run -p rum-cli --release -- --eval "Fold[Plus, 0, {1, 2, 3, 4}] + Length[Select[{1, 2, 3, 4}, EvenQ]]" 2>/dev/null | tail -1   # expect 12
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the stay-list (empty stay-list is a valid finding — say so explicitly).

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

1. What changed — stay-list (or its explicit emptiness), moved file map,
   catalog split, promotion inventory, FULL migration table.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; migration size; advice for control_flow +
   rewrite_builtins (the highest-coupling finale before arithmetic).
4. Stop-loss / open questions (empty if none).
