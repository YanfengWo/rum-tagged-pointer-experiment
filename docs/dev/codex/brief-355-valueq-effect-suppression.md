# brief-355 — `ValueQ` trial evaluation must not escape

## Goal

`ValueQ` promises, in the generated reference, to test "by trial evaluation
without committing side effects". Session state does roll back, but three
effects escape. Close all three **without changing any `ValueQ` answer except
where an effect is now refused**.

Measured on `615d0bd7`:

```text
SeedRandom[123]; e1 = RandomInteger[10^6]; e2 = RandomInteger[10^6]
SeedRandom[123]; ValueQ[RandomInteger[10^6]]; RandomInteger[10^6]
  ->  equals e2, so the trial consumed the first draw

ValueQ[Print["LEAK"]]      ->  prints LEAK when a host print sink is installed
ValueQ[DumpSave[p, x]]     ->  writes the file
```

This is tier 2 of the `ValueQ` work (see `TODO.md`). Tier 1 — answering
provable shapes by lookup without evaluating at all — is a later brief and is
**not** in scope here. Do not add a lookup fast path.

## Director's rulings — implement exactly these

1. **PRNG state is restored** around the trial. `prng_state` is `Copy` and
   `session_api.rs:528` (`evaluate_with_random_state_restored`, used by
   `BlockRandom`) is the existing pattern, including its suspension handling.
2. **The host print sink is suppressed** for the duration of the trial. Output
   produced during it goes to the buffered `prints` vector, which the existing
   rollback snapshot already restores, so nothing reaches the host and nothing
   remains afterwards.
3. **`FileWrite` and `Process` are refused** during the trial by narrowing the
   capability policy, then restoring it. These are the only two capabilities
   whose effects an in-memory snapshot cannot undo.
   - `FileRead`, `Clock`, `Diagnostic` stay allowed: they do not change the
     outside world, and any session-visible consequence is already rolled back.
   - `Random` stays allowed because ruling 1 restores its state.
   - `Interactive` stays allowed and **its current behavior is unchanged**:
     `ValueQ[InputString["…"]]` still answers `True` through the
     `InputRequired` arm, leaves no pending request, and does not suspend.
4. **A refused capability makes `ValueQ` answer `False`**, through the existing
   `Err(_) => false` arm. `ValueQ[DumpSave[path, x]]` therefore changes from
   `True` to `False`. That is the one intended answer change in this brief;
   state it in the report and cover it with a test.
5. **Every other answer is unchanged.** The existing `ValueQ` semantics tests
   are the assertion.

## Context pointers

- `crates/rum-runtime/src/runtime/messages/builtins_messages_inspect.rs:37-56`
  — `eval_value_q`: snapshot, `evaluate`, restore, compare.
- `crates/rum-runtime/src/runtime/session/session_api.rs:505-543` —
  `prng_state` and `evaluate_with_random_state_restored`.
- `crates/rum-runtime/src/runtime/session/session_helpers.rs:227-233` —
  `push_print`: with a sink installed the text leaves immediately; without one
  it goes to `prints`, which the snapshot covers. This is why the existing
  rollback test passes while the leak is real — it runs with no sink.
- `crates/rum-runtime/src/runtime/capabilities.rs:2-13` — the eight
  capabilities; `is_unrestricted()` / `allows()` are the policy API.
  Enforcement points: `eval_dispatch/mod.rs:135`, `eval_core/symbol.rs:62`,
  `builtins_context_loader.rs:68` — verify that inventory and report it.
- `crates/rum-runtime/src/runtime/tests/definitions_and_rules.rs:355-425` —
  `valueq_uses_rollback_trial_evaluation_without_side_effects`, whose `Print`
  assertion checks the buffered vector and so cannot see the sink leak.

## Deliverable

- The three suppressions above, applied only for the trial and restored
  afterwards on **every** exit path: success, error, and suspension.
- Suppression must nest correctly: a `ValueQ` inside a `ValueQ`, and a `ValueQ`
  inside a `BlockRandom`, must each restore exactly one level.
- No change to the default evaluation path. Nothing here may cost anything
  when `ValueQ` is not running.

### Tests

Exact values, never absence of error:

- PRNG: the `SeedRandom` sequence above, asserting the draw after the trial
  equals the draw that would have come without it.
- Print **with a sink installed** — this is the assertion the existing test
  lacks. Capture the sink's output and assert it is empty, then assert a
  normal `Print` outside the trial still reaches it.
- `FileWrite`: `ValueQ[DumpSave[path, x]]` answers `False` and the file does
  not exist afterwards.
- `Process`: the analogous refusal, if a `Process` builtin is reachable in
  tests; if none is, say so.
- `FileRead` still works inside a trial and its session effects roll back.
- `Clock`, `Diagnostic`, `Random` still work inside a trial.
- `Interactive` unchanged: `ValueQ[InputString["…"]]` is `True`, no pending
  request, no suspension, no messages.
- Nesting: `ValueQ[ValueQ[…]]`, and `ValueQ` inside `BlockRandom`, restore
  correctly — assert the sink and the PRNG afterwards.
- Every existing `ValueQ` semantics answer unchanged.
- **Assert after the trial, not only its return value** — the guard line's
  lesson: a gate that only checks the operation's result cannot see what the
  operation left behind.

### A/B

Not a hot path, so the test suite is the gate. If you touch anything on the
default evaluation path, that is a design error — report it instead of
measuring it.

## Scope

- Files you MAY modify:
  `messages/builtins_messages_inspect.rs`; `session/session_helpers.rs`;
  `session/session_api.rs`; `capabilities.rs` if the policy needs a narrowing
  helper; `model/session.rs` if the trial needs a saved-state field; runtime
  and stdlib tests.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any behavior change outside a
  `ValueQ` trial is a stop, except the single intended `FileWrite`/`Process`
  answer change in ruling 4.*
- MUST NOT touch: the `ValueQ` decision logic itself (no lookup fast path),
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.lock`,
  `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **Restore on every exit path**, including the suspension one. A leaked
   suppression would silently disable the host's print sink or capabilities
   for the rest of the session — far worse than the bug being fixed.
2. **Do not restore by assuming the prior value.** Save what was there and put
   that back, so nesting works.
3. **The print sink is `Option<Box<dyn FnMut>>`** — moving it out and back is
   the natural approach; make sure a panic or early return cannot lose it.
4. **`Interactive` must keep working**, including the `InputRequired`
   short-circuit that makes `ValueQ` answer `True` without suspending.
5. **No default-path cost.**
6. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network. Test files go under a temp
  directory, never the repo.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file, mechanically-forced files listed separately.
2. The capability enforcement-point inventory you verified.
3. How restoration is guaranteed on every exit path, including suspension.
4. Verification — each command with verbatim output.
5. The exact-value test matrix, including the sink-installed Print test and
   the nesting tests.
6. The one intended answer change, stated plainly.
7. Stop-loss / open questions.
