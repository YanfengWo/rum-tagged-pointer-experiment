# brief-051: a benign self-fixed-point (`a = a` / `a := a`) must return the symbol, not raise `General::cycle` (the arith_tests silent-skip root cause)

## Goal

rum raises a fatal `General::cycle` / `EvalError::EvaluationCycleDetected` when it
evaluates a symbol whose ownvalue is the symbol itself (`a :> a`, produced by
`a = a` or `a := a`). WolframKernel treats this as a **benign fixed point** and
returns the symbol unchanged — no error, no message. Because a fatal eval error
inside `Get` silently aborts the rest of the file, this single bug makes
`tests/arith_tests.wl` stop mid-file (it runs 41 of 140 `runTests` groups, then
the public-symbol self-alias `nnfConv = HOL\`Auto\`Arith\`nnfConv` on line 399
trips the cycle abort) — **221 of the file's 271 assertions are silently skipped**
(measured: WK 271/0, rum 50/0). When you are done, evaluating a self-fixed-point
symbol returns the symbol (matching WK across all five probed variants below),
`cargo test -p rum-runtime` is green (with the three existing `self := self` cycle
tests redirected to a GENUINE cycle so they still cover real cycle detection), and
new regression tests lock the WK-conformant behavior. The full
`arith_tests.wl` 50→271 re-run on the v8 snapshot is **Claude's gate — you do NOT
run mma-hol**; you verify the minimal rum-cli repros instead.

## Root cause (already diagnosed — do NOT re-investigate, implement)

File `crates/rum-runtime/src/runtime/eval_core.rs`, function `evaluate_symbol`
(~line 106). The ownvalue paths re-evaluate the stored RHS unconditionally:

```rust
if let Some(value) = self
    .definitions
    .get(&symbol)
    .and_then(|definition| definition.own_value.clone())
{
    return self.evaluate_own_value(value);          // own_value path  (a = a)
}

if let Some(value) = self
    .definitions
    .get(&symbol)
    .and_then(|definition| definition.delayed_own_value.clone())
{
    return self.evaluate_definition_rhs(None, value); // delayed path (a := a)
}
```

`evaluate_own_value` (~:189) does `self.evaluate(value)` for any non-literal RHS.
For `a = a` the stored `own_value` is the bare symbol `a`, so `self.evaluate(a)`
re-enters the evaluation of `a` while `a` is still on `self.evaluation_stack`; the
re-entry guard in `evaluate_once_unstable_after_input_check_with_cycle_tracking`
(eval_core.rs:9-18) then returns `EvalError::EvaluationCycleDetected`. The delayed
path hits the same re-entry via `evaluate_definition_rhs`.

**The fix:** in `evaluate_symbol`, before re-evaluating, short-circuit when the
stored RHS is **structurally the symbol itself** — return it as a fixed point
without re-evaluating. Apply this to BOTH the `own_value` and `delayed_own_value`
branches. The symbol-as-`Expr` is `Expr::Atom(Atom::Symbol(symbol))`; compare the
RHS against it (e.g. `matches!(&value, Expr::Atom(Atom::Symbol(s)) if *s == symbol)`,
or build the symbol `Expr` once and compare). Mind the borrow: `symbol` is consumed
by the fallthrough at the end of the function (`Ok(Expr::Atom(Atom::Symbol(symbol)))`),
so the early-return comparison must only borrow it, not move it.

This is intentionally the MINIMAL catch — only the DIRECT self-reference `a :> a`.
A genuine non-trivial cycle (`d := e; e := d`, RHS `e` ≠ `d`) does NOT match the
guard and keeps today's behavior (evaluates `e` → `d` → re-entry → `General::cycle`).
Do not broaden the fix to chains or mutual cycles in this brief.

## WK target behavior (already measured — your repros must reproduce these)

