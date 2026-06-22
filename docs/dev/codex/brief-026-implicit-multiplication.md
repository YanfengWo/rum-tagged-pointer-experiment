# brief-026: parse implicit multiplication (juxtaposition) — unblocks auto/Arith.wl & the number tower

## Goal

rum's parser must accept implicit multiplication by juxtaposition: two adjacent
primaries separated by whitespace are `Times`, e.g. `a b` → `Times[a, b]`,
`2 x` → `Times[2, x]`, `g #` → `Times[g, Slot[1]]`, `a (b + c)` →
`Times[a, Plus[b, c]]`. Today rum has NO juxtaposition multiply, so any such
source fails to parse. This silently breaks the WHOLE mma-hol number tower:
`auto/Arith.wl` line 1963 `assocScale[g_, a_Association] := DeleteCases[Map[g # &,
a], 0]` (the `g #` is `Times[g, #]`) throws `Get::syntax: expected ,, found #`,
so Arith.wl loads NOTHING (`Names["HOL`Auto`Arith`*"]` is empty), `arithProve` is
undefined, and every `arithProve[...]` call in `Int.wl`/`Rat.wl`/the entire
`Real/*` analysis layer returns the inert symbol → the kernel rejects it as
`concl: not a theorem`. So one missing parser feature gates 15 libraries (ℤ→ℚ→ℝ).
When done, juxtaposition parses as `Times`, `auto/Arith.wl` loads (arithProve is
defined), all existing tests stay green, and no currently-valid parse changes.

## Context pointers

- Read `AGENTS.md` and the PROGRESS entry for the checkpoint-chain finding
  (loading stalls at the number tower because of this). This was root-caused
  from the cold load: `FullForm[Hold[g #]]` and `FullForm[Hold[g h]]` both error;
  `FullForm[Hold[g + #]]` works — so the gap is juxtaposition, not Slot.
- Key files:
  - `crates/rum-syntax/src/parser/front.rs` — `parse_binary_expr_internal`
    (around line 69). The led loop (lines 82-118) parses infix operators. At line
    96 `let Some(op) = self.current_binary_op() else { break; };` — when the
    current token is NOT a binary operator the loop BREAKS. **This is where
    juxtaposition must be handled instead of breaking.**
  - `crates/rum-syntax/src/parser/helpers.rs` — `token_starts_primary()` (used by
    `token_starts_expression`, line ~133) tells whether the current token begins
    a primary (number, symbol, `(`, `{`, `<|`, string, Slot `#`, …); use it as
    the gate. `current_binary_op()` maps `TokenKind::Star => BinaryOp::Times`.
  - `BinaryOp::Times` + `BinaryOp::Times.precedence()` + the existing
    `next_min_precedence_for_binary_op` helper.

## The fix (precise)

In `parse_binary_expr_internal`'s led loop, replace the bare `break` when there
is no binary operator with: if the current token STARTS A PRIMARY
(`self.token_starts_primary()`) and `BinaryOp::Times.precedence() >=
min_precedence`, treat the juxtaposition as an implicit `Times`:

```rust
let op = match self.current_binary_op() {
    Some(op) => op,
    None => {
        // Implicit multiplication (juxtaposition): `a b` == Times[a, b].
        let times = BinaryOp::Times;
        if self.token_starts_primary() && times.precedence() >= min_precedence {
            let next_min = next_min_precedence_for_binary_op(times);
            // NOTE: do NOT bump — there is no operator token; the current
            // token is the start of the right operand.
            let rhs = self.parse_binary_expr_internal(next_min, allow_function_postfix)?;
            let span = lhs.span().merge(rhs.span());
            lhs = SyntaxExpr::binary(times, lhs, rhs, span);
            continue;
        }
        break;
    }
};
if op.precedence() < min_precedence { break; }
self.bump();
... (existing rhs handling for the explicit `op`) ...
```

Critical points:
- **Gate on `token_starts_primary()`, NOT `token_starts_expression()`** — the
  latter also returns true for prefix `+ - !`, which must stay as their binary
  operators (`a - b` is subtraction, not `a * (-b)`). `-`/`+` are already binary
  ops via `current_binary_op`, so they never reach this branch; the primary gate
  keeps it that way.
- **Do NOT `self.bump()`** for the implicit case (there is no operator token).
- Use `BinaryOp::Times` precedence so juxtaposition binds like `*`:
  `a + b c` → `Plus[a, Times[b, c]]`; `a b + c` → `Plus[Times[a, b], c]`.
- `f[x]` is unaffected: the postfix `[...]` is consumed at the primary level, so
  `[` never reaches this branch (and `[` is not a primary start).

Multi-factor juxtaposition (`a b c`) may parse left-associatively as
`Times[Times[a, b], c]` rather than flat `Times[a, b, c]`; that is acceptable
(evaluation flattens via Times' Flat/Orderless). Do not add flattening here.

## Scope

- Files you MAY modify:
  - `crates/rum-syntax/src/parser/front.rs` (the led-loop change).
  - ONE test file under `crates/rum-syntax/` (or the existing parser tests
    `crates/rum-syntax/src/parser/tests.rs`) for the regression cases.
- MUST NOT touch: any `mma-hol` clone (for the Arith load check, copy to a
  throwaway scratch dir, or Get read-only — never write to
  `/Users/fft/Developments/mma-hol-acceptance`); `crates/rum-core` `Expr`/`Symbol`
  shape; the lowering (`lowering.rs`) — juxtaposition lowers via the existing
  `BinaryOp::Times` path already; `AGENTS.md`, `PROGRESS.md`, `TODO.md`,
  `docs/**` except your own report; `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Primary gate only.** `a - b`, `a + b`, `a; b`, `a, b`, `a -> b` must be
   unchanged (those tokens are binary ops or separators, not primary starts).
2. **No bump for implicit mult.** Bumping would drop the first token of the
   right operand.
3. **Precedence = Times.** Verify `a + b c`, `a b + c`, `a b c`, `2 x^2`
   (`Times[2, Power[x,2]]`) parse correctly.
4. **`&` and Span are handled before this branch** — keep that order; pure
   functions `g # &` must still attach the `&` (Function) to `Times[g, #]`.
5. **Minimal diff**; no reformatting of the loop beyond this change.
6. **Stop-loss:** if the change breaks more than a couple of existing parser
   tests in non-obvious ways, STOP and report (precedence interaction).

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# 1. juxtaposition now parses:
$BIN --eval 'FullForm[Hold[g #]]'        # expect Hold[Times[g, Slot[1]]]
$BIN --eval 'FullForm[Hold[2 x]]'        # expect Hold[Times[2, x]]
$BIN --eval 'FullForm[Hold[a b]]'        # expect Hold[Times[a, b]]
$BIN --eval 'FullForm[Hold[a (b + c)]]'  # expect Hold[Times[a, Plus[b, c]]]
$BIN --eval 'FullForm[Hold[a + b c]]'    # expect Hold[Plus[a, Times[b, c]]]
$BIN --eval 'FullForm[Hold[Map[g # &, a]]]'  # expect Hold[Map[Function[Times[g, Slot[1]]], a]]
# control — unchanged:
$BIN --eval 'FullForm[Hold[a - b]]'      # expect Hold[Plus[a, Times[-1, b]]] (or rum's current form), NOT Times
$BIN --eval '{2 + 3, 2 x /. x -> 5, a b c}'  # eval still correct: {5, 10, Times[a,b,c]-ish}

# 2. the REAL unblock — auto/Arith.wl now loads (cold, via its dep prefix; ~5 min):
#    Get the prefix ErrorUtil..Simp + Pair + Num + Arith (Stable mode), then check.
#    (Use the pristine clone read-only for Get; do NOT write to it.)
$BIN --eval '$HOLEncapsulationMode="Stable"; $Path=Prepend[$Path,"/Users/fft/Developments/mma-hol-acceptance"]; Get/@("/Users/fft/Developments/mma-hol-acceptance/"<>#&/@{"ErrorUtil.wl","Types.wl","Terms.wl","Kernel.wl","Bootstrap.wl","Equal.wl","Bool.wl","Drule.wl","Tactics.wl","Printer.wl","Parser.wl","auto/PropTaut.wl","auto/Meson.wl","auto/Simp.wl","stdlib/Pair.wl","stdlib/Num.wl","auto/Arith.wl"}); Print["Arith names: ", Length[Names["HOL`Auto`Arith`*"]]]; Print["arithProve DVs: ", Length[DownValues[HOL`Auto`Arith`arithProve]]]; "done"'
#    EXPECT: Arith names > 0 and arithProve DVs > 0 (was 0/0 before this fix),
#    and NO `Get::syntax` error on stderr.
```

Paste verbatim: the parse repros (before showed errors), the control checks, and
the Arith-load result (names/DownValues now nonzero + clean stderr). The Arith
unblock is the acceptance.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone (read-only `Get` is fine).
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo (except read-only Get
  of the pristine clone), no network. Commands: file edits within Scope, `cargo`,
  the release `rum-cli`, `python3`.
- Minimal diff; no drive-by churn.
- You MUST paste real verbatim output; if a step is not green, say so — never
  fake it.

## Report format (final message)

1. What changed — front.rs led-loop change, line range, one-line why.
2. Verification — each command, verbatim output (parse repros, controls, the
   Arith-load names/DownValues).
3. The verdict — juxtaposition parses as Times; `auto/Arith.wl` now defines
   arithProve (names/DVs > 0); controls unchanged; tests green.
4. Stop-loss / open questions — any precedence interaction you were unsure about.
