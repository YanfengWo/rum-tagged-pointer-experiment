# Brief 287 — `TraceDetail`, the lossless trace symbol

Implements `docs/dev/trace-detail-design.md`, ruled by the user on
2026-07-21: **Option A (an Association per step)**, named
**`TraceDetail`**. Read that document first — its §1 is measured
grounding and §2 Option A is the spec.

The substrate is already in place: brief-286 made `TraceEvent` store
`Expr`, so every field below is a live `Expr` needing no parsing.

Grounding below verified on `9af4214`.

## What `Trace[]` throws away (this is what to stop throwing away)

`structured_trace_expr_from_events` (`debug_tools/trace.rs:245`)
destructures four fields away:

| event | uses | **discards** |
|---|---|---|
| `EvaluationPass { before, after }` | both | — |
| `DefinitionStored { symbol, kind, value }` | `value` | **`symbol`, `kind`** |
| `RuleApplied { source, input, rule, result }` | `input`, `result` | **`source`, `rule`** |

Depth is not discarded — it builds the tree — so `TraceDetail` keeps the
tree AND all four fields.

## Output shape

Tree nesting stays exactly as `Trace`'s (`List` nesting = subevaluation
depth). Each STEP becomes an `Association` instead of a bare `Hold[…]`.

Keys per event kind — use exactly these names and this key order:

```
RuleApplied      <|"Step" -> "RuleApplied",
                   "Source" -> "DownValue",          (* RuleApplicationSource, as a string *)
                   "Input"  -> Hold[…],
                   "Rule"   -> Hold[…],
                   "Result" -> Hold[…]|>

EvaluationPass   <|"Step" -> "EvaluationPass",
                   "Before" -> Hold[…],
                   "After"  -> Hold[…]|>

DefinitionStored <|"Step" -> "DefinitionStored",
                   "Symbol" -> "Global`f",            (* the symbol name, as a string *)
                   "Kind"   -> "DownValue",           (* DefinitionKind, as a string *)
                   "Value"  -> Hold[…]|>
