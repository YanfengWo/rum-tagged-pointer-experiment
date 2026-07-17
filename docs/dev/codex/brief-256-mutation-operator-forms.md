# Brief 256 — mutation-family operator forms via `SubValuesHoldAll` (wl15/wfr batch 2b-I)

## Goal

The nine compound-assignment heads — `AddTo`, `AppendTo`, `ApplyTo`,
`AssociateTo`, `DivideBy`, `KeyDropFrom`, `PrependTo`, `SubtractFrom`,
`TimesBy` — carry `SubValuesHoldAll` and gain WL15-style operator
forms: `AppendTo[val][x]` behaves exactly like `AppendTo[x, val]`
(the target `x` arrives HELD thanks to brief-255's attribute; the
capture `AppendTo[val]` is a quiet inert callable following the
existing operator-form convention). Direct binary calls are
byte-identical to today.

## Context pointers

- Read `AGENTS.md` and `docs/dev/wl15-wfr-hold-design.md` (§4 B1
  ruling: FULL FAMILY) before touching anything.
- Key files / mechanism (verified 2026-07-17):
  - `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs:63-139`
    — the operator-wrapper recognizer chain (`is_callable_operator_wrapper`
    OR-chain, `apply_callable_operator_wrapper_to_args`,
    `invalid_callable_operator_wrapper_argx_symbol`). Add ONE new family
    (e.g. `is_mutation_operator_wrapper` / `apply_mutation_operator_wrapper_to_args`)
    following the existing families' shape: recognize a unary capture
    `Head[val]` for the nine names; application to exactly one argument
    rebuilds the data-first binary call `Head[target, val]`; wrong
    application arity goes through the argx helper like the other
    families.
  - `crates/rum-runtime/src/runtime/scoping/callable.rs:8-56` —
    `try_apply_function` consumes the chain; no changes expected there
    (the rebuilt call is evaluated by the existing machinery; the
    rebuilt `Head[target, val]` re-enters ordinary dispatch where
    HoldFirst keeps the target unevaluated).
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:63` — the
    wrapper-head gate; your new family must be reachable from
    `is_callable_operator_wrapper` so this gate lights up (no direct
    edit expected beyond the chain).
  - The nine heads' `BuiltinSpec` entries in the kernel
    `builtin_catalog/` (locate by name): add `SubValuesHoldAll` to each
    entry's attributes. If an attribute-mirror or lockstep test exists
    for these entries, update it and SAY SO in the report.
  - The nine heads' unary-arity handler paths (their eval functions in
    `kernel_data/`): a 1-argument call must become a QUIET inert
    capture (return the call unchanged, NO argx message) — this is the
    established operator-form convention (`Take[2]` alone is quiet
    inert). Verify what each head does today for 1 argument and change
    message-emitting paths to quiet inert.
  - `docs/reference/builtins.md` — GENERATED; attributes render in the
    reference, so regenerate via
    `cargo run -p rum-stdlib --bin generate-builtin-reference` and keep
    the drift test green.
- Acceptance is mechanical: tests + probes + the microbench A/B.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/kernel_data/functional_data.rs`
  - the kernel `builtin_catalog/*.rs` file(s) containing the nine
    heads' `BuiltinSpec` entries (attributes; plus an `argx` message
    declaration ONLY if the wrapper argx path needs one that is
    missing — the push_message debug assertion is the arbiter)
  - the `kernel_data/*.rs` file(s) implementing the nine heads' unary
    arity path (quiet-inert change only)
  - `crates/rum-runtime/src/runtime/tests/` — new tests plus EXISTING
    tests that assert the old unary-argx behavior (each such edit is
    necessary ripple; list every one in the report with the old/new
    expectation)
  - `docs/reference/builtins.md` (regeneration output only)
- MUST NOT touch: brief-255's planner code (`eval_dispatch/mod.rs`,
  `metadata.rs`) — consume the attribute, do not modify the mechanism;
  stdlib catalog; everything else.
- No unconditional default-path cost: the new recognizer runs only on
  compound-head dispatch; symbol-head calls must be untouched.

## Design decisions (already ruled — implement, do not re-litigate)

