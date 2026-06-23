# brief-041: attribute-conformance fixes — drop FullForm's spurious HoldAll, add missing Listable

## Goal

Two behaviorally-verified attribute-table conformance fixes found by the
ATTRIBUTE-axis scan (diff `Attributes` for every System symbol in rum∩WK,
dropping the Protected/ReadProtected/Locked noise):

1. **`FullForm` must NOT hold its argument.** rum registers `FullForm` with
   `HoldAll`; WolframKernel gives it no eval attributes. The bug: `FullForm[1+1]`
   renders `"Plus[1, 1]"` instead of `"2"` (rum holds `1+1` and renders the
   unevaluated input). WK evaluates the argument first, then renders. Remove the
   `HoldAll`.
2. **Add `Listable` to 12 number-theory/arithmetic builtins** that WK marks
   Listable but rum leaves attribute-free, so they do not thread over lists
   (`Abs[{-1,-2}]` stays inert `Abs[{-1, -2}]` instead of `{1, 2}`).

When done, `FullForm[1+1]` is `"2"` and the 12 builtins thread over list
arguments, matching WolframKernel — with every existing test still green and no
default-path cost.

## Context pointers

- Read `AGENTS.md` and the `docs/next-session-handoff.md` `2026-06-24` top block
  (the attribute-axis scan). This brief is the "fix the CONFIRMED attribute
  bugs" follow-up to that scan.
