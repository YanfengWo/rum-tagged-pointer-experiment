# brief-040: implement Dot product — the `.` infix operator + the `Dot` builtin

## Goal

rum has neither the `.` (Dot) infix operator nor the `Dot` builtin. `a . b` fails
to parse (`expected ,, found .`) and `Dot[{1,2,3},{4,5,6}]` stays inert (`Dot`
not registered). Implement BOTH so dot/matrix products work. Found by the
2026-06-23 syntax-axis survey; this is the last 🔴 operator that is a real
COMPUTATION (not just sugar). Two parts (the builtin is PRIMARY/independently
testable; the `.` operator is the sugar on top).

## WolframKernel semantics (verified — match these)

```
Dot[{1,2,3},{4,5,6}]            ==> 32                 (vector·vector -> scalar)
{{1,2},{3,4}} . {5,6}          ==> {17, 39}           (matrix·vector)
{1,2} . {{1,2},{3,4}}          ==> {7, 10}            (vector·matrix)
{{1,2},{3,4}} . {{1,0},{0,1}}  ==> {{1, 2}, {3, 4}}   (matrix·matrix)
{a,b} . {c,d}                   ==> a c + b d           (SYMBOLIC elements OK — builds Plus/Times)
a . b . c                        (n-ary; left-fold Dot[Dot[a,b],c] is fine)
a . b   (a,b NOT lists)         ==> a . b               (inert — can't contract non-lists)
```
Precedence: Dot binds TIGHTER than `*` (Times) and LOOSER than `^`/`**`. rum scale
(higher=tighter): `Times 60 < Dot ~63 < ** 65 < Power 70`. Left-associative.
Verify the grouping with the WK-match probes; adjust the value if a probe fails.

## Part 1 (PRIMARY) — the `Dot` builtin (real computation)

A new builtin (mirror an existing numeric/structure builtin for the
metadata/catalog/dispatch wiring; put the impl in a sensible module, e.g. a new
`linear_algebra.rs` or alongside structure ops). `Dot[a, b, …]`:

- **Contract** the LAST axis of `a` with the FIRST axis of `b`. Works for any
  rank via the general rule; the MUST-HAVES are rank 1 (vector) and rank 2
  (matrix): vec·vec→scalar, mat·vec→vec, vec·mat→vec, mat·mat→mat. (General
  n-dim tensor contraction is a nice-to-have; rank-1/2 correct + higher-rank
  inert is an acceptable first cut — say which you did.)
- The contracted entries are built as `Plus[Times[a_i, b_i], …]` (or the rum
  builders for them), so NUMERIC elements fold to a number (`32`) and SYMBOLIC
  elements stay symbolic (`a c + b d`). Do NOT require numeric elements — Dot
  needs matching DIMENSIONS, not numbers.
- **n-ary**: `Dot[a, b, c]` = left-fold (`Dot[Dot[a, b], c]`).
- **Inert fallback**: if an arg is not a List, or the contracted dimensions do
  not match, return the unevaluated `Dot[…]` call (do not error-throw; WL warns
  + returns unevaluated — inert is acceptable here). `Dot[x]` (1 arg) -> `x`;
  `Dot[]` -> per WK it's `Dot[]` inert (fine).
- rum is rationals-only ([[no floats]]); element products use the existing exact
  arithmetic — `Dot[{1/2, 2}, {2, 1/2}]` -> `2`.

## Part 2 (SECONDARY) — the `.` infix operator

The lexer already has `lex_dot_prefixed_token` (`crates/rum-syntax/src/lexer.rs:293`)
that disambiguates `.`-led tokens: `.5` (number via `lex_number`'s
`has_decimal_point`), `..`/`...` (Repeated/RepeatedNull, brief-037). Add the bare
`.` case: **when a `.` is NOT followed by a digit and NOT followed by another `.`,
emit a `Dot` token.** Then wire it like the brief-036 operators:
- TokenKind `Dot` (`trivia.rs`), `BinaryOp::Dot` (`ast.rs`, token ".", precedence
  ~63, LEFT-assoc), `helpers.rs` mapping, `lowering.rs` arm -> `canonical_call(
  "Dot", vec![lhs, rhs])`.
- `a.b.c` lowering nested `Dot[Dot[a,b],c]` is fine (Dot folds; nested computes
  the same as flat — like StringExpression in brief-038).