```

- Expression-valued fields keep the `Hold[…]` wrapper, exactly as
  `Trace` leaves do — same reason (the step must not re-evaluate) and
  same shape, so a reader moving between the two views is not surprised.
- `Source` and `Kind` render as their variant names in strings
  (`RuleApplicationSource` has 9 variants, `DefinitionKind` 5 — both in
  `runtime/model/messages.rs`). Strings, not symbols: they are data
  labels, and a bare symbol would evaluate.
- Reuse `StructuredTraceBuilder` for the nesting. It takes an arbitrary
  `Expr` per depth (`push_expr(depth, expr)`), so an association goes in
  exactly where a `Hold[…]` goes today — **the tree logic needs no
  change**. Follow `structured_trace_expr_from_events`'s depth handling
  (including its `active_exprs` bookkeeping and the `RuleApplied` arm's
  two-level push) rather than reinventing it.

## Argument surface

Mirror `Trace` exactly: `TraceDetail[expr]`,
`TraceDetail[expr, form]`, `TraceDetail[expr, form, levelspec]`, and the
`Heads -> True|False` option. `{HoldAll, Protected}`, and the same
scoped capture (`prior_enabled` save / set / restore).

**The `form` filter matches the UNDERLYING EXPRESSION, not the
association** (user ruling) — so `TraceDetail[expr, _Plus]` selects the
same steps `Trace[expr, _Plus]` does. Filtering on annotations is what
`Select` over the result is for. Say in your report how you achieved
this, since the filter currently runs over the built structure.

## Registration chain — all of it

Verified by tracing how `Trace` itself is wired. Every one of these
needs a `TraceDetail` counterpart:

1. `builtin_catalog/debug.rs` — a `BuiltinSpec::ordinary_with_hold(
   "TraceDetail", <description>, HoldAttribute::HoldAll,
   builtin_trace_detail)` `.with_messages(&[...])` `.protected()`.
   Give it its own `argform` message mirroring `Trace`'s, plus `ARGTU`.
2. `builtin_adapters.rs` — a `builtin_trace_detail` adapter calling
   `session.eval_trace_detail(args)`, next to `builtin_trace`.
3. `debug_tools/trace.rs` (and `trace/builder.rs` if needed) — the
   implementation.
4. `options/defaults.rs:44` — extend the `"Trace" | "TracePrint"` arm to
   include `"TraceDetail"` so the `Heads` option default exists.
5. `namespace_tools/syntax.rs:153` — same, for syntax information.
6. `capabilities.rs` — capability comes from the `BuiltinSpec`, and the
   debug family's default is `Pure`. **Verify, do not assume**: add
   `assert_eq!(capability("TraceDetail"), Capability::Pure);` to the
   existing test alongside the `Trace` assertion. If it is not `Pure`,
   STOP and report rather than forcing it.
7. `docs/reference/builtins.md` — **regenerate with
   `cargo run -p rum-stdlib --bin generate-builtin-reference`. Do NOT
   hand-edit it**; a drift test (`builtin_reference_is_in_sync`) compares
   it byte-for-byte. It has 641 entries today; expect 642.

Kernel builtins do NOT appear in `builtin_catalog/stdlib_attributes/`
(that mirror is for stdlib symbols) — verified, nothing to add there.

## Tests

Add to `runtime/tests/debug_tools.rs`:

- basic shape on a small expression, pinning the full association;
- **the headline case**: a symbol with several overlapping `DownValue`s
  where more than one could match, asserting that `"Rule"` names the one
  that actually fired. This is the reason the symbol exists — if this
  test is weak, the feature is unproven;
- a `DefinitionStored` step, asserting `"Symbol"` and `"Kind"`;
- `form` filtering, showing it matches the underlying expression;
- `levelspec` and `Heads`;
- equivalence: for the same input, `TraceDetail`'s tree SHAPE (nesting)
  matches `Trace`'s. State how you checked.

## Scope

- MAY modify: `builtin_catalog/debug.rs`, `builtin_adapters.rs`,
  `debug_tools/trace.rs`, `debug_tools/trace/builder.rs`,
  `options/defaults.rs`, `namespace_tools/syntax.rs`,
  `capabilities.rs` (the one added assert only),
  `runtime/tests/debug_tools.rs`,
  `runtime/tests/syntax_information_debug_catalog.rs` (only if its
  catalog assertions cover the new symbol),
  `docs/reference/builtins.md` (REGENERATED, never hand-edited).
- MUST NOT touch: `Trace`/`TracePrint` behavior — they are unchanged and
  every existing trace test must stay green untouched;
  `runtime/model/messages.rs` (the event types are already right);
  `rum-host/**`, `rum-repl/**`, `rum-jupyter/**`, `rum-terminal/**`
  (**no REPL surface in this brief** — `:trace --rules` is a separate,
  later decision); any `Cargo.toml`; `TODO.md`; `PROGRESS.md`;
  `AGENTS.md`; `.git`; `Cargo.lock`.
- No new dependencies, no new `unsafe`, no new capability.
- No default-path cost: `TraceDetail` must use the same scoped-capture
  guard discipline as `Trace`; nothing changes for sessions that never
  call it.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Baseline is `1662 passed / 0 failed` on `9af4214`; state your totals and
the reference count (641 → expected 642). Sandbox ZeroMQ caveat applies;
Claude re-runs the authoritative gate outside the sandbox.

Paste verbatim release-CLI output (stdin from `/dev/null`) for:

```
TraceDetail[1+1]
TraceDetail[(1+2)*3]
f[x_] := x + 1; TraceDetail[f[2]]
TraceDetail[g = 5]
TraceDetail[(1+2)*3, _Plus]
Trace[1+1]                          (* regression: unchanged *)
```

No microbench: a new symbol on no existing path.

## Stop-loss

- The `form` filter cannot be made to match the underlying expression
  without changing `Trace`'s behavior → STOP, report.
- `TraceDetail` does not come out `Capability::Pure` → STOP, report.
- Any existing `Trace`/`TracePrint` test changes → STOP, report which.
- The registration chain needs a file not listed above → STOP, name it.

(`git checkout --` fails under the sandbox with `Unable to create
'.git/index.lock': Operation not permitted` — restore by scoped patch
instead and say so.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes
  in the working tree; Claude runs the authoritative gate and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and
  write that in the report instead.

## Report format

1. What you changed, file by file.
2. Verbatim verification output, incl. the six probes and the reference
   count.
3. How you made `form` match the underlying expression, and how you
   checked tree-shape equivalence with `Trace`.
4. Stop-loss / open questions.
