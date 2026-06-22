# brief-027: `#` slot spec must be adjacent — `# 6` is `Times[#, 6]`, not `Slot[6]`

## Goal

A slot specifier binds to `#` only when IMMEDIATELY adjacent (no whitespace):
`#6` is `Slot[6]`, `#x` is `Slot["x"]`, but `# 6` (with a space) is
`Times[Slot[1], 6]` and `# x` is `Times[Slot[1], x]` (a bare `Slot[1]` followed
by a separate juxtaposed token). Today rum's `parse_slot` consumes a following
Integer/Symbol as the slot spec EVEN ACROSS WHITESPACE, so `# 6` wrongly becomes
`Slot[6]` and `# x` becomes `Slot["x"]`. This silently corrupts the mma-hol
arithmetic prover: `auto/Arith.wl`'s `clearProvToInt` does
`Map[(# lcm) &, prov]` (scale each Association value by `lcm`), but `# lcm` is
mis-parsed as the named slot `Slot["lcm"]`, so the Farkas certificate is garbage
and `farkasFM` throws `certificate is not a positive constant` — stopping the
load at `stdlib/Finite.wl` and gating the whole number tower (it is reached only
now that brief-026 made `auto/Arith.wl` parse at all). When done, a
whitespace-separated token after `#` is NOT part of the slot, and `clearProvToInt`
scales correctly.

## Context pointers

- Read `AGENTS.md` and the PROGRESS entry for the checkpoint-chain findings
  (this is the new frontier after the juxtaposition fix `67a3bdc`). Root-caused
  from the rebuild: `farkasFM: certificate is not a positive constant` →
  `clearProvToInt`'s `Map[(# lcm) &, prov]` → `# lcm` parses as `Slot["lcm"]`.
  Confirmed minimal: `FullForm[Hold[# 6]]` gives `Hold[Slot[6]]` (should be
  `Hold[Times[Slot[1], 6]]`); `#6` (no space) correctly gives `Slot[6]`.
- Key file: `crates/rum-syntax/src/parser/helpers.rs` — `parse_slot` (lines
  4-35). The `is_sequence` check (line 7-8) ALREADY requires adjacency via
  `self.current().leading_trivia.is_empty()`. The slot-spec match arms (around
  lines 15, 20, 25) do NOT — that is the bug.

## The fix (precise)

In `parse_slot`, require the consumed Integer/Symbol to be ADJACENT to the `#`
(no leading trivia), mirroring the existing `is_sequence` adjacency check. Add an
`if self.current().leading_trivia.is_empty()` guard to the three consuming arms:

```rust
let spec = match (&self.current().kind, is_sequence) {
    (TokenKind::Integer(value), false) if self.current().leading_trivia.is_empty() => {
        let value = value.clone();
        end_span = self.bump();
        SlotSpec::Number(value)
    }
    (TokenKind::Symbol(value), false) if self.current().leading_trivia.is_empty() => {
        let value = value.clone();
        end_span = self.bump();
        SlotSpec::Named(value)
    }
    (TokenKind::Integer(value), true) if self.current().leading_trivia.is_empty() => {
        let value = value.clone();
        end_span = self.bump();
        SlotSpec::SequenceNumber(value)
    }
    _ if is_sequence => SlotSpec::Sequence,
    _ => SlotSpec::Anonymous,
};
```

With this, `# 6` leaves the `6` unconsumed (it has leading whitespace) → the slot
is `SlotSpec::Anonymous` (`Slot[1]`), and brief-026's juxtaposition handles the
separate `6` → `Times[Slot[1], 6]`. `#6` (no whitespace) is unchanged. Do NOT
touch the lexer (`'#' => self.single(TokenKind::Hash)` is correct) or the
juxtaposition code.

## Scope

- Files you MAY modify:
  - `crates/rum-syntax/src/parser/helpers.rs` (the three guards).
  - the parser test file `crates/rum-syntax/src/parser/tests.rs` (regression
    cases) and/or another test module if needed.
- MUST NOT touch: any `mma-hol` clone (read-only `Get` is fine for verification);
  `crates/rum-core`; the lexer or juxtaposition code beyond these guards;
  `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except your own report;
  `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **No-space cases unchanged.** `#6`→`Slot[6]`, `#x`→`Slot["x"]`, `#`→`Slot[1]`,
   `#1`→`Slot[1]`, `##`→`SlotSequence[1]`, `##2`→`SlotSequence[2]`. Only the
   WHITESPACE-separated cases change.
2. **`##` adjacency** (line 7-8) is already correct — keep it; add the same guard
   to the SequenceNumber arm so `## 2` is `Times[##, 2]`.
3. **Minimal diff** — only add the three `if` guards; no other edits to
   `parse_slot`.
4. **Stop-loss:** if existing slot tests break in a way that is NOT a genuine
   no-space case, STOP and report.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# 1. adjacency: no-space (unchanged) vs space (now Times):
$BIN --eval 'FullForm[Hold[#6]]'     # Hold[Slot[6]]            (unchanged)
$BIN --eval 'FullForm[Hold[# 6]]'    # Hold[Times[Slot[1], 6]]  (was Slot[6])
$BIN --eval 'FullForm[Hold[#x]]'     # Hold[Slot["x"]]          (unchanged)
$BIN --eval 'FullForm[Hold[# x]]'    # Hold[Times[Slot[1], x]]  (was Slot["x"])
$BIN --eval 'FullForm[Hold[#]]'      # Hold[Slot[1]]
$BIN --eval 'FullForm[Hold[##2]]'    # Hold[SlotSequence[2]]    (unchanged)
$BIN --eval 'FullForm[Hold[## 2]]'   # Hold[Times[SlotSequence[1], 2]]

# 2. the pure function now multiplies (was: returned the wrong slot):
$BIN --eval '(# 6 &)[5]'             # expect 30 (was Slot[6]-ish garbage)

# 3. the REAL unblock — clearProvToInt scales correctly now:
$BIN --eval 'cp[p_]:=Module[{lcm,scaled,g},lcm=Apply[LCM,Denominator/@Values[p]];scaled=Map[(# lcm)&,p];g=Apply[GCD,Values[scaled]];If[g===0,scaled,Map[(#/g)&,scaled]]]; cp[<|1 -> 1/2, 2 -> 1/3|>]'
#    expect Association[Rule[1, 3], Rule[2, 2]]  (was garbage with Slot["lcm"])
```

Paste verbatim: the adjacency repros (no-space unchanged, space now Times), the
pure-function `(# 6 &)[5] == 30`, and the `clearProvToInt` result
`<|1 -> 3, 2 -> 2|>`. (The full Finite.wl / number-tower re-verification is
Claude's follow-up via the checkpoint chain — you do NOT need to run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`, `python3`.
- Minimal diff; no drive-by churn.
- You MUST paste real verbatim output; if a step is not green, say so — never
  fake it.

## Report format (final message)

1. What changed — helpers.rs guards, line range, one-line why.
2. Verification — each command, verbatim output (adjacency repros, `(# 6 &)[5]`,
   `clearProvToInt`).
3. The verdict — whitespace after `#` is no longer part of the slot; no-space
   cases unchanged; `clearProvToInt` now gives `<|1 -> 3, 2 -> 2|>`; tests green.
4. Stop-loss / open questions — anything unsure.
