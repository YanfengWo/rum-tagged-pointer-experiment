# brief-046: MatchQ/Cases pattern argument must evaluate its ordinary subexpressions (the v7 batch-A 50-FAIL root cause)

## Goal

The 50 open v7 batch-A failures (num 12 + rat 23 + set 15) all reduce to ONE rum
bug, confirmed against WolframKernel this session. rum hard-holds the pattern
argument of `MatchQ` (and the whole `Cases`/`Count`/`Position`/`DeleteCases`/
`ReplacePart`/`Catch`-tag family) and then, in `resolve_matcher_pattern_expr_owned`,
applies an all-or-nothing gate: if the held pattern argument contains ANY pattern
syntax (`Blank`/`Pattern`/…) ANYWHERE, the WHOLE argument is returned
**unevaluated**. So an ordinary subexpression that should evaluate is skipped:

- the mma-hol test helper `quantNestPat[q,n,body] := Nest[comb[const[q,_],
  abs[bvar[0,_],#,_]]&, body, n]` is NEVER expanded (the `Nest` never fires)
  because its `body` argument carries literal `_` → num/rat tests match the
  theorem against the literal, unexpanded `quantNestPat[...]` and get `False`;
- the set tests' `comb[comb[const["=",_], const[pair[[2]], _]], _]` is held with
  `pair[[2]]` (a `Part`) **unevaluated**, so the const name never becomes `"IN"`.

