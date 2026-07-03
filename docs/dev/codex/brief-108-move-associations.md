# Brief 108 — Stage-3 batch 7: associations → rum-stdlib (data layer stays kernel-side)

## Goal

Seventh Stage-3 move (pipeline + mass-test rules per briefs 106/107, commits
d876645/de49b62). The `associations` BUILTIN family moves to `rum-stdlib`,
while the association DATA LAYER (expression-form parsing/rendering shared
with the kernel and with not-yet-moved families) STAYS kernel-side — the
`text_rendering.rs` precedent. After this brief: runtime greps clean of the
family module; `cargo test --workspace` green; CLI smoke
`--eval "Lookup[Association[1 -> 10, 2 -> 20], 2] + Length[Keys[Association[97 -> 1]]]"`
→ `21`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror briefs 106/107.
  All standing rules apply (verbatim bodies incl. in-transit free-fn
  conversion, spec identity, promotions per compiler demand ≤ small,
  path-precise gates, Option-1 test migration with the ≥50% ratio rule, full
  accounting, probes never rewritten).
- Facts (verified 2026-07-04):
  - Implementation: `runtime/associations/` — 18 files, ~5.6k lines, 35
    `eval_*` fns; specs: `builtin_catalog/associations.rs` (34 references;
    per-entry ownership by definition site as always — `MissingQ` lives here
    and its handler module must be checked before deciding it moves or stays).
  - **The split that defines this brief:** the association data layer
    (`AssociationEntryForm`, `association_entries_from_expr`,
    `association_entry_forms_from_expr`, `render_association`,
    `render_association_entry_forms_preserving_heads`, `list_items`,
    `parse_association_entry_form`, `missing_key_absent_expr`,
    `is_missing_expr`, and whatever else kernel-side code or not-yet-moved
    families reference) is SHARED MACHINERY: `eval_dispatch` checks
    `is_association_expr`, `kernel_api` re-exports the data helpers (fan-out
    ≥6 families). Decision rule: an item inside `associations/` that is
    referenced by ANY code outside the family (kernel modules, kernel_api,
    or other families) STAYS kernel-side; collect the stays VERBATIM into a
    new `runtime/association_data.rs` (or a small module dir if size demands),
    keep `kernel_api` re-exports pointing at it. Everything else (the 35
    handlers + family-only helpers) moves.
  - Kernel needs already pub from prior batches (`invalid_call_with_message`
    ×78 was promoted in brief-104; `suspend_with_frame`, `evaluate`, …).
    Expected genuinely NEW promotions ≤5; list each.
  - Tests: 13 dedicated `runtime/tests/associations*` files move whole first;
    then the measured-failure pass — `Association[` appears in 28 external
    test files, so expect the biggest migration so far (logic 31,
    predicates 33; this may be 3-5x). That is Option-1-expected, NOT a stop
    condition. Persist raw logs to /tmp before parsing.

## Scope

- MAY modify: `crates/rum-stdlib/**`;
  `crates/rum-runtime/src/runtime/associations/**` (DELETE after the split) +
  the NEW `runtime/association_data.rs` (verbatim stays); `runtime/mod.rs`;
  `builtin_catalog/associations.rs` (family-entry removals; file stays if
  other entries remain); `runtime/tests/**` per migration rules +
  `runtime/tests/mod.rs`; `kernel_api.rs` (retarget data-helper re-exports) +
  promotion definition sites/facades (visibility + one-line doc only, list
  each); `Cargo.lock` NOT expected to change.
- MUST NOT touch: other families, `eval_dispatch/` (its `is_association_expr`
  call sites keep working via the kernel-side data layer — if a dispatch file
  must change beyond imports the split is wrong; STOP), other crates, any
  `mma-hol` clone, `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **The data/builtin split is the whole risk.** Get the stay-list by
   REFERENCE ANALYSIS (grep each associations item for uses outside the
   family), not by intuition. Stays are VERBATIM relocations into
   `association_data.rs` — no rewrites, no signature changes beyond
   visibility. Paste the stay-list in the report.
2. **Bodies verbatim** through free-fn conversion and the move; message
   text/order identical (the `Missing[…]` semantics and key-arity diagnostics
   especially).
3. **Spec identity** modulo handler path; install order core-then-stdlib.
4. **Mass migration discipline**: ratio rule per file; nothing deleted or
   weakened; totals conserved (currently 1300/0); full table in the report.
   If a crate-internal MECHANISM test needs a moved builtin, use the
   test-local registration pattern (brief-107's SameQ precedent) and list it.
5. Path-precise grep gates only (brief-107 lesson).
6. No microbench (ownership move only — state it).
7. Stop-loss: >5 genuinely new promotions, any eval_dispatch change beyond
   imports, unattributable non-target failures, same compile error twice →
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "runtime/associations\|mod associations\|associations::" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits (association_data.rs is a different path)
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                                          # report (163 + moved)
cargo run -p rum-cli --release -- --eval "Lookup[Association[1 -> 10, 2 -> 20], 2] + Length[Keys[Association[97 -> 1]]]" 2>/dev/null | tail -1   # expect 21
```

Paste all outputs verbatim + workspace accounting + the FULL migration table +
the data-layer stay-list.

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

1. What changed — the data-layer stay-list (item → outside referencer), moved
   file map, catalog split, promotion inventory, the FULL migration table.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health; migration size vs 31/33; advice for
   structure_ops/functional_ops/control_flow/rewrite_builtins (arithmetic
   last).
4. Stop-loss / open questions (empty if none).