- **`Evaluate` is NOT a bug — DO NOT TOUCH IT.** The scan flagged `Evaluate`
  with EXTRA `HoldAll` and the handoff guessed it was broken
  (`Hold[Evaluate[1+1]]` "stays unevaluated"). That was a FullForm-induced
  MISDIAGNOSIS: `Hold[Evaluate[1+1]]` already evaluates to `Hold[2]` correctly
  (`Hold[Evaluate[1+1]] === Hold[2]` is `True`); the "broken" appearance came
  from inspecting it with `FullForm[...]`, which (because of bug #1) held the
  whole expression and rendered the un-fired input
  `"Hold[Evaluate[Plus[1, 1]]]"`. Fixing FullForm removes that artifact. Leave
  `Evaluate`'s attributes and its evaluator special-casing exactly as they are.
  (This is the [[diagnostic-tool-trust]] lesson: the instrument was the defect.)
- This is a pure ATTRIBUTE-TABLE change. The Listable threading is generic in
  the evaluator: adding the attribute alone makes a builtin thread (verified —
  `Boole` is Listable and `Boole[{True,False,True}]` already gives
  `{1, 0, 1}` with no handler code). You do NOT touch any `BuiltinHandler`
  implementation, the evaluator, or the dispatch surface.
- Registration mechanism (single source of truth — the `BuiltinSpec` constants):
  - `BuiltinSpec::ordinary(name, doc, handler)` — no attributes.
  - `BuiltinSpec::ordinary_with_attributes(name, doc, AttributeSet, handler)` —
    explicit attribute set; e.g. `numeric.rs` `Plus`/`Times` use
    `AttributeSet::empty().with(Attribute::Listable)...`. For a single
    attribute the existing idiom is
    `AttributeSet::from_attribute(Attribute::Listable)` (see
    `scoping.rs` `HoldComplete`/`Unevaluated`).
  - `BuiltinSpec::ordinary_with_hold(name, doc, HoldAttribute::HoldAll, handler)`
    — what `FullForm` currently (wrongly) uses.
- `Attribute::Listable` exists (`crates/rum-runtime/src/runtime/metadata.rs:36`);
  `AttributeSet::from_attribute` is the single-attribute constructor.

## WolframKernel semantics to match (verified this session against WK)

FullForm (the held-arg artifact and the regressions that must stay unchanged):

```
FullForm[1+1]                  => "2"                     (was "Plus[1, 1]" — THE FIX)
FullForm[Hold[1+1]]            => "Hold[Plus[1, 1]]"      (UNCHANGED: Hold holds, arg inert)
FullForm[{1,2}]                => "List[1, 2]"            (UNCHANGED)
Hold[Evaluate[1+1]] === Hold[2] => True                  (UNCHANGED guard: Evaluate untouched)
Attributes[FullForm]           => {}                       (HoldAll removed; was {HoldAll})
```

The 12 Listable builtins (scalar results already correct in rum — verified
`Abs[-5]=5 GCD[12,18]=6 LCM[4,6]=12 Mod[17,5]=2 Quotient[17,5]=3
PowerMod[2,10,7]=2 Numerator[3/4]=3 Denominator[3/4]=4 Factorial[4]=24
Factorial2[5]=15 CoprimeQ[4,9]=True Round[5/2]=2`; this brief only adds the
list-threading). WK reference (rum-cli prints lists as `List[...]`):

```
Abs[{-1,-2,3}]            => {1, 2, 3}
GCD[{12,18},6]           => {6, 6}
LCM[{4,6},3]             => {12, 6}
Mod[{5,7},3]             => {2, 1}
Quotient[{17,9},5]       => {3, 1}
PowerMod[{2,3},10,7]     => {2, 4}
Numerator[{3/4,5/6}]     => {3, 5}
Denominator[{3/4,5/6}]   => {4, 6}
Factorial[{3,4}]         => {6, 24}
Factorial2[{4,5}]        => {8, 15}
CoprimeQ[{4,9},3]        => {True, False}
Round[{3/2,5/2,7/3}]     => {2, 2, 2}
```

## The exact edits

### Edit 1 — `crates/rum-runtime/src/runtime/builtin_catalog/structure.rs` (around line 4)

Change the `FullForm` spec from `ordinary_with_hold(..., HoldAttribute::HoldAll, ...)`
to a plain `ordinary(...)`:

```rust
    BuiltinSpec::ordinary(
        "FullForm",
        "Renders the held argument as a canonical structural-form string in the current core expression subset.",
        BuiltinHandler::FullForm,
    ),
```

(The doc string mentions "held argument" — you MAY leave the string as-is to keep
the diff minimal, or trim "held " if it is one token; do not rewrite the
sentence. Minimal diff preferred — leaving it is fine.)

### Edit 2 — `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs`

For each of these 11 builtins, change `BuiltinSpec::ordinary(name, doc, handler)`
to `BuiltinSpec::ordinary_with_attributes(name, doc,
AttributeSet::from_attribute(Attribute::Listable), handler)`, keeping the same
name/doc/handler. They are all currently `ordinary`:

`Abs` (~:44), `Round` (~:49), `Factorial` (~:54), `Factorial2` (~:59),
`Denominator` (~:69), `Numerator` (~:74), `Mod` (~:79), `Quotient` (~:84),
`PowerMod` (~:89), `GCD` (~:94), `LCM` (~:99).

Example (Abs):

```rust
    BuiltinSpec::ordinary_with_attributes(
        "Abs",
        "Returns the absolute value of exact integers and rationals.",
        AttributeSet::from_attribute(Attribute::Listable),
        BuiltinHandler::Abs,
    ),
```

Do NOT touch `Total` (~:64) — it sits between these and is correctly non-Listable.
Do NOT touch `Power`/`Divide`/`Subtract`/`Max`/`Min` in this file (out of scope —
see "open questions").

### Edit 3 — `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs` (around line 298)

Change `CoprimeQ` the same way (`ordinary` → `ordinary_with_attributes` with
`AttributeSet::from_attribute(Attribute::Listable)`).

### Edit 4 — tests

Add a focused regression test (extend an existing module under
`crates/rum-runtime/src/runtime/tests/`, e.g. `arithmetic.rs`, or a new small
module wired into the test mod tree the same way its neighbors are). Cover:
- `FullForm[1+1]` → `"2"`; `FullForm[Hold[1+1]]` → `"Hold[Plus[1, 1]]"`;
  `Hold[Evaluate[1+1]] === Hold[2]` → `True` (the Evaluate-untouched guard).
- Each of the 12 threaded cases from the table above.
- One scalar guard each for a couple of them (e.g. `Abs[-5]` → `5`,
  `GCD[12,18]` → `6`) so the threading change did not disturb the scalar path.

If `AttributeSet` / `Attribute` are not already in scope in `numeric.rs` /
`logic.rs`, they are re-exported through `use super::*;` (the file already
references `BuiltinSpec`, `BuiltinHandler`; `Plus` in `numeric.rs` already uses
`AttributeSet`/`Attribute`, so the imports exist). Do not add new `use` lines if
the symbols already resolve.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/builtin_catalog/structure.rs` (FullForm)
  - `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs` (11 Listable)
  - `crates/rum-runtime/src/runtime/builtin_catalog/logic.rs` (CoprimeQ)
  - one test module under `crates/rum-runtime/src/runtime/tests/`.
- MUST NOT touch: any `BuiltinHandler` implementation, the evaluator, the
  dispatch surface, `Evaluate`'s registration or special-casing, any other
  builtin's attributes; any `mma-hol` clone or the v5/v6 snapshot checkpoints;
  `crates/rum-core` `Expr`/`Symbol`/serde shape; `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**` except this brief's report; `.git`, git config,
  `Cargo.lock`.
- **No default-path cost.** This is a compile-time constant-table change; the
  attribute is read on the existing attribute-lookup path. Add nothing per-call.

## rum / Rust pitfalls (read twice)

1. **Attribute-only change — no handler edits.** Listable threading is generic;
   the proof is `Boole[{True,False,True}] == {1,0,1}` already works. If you find
   yourself editing a `BuiltinHandler` body, STOP — that is out of scope and
   means you misread the mechanism.
2. **Do NOT touch `Evaluate`.** It is correct; the scan's "Evaluate broken" was
   a FullForm artifact (see Context). Changing it will break held-context
   evaluation.
3. **FullForm regressions are the `Hold[...]` forms.** Every existing FullForm
   test inspects `FullForm[Hold[...]]`; those are UNCHANGED by removing
   FullForm's own hold (the inner `Hold` still holds). If `cargo test` shows a
   FullForm test now failing, read it: a test that asserted the OLD buggy
   `FullForm[<unevaluated arithmetic>]` output must be corrected to WK behavior
   (evaluate-then-render) — NEVER revert the fix to satisfy a test that encoded
   the bug ([[prefer-general-wl-semantics]]). Report any such test you correct.
4. **Minimal diff.** Only the spec constructor per symbol changes; no reordering,
   no doc rewrites, no drive-by edits.
5. **Stop-loss.** If a threaded result disagrees with the table for a reason you
   cannot localize to the attribute change, STOP and report — do not start
   editing handlers to force it.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# FullForm fix + regressions:
$BIN --eval 'FullForm[1+1]'                    # "2"
$BIN --eval 'FullForm[Hold[1+1]]'              # "Hold[Plus[1, 1]]"
$BIN --eval 'FullForm[{1,2}]'                  # "List[1, 2]"
$BIN --eval 'Hold[Evaluate[1+1]] === Hold[2]'  # True   (Evaluate untouched)
$BIN --eval 'Attributes[FullForm]'             # List[]
$BIN --eval 'Attributes[Evaluate]'             # List[HoldAll]  (UNCHANGED — guard)

# Listable threading (rum-cli renders lists as List[...]):
$BIN --eval 'Abs[{-1,-2,3}]'                    # List[1, 2, 3]
$BIN --eval 'GCD[{12,18},6]'                    # List[6, 6]
$BIN --eval 'LCM[{4,6},3]'                      # List[12, 6]
$BIN --eval 'Mod[{5,7},3]'                      # List[2, 1]
$BIN --eval 'Quotient[{17,9},5]'                # List[3, 1]
$BIN --eval 'PowerMod[{2,3},10,7]'              # List[2, 4]
$BIN --eval 'Numerator[{3/4,5/6}]'              # List[3, 5]
$BIN --eval 'Denominator[{3/4,5/6}]'            # List[4, 6]
$BIN --eval 'Factorial[{3,4}]'                  # List[6, 24]
$BIN --eval 'Factorial2[{4,5}]'                 # List[8, 15]
$BIN --eval 'CoprimeQ[{4,9},3]'                 # List[True, False]
$BIN --eval 'Round[{3/2,5/2,7/3}]'              # List[2, 2, 2]
$BIN --eval 'Attributes[Abs]'                   # List[Listable]

# scalar guards (must be unchanged):
$BIN --eval 'Abs[-5]'                           # 5
$BIN --eval 'GCD[12,18]'                        # 6
$BIN --eval 'Round[5/2]'                        # 2
$BIN --eval 'Mod[17,5]'                         # 2
```

Paste every line verbatim. (rum-cli may render a one-element / quoted form
slightly differently — match the VALUE; if a render form differs from the
comment, note it, do not fake it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate and commits.
- NEVER edit any `mma-hol` clone or the snapshot checkpoints.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Do NOT touch `Evaluate`, any handler body, the evaluator, or the dispatch
  surface — this is an attribute-table-only change.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the FullForm spec edit, each of the 12 Listable spec edits
   (file + symbol + line), the test added; one line each.
2. Verification — each command with verbatim output (test counts + every eval).
3. Verdict — FullForm renders evaluated args, the 12 builtins thread per the
   table, scalar paths unchanged, Evaluate untouched, all tests green.
4. Stop-loss / open questions — note any FullForm test you had to correct, and
   anything unsure (empty if none).

## Open questions / deliberately deferred (NOT in this brief)

- WK also marks `Power`, `Divide`, `Subtract` Listable, and `Attributes`,
  `Range`, `Characters`, `ToExpression`, `ToUpperCase`, `Coefficient`, `Factor`,
  `Together` (the rest of the 23-symbol missing-Listable set). Deferred to a
  follow-up brief: the arithmetic-core trio (`Power`/`Divide`/`Subtract`) sit on
  hot canonicalization paths and need their own regression pass, and the rest are
  lower value for the HOL load. Do NOT add them here.
- rum has EXTRA `Listable` on `TextWords`/`ToCharacterCode` (WK has neither) and
  a batch of EXTRA/`mismatched`-`Hold*` on control/scoping symbols (`Catch`,
  `Until`, `BlockRandom`, `Stack`, `ReleaseHold`, `OptionValue`, ...). Those are
  a SEPARATE per-symbol triage (most are benign — rum hardcodes hold in the
  evaluator, not via the attr table) and are NOT in this brief.