WolframKernel does NOT Hold the pattern argument — it evaluates it normally;
pattern leaves (`Blank`, `Pattern`, …) survive because they are inert. The fix is
to make rum reproduce that: evaluate the ordinary subexpressions of the held
pattern (function calls, `Part`, ownvalue symbols) while leaving genuine pattern
constructs held. When done, the WK-calibrated probes below flip `False -> True`
and the rum-runtime suite stays green. (The real batch-A 50→0 re-run on a rebuilt
v7 snapshot is **Claude's gate** — you do NOT run mma-hol.)

## Root cause (already diagnosed — do NOT re-investigate, implement)

File `crates/rum-runtime/src/runtime/rewrite_builtins/cases/match_q.rs`, function
`resolve_matcher_pattern_expr_owned` (~line 26):

```rust
let normalized = strip_hold_pattern(&expr).clone();
let resolved = if contains_pattern_syntax(&normalized) {
    normalized              // BUG: any Blank anywhere -> whole arg held unevaluated
} else {
    self.evaluate(normalized)?
};
Ok(self.normalize_match_expr(&resolved))
```

`contains_pattern_syntax` (same file, ~line 129) recurses, so one nested `_`
freezes the entire argument. `matches_pattern_head` (~line 144) is the canonical
list of pattern-construct heads (`Blank`/`Pattern`/`Condition`/`Alternatives`/
`Optional`/`PatternTest`/`Repeated`/`Verbatim`/`HoldPattern`/…). Both helpers stay
as-is; you reuse them.

This function is the single chokepoint: `resolve_cases_spec_expr_for_matching_owned`
(`definition_catalog/rewrite_rules.rs:24`) falls through to it, and `DeleteCases`/
`Position`/`Count`/`ReplacePart`/`Catch`-tag (`nonlocal.rs`) call it directly. One
fix corrects the whole family.

### Why a blunt "always evaluate" is WRONG (verified this session — do NOT do it)

rum's `Pattern` is **not** `HoldFirst` in the evaluator: with `x = 5`,
`FullForm[x_]` evaluates to `Pattern[5, Blank[]]` (the name is corrupted). Today
`MatchQ[7, x_]` returns `True` ONLY because the hold protects `Pattern[x, _]` from
evaluation. So the fix MUST keep pattern-construct nodes held; it may only
evaluate ordinary-headed subexpressions.

## The fix (precise — implement EXACTLY)

Replace the gate with a recursive resolver. In `match_q.rs`:

```rust
pub(in crate::runtime) fn resolve_matcher_pattern_expr_owned(
    &mut self,
    expr: Expr,
) -> Result<Expr, EvalError> {
    let resolved = self.resolve_match_pattern_subexprs(strip_hold_pattern(&expr))?;
    Ok(self.normalize_match_expr(&resolved))
}

/// Evaluate the ordinary (non-pattern) subexpressions of a held matcher pattern,
/// leaving genuine pattern constructs (Blank/Pattern/Condition/...) unevaluated.
/// WK does not Hold the MatchQ pattern argument; it evaluates it normally and the
/// pattern leaves survive because they are inert. rum hard-holds the argument, so
/// this reproduces that selectively: a pattern leaf stays held (rum's `Pattern` is
/// NOT HoldFirst — evaluating `Pattern[x,_]` with a bound `x` corrupts the name),
/// but an ordinary head over pattern-bearing args (a user `quantNestPat[...]`
/// helper, or `comb[const[pair[[2]],_],_]` with a `Part`) is evaluated so it expands.
fn resolve_match_pattern_subexprs(&mut self, expr: &Expr) -> Result<Expr, EvalError> {
    // No pattern syntax anywhere -> safe to fully evaluate (Part[...], function
    // calls, ownvalue symbols all resolve). This is the historical else-branch,
    // now reachable at every depth instead of only at the top.
    if !contains_pattern_syntax(expr) {
        return self.evaluate(expr.clone());
    }
    match expr {
        // Atoms carry no pattern syntax, so the guard above already handled them;
        // keep the arm total.
        Expr::Atom(_) => Ok(expr.clone()),
        Expr::Call { head, args } => {
            // Genuine pattern construct: keep held (protect pattern names / held tests).
            if matches_pattern_head(head) {
                return Ok(expr.clone());
            }
            // Ordinary head with pattern syntax in some argument: resolve each child,
            // then evaluate the rebuilt node so the ordinary head fires (expands
            // quantNestPat[...]; anonymous Blanks pass through unchanged).
            let resolved_head = self.resolve_match_pattern_subexprs(head)?;
            let mut resolved_args = Vec::with_capacity(args.len());
            for arg in args.iter() {
                resolved_args.push(self.resolve_match_pattern_subexprs(arg)?);
            }
            self.evaluate(Expr::call(resolved_head, resolved_args))
        }
    }
}
```

Notes:
- `strip_hold_pattern` is applied ONCE at the top (preserving the current
  outer-`HoldPattern` behavior); nested `HoldPattern` is a `matches_pattern_head`
  node and is kept held by the recursion. Do not strip inside the recursion.
- Use whatever `Expr::call` / argument constructor the surrounding code uses
  (match the existing `normalize_match_expr` reconstruction in `resolution.rs`);
  keep `Arc`-shared args, don't deep-clone trees needlessly.
- Trace through the two real shapes to convince yourself:
  - `comb[comb[const["=",_], const[pair[[2]],_]], _]`: outer `comb` ordinary →
    recurse → inner `const[pair[[2]],_]` ordinary → recurse → `pair[[2]]` has no
    pattern syntax → evaluate → `"IN"`; `_` is a Blank → kept. Rebuild
    `const["IN",_]`, evaluate (inert) → unchanged. Result
    `comb[comb[const["=",_], const["IN",_]], _]`. ✔
  - `quantNestPat["∀",2, comb[...,_,...]]`: ordinary head → recurse args (Blanks
    kept) → evaluate the rebuild → the `Nest` fires → fully expanded pattern. ✔

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/rewrite_builtins/cases/match_q.rs` — replace the
    gate in `resolve_matcher_pattern_expr_owned`, add the private
    `resolve_match_pattern_subexprs` helper. Reuse `contains_pattern_syntax`,
    `matches_pattern_head`, `strip_hold_pattern`, `normalize_match_expr`.
  - a test module under `crates/rum-runtime/src/runtime/tests/` — add the
    regression cases below (extend an existing pattern-matching test file rather
    than creating a new one if one fits, e.g. `definitions_and_rules_part2.rs`).
- MUST NOT touch:
  - `contains_pattern_syntax` / `matches_pattern_head` / `strip_hold_pattern`
    bodies, the matcher core (`pattern_matching/`), `normalize_match_expr`
    (`resolution.rs`), `Pattern`'s evaluator attributes — none of these is the
    fix;
  - `crates/rum-core` `Expr`/`Symbol`/serde shapes;
  - any `mma-hol` clone or snapshot; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**`
    except appending to your own report; git.
- Do NOT add unconditional default-path cost. This path runs only when a matcher
  resolves a pattern argument; the recursion is bounded by pattern depth. Do not
  add counters/instrumentation.

## rum / Rust pitfalls (read twice)

1. **Keep pattern-construct nodes held.** The `matches_pattern_head` early-return
   is load-bearing: without it, `MatchQ[7, x_]` with a bound `x` corrupts to
   `Pattern[5,_]`. Verify the guard probe `x = 5; MatchQ[7, x_]` stays `True`.
2. **Suspension-safety.** `self.evaluate` can return
   `Err(EvalError::InputRequired{..})`; propagate it with `?` (the recursion does).
   The caller already wraps this resolver in `suspend_with_frame` and re-runs it on
   resume — pattern resolution is idempotent, so the re-run is fine. Do NOT swallow
   or special-case the suspension.
3. **`_+_` becomes `2 _` now — that is correct WK conformance, NOT a regression.**
   With the fix, an ordinary arithmetic head over Blanks evaluates:
   `_ + _` → `Times[2, Blank[]]` (WK does the same), so `MatchQ[a+b, _+_]` is
   `False`. If a rum unit test encoded the old held `_+_` behavior and now flips,
   STOP and report it in your final message (do NOT edit the test) — Claude
   decides at the gate (it is likely a test that encoded the pre-fix divergence,
   like the brief-041 `FullForm[1+2]` correction). mma-hol uses inert ADT
   constructors (`comb`/`const`/`abs`/`var`/`bvar`), never `Plus` patterns, so the
   real tower is unaffected.
4. **Minimal diff.** One function rewritten + one private helper + one test block.
   No reformatting, no renames elsewhere.
5. **Stop-loss.** If the suite shows a failure you cannot tie to point 3, or a
   guard probe below is wrong for a reason you cannot localize, STOP and report —
   do not keep tuning.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime           # 691/0 baseline + your new cases, all green
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# === the bug, minimal repros (WK gives True for all; rum currently gives False) ===
$BIN --eval 'qq[n_, b_] := Nest[w[#]&, b, n]; MatchQ[w[w[5]], qq[2, _]]'          # True
$BIN --eval 'MatchQ[f[1, 2], f[{10, 1}[[2]], _]]'                                  # True
$BIN --eval 'pair = {thm, "IN"}; MatchQ[cc["IN", 9], cc[pair[[2]], _]]'           # True
$BIN --eval 'wrap[b_] := h[cc, b]; MatchQ[h[cc, 9], wrap[_]]'                      # True
$BIN --eval 'MatchQ[abs[1, 2, 3], abs[_, _, _]]'                                   # True (already; keep)

# === pattern-leaf preservation guards (must stay as shown) ===
$BIN --eval 'x = 5; MatchQ[7, x_]'                # True  (Pattern name NOT corrupted)
$BIN --eval 'x = 5; MatchQ[7, x_ /; x > 3]'       # True
$BIN --eval 'x = 5; MatchQ[2, x_ /; x > 3]'       # False
$BIN --eval 'MatchQ[f[a, b], f[_, _]]'            # True
$BIN --eval 'MatchQ[f[a], f[_, _]]'               # False
$BIN --eval 'MatchQ[7, _Integer]'                 # True
$BIN --eval 'MatchQ["hi", _Integer]'              # False

# === WK-conformance consequence (paste actual output; expected per the comment) ===
$BIN --eval 'MatchQ[a + b, _ + _]'                # False  (pattern is 2 _, WK-conformant)

# === the family also benefits (Cases/Count/Catch route through the same resolver) ===
$BIN --eval 'qn[n_, b_] := Nest[g[#]&, b, n]; Cases[{g[g[1]], g[2], x}, qn[2, _]]' # {g[g[1]]}
$BIN --eval 'tg = "$t$"; Catch[Throw[42, tg], tg]'                                 # 42 (Task-1, unchanged)
```

Paste EVERY line's verbatim output. Add unit tests covering: the `qq`/`Part`/
`wrap` resolve cases, the `x=5; MatchQ[7,x_]` name-preservation guard, and the
`Cases` family case.

## Hard rules

- Do NOT `git commit`/branch/push or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate (rebuild snapshot + re-run the
  real num/rat/set batch-A, confirming 50→0) and commits.
- Do NOT modify files outside Scope; do NOT edit any `mma-hol` clone/snapshot; do
  NOT run mma-hol or `wolframscript`.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  Scope, `cargo`, release `rum-cli`.
- Paste real verbatim output. If any probe is not as the comment says, or a suite
  test flips for a reason other than pitfall 3, STOP and report it — never fake a
  result, never patch a test to pass.

## Report format

1. What changed — `resolve_matcher_pattern_expr_owned` + the new helper + the test,
   with line ranges and a one-line why each.
2. Verification — `fmt`, the test count, EVERY probe line verbatim.
3. Verdict — the resolve bug is fixed (ordinary subexprs evaluate, pattern leaves
   held); any suite test that flipped due to the `_+_` conformance change (with the
   exact test name), or "none".
4. Stop-loss / open questions.
