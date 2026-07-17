# Brief 255 — `SubValuesHoldAll` attribute: mechanism only (wl15/wfr batch 2a)

## Goal

A new attribute `SubValuesHoldAll` (WL15 name, user ruling A1 in
`docs/dev/wl15-wfr-hold-design.md` §6) exists end-to-end: it parses,
prints, round-trips through `SetAttributes`/`ClearAttributes`/
`Attributes`, and changes exactly one behavior — for a call whose HEAD
is itself a Call (`f[…][args]`), if the base symbol (`f`) carries
`SubValuesHoldAll`, the OUTER arguments are prepared exactly as
`HoldAll` would prepare them (held; `Evaluate` pierces per argument;
`Unevaluated` stripped with the no-consumer restore; Sequence handling
identical to current `HoldAll`, which flattens). No builtin gains the
attribute in this brief (that is batch 2b); user-defined heads are the
only consumers. Symbol-head calls MUST NOT gain any new branch, lookup,
or allocation.

## Context pointers

- Read `AGENTS.md` and `docs/dev/wl15-wfr-hold-design.md` (§2 probe
  facts, §3 candidate A, §6 rulings) before touching anything.
- Key files:
  - `crates/rum-runtime/src/runtime/metadata.rs` — `Attribute` enum
    (`:32-48`), `bit()` (`:52-68`; use `1 << 15`, the LAST free u16
    bit — note in code that a 16th attribute needs a wider mask),
    name render (`:79` area), name parse (`:109` area), the
    all-attributes const list (`:249` area).
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs:4-23` —
    `build_call_dispatch_plan`: `attributes =
    cached_attribute_set_for_head(head)` yields the empty set for a
    compound head today; `base_head_symbol_of_nested_call(head)` is
    ALREADY computed at `:15-17` for the value-definition lookup.
    Piggyback there: when `head_symbol` is None and the base symbol's
    attribute set contains `SubValuesHoldAll`, make the plan's
    argument-preparation behave as `HoldAll`. Zero new walks.
  - `crates/rum-runtime/src/runtime/eval_call_prepare.rs` — the hold
    plan / argument preparation the flag must flow into (the same
    machinery `HoldAll` uses: Evaluate pierce, Unevaluated
    strip+restore, splice policy).
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs:22` area —
    add the system name next to the other attribute names.
  - `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs:390` area —
    the attribute-name string match; add the new name (MECHANICAL).
  - `crates/rum-stdlib/src/docgen.rs:397` area — the
    `Attribute -> name` arm (MECHANICAL).
  - `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs`
    — home for the new behavior tests.
