# Brief 105 — Stage-3 batch 4: the strings family → rum-stdlib

## Goal

Fourth Stage-3 move (pipeline per briefs 102/103/104, commits
47716e6/5566c5c/a0cfa6a). Single family this batch because it is the largest so
far: `strings` = facade `runtime/strings.rs` (194 lines) + `runtime/strings/`
(21 files, ~6.5k lines: `encoding.rs`, `patterns.rs` + `patterns/`,
`similarity/`, `template.rs`, `text.rs` + `text/`), 66 `eval_*` fns. After this
brief: `rum-stdlib` owns the family; `rum-runtime` greps clean of it;
`cargo test --workspace` green; CLI smoke
`--eval "StringLength[StringJoin[{\"ab\", \"cd\"}]] + ToExpression[\"3*4\"]"` → `16`.

## Context pointers

- Read `AGENTS.md`, `docs/crate-rebalance-plan.md`; mirror the prior move
  briefs' structure. All established rules apply: bodies VERBATIM
  (receiver-only conversion), spec identity modulo handler path, promotion
  strictly per compiler demand (+ one-line doc), facade per-item `pub use`,
  handler-vs-helper by catalog reference, family-internal items do NOT route
  through kernel_api, multi-file layout preserved (do not flatten).
- Facts (verified 2026-07-04):
  - NO external-crate baggage: Compress/Hash/etc. are folk implementations —
    rum-runtime has no flate2/sha2-style deps to carry over.
  - `strings` DOES use the parser (`parse_expr` in `encoding.rs` and
    `text/builtins.rs` — ToExpression/SyntaxQ): add `rum-syntax` (workspace
    path dep, style-matched to rum-runtime's Cargo.toml) to
    `crates/rum-stdlib/Cargo.toml`. That plus the resulting `Cargo.lock`
    update are the ONLY sanctioned manifest changes.
  - Expected kernel-method promotions (sampled; the compiler is the referee,
    list every one): `suspend_with_frame` (the suspension API head — promote
    directly with a real doc comment, do NOT wrap it),
    `lower_syntax_expr` (already `pub` via the crate root — verify),
    `restore_transport_expr`, plus already-pub `evaluate`/`invalid_call_*`.
  - Specs live in `builtin_catalog/strings.rs` (61 handler references) — but
    catalog file grouping ≠ family grouping (standing lesson): delete ONLY the
    entries whose handlers are defined in the strings implementation module;
    any other family's entries in that catalog file stay (report the split).
    Runtime catalog count: 482 → report the measured result; stdlib count
    31 → 31 + moved.
  - Tests: `runtime/tests/strings/` (a directory) moves whole; the family's
    builtins also appear incidentally in ~5-9 other runtime test files
    (`StringJoin[` 5, `ToString[` 9, `StringLength[` 5) — apply the
    measured-failure rule (run `cargo test -p rum-runtime` after the move;
    failing fns move whole to stdlib test modules). Account for every one.

## Scope

- MAY modify: `crates/rum-stdlib/**` (incl. Cargo.toml per above);
  `Cargo.lock` (the rum-syntax edge only);
  `crates/rum-runtime/src/runtime/strings.rs` +
  `crates/rum-runtime/src/runtime/strings/**` (DELETE); `runtime/mod.rs`;
  `builtin_catalog/strings.rs` (family-entry removals; file may stay with
  other families' entries); `runtime/tests/strings/**` (MOVE out),
  `runtime/tests/mod.rs`, and runtime test files with MEASURED failing
  functions (removals of exactly those fns); promotion definition sites +
  `kernel_api.rs` + facades (visibility + one-line doc only, list each).
- MUST NOT touch: any other family, `eval_dispatch/`, `pattern_matching/`
  (the string-pattern compiler inside the strings family moves WITH it; the
  kernel expression-pattern matcher stays — if the two turn out to be
  entangled, STOP and report rather than splitting kernel code), other crates,
  any `mma-hol` clone, `docs/**` except your report, `.git`.

## rum / Rust pitfalls (read twice)

1. **Bodies verbatim** — especially template.rs (StringTemplate/TemplateApply)
   and the similarity metrics; message text/order identical.
2. **ToString/TextString formatting**: they render via kernel formatting
   machinery — whatever they call gets promoted per demand; do not copy or
   fork any formatting code into the stdlib.
3. **Suspension-safety**: several string builtins suspend (`suspend_with_frame`
   sampled in text.rs) — continuation shapes move verbatim.
4. **Test migration by measured failure**; both suites green; totals accounted
   (currently 1299/0).
5. No microbench (ownership move only — state it).
6. **Stop-loss thresholds for THIS batch**: >12 NEW kernel promotions, or >40
   failing runtime test fns to migrate, or the string-pattern/kernel-matcher
   entanglement above → STOP and report with the measured numbers (that is a
   design decision point, not a grind-through).

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
grep -rn "mod strings\|strings::\|eval_string_join\|eval_to_expression" crates/rum-runtime/src/ | grep -v "^Binary"   # expect ZERO hits
grep -c "BuiltinSpec::" crates/rum-stdlib/src/catalog.rs                                                               # report the count
cargo run -p rum-cli --release -- --eval "StringLength[StringJoin[{\"ab\", \"cd\"}]] + ToExpression[\"3*4\"]" 2>/dev/null | tail -1   # expect 16
```

Paste all outputs verbatim + workspace-totals accounting + moved-test inventory
+ the catalog-file split (moved vs stayed entries).

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

1. What changed — moved file map, catalog split, spec counts, promotion
   inventory (item → demander), moved-test inventory.
2. Verification — each command with exact verbatim output.
3. Verdict — batch health + the measured numbers vs the stop-loss thresholds;
   anything the remaining 7 families should know.
4. Stop-loss / open questions (empty if none).
