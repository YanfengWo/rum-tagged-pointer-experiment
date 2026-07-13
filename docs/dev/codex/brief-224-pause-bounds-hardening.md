# Brief 224 — Pause bounds hardening (no host-process panic)

Fix brief — unit-10 batch 1 (safety, S). Source:
`docs/dev/usability-pass/kernel-c-findings.md` (top-gap #2, Evidence E5).
The cheapest real safety win of the unit: a script must not be able to
crash the process.

## The defect (gate-reproduced)

```
printf '' | rum-cli --eval 'Pause[10^100]'
thread 'main' panicked at .../core/src/time.rs:944:23:
cannot convert float seconds to Duration: value is either too big or NaN
(process EXIT CODE 101)
```

`expr_as_nonnegative_duration` (`runtime/builtins_input.rs:419-435`)
parses the argument to `f64`, rejects only non-finite and negative
values, then calls `Duration::from_secs_f64(seconds)`. A large finite
value like `1e100` passes the check but overflows `Duration`'s
representable range, and `Duration::from_secs_f64` PANICS on overflow —
terminating the host process (exit 101). `Pause[-1]` already fails closed
correctly (`Pause::numnm`).

## Required change

- A finite non-negative `seconds` that is NOT representable as a
  `Duration` returns `None` from `expr_as_nonnegative_duration` (same
  path as the negative case), so the caller emits the EXISTING
  `Pause::numnm` message ("Non-negative machine-sized number expected …")
  and returns inert — no panic, no process exit. The message text already
  fits: `10^100` is not machine-sized.
- Determine the representable bound HONESTLY: `Duration::from_secs_f64`
  panics when the value is non-finite, negative, or `>= 2^64` seconds
  (whole-seconds overflow). Use `Duration::try_from_secs_f64` if this
  crate's MSRV (check `Cargo.toml` / `rust-version`; the tree targets a
  pinned MSRV) provides it — it returns a `Result` and is the clean
  fix — OR, if not available at MSRV, guard with an explicit bound
  (`seconds < 2f64.powi(63)` is a safe conservative ceiling; state the
  exact bound you chose and why it cannot panic).
- Valid durations are BYTE-IDENTICAL: `Pause[0]`, `Pause[1/1000]`,
  small integer/rational seconds behave exactly as today. Do not change
  the sleep semantics, the capability (`Clock`), or the message tag.

## Method

1. Confirm the MSRV and whether `try_from_secs_f64` is available; state
   the finding. Pick the conversion accordingly.
2. Baseline-verify the preservation pins (`Pause[0]`, a small rational)
   before AND after — they must be identical.
3. Tests: `Pause[10^100]` → `Pause::numnm` + inert, NO panic;
   `Pause[-1]` still `numnm`; `Pause[0]`/`Pause[1/1000]` unchanged; a
   huge rational (`10^100/3`) also fails closed. Put them where the
   existing Pause tests live (`tests/holding_and_control_io.rs` — survey
   and name the file).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace
printf '' | cargo run -q -p rum-cli -- --eval 'Pause[10^100]' ; echo "exit=$?"
# ^ must print the Pause::numnm message and exit 0 (NOT 101); also probe
#   Pause[-1], Pause[0], Pause[1/1000] and paste them
git status --short            # only MAY-listed files
```

Note: measure the CLI exit code with a DIRECT invocation (a trailing
`| tail` reports the pipe's exit code, not rum-cli's — the unit-10
head-start hit exactly this).

## Scope

MAY modify:
- `crates/rum-runtime/src/runtime/builtins_input.rs` (the
  `expr_as_nonnegative_duration` bound only)
- Tests: `crates/rum-runtime/src/runtime/tests/holding_and_control_io.rs`
  (or the actual Pause test home the survey names), one new test allowed

MUST NOT touch: the `Clock` capability, the message tag/text, sleep
semantics, `Input`/`InputString`/`Dialog` (separate decision), anything
else. No A/B (no hot-path change), no mma-hol gate (no loader change).

## Hard rules

- Do NOT `git commit`, branch, or push. No network, no new dependencies,
  no new `unsafe`.
- The fix must make the panic UNREACHABLE for any finite input — verify
  by construction (bound-check or `try_*`), not just by the one probe.
- Baseline-verify the preservation pins.

## Report format

1. The change: MSRV finding, conversion chosen, the exact bound.
2. Probe panel before/after (the panic gone; pins identical), verbatim,
   with the direct-invocation exit code.
3. Verification outputs verbatim.
4. Deviations / open questions (empty if none).
