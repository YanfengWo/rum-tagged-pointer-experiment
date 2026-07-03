# Brief 109 — Stage-3 batch 8: structure_ops → rum-stdlib (Part machinery may stay)

## Goal

Eighth Stage-3 move (pipeline + rules per briefs 106-108, commits
d876645/de49b62/f01e1b1, incl. the brief-108 data/builtin-split precedent).
The `structure_ops` BUILTIN family moves to `rum-stdlib`; any structure/Part
machinery referenced by kernel-side code or not-yet-moved families STAYS
kernel-side (new `runtime/structure_data.rs` if needed — reference analysis
decides, not intuition). After this brief: runtime greps clean of the family
module; `cargo test --workspace` green; CLI smoke
`--eval "Length[Flatten[{{1, 2}, {3}}]] + First[Sort[{9, 4, 7}]]"` → `7`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror brief-108's
  structure INCLUDING its stay-list method. All standing rules apply
  (verbatim bodies + in-transit free-fn conversion, spec identity,
  path-precise gates, Option-1 test migration, probes never rewritten,
  test-local registration precedent for crate-internal mechanism tests,
  behavior-equivalence adaptations like "registered spec == known" are legit
  and must be reported individually).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/structure_ops/` — 27 files, ~6.3k lines, 47
    `eval_*` fns. Specs: `builtin_catalog/structure.rs` (38 references;
    ownership by definition site).
  - EXPECT kernel entanglement around Part/Extract/ReplacePart and the
    part-path selectors: `exact_position_paths_from_part_selector_exprs` was
    already promoted (brief-108) and Part-lvalue assignment lives in kernel
    definitions code. Items referenced from outside the family stay
    kernel-side, verbatim.
  - Incidental test footprint: Length/First/Sort/Flatten are everywhere —
    expect the LARGEST migration of the line (associations was ~62 fns).
    That is Option-1-expected; not a stop condition. Persist raw logs first.
  - Promotion stop line for THIS family: ≤8 genuinely new items (sized to its
    audited 15 kernel imports). **Stop means stop**: brief-108 finished at
    6/5 instead of stopping — if you hit 9, you STOP with the inventory and
    report, even mid-edit.
- Note `tests/structure_ops/` has already been partially migrated by earlier
  batches (join/selectors/surface/replace/ordering_sets fragments live in
  stdlib) — coordinate module names with the existing stdlib test tree.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/structure_ops/**` (DELETE after the split)
  + NEW `runtime/structure_data.rs` (verbatim stays, only if reference
  analysis demands); `runtime/mod.rs`; `builtin_catalog/structure.rs`
  (family-entry removals); `runtime/tests/**` per migration rules +
  `runtime/tests/mod.rs`; `kernel_api.rs` retargets + promotion sites/facades
  (visibility + one-line doc only, list each); `Cargo.lock` NOT expected to
  change.
- MUST NOT touch: other families, `eval_dispatch/`,
  `definition_application`/`definitions` beyond compiler-forced import/path
  retargets (list each; if Part-lvalue LOGIC needs changes the split is wrong
  — STOP), other crates, any `mma-hol` clone, `docs/**` except your report,
  `.git`.

## rum / Rust pitfalls (read twice)

1. **The stay-list is reference-analysis-driven**; paste it (item → outside
   referencer). Stays are verbatim relocations.
2. **Bodies verbatim**; message text/order identical (Take/Drop/Part
   out-of-range diagnostics especially).
3. **Spec identity** modulo handler path; install order core-then-stdlib.
4. **Mass migration discipline**: ratio rule, nothing deleted/weakened,
   totals conserved (currently 1300/0), full table; test-local registrations
   listed.
5. Suspension-safety: the resumable structure ops (sort-by, select-first,
   …) keep their continuation shapes byte-stable.
6. No microbench (ownership move only — state it).
7. Stop-loss: >8 genuinely new promotions (STOP AT 9, even mid-edit), any
   Part-lvalue logic change, unattributable non-target failures, same
   compile error twice → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod structure_ops\|structure_ops::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                     # report (196 + moved)
cargo run -p rum-cli --release -- --eval "Length[Flatten[{{1, 2}, {3}}]] + First[Sort[{9, 4, 7}]]" 2>/dev/null | tail -1   # expect 7
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the stay-list.

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

1. What changed — stay-list, moved file map, catalog split, promotion
   inventory, FULL migration table.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; migration size; advice for functional_ops (next).
4. Stop-loss / open questions (empty if none).