- Acceptance is mechanical: the named tests + the microbench A/B gate.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/metadata.rs`
  - `crates/rum-runtime/src/runtime/eval_dispatch/mod.rs`
  - `crates/rum-runtime/src/runtime/eval_call_prepare.rs`
  - `crates/rum-runtime/src/runtime/namespace_tools/names.rs`
    (MECHANICAL: one name)
  - `crates/rum-runtime/src/runtime/expr_runtime_helpers.rs`
    (MECHANICAL: one match arm)
  - `crates/rum-stdlib/src/docgen.rs` (MECHANICAL: one match arm)
  - `crates/rum-runtime/src/runtime/tests/holding_and_control_part2.rs`
  - `crates/rum-runtime/src/runtime/tests/namespace_names.rs` ONLY if
    a names-inventory test enumerates attribute names and needs the
    addition (state it in the report if so).
- Files/areas you MUST NOT touch: any stdlib catalog attribute
  assignment (no builtin gets the attribute), `docs/**` except your own
  report, `Cargo.lock`, everything else.
- Do NOT add unconditional cost to the default evaluation path: a
  SYMBOL-head call must compile to the identical fast path (no new
  branch/lookup). The compound-head path may add at most one attribute
  lookup on the already-computed base symbol.

## Design decisions (already ruled — implement, do not re-litigate)

1. Name: `SubValuesHoldAll`. Bit: `1 << 15`.
2. Semantics: one rule — outer arguments of a compound-head call whose
   base symbol carries the attribute are prepared EXACTLY as if the
   call head carried `HoldAll`. All secondary semantics (Evaluate
   pierce, Unevaluated strip/restore, Sequence flattening, upvalue
   stage still runs) INHERIT from the existing HoldAll machinery —
   do not hand-roll a second hold path.
3. Depth: the rule applies at every application level (`f[a][s1][s2]`:
   when `f[a][s1]` is itself being applied to `s2`, the same base-`f`
   check governs `s2`; the inner application already governed `s1`).
   Normal (inner-most direct) arguments evaluate as today.
4. Introspection: `Attributes[f]` lists it after `SetAttributes[f,
   SubValuesHoldAll]`; `ClearAttributes` removes it; protection
   interplay is whatever `SetAttributes` already enforces — no special
   cases.
5. If honoring the attribute requires touching special-form dispatch,
   suspension frames, or the plan cache invalidation story in a
   non-obvious way: STOP and report instead of improvising.

## Deliverable

- The attribute end-to-end + the planner change.
- Behavior tests (session-level, in `holding_and_control_part2.rs`):
  - baseline pin: WITHOUT the attribute, both side-effecting args
    evaluate (the E9 shape: counter reaches 2, `HoldComplete[1, 2]`).
  - with `SetAttributes[op, SubValuesHoldAll]`: inner captured arg
    still evaluates, outer arg arrives HELD (assert via a counter that
    stays at 1 and/or the held expression shape — VERIFY the exact
    rendered form empirically first; do not assert from prediction).
  - `Evaluate[…]` in the outer position re-enables evaluation.
  - `Sequence` in the outer position behaves exactly as it does for a
    `HoldAll` symbol head today (probe first, then pin).
  - attribute round-trip: set/list/clear.
- Microbench A/B (MANDATORY, this is call-planning): base = HEAD before
  your change, candidate = with it. Report the downvalue-tier labels
  (`downvalue-exact-hit` and same-tier neighbors) plus any
  subvalue/compound-head label present in the panel; if the panel has
  no compound-head label, SAY SO explicitly. Best-of-2 per side; the
  brief's verdict standard: no hot-label regression outside the
  same-run noise floor (unaffected control labels define the floor).

## rum / Rust pitfalls (read twice)

1. Timing numbers come from `scripts/core-language-microbench.sh`
   (per-label `timings.tsv`), profiling env OFF, one variable per A/B.
2. The attribute cache: `cached_attribute_set_for_head` may cache per
   head — make sure a `SetAttributes[f, SubValuesHoldAll]` AFTER a
   prior application of `f[…][…]` still takes effect (write a test
   exercising exactly that order; if the existing cache already
   invalidates on SetAttributes for symbol heads, the compound path
   must inherit that invalidation).
3. Evaluator flows are suspension-safe: do not collapse any
   `suspend_with_frame*` shape.
4. Hot paths avoid owned-tree clones: `Arc` clone over deep clone.
5. Expected values in tests must be MEASURED, not predicted (this
   pass's standing lesson) — probe the release CLI first, paste what
   you saw, then pin it.
6. rum-cli probes MUST feed stdin: `printf '' | … rum-cli --eval '…'`
   (open-stdin filter mode otherwise blocks).
7. Stop-loss is real: same failure twice for the same reason → STOP
   and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
```

(9 rum-jupyter native ZeroMQ tests fail deterministically under the
sandbox's loopback denial; run
`cargo test --workspace --exclude rum-jupyter` as your green gate and
note the exclusion — the director's out-of-sandbox gate covers them.)

Then the microbench A/B per the Deliverable, and paste VERBATIM live
probes (all through `printf '' | cargo run -q -p rum-cli -- --eval …`):

```
c = 0; op[a_][b_] := HoldComplete[a, b]; {op[c = c + 1][c = c + 1], c}
c = 0; SetAttributes[op, SubValuesHoldAll]; op[a_][b_] := HoldComplete[a, b]; {op[c = c + 1][c = c + 1], c}
c = 0; SetAttributes[op, SubValuesHoldAll]; op[a_][b_] := HoldComplete[a, b]; {op[c = c + 1][Evaluate[c = c + 1]], c}
SetAttributes[f, SubValuesHoldAll]; {Attributes[f], ClearAttributes[f, SubValuesHoldAll], Attributes[f]}
```

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- Do NOT modify files outside Scope; if you believe you must, STOP and
  report.
- Touch NOTHING outside this repo, no network. Commands: file edits in
  Scope, `cargo`, the release `rum-cli`, the microbench scripts,
  `python3` for parsing.
- Testing Policy hard line: rum's own documented behavior only; no
  external-engine comparisons.
- No new `unsafe`.
- Probe budget: best-of-2 per A/B side, cap total probe runs, state the
  cap; derive timeouts from measured runs via `scripts/run-bounded.py`.
- Minimal diff. Paste real verbatim output; never fake a green.

## Report format (your final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — commands + verbatim output (test counts, A/B
   per-label rows for target + same-tier neighbors, probes).
3. Verdict — behavior table + the A/B reading (state the noise floor
   from the unaffected controls).
4. Stop-loss / open questions (empty if none).
