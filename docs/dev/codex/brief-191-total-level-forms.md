# Brief 191 — Total level forms: Total[list,n] / Total[list,{k}] / Total[list,Infinity] (unit-5 fix 4b)

Implementation brief. Executes part of unit-5 triage "cheap flexibility"
(tracker `docs/dev/usability-pass.md`, unit-5 section; evidence E5 in
`docs/dev/usability-pass/math-findings.md`). `Total` currently accepts
ONLY one argument (sum the top-level elements). Add the WL-documented
level specifications. Folk-cheap: build a nested `Plus`/`List` expression
tree and evaluate it ONCE (the same "build `Plus`, evaluate" style the
current `eval_total` already uses). No general tensor machinery, no new
attributes.

All in `crates/rum-runtime/src/runtime/arithmetic/parts.rs` (`eval_total`
at `parts.rs:4`).

## WL semantics (the acceptance spec — derived + fixed)

For `m = {{1, 2}, {3, 4}}`:
```
Total[m]                -> {4, 6}    (default = level 1; UNCHANGED)
Total[{1,2,3}]          -> 6         (UNCHANGED)
Total[m, 1]             -> {4, 6}    (levels 1..1)
Total[m, 2]             -> 10        (levels 1..2, cumulative)
Total[{1,2,3}, 2]       -> 6         (no level 2; still the total)
Total[m, {1}]           -> {4, 6}    (contract level 1 only)
Total[m, {2}]           -> {3, 7}    (contract level 2 only: {1+2, 3+4})
Total[m, Infinity]      -> 10        (sum all levels)
```

## Change — extend `eval_total`

Keep the 1-argument form byte-identical (route it through the level-1
path below). For `Total[list, spec]` (2 args), dispatch on `spec`:

- `spec` a CONCRETE POSITIVE integer `n` → `build_total_levels(list, n)`
  (cumulative levels 1..n).
- `spec` = the symbol `Infinity` (`is_system_symbol_name(spec,
  "Infinity")`) → sum ALL levels (`build_total_levels` recursing to the
  leaves).
- `spec` a `List` with exactly ONE element that is a concrete positive
  integer `k` (`{k}`) → `build_total_at_level(list, k)` (contract level
  `k` only).
- anything else (non-positive, symbolic, ragged, multi-element level
  list, `spec` not applicable) → return the inert call unchanged.

Build a pure expression tree with these helpers, then `self.evaluate` it
ONCE (mirror the current `eval_total` which builds `Plus` and evaluates):

```
// cumulative levels 1..n; a non-List leaf passes through
build_total_levels(expr, n):
    if expr is not List[...]: return expr
    elements = items(expr)
    if n <= 1: return Plus @@ elements
    else:      return Plus @@ [ build_total_levels(el, n-1) for el in elements ]

// contract level k only; returns None (=> inert) if the structure is
// not uniformly deep enough (a non-List where a List is needed)
build_total_at_level(expr, k) -> Option:
    if expr is not List[...]: return None
    elements = items(expr)
    if k <= 1: return Some(Plus @@ elements)
    else:
        mapped = [ build_total_at_level(el, k-1)? for el in elements ]   // any None -> None
        return Some(List @@ mapped)
```

`Infinity` uses `build_total_levels` recursing until every branch bottoms
out at a leaf (equivalently a saturating-large `n`). `Plus @@ xs` =
`Expr::call(system_expr("Plus"), xs)`; `List @@ xs` similarly. Ragged
lists where `Plus` cannot thread simply evaluate to whatever `Plus`
yields (folk — do not special-case).

Verify with the acceptance table: `Total[m, 2]` → `10`,
`Total[m, {2}]` → `{3, 7}`, `Total[m, {1}]` → `{4, 6}`,
`Total[m, Infinity]` → `10`, `Total[{1,2,3}, 2]` → `6`, and the
1-argument forms unchanged.

## Tests

Add tests in `crates/rum-runtime/src/runtime/tests/arithmetic.rs` (where
the Total/Mod arithmetic tests live): the full acceptance table above,
plus a fail-soft case — `Total[{{1, 2}, {3}}, {2}]` (ragged) and
`Total[m, 0]` / `Total[m, x]` (non-positive / symbolic spec) stay inert.
Keep existing Total tests passing.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Total[{{1,2},{3,4}}], Total[{1,2,3}], Total[{{1,2},{3,4}}, 2], Total[{{1,2},{3,4}}, {2}], Total[{{1,2},{3,4}}, {1}], Total[{{1,2},{3,4}}, Infinity], Total[{1,2,3}, 2], Total[{{1,2},{3,4}}, 0], Total[{{1,2},{3,4}}, x]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
the probe returns `{4,6}, 6, 10, {3,7}, {4,6}, 10, 6, Total[...,0]
(inert), Total[...,x] (inert)}`.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/arithmetic/parts.rs`
  (`eval_total` + the new helpers) and
  `crates/rum-runtime/src/runtime/tests/arithmetic.rs` (new tests).
- MUST NOT touch: `Denominator`/`Numerator` (same file, leave alone), any
  other builtin; `Commonest`/`Quantile`/`PowerMod` (other briefs); the
  guide, `docs/reference/builtins.md`, `TODO.md`, the tracker, findings
  files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve any suspend/resume shape (the final evaluate
  mirrors the current `eval_total`).
- Compat: the 1-argument `Total` is byte-identical to today. New level
  forms build an expression tree and evaluate once — no new algorithm.
- Folk fallback: any spec that is not a positive integer / `Infinity` /
  `{positive integer}`, or a structure too shallow/ragged for a `{k}`
  contraction, returns the inert call. Do NOT guess or error.
- No A/B gate needed (not an eval/dispatch/matcher hot path).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — `eval_total` dispatch + the helpers, line ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
