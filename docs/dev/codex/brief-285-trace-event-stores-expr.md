# Brief 285 — `TraceEvent` stores `Expr`, not `String`

Removes a render→parse round trip from the tracing path. Prerequisite
for the lossless rule-aware trace symbol (that is a later brief; do NOT
build it here).

## The waste

`push_rule_applied_trace`
(`definitions/application/mod.rs:52`) renders every expression at
capture time:

```rust
TraceEvent::RuleApplied {
    source,
    input: input.to_string(),    // full recursive render + allocation
    rule: rule.to_string(),
    result: result.to_string(),
}
```

`Trace[]` then parses each one back — `trace.rs:296`,
`trace_expr_from_text` → `lower_text`, with a silent fallback to
`Expr::string(text)` when parsing fails. So the pipeline is

```
Expr → render → String → lex + parse + lower → Expr
```

`Expr::Call { head: SharedExpr, args: SharedArgs }` and
`SharedExpr(Arc<Expr>)` (`rum-core/src/expr.rs:507`), so cloning an
`Expr` is a refcount bump — O(1), allocation-free. The round trip buys
nothing in the in-kernel path.

Measured on `f[k_] := If[k < 1, 0, f[k-1] + 1]` (release CLI):

| n | untraced | traced | ratio |
|---|---|---|---|
| 60 | 1.45 ms | 4.93 ms | 3.4× |
| 120 | 2.15 ms | 8.40 ms | 3.9× |

**Attribution honesty**: that ratio is total tracing overhead, of which
the round trip is one component — event allocation and structured-tree
building are others. Do not claim the round trip caused all of it.
Measuring the actual delta is part of this brief's job.

## What to change

**Store `Expr` in `TraceEvent`; render only at the host boundary.**

1. `rum-runtime/src/runtime/model/messages.rs` — `TraceEvent`'s
   expression payloads become `Expr`: `EvaluationPass { before, after }`,
   `DefinitionStored { value }`, `RuleApplied { input, rule, result }`.
   `symbol` stays a `String` (it is a name, not an expression) and
   `kind`/`source` are unchanged. `Expr` implements `Eq`
   (`expr.rs:681`), so the existing derives still work.

2. `definitions/application/mod.rs` — drop the `.to_string()` calls;
   clone the `Expr` instead.
   **Also collapse the empty-payload branch.** Each constructor
   currently has an `else` arm building the event with `String::new()`
   for when tracing is off but a monitor is active — its only purpose
   was to dodge the render cost. Cloning is now free, so populate
   unconditionally and delete the branch. Keep the early-return guard at
   the top (`!tracing_enabled && no active monitor`) exactly as is —
   **that guard is the no-default-path-cost line and must not move.**

3. `debug_tools/trace.rs` — use the stored `Expr` directly. Delete
   `trace_expr_from_text` and its `Expr::string(text)` fallback.

4. `debug_tools/resources/memory.rs` —
   `render_trace_event_memory_proxy_bytes` currently sums
   `String::capacity()`. Switch to the existing
   `expr_memory_proxy_bytes` (line 257) for the `Expr` payloads, keeping
   `String::capacity()` for `symbol`.

5. `rum-host/src/model/outcome.rs` — `From<TraceEvent> for
   HostTraceEvent` now renders: `expr.to_string()` per payload.
   **`HostTraceEvent` itself does NOT change** — it keeps its `String`
   fields. Rendering is session-free (`Display` on `Expr`), so this is
   mechanical.

That last point is deliberate and load-bearing: it keeps
`rum-jupyter/src/adapter/render.rs` and `rum-terminal/src/lib.rs`
untouched, since they consume `HostTraceEvent`, not `TraceEvent`.
**If you find yourself needing to edit either, STOP** — it means the
boundary was drawn wrong.

## Acceptance: output differences are possible, and must be surfaced

