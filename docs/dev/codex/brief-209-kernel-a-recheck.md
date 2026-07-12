# Brief 209 — Re-run the write guard in the held-update store step (unit-8 fix batch 1a)

Implementation brief (repurposed — see the note at the end). A single
localized correctness fix in the compound-assignment store path, plus one
regression test. Read `AGENTS.md` (Testing Policy) and
`docs/dev/usability-pass/kernel-a-findings.md` (the compound-assignment
section and its GATE block, and Evidence E4) before starting.

## Goal

The held-update path guards the target once, at the CHECK step, then
evaluates the update, then stores — and the store step writes without
re-running the guard. For `ApplyTo`, whose update is `function[current]`
and therefore runs user code, that user code can change the target's
`Protected` state during the update, and the store step then writes anyway.

Make the store step re-run the same guard the check step already uses, so a
target that is protected at store time is not written. When you are done:
the eleven compound-assignment heads all pass their guard at the moment of
the write, `ApplyTo` with an update that protects its target during
evaluation fails closed with the standard `wrsym` message and leaves the
target unchanged, and every existing test stays green.

## The exact mechanism (already diagnosed — no re-investigation needed)

- CHECK step: `current_symbol_held_update_value` / `current_part_held_update_value`
  call `self.ensure_symbol_not_protected_for_write(operator, &target)`
  (`crates/rum-runtime/src/runtime/definitions/definition_updates.rs:269`,
  `:298`). This returns `false` and emits the message if the target is
  protected. Good — but this runs BEFORE the update is evaluated.
- STORE step: `finish_held_update` (`:361-375`) evaluates the update
  (`self.evaluate(update)`), then calls `store_held_update` (`:377-418`),
  which writes `definition.own_value` with NO guard call.
- Only `ApplyTo` builds an update that runs user code:
  `update = Expr::call(function.clone(), vec![current])` (`:102`). The other
  ten build `Plus[current, arg]` / `Subtract[...]` / an `Append`-shaped call
  etc. (`:41`, `:59`, `:73`, `:88`, `:117`, `:159`, …) whose arguments are
  already evaluated, so their update evaluation runs no user code and cannot
  change protection state. The fix is shared regardless; `ApplyTo` is the
  reachable case and the one the test must cover.

`ensure_symbol_not_protected_for_write(operator: &str, symbol: &Symbol) -> bool`
(`crates/rum-runtime/src/runtime/attributes.rs:210`) is the guard: it pushes
the `wrsym` message and returns `false` when protected. Reuse it verbatim —
do not write a second protection check.

## The change

1. Thread the operator name into the store path. Add an
   `operator: &'static str` parameter to `finish_held_update` and
   `store_held_update`. All nine `finish_held_update` call sites (`:42`,
   `:60`, `:74`, `:89`, `:103`, `:137`, `:171`, `:189`, `:242`) already have
   the operator string literal in hand (they pass it to
   `current_held_update_value` on the line just above) — pass the same
   literal. Update the `suspend_with_frame` closure at `:371` (which also
   calls `store_held_update`) to capture and forward it.
2. At the TOP of `store_held_update`, before any `definitions` mutation,
   pull the owner symbol from the target — `HeldUpdateTarget::Symbol(sym)`
   and `HeldUpdateTarget::Part { symbol, .. }` both carry one — and call
   `self.ensure_symbol_not_protected_for_write(operator, &owner)`. If it
   returns `false`, DO NOT write: return a fail-closed value and skip the
   store entirely. The message is already emitted by the guard.
3. Fail-closed return shape: match the existing CHECK-stage behavior as
   closely as the available data allows. The CHECK stage returns the inert
   held call (`Expr::call(system_expr(operator), args.to_vec())`); at store
   time you no longer hold the original `args`, so returning the inert head
   call is not reconstructable identically. Return the pre-update value
   instead: read the owner's current `own_value` and return it (the target
   is unchanged, and the caller sees the un-mutated value). Do NOT return
   `updated` — that would surface a value that was never stored. Whatever you
   choose, it MUST NOT look like a silent success and MUST NOT store; the
   test pins the observable contract (message + unchanged target).

