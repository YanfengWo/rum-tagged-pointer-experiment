# Brief 007 — Drive the fast match from the precompiled `MatcherNode` (implementation)

## Goal

`downvalue-pattern-hit` (~`7.2x`) is the worst primitive. Brief-006 found (and
the director gate independently confirmed) that `fast_match_definition_pattern`
spends `42-62%` of its time RE-PROBING the raw pattern `Expr` through the
~11-step `pattern_*` predicate chain every call — even though a precompiled
`MatcherNode` already sits on every `DownValue`/`UpValue`/`SubValue`. Add a
compiled fast matcher that reads the `MatcherNode` directly (no predicate
chain), producing identical bindings, and use it on the dispatch fast path when
the matcher is current. Fall back to the existing raw-`Expr` matcher unchanged
for the cases the fast compiled path does not handle.

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` first. Diagnosis is
  in `PROGRESS.md` "Brief-006 Diagnosis: pattern-matcher cost".
- The existing raw-`Expr` matcher is `fast_match_definition_pattern`
  (`crates/rum-runtime/src/runtime/definition_application.rs` ~line 895) and its
  wrapper `try_fast_definition_match` (~line 876). MIRROR its case logic exactly.
- `MatcherNode` and `MatcherArg` are in
  `crates/rum-runtime/src/runtime/session_state.rs` (`MatcherNode` ~line 170,
  `MatcherArg` ~line 204). Each `DownValue`/`UpValue`/`SubValue` carries
  `matcher: MatcherNode` and `matcher_attribute_revision: u64`.
- Reuse `match_blank(head_constraint: Option<&Expr>, value: &Expr) -> bool`
  (`matcher_and_symbols.rs`) and `bind_pattern_name(name: &Symbol, value: Expr,
  bindings: &mut SymbolExprMap) -> bool` (`binding_substitution.rs`).
- The 3 fast-path call sites are in `definition_application.rs` at ~lines 166
  (down), 449 (up), 653 (sub).

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/definition_application.rs` (the new compiled
  matcher + the 3 call-site swaps), and a runtime test module under
  `crates/rum-runtime/src/runtime/tests/`.
- MUST NOT change `MatcherNode`/`MatcherArg`, the matcher compiler, the raw
  `fast_match_definition_pattern` (keep it as the fallback), `Expr`/`Symbol`,
  docs, `AGENTS.md`, `PROGRESS.md`, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

1. **Add `fast_match_compiled(&self, matcher: &MatcherNode, value: &Expr,
   bindings: &mut SymbolExprMap) -> Option<bool>`** mirroring
   `fast_match_definition_pattern` case-for-case, but reading the node:
   - `MatcherNode::Exact(p)` → `Some(p == value)`.
   - `MatcherNode::Blank { head_constraint }` →
     `Some(match_blank(head_constraint.as_ref(), value))`.
   - `MatcherNode::Named { name, inner }` → recurse `fast_match_compiled(inner,
     value, bindings)?`; if false return `Some(false)`; else
     `Some(bind_pattern_name(name, value.clone(), bindings))`.
   - `MatcherNode::Call { head, args }` → require `value` to be a
     `Expr::Call { head: vhead, args: vargs }` (else `Some(false)`); require
     EVERY arg to be `MatcherArg::Single(_)` (else return `None` — variable
     arity, fall to the full matcher); require `args.len() == vargs.len()`
     (else `Some(false)`); replicate the existing head-attribute guard — if the
     value head's attributes contain `Flat`, `Orderless`, or `OneIdentity`,
     return `None`; then recurse `fast_match_compiled(head, vhead, bindings)`
     and each `Single` arg against the matching value arg, short-circuiting on
     `false`/`None`.
   - `MatcherNode::Condition | PatternTest | Except | Alternatives |
     KeyValuePattern` → `None` (these need the full matcher; do NOT attempt
     them here).
   - Any `MatcherArg` that is not `Single` → `None`.

2. **Add `try_fast_definition_match_compiled(&self, matcher: &MatcherNode,
   candidate: &Expr) -> Option<CompiledMatchOutcome>`** mirroring
   `try_fast_definition_match` but calling `fast_match_compiled`.

3. **Swap the 3 call sites** (down/up/sub). Replace
   `self.try_fast_definition_match(&X.lhs, &normalized_candidate)` with:
   use the compiled path WHEN the matcher is current
   (`X.matcher_attribute_revision == self.attribute_revision`), else keep the
   existing raw-`Expr` call:

   ```
   let fast_outcome = if X.matcher_attribute_revision == self.attribute_revision {
       self.try_fast_definition_match_compiled(&X.matcher, &normalized_candidate)
   } else {
       self.try_fast_definition_match(&X.lhs, &normalized_candidate)
   };
   ```

   Everything downstream of `fast_outcome` stays as is.

4. **Why this is correct (state it in your report):** the `MatcherNode` is
   compiled by `compile_matcher` from the same LHS the raw prober reads, and
   `fast_match_compiled` mirrors `fast_match_definition_pattern` case-for-case,
   producing the same bindings and the same `None`/`false` boundaries, so the
   compiled path returns exactly what the raw path would for the cases it
   handles and defers (returns `None`) for the rest. Gating on
   `matcher_attribute_revision` ensures the node reflects current attributes.

## Tests (add focused regressions)

- exact pattern hit binds and rewrites: `f[x_Integer] := x + 1; f[1]` → `2`;
- typed-blank miss falls through: `f[x_Integer] := x + 1; f["a"]` stays
  unevaluated (`Blank[Integer]` does not match a string);
- named multi-arg: `g[x_, y_] := x - y; g[5, 2]` → `3`;
- exact-vs-pattern precedence still holds: `h[x_] := x + 1; h[1] = 10; h[1]` →
  `10`, `h[2]` → `3`;
- fall-through cases STILL work through the full matcher (these hit the `None`
  branches): a `Condition` `p[x_] /; x > 0 := x; p[3]` → `3`, `p[-1]` stays
  unevaluated; a sequence `q[x__] := {x}; q[1, 2, 3]` → `{1, 2, 3}`; an
  `Alternatives` `r[1 | 2] := ok; r[2]` → `ok`.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green, including the existing matcher/precedence tests and your new
regressions. Then the performance gate (profiling OFF; a clean same-session A/B
if you can, else cross-run and say so):

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_007_micro
```

Paste the `ratios.tsv` rows for `downvalue-pattern-hit` AND same-tier neighbors
(`downvalue-pattern-miss`, `downvalue-exact-hit`, `matchq-integer-success`,
`matchq-integer-fail`). `downvalue-pattern-hit` must drop with NO same-tier
regression. If it does not drop, STOP and report.

## rum / Rust pitfalls (read twice)

1. Measure the RELEASE binary; profiling OFF for any timing.
2. `fast_match_compiled` must return `None` (not `Some(false)`) for the cases it
   does not handle, so the existing full matcher runs — a wrong `Some(false)`
   there would silently drop valid matches.
3. Keep `fast_match_definition_pattern` as the fallback; do not delete it.
4. No new per-call allocation beyond what the raw path already did; reuse the
   same `bind_pattern_name`/`match_blank`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; leave the working
  tree. Nothing outside Scope; if you must, STOP and report.
- Full machine access, repo + `/tmp` only, no network. Commands: `cargo`,
  release `rum-cli`, the microbench script, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, or no `downvalue-pattern-hit`
  movement → STOP and report.
- Minimal diff. Paste the real `cargo test` summary and the real ratio rows;
  never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (point 4).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, and before/after `downvalue-pattern-hit` + same-tier neighbor ratios.
4. Stop-loss / open questions (empty if none).
