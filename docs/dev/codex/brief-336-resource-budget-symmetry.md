# brief-336 — close three resource-budget gaps

Three builtins do work proportional to a user-supplied size without charging it
against a budget, while their siblings do. This is an asymmetry within families
that already have the mechanism, not a missing mechanism.

## Measured today (release CLI, verified by the director — do not re-derive)

Charged, and refusing as they should:

```
StringRepeat["a", 10^6]          → ! evaluation exceeded generated element limit (100000)
PadLeft[{1}, 10^6]               → same
ConstantArray[0, 10^6]           → same
Table[0, {10^6}]                 → same
Table[i, {i,1,10^6}]             → same
2^(10^7)                         → ! evaluation exceeded integer work limit (100000)
```

Not charged:

```
StringPadLeft["a", 10^8]         → succeeds, 0.7 s, builds a 100 MB string
StringPadRight["a", 10^8]        → succeeds, 0.7 s, same
Table[0, {i,1,10^4},{j,1,10^4}]  → succeeds, 3.3 s — 10^8 elements, each
                                   iterator under the limit, the product far over
Table[0, {i,1,10^5},{j,1,10^5}]  → still running at 25 s
Fibonacci[10^6]                  → succeeds, 21.3 s
Fibonacci[10^8]                  → still running at 25 s
```

The `Table` line is the interesting one: the single-iterator form is charged
and the multi-iterator form is not, so two iterators that each pass the check
multiply past it.

## The mechanisms already exist

- `session.enforce_generated_element_limit_for_count(count)` and
  `…_for_big_count(&big)` — used across `random.rs`, `integer_digits.rs`,
  `list_plumbing.rs`.
- `IntegerWorkBudget::new(session.resource_limits().integer_work_limit)` — used
  throughout `number_theory.rs`.

**Use them. Do not invent a new limit, a new message tag, or a new config
knob.** `random.rs:528` already charges a *product* of dimensions through
`…_for_big_count`; that is the shape the multi-iterator `Table` wants.

## What to build

1. **`StringPadLeft` / `StringPadRight`** — charge the padded length, the way
   `StringRepeat` charges its repeat count.
2. **`Table` with multiple iterators** — charge the product of the iterator
   counts, before generating. Compute it in a width that cannot overflow, since
   the point is that the product exceeds what each factor suggests.
3. **`Fibonacci`** — charge through `IntegerWorkBudget`, as its `number_theory`
   siblings do.

Then look at the rest of each family and say what you found. The three above
are what the director's probes reached; a sibling with the same shape and no
charge is the same defect, and this is the pass where it gets found. Report any
you fix and any you deliberately leave.

One observation to check rather than act on: `StringTake["abc", 10^9]` returns
`"abc"` rather than reporting an out-of-range request. Say whether that matches
the documented behavior. Do not change it in this brief.

## What must not change

- **Anything under the limits.** Ordinary sizes must behave exactly as they do
  today, in value and in message.
- The limit values themselves, and the two existing messages.
- The builtins that already charge correctly.
- `docs/reference/builtins.md` unless a catalog spec genuinely changes; if it
  does, regenerate it and say why.

## Scope

Files you MAY modify:

- `crates/rum-stdlib/src/strings/` — the padding builtins
- `crates/rum-stdlib/src/functional_ops/arrays/generators.rs` and the `Table`
  iterator path
- `crates/rum-stdlib/src/number_theory.rs`
- other `rum-stdlib` sources holding siblings you find with the same gap
- test modules under `crates/rum-stdlib/src/tests/`

Out of scope: the limit mechanisms themselves, `rum-core`, `rum-runtime`,
session configuration, and the limit values.

Do not add unconditional cost to the default evaluation path — a budget check
on a small argument must stay a comparison, not an allocation.

## Deliverable

- The three charged, through the existing mechanisms.
- The family sweep, with what you found and what you left.
- Tests: just under the limit (accepted, correct value), just over (refused
  with the existing message), and the multiplying `Table` case specifically.
- A test that the multi-iterator product is charged even when every individual
  iterator passes.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise. **Run each probe as its own
script**: the CLI stops at the first error, so a combined script hides
everything after the first refusal. That mistake was made while writing this
brief.

Probe both directions for each of the three: a size that must still work, and
the measured sizes above that must now be refused. Report elapsed time — a
refusal that takes 20 seconds is not a refusal.

No microbench A/B: these are argument-size checks, not evaluation hot paths.
Say so if you believe otherwise.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.**
- **Leave no `TODO` or `FIXME` markers.** This repository has none by policy.
- Do not change a limit's value or its message text.
- Minimal diff; no drive-by reformatting.
- **Stop-loss is real.** If charging one of the three cannot be done without
  changing results under the limit, stop and report that one.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per builtin, with line ranges and which mechanism it uses.
2. The family sweep: siblings checked, fixed, and left alone with reasons.
3. Verification — per-probe, both directions, with elapsed times.
4. The `StringTake` observation.
5. Stop-loss / open questions.
