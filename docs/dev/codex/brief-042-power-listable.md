# brief-042: add Listable to `Power` (the one remaining real arithmetic Listable gap)

## Goal

`Power` is the only substantive remaining missing-Listable bug from the
attribute-axis scan: WolframKernel marks `Power` Listable, rum does not, so
`Power[{2,3},2]` stays inert `Power[List[2, 3], 2]` instead of threading to
`{4, 9}`. The 12 number-theory builtins were fixed in brief-041; this is the
arithmetic-core follow-up that was deliberately split out (Power sits on a hot
canonicalization path, so it gets its own brief with heavy regression). When
done, `Power` threads over list arguments exactly like WK, and every existing
scalar / symbolic / Times-canonicalization behavior is byte-for-byte unchanged.

## Context pointers

- Read `AGENTS.md` and the `docs/next-session-handoff.md` `2026-06-24` top block.
  brief-041 (committed `b7840a7`) already added Listable to
  Abs/Round/Factorial/Factorial2/Denominator/Numerator/Mod/Quotient/PowerMod/
  GCD/LCM/CoprimeQ and dropped FullForm's HoldAll.
- This is an ATTRIBUTE-TABLE-only change, exactly like brief-041: Listable
  threading is generic in the evaluator; adding the attribute alone makes
  `Power` thread (proven — brief-041's 12 builtins thread with no handler code).
  You do NOT touch `BuiltinHandler::Power`, the evaluator, or dispatch.
- **There are PRE-EXISTING `Power`/`Times` canonicalization gaps that are OUT OF
  SCOPE — do NOT try to fix them, and do NOT treat them as regressions.** rum
  (current, before your change) already differs from WK here, and adding
  Listable must leave these EXACTLY as they are now:
  - `x^2 x^3` → rum `Times[Power[x, 2], Power[x, 3]]` (WK combines to `x^5`; rum
    does not — pre-existing, unrelated to Listable).
  - `x^2/x` → rum `Times[Power[x, -1], Power[x, 2]]` (WK gives `x`).
  - `Power[0,0]` → rum inert `Power[0, 0]` (WK `Indeterminate`).
  These are recorded so you assert rum's CURRENT output as the unchanged guard,
  not WK's. If your change alters ANY of them, that is a real regression — stop.
- Registration site: `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`
  (~line 29), currently:
  ```rust
  BuiltinSpec::ordinary(
      "Power",
      "Raises an exact number to an integer power when possible.",
      BuiltinHandler::Power,
  ),
  ```
- The single-attribute idiom is `AttributeSet::from_attribute(Attribute::Listable)`
  (same as brief-041's edits in this file; `AttributeSet`/`Attribute` already
  resolve here via `use super::*;` — `Plus`/`Times` in this file use them).
- **Only `Listable`.** WK's `Power` also has `OneIdentity`/`NumericFunction`;
  do NOT add those — they are a different axis (pattern-matching / numeric
  tagging), not the identified bug, and OneIdentity can change pattern matching.
  Listable only.

## WolframKernel semantics to match (verified this session against WK)

New threaded behavior (rum currently inert on all of these; rum-cli renders
lists as `List[...]`):

```
Power[{2,3},2]        => {4, 9}            rum-cli: List[4, 9]
Power[{2,3},{3,2}]    => {8, 9}            rum-cli: List[8, 9]
Power[{a,b},2]        => {a^2, b^2}        rum-cli: List[Power[a, 2], Power[b, 2]]
Power[2,{2,3}]        => {4, 8}            rum-cli: List[4, 8]
Power[{2,3},0]        => {1, 1}            rum-cli: List[1, 1]
Power[{1,2,3},-1]     => {1, 1/2, 1/3}     rum-cli: List[1, 1/2, 1/3]
```

Scalar / symbolic / canonicalization guards — these are rum's CURRENT outputs
and MUST stay byte-identical after the change (the last three are the
out-of-scope pre-existing gaps; assert rum's value, not WK's):

```
Power[2,3]     => 8
Power[5,1]     => 5
Power[7,0]     => 1
Power[1,9]     => 1
Power[a,2]     => Power[a, 2]
2^10           => 1024
x^2 x^3        => Times[Power[x, 2], Power[x, 3]]     (pre-existing gap; unchanged)
(2 x)^2        => Power[Times[2, x], 2]               (pre-existing gap; unchanged)
x^2/x          => Times[Power[x, -1], Power[x, 2]]    (pre-existing gap; unchanged)
Power[0,0]     => Power[0, 0]                          (pre-existing gap; unchanged)
```

## The exact edit

`crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs` (~line 29) — change
`Power` from `ordinary` to `ordinary_with_attributes` with Listable, keeping the
same name/doc/handler:

```rust
    BuiltinSpec::ordinary_with_attributes(
        "Power",
        "Raises an exact number to an integer power when possible.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Power,
    ),
```

Do NOT touch any other spec in this file (Divide/Subtract/Max/Min/Total and the
brief-041 builtins stay as they are).

### Tests

Extend the same test module brief-041 used,
`crates/rum-runtime/src/runtime/tests/structure_ops/surface.rs` (it already has
the `eval_to_string` helper and the
`arithmetic_and_number_theory_builtins_thread_over_lists` test), with a focused
`power_threads_over_lists_without_disturbing_canonicalization` test covering BOTH
the 6 threaded cases AND the 10 unchanged guards above (so the canonicalization
gaps are locked in as deliberate, not silently changed).

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs` (Power only)
  - `crates/rum-runtime/src/runtime/tests/structure_ops/surface.rs` (one test)
- MUST NOT touch: `BuiltinHandler::Power` or any handler body, the evaluator, the
  dispatch surface, any other builtin's attributes, the `Power`/`Times`
  canonicalization logic; any `mma-hol` clone or snapshot checkpoints;
  `crates/rum-core` `Expr`/`Symbol`/serde shape; `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**` except this brief's report; `.git`, git config,
  `Cargo.lock`.
- **No default-path cost.** Compile-time constant-table change only.

## rum / Rust pitfalls (read twice)

1. **Attribute-only — no handler/canonicalization edits.** If you find yourself
   editing `BuiltinHandler::Power` or any Times/Plus combination logic, STOP —
   out of scope and a sign you misread the mechanism.
2. **The 3 pre-existing canonicalization gaps are NOT yours to fix.** `x^2 x^3`
   staying `Times[Power[x, 2], Power[x, 3]]`, `x^2/x` not simplifying, and
   `Power[0,0]` inert are deliberate out-of-scope guards. Assert rum's CURRENT
   output. Do not "improve" them; do not flag them as failures.
3. **Listable threads BEFORE the scalar handler**, so each element hits the
   unchanged `Power` handler — that is why no handler change is needed and why
   internal scalar Power calls (always on non-list args) are unaffected.
4. **Minimal diff.** Only the Power spec constructor changes; no reordering, no
   doc rewrites, no drive-by edits.
5. **Stop-loss.** If any guard above changes, or a threaded result disagrees with
   the table for a reason you cannot localize to the attribute, STOP and report —
   do NOT edit handlers to force it.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# new threading:
$BIN --eval 'Power[{2,3},2]'         # List[4, 9]
$BIN --eval 'Power[{2,3},{3,2}]'     # List[8, 9]
$BIN --eval 'Power[{a,b},2]'         # List[Power[a, 2], Power[b, 2]]
$BIN --eval 'Power[2,{2,3}]'         # List[4, 8]
$BIN --eval 'Power[{2,3},0]'         # List[1, 1]
$BIN --eval 'Power[{1,2,3},-1]'      # List[1, 1/2, 1/3]
$BIN --eval 'Attributes[Power]'      # List[Listable]

# unchanged guards (scalar/symbolic/canonicalization — must be byte-identical):
$BIN --eval 'Power[2,3]'             # 8
$BIN --eval 'Power[7,0]'             # 1
$BIN --eval 'Power[a,2]'             # Power[a, 2]
$BIN --eval '2^10'                   # 1024
$BIN --eval 'x^2 x^3'                # Times[Power[x, 2], Power[x, 3]]   (pre-existing gap)
$BIN --eval 'x^2/x'                  # Times[Power[x, -1], Power[x, 2]]  (pre-existing gap)
$BIN --eval 'Power[0,0]'             # Power[0, 0]                        (pre-existing gap)
```

Paste every line verbatim; match the VALUE (note any render-form nuance, do not
fake it).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate and commits.
- NEVER edit any `mma-hol` clone or the snapshot checkpoints.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Do NOT touch any handler, the evaluator, dispatch, or canonicalization — this
  is an attribute-table-only change.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the Power spec edit (file + line) and the test added; one line
   each.
2. Verification — each command with verbatim output (test counts + every eval).
3. Verdict — Power threads per the table, all scalar/symbolic/canonicalization
   guards byte-identical, no handler touched, tests green.
4. Stop-loss / open questions — anything unsure (empty if none).