| input | WK result |
| --- | --- |
| `a = a; a` | `a` (OwnValues `{HoldPattern[a] :> a}`, no error, no message) |
| `b := b; b` | `b` (no message) |
| `s[x_] := x + 1; s = s; s` then `s[5]` | `s` then `6` |
| `c[x_] := x + 1; c := c; c` then `c[5]` | `c` then `6` |
| `d := e; e := d; d` | `TerminatedEvaluation[IterationLimit]` (genuine cycle — OUT OF SCOPE, leave rum's current behavior) |

## Context pointers

- Read `AGENTS.md` (operational notes; the unmodified-`mma-hol` hard line).
- Key files:
  - `crates/rum-runtime/src/runtime/eval_core.rs` — `evaluate_symbol` (the fix),
    `evaluate_own_value` (:189), the re-entry guard
    `evaluate_once_unstable_after_input_check_with_cycle_tracking` (:4-45, for
    understanding only — do NOT change it).
  - `crates/rum-runtime/src/runtime/tests/messages_part2.rs` — three tests
    (~:243 `non_loader_evaluation_errors_emit_general_messages_at_top_level`,
    ~:268 `check_can_catch_non_loader_evaluation_errors_via_general_messages`,
    ~:290 `check_can_match_specific_general_error_selectors`) set up `self := self`
    and assert it yields `General::cycle`. That setup now returns `self` (the
    correct WK behavior), so those tests must be **re-pointed at a genuine cycle**:
    replace the `self := self` setup with a mutual cycle, e.g.
    `session.eval_text("loopA := loopB").unwrap(); session.eval_text("loopB := loopA").unwrap();`
    and evaluate `loopA` — it still produces `EvaluationCycleDetected` /
    `General::cycle`, so the assertions' shape is unchanged (only the trigger and
    the `expr ==` string change, from `"self"` to `"loopA"`). Keep all three tests
    meaningful; do not delete them.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. Acceptance is
  mechanical: the named tests + the verbatim rum-cli probe outputs decide.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/eval_core.rs` (the `evaluate_symbol`
    self-fixed-point short-circuit only);
  - `crates/rum-runtime/src/runtime/tests/messages_part2.rs` (redirect the three
    `self := self` tests to a genuine mutual cycle; add the new regression tests —
    OR put the new regression tests in the most appropriate existing
    `crates/rum-runtime/src/runtime/tests/*.rs` module if that fits better, e.g.
    alongside other ownvalue/Set tests).
- Files/areas you MUST NOT touch:
  - the re-entry guard / fixed-point loop themselves (`eval_core.rs` :4-45,
    `resolution.rs` `evaluate_profiled_fixed_point` / `should_track_evaluation_cycle`)
    — the fix is a pre-eval short-circuit in `evaluate_symbol`, nothing else;
  - anything under any `mma-hol` clone or the acceptance standard — NEVER edit it
    to pass; you do NOT run mma-hol in this brief;
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation and any public
    type/serde shape;
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your own
    report, `.git`, `Cargo.lock`.
- Do NOT add any default-path cost: the new check runs only when a symbol already
  HAS an ownvalue (already in the `if let Some(value) = …own_value…` arm), so it
  adds at most one cheap structural compare on a path that was about to call
  `self.evaluate` anyway — keep it that cheap, no allocation in the hot fallthrough.

## Deliverable

- The `evaluate_symbol` short-circuit (both ownvalue branches).
- The three `messages_part2.rs` cycle tests redirected to a genuine mutual cycle,
  still asserting `EvaluationCycleDetected` / `General::cycle`.
- New regression test(s) asserting the WK table above (rows 1-4), e.g.
  `a = a; a` → `a`; `b := b; b` → `b`; `s[x_] := x + 1; s = s; s` → `s` and
  `s[5]` → `6`. Use the in-crate `Session` test harness (`session.eval_text(...)`,
  `.value.to_string()`), matching the existing tests' style.

## rum / Rust pitfalls (read twice)

1. **Minimal catch only.** Match ONLY the direct self-reference `RHS === this
   symbol`. Do not try to detect mutual/longer cycles; that is a separate
   conformance item (WK returns `TerminatedEvaluation[IterationLimit]` for those,
   which rum does not yet do, and is OUT OF SCOPE here).
2. **Borrow discipline.** `symbol` is moved by the function's final fallthrough.
   The early-return comparison must borrow `symbol` (PartialEq takes refs) or copy
   it; do not introduce a move that breaks the later `Ok(Expr::Atom(Atom::Symbol(symbol)))`.
3. **Suspension-safety.** `evaluate_symbol` is on the eval path; the new branch is
   a plain early `Ok(...)` return — do not wrap it in or remove any
   `suspend_with_frame` machinery.
4. **Do not touch the guard.** The re-entry guard and the `fixed_point_seen` loop
   stay exactly as-is; genuine cycles must keep erroring.
5. **One behavior change.** The only observable change is: self-fixed-point symbols
   now return themselves. Verify a neighbor did not regress: ordinary ownvalues
   (`x = 5; x` → `5`) and downvalues (`f[x_] := x + 1; f[3]` → `4`) must still work.

## Verification (MANDATORY — you run it; iterate to green)

Run exactly these and paste results verbatim:

```
cargo fmt --check
cargo test -p rum-runtime
```

Then build the release CLI and paste the verbatim output of these minimal
WK-conformance repros (each must match the WK table; none may error / exit non-zero):

```
cargo build --release -p rum-cli
target/release/rum-cli --eval 'a = a; Print["a=", a]'
target/release/rum-cli --eval 'b := b; Print["b=", b]'
target/release/rum-cli --eval 's[x_] := x + 1; s = s; Print["s=", s]; Print["s5=", s[5]]'
target/release/rum-cli --eval 'x = 5; Print["x=", x]; f[y_] := y + 1; Print["f3=", f[3]]'   # neighbor: no regression
```

(For comparison, `git stash` is not needed; the pre-fix binary errors on the first
three with `General::cycle` and exits 1 — that is the bug you are removing.)

Loop: edit → `cargo test -p rum-runtime` → fix → rerun until green, then run the
four rum-cli probes and paste their stdout. Never claim a result without the
verbatim line.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate (incl. the arith_tests 50→271
  re-run on the v8 snapshot) and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and write that
  in the report instead.
- Full machine access, but touch NOTHING outside this repo, no network. The only
  commands you run are file edits within Scope, `cargo`, and the release `rum-cli`.
- Never edit any `mma-hol` file; do not run mma-hol here.
- Stop-loss: if `cargo test -p rum-runtime` is still red after two focused fix
  attempts for the same failure, STOP and report the failing test + output.
- Minimal diff: no drive-by reformatting, renames, or comment changes outside the
  task.
- You MUST paste the real verbatim verification output. If a run did not reach
  green, say so explicitly — never fake it.

## Report format (your final message)

1. What changed — per file, with line ranges and a one-line why.
2. Verification — each command run, with the exact verbatim output lines (test
   counts, the four rum-cli probe stdout lines).
3. The finding / verdict — confirm the self-fixed-point now returns the symbol and
   the genuine-cycle tests still error; note the neighbor checks.
4. Stop-loss / open questions — where you stopped and why (empty if none).
