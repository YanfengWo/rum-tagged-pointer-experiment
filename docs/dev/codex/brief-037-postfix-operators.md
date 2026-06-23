# brief-037: add postfix operators — `!` `!!` `'` `..` `...`

## Goal

Add five missing POSTFIX operators to rum's parser, mirroring the existing
postfix machinery (`a++` → `Increment[a]`, `PostfixOp` enum in `ast.rs:241/386`,
lowered in `lowering.rs`). Run AFTER brief-036 (shared parser files). Found by the
2026-06-23 syntax-axis survey. One new tiny builtin is needed (`Factorial2`);
the other targets exist (`Factorial`, `Derivative`, `Repeated`, `RepeatedNull`).

| op | meaning | lowers to | note |
|----|---------|-----------|------|
| `n!`   | Factorial      | `Factorial[n]`        | `Factorial` builtin exists |
| `n!!`  | Factorial2     | `Factorial2[n]`       | **`Factorial2` builtin MISSING — add it** (double factorial) |
| `f'`   | Derivative     | `Derivative[k][f]`    | k = number of consecutive primes: `f'`→`Derivative[1][f]`, `f''`→`Derivative[2][f]` |
| `a ..` | Repeated       | `Repeated[a]`         | `Repeated` matcher already works |
| `a ...`| RepeatedNull   | `RepeatedNull[a]`     | `RepeatedNull` exists |

Precedence: postfix `!`/`!!`/`'` bind VERY tight (tighter than `Power`=70; place
~78, above `Power`/`Map` but below `Pattern`=80). `..`/`...` are looser (used in
patterns; place ~76 and verify). Use the same precedence slot the existing
`Increment`/`Decrement` postfix ops use as the anchor; verify with the WK probes.

## WolframKernel semantics (verified — match these)

```
2 a!     ==> Times[2, Factorial[a]]        (! tighter than *)
a! + b   ==> Plus[Factorial[a], b]
a!^2     ==> Power[Factorial[a], 2]        (! tighter than ^)
f''      ==> Derivative[2][f]
2 f'     ==> Times[2, Derivative[1][f]]
{a ..}   ==> List[Repeated[a]]
{a ...}  ==> List[RepeatedNull[a]]
5!       ==> 120        6!! ==> 48   (= 6*4*2)   7!! ==> 105 (= 7*5*3*1)   0!! ==> 1
```

## Touch points