Keep the diff minimal: the parameter threading plus the guard call plus the
early return. No refactor of the surrounding held-update machinery, no
behavior change to the ten arithmetic/list forms (their update cannot
trigger the guard, so they are byte-identical), no new `unsafe`, no change
to any public type.

## Regression test

Add to `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs`,
right after the existing `held_updates_respect_protected_symbols` (line
354). Model it on that test. Assert all three observations:

```rust
#[test]
fn apply_to_rechecks_protection_at_store_time() {
    let mut session = core_session();

    session
        .eval_text("x = 1; f = Function[v, (Protect[x]; v + 100)]")
        .unwrap();
    let blocked = session.eval_text("ApplyTo[x, f]").unwrap();

    // message emitted by the store-time guard
    assert_eq!(blocked.messages[0].symbol, "ApplyTo");
    assert_eq!(blocked.messages[0].tag, "wrsym");
    // the write did NOT land: x is still its pre-update value
    assert_eq!(session.eval_text("x").unwrap().value.to_string(), "1");
    // and the symbol is (now) protected
    assert_eq!(
        session.eval_text("Attributes[x]").unwrap().value.to_string(),
        "{Protected}"
    );
}
```

Before your fix this test FAILS (x becomes 101, no message). Run it once
before the fix to confirm it captures the bug, then after to confirm green
— paste both. Keep the existing `held_updates_respect_protected_symbols`
passing (the already-protected `AddTo[x, 2]` path is unchanged).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/definitions/definition_updates.rs` (the
    parameter threading + the store-step guard + early return).
  - `crates/rum-stdlib/src/tests/incidental_definitions_updates.rs` (the one
    new test).
- MUST NOT touch: `attributes.rs` (reuse the existing guard as-is), any
  other crate file, `docs/**`, `TODO.md`, the tracker, the findings file,
  `Cargo.*`, git.

## Verification (MANDATORY — run all, paste verbatim)

```
# the new test fails BEFORE the fix (run once to prove it catches the bug):
cargo test -p rum-stdlib apply_to_rechecks_protection_at_store_time
# ... apply the fix ...
cargo test -p rum-stdlib apply_to_rechecks_protection_at_store_time   # now green
cargo test -p rum-stdlib incidental_definitions_updates               # the whole file green
cargo fmt --check
cargo test --workspace                                                # full tree green
```

Paste the pass/fail line for the new test both before and after, the file
suite total, and the workspace total.

## Hard rules

- No `git commit`, branch, push, or git config. Leave changes in the working
  tree for the gate.
- No new `unsafe`; `rum-core`/`rum-runtime` stay zero-`unsafe`.
- This is NOT a hot path (the held-update store runs once per compound
  assignment, not per eval step), so no microbench A/B is required — but do
  not add per-call cost to the ten forms that already fail closed; the guard
  call is one `symbol_is_protected` lookup at store time, which is
  acceptable and paid only on the mutation path.
- Minimal diff; no drive-by reformatting or renaming.
- Reuse `ensure_symbol_not_protected_for_write` — do not hand-roll a second
  protection check or a second message.
- Paste real verbatim output. If the workspace suite is not green, say so;
  do not claim a total you did not see.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — the before/after lines for the new test, the file suite
   total, the workspace total, `fmt` clean, all verbatim.
3. Confirmation that the ten arithmetic/list forms are behaviorally
   unchanged (their update evaluates no user code).
4. Stop-loss / open questions (empty if none).

## Repurpose note

This file previously held a re-verification brief (have Codex re-check the
disputed claim). That step is superseded: the claim was settled directly —
the `ApplyTo` bypass is real and reproduced, the other ten forms fail
closed, and the root cause is the missing store-step guard fixed here. The
findings file records the full reversal. Straight to the fix.