**Stop-loss:** if the bare-`.` lexing regresses decimals (`1.5`/`.5`/`1.`) or
`..`/`...`, SHIP PART 1 (the builtin) ALONE and report Part 2 with what you found
— do NOT break numbers or Repeated to force `.`.

## Scope

- MAY modify: `crates/rum-syntax/src/{lexer.rs,trivia.rs,ast.rs,parser/helpers.rs,
  parser/tests.rs}`, `crates/rum-runtime/src/runtime/{lowering.rs,metadata.rs,
  eval_dispatch/surface.rs (or symbolic.rs),builtin_catalog/*.rs}` + a new impl
  module for Dot + test modules.
- MUST NOT touch: decimal-number lexing or `..`/`...` (Repeated) behavior (only
  ADD bare-`.`); `crates/rum-core` shape; any `mma-hol` clone or v5/v6
  checkpoints; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except this report;
  `.git`, git config, `Cargo.lock`.
- New builtin/operator only runs when used — no default-path cost.

## rum / Rust pitfalls (read twice)

1. **Bare-`.` lexing must not regress numbers or Repeated.** Mandatory regression
   tests: `1.5`/`.5`/`1.`/`1.5e3` stay numbers, `x_..`/`x_...` stay Repeated/
   RepeatedNull. Use the stop-loss rather than break them.
2. **Dot computes symbolically** — match DIMENSIONS, not element types; build
   `Plus[Times[…]]` so numbers fold and symbols stay.
3. **Precedence is best-estimate** — if `a.b*c` / `a.b^c` probes disagree with
   WK, adjust the value and rerun.
4. **Inert, not error,** on dimension mismatch / non-list args.
5. **Minimal diff**; mirror brief-036 (operator) + an existing builtin (compute).

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# Part 1 — builtin:
$BIN --eval 'Dot[{1,2,3},{4,5,6}]'                   # 32
$BIN --eval '{{1,2},{3,4}} . {5,6}' || $BIN --eval 'Dot[{{1,2},{3,4}},{5,6}]'   # {17, 39}
$BIN --eval 'Dot[{1,2},{{1,2},{3,4}}]'               # {7, 10}
$BIN --eval 'Dot[{{1,2},{3,4}},{{1,0},{0,1}}]'       # {{1, 2}, {3, 4}}
$BIN --eval 'Dot[{a,b},{c,d}]'                       # a c + b d  (symbolic)
$BIN --eval 'Dot[{1/2,2},{2,1/2}]'                   # 2
$BIN --eval 'FullForm[Dot[x,y]]'                     # Dot[x, y]  (inert, non-list)
# Part 2 — operator (if shipped):
$BIN --eval 'FullForm[Hold[a . b]]'                  # Hold[Dot[a, b]]
$BIN --eval '{1,2,3} . {4,5,6}'                      # 32
$BIN --eval 'FullForm[Hold[a.b*c]]'                  # Hold[Times[Dot[a,b],c]]   (Dot tighter than *)
$BIN --eval 'FullForm[Hold[a.b^c]]'                  # Hold[Dot[a,Power[b,c]]]    (^ tighter than Dot)
# regressions — numbers / Repeated unchanged:
$BIN --eval 'FullForm[Hold[1.5]]'                    # number (3/2), NOT Dot
$BIN --eval 'FullForm[Hold[.5]]'                     # number (1/2)
$BIN --eval 'FullForm[Hold[x_..]]'                   # Hold[Repeated[Pattern[x,Blank[]]]]
$BIN --eval 'FullForm[Hold[x_...]]'                  # Hold[RepeatedNull[Pattern[x,Blank[]]]]
```

## Hard rules

- Do NOT commit/branch/push/touch git config. Leave changes in the tree; Claude gates.
- NEVER edit any `mma-hol` clone or the checkpoints. Minimal diff.
- Files within Scope only; full machine access, no network; commands = file edits,
  `cargo`, release `rum-cli`. Paste real verbatim output; if Part 2 hits the
  stop-loss, ship Part 1 and say so; fix any WK-disagreeing precedence probe.

## Report format

1. What changed — Part 1 (the `Dot` builtin + which ranks) and Part 2 (the `.`
   lexing/operator) or its stop-loss, files + line ranges.
2. Verification — each command verbatim.
3. Verdict — Dot computes vec/mat products (numeric + symbolic), `.` parses with
   correct precedence, number/Repeated regressions clean.
4. Stop-loss / open questions.