1. **Lexer** (`lexer.rs`): emit a `'` token (Prime) — a NEW char arm. For `!`,
   the `!` token already exists (Not / `!=` / `=!=`); postfix use is a PARSER
   decision (see #4), no new lexer token, but confirm `!=`/`=!=`/`!` still lex.
   For `!!`: prefer the parser consuming TWO `!` postfix as `Factorial2` OR a `!!`
   token — pick the cleaner; if a `!!` token, place it before `!`. For `..`/`...`:
   add `..` and `...` tokens (most-specific `...` first), but a numeric literal
   (`1.5`, `.5`, `1.`) MUST still lex as a number — only a `.`/`..`/`...` NOT part
   of a number becomes the operator.
2. **TokenKind** (`trivia.rs`): add the new kinds + `as_str`.
3. **PostfixOp** (`ast.rs:241`): add `Factorial`, `Factorial2`, `Derivative`,
   `Repeated`, `RepeatedNull` to the enum; fill `token()` and `precedence()`.
4. **Parser**: recognize these in POSTFIX position (after an operand), like the
   existing `++`/`--`. `!` is prefix `Not` in operand position but `Factorial` in
   postfix position — the Pratt parser distinguishes these naturally (nud vs led);
   mirror how `++` (postfix Increment) vs `++a` (prefix PreIncrement) is handled.
   For `'`: consume a RUN of consecutive `'` and record the count k.
5. **Lowering** (`lowering.rs`): `Factorial`→`canonical_call("Factorial",[x])`;
   `Factorial2`→`canonical_call("Factorial2",[x])`; `Derivative`→
   `Expr::call(Expr::call(Derivative, [Integer k]), [x])` i.e. `Derivative[k][x]`;
   `Repeated`→`canonical_call("Repeated",[x])`; `RepeatedNull`→`…("RepeatedNull",[x])`.
6. **New builtin `Factorial2`** (`crates/rum-runtime/src/runtime/arithmetic.rs` +
   `builtin_catalog/numeric.rs` + `metadata.rs` `BuiltinHandler` + dispatch in
   `eval_dispatch/surface.rs`) — mirror the existing `Factorial`/`eval_factorial`
   (grep it). Double factorial: `n!! = n(n-2)(n-4)…` down to 1 or 2; `0!!=1`,
   `1!!=1`; non-negative-integer only, else inert. Reuse the integer work-limit
   guard the existing `Factorial` uses.

## Scope

- MAY modify: `crates/rum-syntax/src/{lexer.rs,trivia.rs,ast.rs,parser/**}`,
  `crates/rum-runtime/src/runtime/{lowering.rs,arithmetic.rs,metadata.rs,
  eval_dispatch/surface.rs,builtin_catalog/numeric.rs}` + the matching test
  modules.
- MUST NOT touch: the 1-arg `.`/number lexing for actual numeric literals (only
  ADD operator `.`/`..`/`...` for non-number positions); `crates/rum-core` shape;
  any `mma-hol` clone or checkpoints; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**`
  except this report; `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Number vs operator `.`** is the trap: `1.5`, `.5`, `1.`, `1.5e3` MUST stay
   numbers; `a ..` / `a ...` are operators. Add a regression test for both.
2. **Prefix-vs-postfix `!`**: `!a` is `Not[a]`, `a!` is `Factorial[a]`, `a != b`
   is `Unequal`. Keep all three correct — the parser position decides.
3. **`'` run-counting**: `f'''` → `Derivative[3][f]`. Lower as `Derivative[k][f]`
   (a curried call), NOT `Derivative[f]`.
4. **`Factorial2` is non-negative-integer only**; reuse the work-limit guard;
   non-integer/negative → inert call.
5. **Minimal diff**; mirror `Increment`/`eval_factorial`. Precedence is a
   best-estimate — if a WK-match probe fails, adjust and rerun.

## Verification (MANDATORY — paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'FullForm[Hold[2 a!]]'        # Hold[Times[2, Factorial[a]]]
$BIN --eval 'FullForm[Hold[a! + b]]'      # Hold[Plus[Factorial[a], b]]
$BIN --eval '5!'                          # 120
$BIN --eval 'FullForm[Hold[f'']]'         # Hold[Derivative[2][f]]
$BIN --eval 'FullForm[Hold[2 f']]'        # Hold[Times[2, Derivative[1][f]]]
$BIN --eval 'FullForm[Hold[{a ..}]]'      # Hold[List[Repeated[a]]]
$BIN --eval 'FullForm[Hold[{a ...}]]'     # Hold[List[RepeatedNull[a]]]
$BIN --eval '6!!'                         # 48
$BIN --eval '7!!'                         # 105
$BIN --eval '0!!'                         # 1
$BIN --eval 'MatchQ[{1, 1, 1}, {1 ..}]'   # True
# regressions — numbers and Not/Unequal unchanged:
$BIN --eval 'FullForm[Hold[1.5]]'         # Hold[1.5`...]  (a Real number, NOT an operator)
$BIN --eval 'FullForm[Hold[! a]]'         # Hold[Not[a]]
$BIN --eval 'FullForm[Hold[a != b]]'      # Hold[Unequal[a, b]]
```

## Hard rules

- Do NOT commit/branch/push/touch git config. NEVER edit `mma-hol`/checkpoints.
- Files within Scope only; full machine access, no network; commands = file edits,
  `cargo`, release `rum-cli`. Minimal diff. Paste real verbatim output; fix any
  WK-disagreeing probe before reporting.

## Report format

1. What changed — per op + the `Factorial2` builtin, touch points, precedences used.
2. Verification — each command verbatim.
3. Verdict — postfix ops parse/lower right, `Factorial2` computes, number/Not
   regressions clean.
4. Stop-loss / open questions.
