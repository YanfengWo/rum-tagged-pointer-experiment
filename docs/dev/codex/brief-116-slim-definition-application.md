# Brief 116 — file-slimming batch 3: definition_application.rs (55KB, HOTTEST) — value-application domain split

## Goal

Third slimming batch. Split
`crates/rum-runtime/src/runtime/definition_application.rs` (55,569 chars,
1488 lines — ONE `impl Session` block of 1423 lines + tail helpers) into a
`definition_application/` directory per brief-114's SPLITTING PRINCIPLES
(read them; split map first). This is the HOTTEST file of the line — the
down/up/sub-value dispatch and fast-match layer that every definition
application crosses (the Pair-descent probes live here historically). Zero
behavior change; workspace green (1301/0); every new file ≤20k; microbench
A/B required (you run it, 115-style).

## Context pointers

- Read `AGENTS.md`, `docs/code-organization-guidelines.md`, brief-114
  principles, brief-115 (the hot-split precedent incl. the STALE-BINARY
  lesson: rebuild release rum-cli before the post-edit bench run).
- Measured method-domain structure (verify before mapping):
  1. VALUE-APPLICATION entries: `try_apply_down_values_from_symbol`,
     `try_apply_up_values_from_list`, `try_apply_sub_values_from_symbol`,
     direct/exact default-value handling, `finish_definition_rhs_evaluation`;
  2. FAST-MATCH layer (the hottest): `normalize_definition_candidate_for_matching`,
     `definition_lhs_could_match_candidate`, `matcher_could_match_candidate_shape`,
     `pattern_args_may_have_variable_arity`, `try_fast_definition_match`,
     `try_fast_definition_match_compiled`, `fast_match_compiled`,
     `fast_match_definition_pattern`;
  3. REPLACE-ALL drivers: `apply_replace_all_owned`, `apply_replace_all_after_miss`,
     `apply_replace_all_args`, `apply_rules_once_from_list` (consumed by the
     rewrite_data wrappers);
  4. trace + tail profile-outcome helpers + `default_value_lhs` /
     `exact_down_value_rhs` free fns.
- Suggested map (verify cohesion; adjust with justification in the map):
  `mod.rs` (facade + shared/tail helpers), `value_application.rs` (domain 1),
  `fast_match.rs` (domain 2), `replace_all.rs` (domain 3). Splitting one
  `impl Session` block into multiple `impl Session` blocks across files is
  fine and idiomatic — method bodies VERBATIM.
- HOT RED LINES: bodies verbatim, no new indirection, the fast-match layer's
  early-out ordering untouched. Same-crate boundaries don't block inlining —
  prove with the A/B anyway.

## Microbench A/B (you run it; 115-style)

Base BEFORE any edit; REBUILD `cargo build --release -p rum-cli` before the
post-edit run (stale-binary lesson); one suspicious-label rerun allowed.
Judge on: downvalue-* (all six), do-small, module-local, replaceall-simple,
matchq-*. Machine noise is ±10-15% on repeated cycles — consistent direction
only.

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_116_micro_base
# ... split ...
cargo build --release -p rum-cli
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_116_micro_new
```

## Scope

- MAY modify: `definition_application.rs` → `definition_application/`
  (directory); `use` lines inside it; `runtime/mod.rs` only if declaration
  syntax demands (likely zero).
- MUST NOT touch: any caller outside the directory, any behavior,
  `Cargo.toml`/`Cargo.lock`, any `mma-hol` clone, `docs/**` except your
  report, `.git`.

## Verification (MANDATORY)

```
cargo fmt --check
cargo test --workspace
ls crates/rum-runtime/src/runtime/definition_application.rs 2>&1   # No such file
find crates/rum-runtime/src/runtime/definition_application -name "*.rs" | while read f; do wc -c "$f"; done | sort -rn
git status --short -- crates/                                       # exactly: 1 deletion + new directory
```

Plus the A/B table with judgment. Every new file ≤20k (justify exceptions).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Nothing outside this repo, no network; only Scope edits, `cargo`, the
  microbench script, `grep`, `python3`. Never edit any `mma-hol` file.
  **No new `unsafe`.**
- Probe budget: two A/B runs + one rerun.
- Stop-loss: consistent regression on the judged labels, a boundary that
  can't keep callers unchanged, same compile error twice → STOP and report.

## Report format

1. SPLIT MAP (module → responsibility → chars; DAG; hot-layer isolation
   note).
2. What changed — inventory; untouched-callers evidence.
3. Verification — verbatim outputs incl. size + A/B tables.
4. Verdict — red-line assertion; advice for the remaining debt files
   (vm_rhs_template 53k, arithmetic 42k, lexer 41k, 2 test files).
5. Stop-loss / open questions.