The round trip was `Expr → render → parse → lower → Expr`. That is not
guaranteed to be the identity. Removing it may therefore **change some
`Trace[]` output** — and where it does, the new output is very likely
the more faithful one.

So the criterion is NOT "output is byte-identical". It is:

1. Build a corpus of at least 25 traced expressions covering: plain
   arithmetic, user `DownValue`s, recursion, `Module` locals (gensym
   rendering), strings containing brackets and quotes, `Hold`/held
   arguments, `Association` lookups, `ReplaceAll`/`ReplaceRepeated`,
   `Sow`/`Reap`, and any construct you suspect of not round-tripping.
2. Capture `Trace[...]` and `TracePrint[...]` output for the whole
   corpus on `HEAD` **before** any edit.
3. Capture again after.
4. `diff` them. **Report every difference**, each with a one-line
   judgment of which output is correct and why. A difference is not a
   failure — an unexplained difference is.
5. If a difference makes the new output WORSE, stop and report rather
   than pinning it.

Existing tests that assert trace output are the other half of this: if
one changes, treat it under the same rule — explain, do not
reflexively re-pin.

## Performance

Re-run the measurement from §The waste on both builds and report the
before/after ratio. The expected direction is "traced workloads get
faster; untraced workloads unchanged".

**The untraced number is the important one**: it must not regress. If it
moves, the early-return guard was disturbed. Also run the standard
`scripts/core-language-microbench.sh` A/B (base built before any edit,
copied aside) and state the direction mix — no label should move, since
nothing on the untraced path changed.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/model/messages.rs`;
  - `crates/rum-runtime/src/runtime/definitions/application/mod.rs`;
  - `crates/rum-runtime/src/runtime/debug_tools/trace.rs` and
    `debug_tools/trace/builder.rs`;
  - `crates/rum-runtime/src/runtime/debug_tools/monitor.rs`
    (MECHANICAL only — it matches on the variant, not the payload);
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`;
  - `crates/rum-host/src/model/outcome.rs` (the `From` impl only —
    `HostTraceEvent`'s own shape must not change);
  - MECHANICAL ONLY, and only if the type change forces it:
    `runtime/model/session.rs`, `runtime/model/mod.rs`,
    `runtime/model/requests.rs`, `runtime/session/session_api.rs`,
    `runtime/session/session_state.rs`;
  - tests that construct or destructure `TraceEvent`:
    `runtime/tests/holding_and_control_part2.rs`,
    `runtime/tests/session_requests/api.rs`,
    `runtime/tests/session_config.rs`.
- MUST NOT touch: `crates/rum-jupyter/**`, `crates/rum-terminal/**`,
  `crates/rum-repl/**`, `crates/rum-host/tests/**`, `crates/rum-cli/**`,
  any `Cargo.toml`, `docs/**`, `TODO.md`, `PROGRESS.md`, `AGENTS.md`,
  `.git`, `Cargo.lock`.
- Do NOT add the rule-aware trace symbol. This brief only changes how
  events are stored. `Trace[]`'s own behavior stays as-is apart from
  fidelity differences surfaced above.
- No new `unsafe`. No new dependencies.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

Baseline is `1662 passed / 0 failed` on `f028f13`; state your totals.
Sandbox ZeroMQ caveat applies as usual; Claude re-runs the
authoritative gate outside the sandbox.

Paste: the corpus diff (with per-difference judgments), the before/after
timing table, and the microbench direction mix.

## Stop-loss

- A corpus difference makes output worse and you cannot fix it within
  scope → STOP, report.
- `rum-jupyter` or `rum-terminal` needs editing → STOP, the boundary is
  wrong.
- The untraced timing regresses → STOP, the guard moved.
- The change requires touching a MUST-NOT file → STOP.

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
2. The corpus diff and your judgment on each difference (or "no
   differences", stated explicitly).
3. Before/after timing, plus the microbench direction mix.
4. Verbatim verification output.
5. Stop-loss / open questions.
