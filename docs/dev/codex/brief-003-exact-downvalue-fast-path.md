# Brief 003 — Exact (ground) downvalue fast path (implementation)

## Goal

`downvalue-exact-hit` is the worst Stage 0 primitive (~`7.76x` WK). A literal
downvalue like `fExact[1] = 2` is dispatched through the full pattern machinery
every call. A per-phase timing split (`RUM_PROFILE_LEXICAL_SUBSTITUTION=1`,
`rum-profile runtime-path-self-timing`) shows the two fattest dispatch phases
for an exact hit are `down_values.fast_match` (~`267 ns/call`) and
`down_values.prefilter` (~`70 ns/call`) — and both are redundant for a LHS that
contains no pattern syntax: such a LHS matches a candidate iff they are
structurally equal, with empty bindings. Add a fast path that, for a
pattern-free (ground) downvalue LHS, decides the match with a single
`Expr` equality and skips the shallow-signature prefilter and the matcher
entirely. This must be provably equivalent and must not change any dispatch
result or ordering.

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` (the corrected
  baseline) first. The diagnosis is in `PROGRESS.md`
  "Brief-002 Diagnosis: downvalue-exact-hit".
- Dispatch entry: `try_apply_down_values_from_symbol` in
  `crates/rum-runtime/src/runtime/definition_application.rs` (around line 80).
  The current per-entry flow is: get `down_value` → prefilter
  (`definition_lhs_could_match_candidate`) → `try_fast_definition_match` →
  matched branch (substitute RHS, option scope, trace, `evaluate_definition_rhs`).
- `DownValue` struct: `crates/rum-runtime/src/runtime/model.rs` (line 686).
- The ONLY `DownValue { ... }` construction site is
  `crates/rum-runtime/src/runtime/definition_tags.rs` (~line 204).
- Helpers that already exist: `strip_hold_pattern` (in
  `function_and_subst_helpers.rs`) and `expr_contains_pattern_syntax` (in
  `model.rs`, ~line 822 — make it `pub(super)` if it is not already reachable
  from the construction site).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model.rs` (add the `DownValue` field; widen
    `expr_contains_pattern_syntax` visibility if needed),
  - `crates/rum-runtime/src/runtime/definition_tags.rs` (compute the field at
    the one construction site),
  - `crates/rum-runtime/src/runtime/definition_application.rs` (the fast path),
  - a runtime test module under
    `crates/rum-runtime/src/runtime/tests/` for the new regression.
- MUST NOT change: the `Expr`/`Symbol`/serde shapes, the matcher compiler, the
  non-ground dispatch path, any `mma-hol` clone, docs, `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `.git`.
- Do NOT add unconditional default-path cost beyond the one precomputed `bool`
  field and one equality check; no new per-call allocation.

## Design (implement exactly this; idiomatic Rust is yours)

1. **Precompute a ground flag on `DownValue`.** Add
   `pub(super) lhs_is_ground: bool`. At the construction site, compute it as
   `!expr_contains_pattern_syntax(strip_hold_pattern(&lhs))`. A ground LHS has
   no `Blank`, `Pattern`, `Condition`, `PatternTest`, `Alternatives`,
   `Optional`, `Except`, sequence, or key-value pattern syntax anywhere — it is
   a literal call like `fExact[1]`.

2. **Fast path in `try_apply_down_values_from_symbol`.** Immediately after
   obtaining `down_value` for the current `index` (BEFORE the prefilter and
   `try_fast_definition_match`), if `down_value.lhs_is_ground`:
   - compute `matched = strip_hold_pattern(&down_value.lhs) == &normalized_candidate`;
   - if NOT matched: `return self.try_apply_down_values_from_symbol(candidate, normalized_candidate, symbol, index + 1)` (exactly what a prefilter miss does);
   - if matched: this is an exact match with EMPTY bindings. Produce the RHS the
     same way the existing matched branch does — `substitute_rule_bindings_for_rhs(&down_value.rhs, &SymbolExprMap::default(), down_value.rhs_may_splice_without_bindings)`,
     then `option_value_scope_for_definition`, `push_rule_applied_trace(RuleApplicationSource::DownValue, ...)`, `record_profile_down_value_application(&symbol)`, and `evaluate_definition_rhs(option_scope, substituted).map(Some)`. Return it.
   - The profiling-only `runtime_path_timing` / `definition_dispatch_phase_timing`
     hooks may be omitted on the fast path, but KEEP
     `record_profile_down_value_application` so counters stay correct.
   - The non-ground branch (prefilter + `try_fast_definition_match` + compiled
     matcher) stays exactly as it is today.

3. **Why this is correct (state it in your report):** a LHS with no pattern
   syntax matches a candidate if and only if they are structurally equal, and
   then the binding set is empty and the substituted RHS is the stored RHS.
   Ground downvalues are also maximally specific, so they are stored ahead of
   any pattern downvalue (`replace_or_insert_definition_by_specificity`); taking
   the ground match at this index is the same entry the scan would have taken.

## Tests (add a focused regression)

Add runtime tests proving the fast path does not change semantics:

- exact hit returns the stored RHS: `f[1] = 10; f[1]` → `10`;
- exact miss falls through: `f[1] = 10; f[2]` stays `f[2]` (unevaluated);
- exact beats a later pattern: `f[x_] := x + 1; f[1] = 10; f[1]` → `10`, and
  `f[2]` → `3` (pattern still applies to non-exact args);
- a delayed exact downvalue still works: `g[1] := 10; g[1]` → `10`;
- a conditional/pattern LHS is NOT treated as ground: `h[1] /; True := 5; h[1]`
  → `5` (goes through the matcher, not the fast path).

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green, including the existing definition precedence / dispatch tests and
your new regression. Then the performance gate (profiling env OFF):

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_003_micro
```

Paste the `ratios.tsv` rows for `downvalue-exact-hit` AND its same-tier
neighbors (`downvalue-exact-miss`, `downvalue-pattern-hit`,
`downvalue-pattern-miss`) BEFORE (from the committed baseline note, ~`7.76x`)
and AFTER. The target ratio must drop with NO same-tier regression. If it does
not drop, STOP and report the numbers — do not keep tuning.

## rum / Rust pitfalls (read twice)

1. Build/measure the RELEASE binary; profiling env OFF for any timing number.
2. The fast path must compare the STRIPPED lhs (`strip_hold_pattern`) against
   the normalized candidate — match the existing matched semantics.
3. Keep `record_profile_down_value_application(&symbol)` on the fast path or the
   counters and the brief-002 diagnosis stop reconciling.
4. One precomputed `bool` + one equality on the hot path; no new allocation, no
   `Mutex`, nothing paid when the field is `false`.
5. Do not reorder or alter how downvalues are stored or their specificity.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; leave changes in the
  working tree.
- Nothing outside Scope; if you must, STOP and report.
- Full machine access, repo + `/tmp` only, no network. Commands: `cargo`,
  release `rum-cli`, the microbench script, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: if `cargo test` fails the same way twice, or the ratio does not
  move, STOP and report.
- Minimal diff. Paste the real `cargo test` summary and the real ratio rows; if
  a run did not reach green, say so — never fake it.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument for the ground fast path (point 3 above).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, and the before/after `downvalue-exact-hit` + same-tier neighbor
   ratios.
4. Stop-loss / open questions (empty if none).
