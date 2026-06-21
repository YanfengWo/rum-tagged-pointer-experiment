# Brief 008 — Skip empty Down/Up/SubValue dispatch stages (implementation)

## Goal

After brief-007, the worst primitive is `if-true` (~`6.4x`), now dominated by
shared call-dispatch overhead. `build_call_dispatch_plan` unconditionally pushes
the `UpValues`, `DownValues`, and `SubValues` stages for every ordinary call,
even when the head symbol has none of those values. Per-phase timing on
`If[True, 1, 2]` (which has NO down/up/sub values) shows these EMPTY stages cost
`down_values 307 ns`, `sub_values 181 ns`, `up_values 51 ns` profiled — about
`539 ns`, comparable to the actual builtin work (`604 ns`). Skip a value stage
when the call cannot possibly have that value, so pure-builtin and
undefined-symbol calls stop entering three no-op stages. This helps every
builtin call and is independent of the (larger, deferred) owned-args rework.

## Context pointers

- Read `AGENTS.md` and `docs/performance-stage0-baseline.md` first.
- `build_call_dispatch_plan` is in
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (it already looks up
  `cached_attribute_set_for_head`, `expr_as_symbol(head)`, and
  `cached_builtin_dispatch_spec_for_symbol`).
- The stage bodies confirm the guards: `try_apply_up_values` early-returns when
  `!self.has_up_value_definitions`; `try_apply_down_values` /
  `try_apply_sub_values` early-return when the head symbol's definition has
  empty `down_values` / `sub_values` (or the head is not a symbol, or there is
  no definition). The plan-time guard must mirror these exactly.

## Scope

- Files you MAY modify:
  `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs` (the plan builder), and
  a runtime test module under `crates/rum-runtime/src/runtime/tests/` if needed.
- MUST NOT change the stage bodies, `CallDispatchPlan`/`CallDispatchStage`, the
  special-form / Listable / Function / OrdinaryBuiltin pushes, `Expr`/`Symbol`,
  docs, any `mma-hol` clone, `.git`.

## Design (implement exactly; idiomatic Rust is yours)

In `build_call_dispatch_plan`, gate the three value-stage pushes on whether the
call can have that value. Look up the head symbol's definition once (reuse the
`expr_as_symbol(head)` already computed for the builtin spec):

- **UpValues**: push only if `!attributes.contains(HoldAllComplete)` AND
  `self.has_up_value_definitions`. (UpValues are keyed by the args' symbols, not
  the head, so the existing global flag is the correct and complete guard.)
- **DownValues**: push only if the head is a symbol whose definition has a
  non-empty `down_values`. (Downvalues are always keyed by the head symbol; a
  non-symbol head or absent/empty definition means the stage can only
  early-return.)
- **SubValues**: push only if the head is a symbol whose definition has a
  non-empty `sub_values`.

Everything else (special-form early return, `PrepareArguments`, `Listable`,
`OrdinaryBuiltin`, `Function`) stays exactly as is. Do a single
`self.definitions.get(symbol)` (or reuse one borrow) for the down/sub checks; do
not add more than one definition lookup.

**Why this is correct (state it in your report):** the guards are exactly the
conditions under which each stage body returns `Ok(None)` without effect, so
omitting a stage produces the identical dispatch result as running it and
getting a no-op. The plan is rebuilt per call, so a later `Set`/`UpSet`/`f[..]:=`
that adds a value is reflected on the next call's plan; there is no stale-plan
risk.

## Tests (add focused regressions)

- a downvalue added AFTER first use still applies: evaluate `k[1]` (stays
  `k[1]`), then `k[1] = 9`, then `k[1]` → `9`;
- an upvalue still fires when one exists: `m /: f[m] := ok; f[m]` → `ok`;
- a subvalue still fires: `d[1][2] := 12; d[1][2]` → `12` (curried / SubValue);
- a pure builtin with no values is unchanged: `SameQ[1, 1]` → `True`,
  `If[True, 1, 2]` → `1`, `Length[{1, 2, 3}]` → `3`.

## Verification (MANDATORY — run and paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

All green (especially up/down/sub value and dispatch tests, plus your new
regressions). Then the performance gate (profiling OFF). Because whole-label
microbench A/B has cross-run noise, ALSO measure the isolated stage cost to
prove the win directly: run the per-phase timer on `If[True, 1, 2]` before and
after and confirm `call.stage.down_values` / `sub_values` / `up_values` no
longer appear (or drop to near zero):

```
RUM_PROFILE_LEXICAL_SUBSTITUTION=1 target/release/rum-cli --eval \
  'Do[Do[If[True,1,2],{i,1,100000}],{j,1,10}]' 2>&1 \
  | grep -E 'runtime-path-self-timing' | grep -iE 'call.stage'
```

Then:

```
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_008_micro
```

Paste the `ratios.tsv` rows for `if-true`, `sameq-integer`, `length-list`,
`head-list`, and `downvalue-exact-hit` (still has a real downvalue, must NOT
regress). The builtin labels should improve; the downvalue/subvalue labels must
not regress. If the isolated stage cost does not drop, STOP and report.

## rum / Rust pitfalls (read twice)

1. Measure the RELEASE binary; profiling OFF for any timing, ON only for the
   isolated stage check.
2. The guard must EXACTLY match each stage body's early-return condition — a
   too-aggressive skip would silently drop a real value match. Mirror them.
3. At most one extra `definitions.get` in the plan builder; no per-call alloc.
4. Keep the plan correct when a definition is added/removed between calls (it is
   rebuilt per call, so just read current state).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; leave the working
  tree. Nothing outside Scope; if you must, STOP and report.
- Full machine access, repo + `/tmp` only, no network. Commands: `cargo`,
  release `rum-cli`, the microbench script, `python3`.
- Never edit any `mma-hol` file.
- Stop-loss: same `cargo test` failure twice, or the isolated stage cost does
  not drop → STOP and report.
- Minimal diff. Paste the real `cargo test` summary, the before/after isolated
  stage lines, and the ratios; never fake a result.

## Report format

1. Per-file changes, line ranges + one-line why.
2. The correctness argument (the guard == the stage early-return condition).
3. Verification — verbatim `cargo fmt --check`, `cargo test -p rum-runtime`
   summary, the before/after `call.stage.*` isolated lines, and the ratio rows.
4. Stop-loss / open questions (empty if none).