1. Family list: exactly the nine heads above (user ruling B1).
2. Capture semantics: `Head[val]` under HoldFirst stores `val` AS
   GIVEN (held); the value evaluates when the rebuilt binary call
   evaluates it by its normal rules — i.e. capture-time side effects
   do NOT fire, application-time they fire once. Pin this with a
   counter test (`w = AppendTo[c = c + 1]` leaves c at 0; `w[x]`
   brings c to 1).
3. Unary capture is QUIET inert (no message) — matches `Take[2]`.
4. Application arity: a capture applied to ≠1 arguments goes through
   `invalid_callable_operator_wrapper_argx_symbol` with the same
   message shape the other families use.
5. The rebuilt call is `Head[target, val]` with `target` the RAW held
   outer argument — never evaluate it during the rewrite.
6. Do NOT touch `ConformSchema`/`SchemaErrors` (that is brief-257).

## Deliverable

- The wrapper family + attribute adoption + quiet-inert unary paths.
- Tests (runtime tests; measured expectations — probe first, then pin):
  - `x = {1, 2}; AppendTo[3][x]; x` → `{1, 2, 3}` and the call returns
    the updated value, byte-identical to `AppendTo[x, 3]`.
  - one probe per remaining head (AddTo/SubtractFrom/TimesBy/DivideBy
    on a numeric symbol; AssociateTo/KeyDropFrom on an association
    symbol; PrependTo on a list; ApplyTo with a pure function).
  - capture-timing counter test (decision 2).
  - protection still enforced: a capture applied to a `Protected`
    symbol fails with the same message as the direct binary call.
  - unary capture quiet: `AppendTo[3]` evaluates to itself, zero
    messages.
  - direct binary no-regression pins for at least AppendTo and AddTo.
- Microbench A/B (MANDATORY): base = HEAD before your change,
  candidate = with it; best-of-2 per side; report downvalue-tier
  labels + same-tier neighbors; the noise-floor rule (unaffected
  controls define the floor). Note: the panel has no compound-head
  label — say so.

## rum / Rust pitfalls (read twice)

1. Expected values in tests must be MEASURED via the release CLI
   first, never predicted (standing lesson).
2. rum-cli probes MUST feed stdin: `printf '' | … --eval '…'`.
3. The nine heads are lvalue-writers: their binary handlers run the
   protection recheck machinery (`store_held_update` — brief-209).
   Your rewrite path must land in the SAME binary handler, inheriting
   those guards; add no second store path.
4. Suspension safety: do not collapse `suspend_with_frame*` shapes.
5. Hot paths avoid owned-tree clones; the recognizer works on borrowed
   `&Expr`.
6. Stop-loss: same failure twice for the same reason → STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace --exclude rum-jupyter   # sandbox loopback denial excludes the 9 native ZeroMQ tests; note it
```

Then the microbench A/B, and paste VERBATIM live probes (all via
`printf '' | cargo run -q -p rum-cli -- --eval …`):

```
x = {1, 2}; {AppendTo[3][x], x}
y = 10; {AddTo[5][y], y}
a = <|"k" -> 1|>; {AssociateTo["j" -> 2][a], a}
a2 = <|"k" -> 1, "j" -> 2|>; {KeyDropFrom["k"][a2], a2}
z = 7; {ApplyTo[Function[t, t + 1]][z], z}
c = 0; w = AppendTo[c = c + 1]; l = {}; {c, w[l], c, l}
Protect[p]; AppendTo[1][p]
AppendTo[3]
x2 = {1, 2}; {AppendTo[x2, 3], x2}
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; must-touch conflicts → STOP and
  report.
- Nothing outside this repo, no network. Commands: Scope edits,
  `cargo`, release `rum-cli`, microbench scripts, `python3`.
- Testing Policy hard line: rum's own documented behavior only.
- No new `unsafe`. Minimal diff. Probe budget best-of-2 per A/B side.
- Paste real verbatim output; never fake a green.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why (include the
   located catalog entries and every ripple-test edit).
2. Verification — commands + verbatim output.
3. Verdict — behavior table + A/B reading with the stated noise floor.
4. Stop-loss / open questions (empty if none).
