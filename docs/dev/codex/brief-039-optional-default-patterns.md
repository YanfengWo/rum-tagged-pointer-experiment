# brief-039: fix default/optional patterns — `x_:v` -> Optional, and `x_.` syntax

## Goal

Make rum's default/optional patterns correct. This is a PARSER-only fix — rum's
matcher + `Default` infrastructure already work when handed the right
`Optional[...]` form (verified: `MatchQ[f[1], f[1, Optional[x_, 9]]]` -> `True`,
`g[1, Optional[x_, 9]] := {1, x}; g[1]` -> `{1, 9}`, and 1-arg
`Optional[x_]` fills from `Default[head]`). Two parts:

1. **PRIMARY — `:` lowers wrong.** `x_:9` currently lowers to
   `Pattern[Pattern[x, Blank[]], 9]` (a nested Pattern) instead of WL's
   `Optional[Pattern[x, Blank[]], 9]`. WL's rule for the `:` operator: **`lhs : rhs`
   is `Pattern[lhs, rhs]` when `lhs` is a bare Symbol, but `Optional[lhs, rhs]`
   when `lhs` is anything else** (a `Blank`/`Pattern`/etc.). rum always emits
   `Pattern`. Fix the one lowering arm. This makes `x_:9`, `_:9`, `x_h:0` etc.
   work and is the core "default pattern" bug.
2. **SECONDARY — `x_.` syntax missing.** `x_.` (= `Optional[Pattern[x, Blank[]]]`,
   default looked up from `Default[head]`) does not parse (`expected ,, found .`).
   `x_..` (Repeated) and `x_...` (RepeatedNull) already work (brief-037); only the
   single-dot `_.` is missing. Add it so it lowers to a 1-arg `Optional[...]`,
   which plugs into the already-working `Default` matcher.

## WolframKernel semantics (verified — match these)

```
FullForm[Hold[x_:9]]        ==> Hold[Optional[Pattern[x, Blank[]], 9]]
FullForm[Hold[_:9]]         ==> Hold[Optional[Blank[], 9]]
FullForm[Hold[x:_Integer]]  ==> Hold[Pattern[x, Blank[Integer]]]   (bare symbol LHS -> Pattern, UNCHANGED)
FullForm[Hold[x:y]]         ==> Hold[Pattern[x, y]]                (bare symbol LHS -> Pattern)
MatchQ[f[1], f[1, x_:9]]    ==> True
g[1, x_:9] := {1, x}; g[1]      ==> {1, 9}
g2[1, x_:9] := {1, x}; g2[1, 5] ==> {1, 5}
FullForm[Hold[x_.]]         ==> Hold[Optional[Pattern[x, Blank[]]]]
FullForm[Hold[_.]]          ==> Hold[Optional[Blank[]]]
```

## Touch points

### Part 1 (PRIMARY) — `crates/rum-runtime/src/runtime/lowering.rs:352`
The `BinaryOp::Pattern` arm currently is:
```rust
BinaryOp::Pattern => Ok(canonical_call(heads::patterns::PATTERN, vec![lowered_lhs, lowered_rhs])),
```
Change it to branch on the LHS: a BARE SYMBOL stays `Pattern`, anything else
becomes `Optional`:
```rust
BinaryOp::Pattern => {
    if matches!(lowered_lhs, Expr::Atom(Atom::Symbol(_))) {
        Ok(canonical_call(heads::patterns::PATTERN, vec![lowered_lhs, lowered_rhs]))
    } else {
        Ok(canonical_call("Optional", vec![lowered_lhs, lowered_rhs]))
    }
}
```
(A bare symbol is `Expr::Atom(Atom::Symbol(_))` — see `crates/rum-core/src/expr.rs`.
Confirm the `Atom`/`Expr` imports are in scope in lowering.rs; add if needed.)

### Part 2 (SECONDARY) — `x_.` lexing
Add the single-dot default-blank. After a blank (`_`/`__`/`___`), a SINGLE `.`
(not `..`/`...`, not a digit) means Optional. Mirror how `..`/`...` (brief-037)
were added in the lexer/parser, but for exactly one dot -> wrap the blank/pattern
in a 1-arg `Optional[...]`. Disambiguation that MUST hold: `x_.` -> `Optional`,
`x_..` -> `Repeated` (unchanged), `x_...` -> `RepeatedNull` (unchanged), and a
numeric `.5`/`1.`/`1.5` is still a number. **Stop-loss:** if the single-dot vs
`..`/`...` disambiguation fights the brief-037 lexing, SHIP PART 1 ALONE (the
primary bug) and report Part 2 as not-done with what you found — do NOT regress
`..`/`...` or decimals to force `_.`.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/lowering.rs`; for Part 2
  `crates/rum-syntax/src/{lexer.rs,trivia.rs,ast.rs,parser/**}` + the lowering;
  plus test modules under `crates/rum-runtime/src/runtime/tests/` and
  `crates/rum-syntax/src/parser/tests.rs`.
- MUST NOT touch: the matcher / `Default` runtime (it already works — do NOT
  change it); the bare-symbol `Pattern` behavior (`x:_Integer` must stay
  `Pattern`); `..`/`...` (Repeated) and decimal lexing; `crates/rum-core` shape;
  any `mma-hol` clone or v5/v6 checkpoints; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except this report; `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Only the `:` LHS-is-symbol decision changes for Part 1.** Do not touch the
   matcher — `Optional` matching + `Default` lookup already work; this is purely
   producing `Optional` instead of `Pattern` at parse/lower time.
2. **Bare-symbol Pattern must NOT regress.** `x:_Integer` -> `Pattern[x,
   Blank[Integer]]`, `x:y` -> `Pattern[x, y]`. Only NON-symbol LHS becomes
   `Optional`.
3. **Part 2 single-dot disambiguation** is the risky bit — guard `..`/`...` and
   decimals with regression tests; use the stop-loss rather than break them.
4. **Minimal diff.** Part 1 is a few lines.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# Part 1 (primary):
$BIN --eval 'FullForm[Hold[x_:9]]'              # Hold[Optional[Pattern[x, Blank[]], 9]]
$BIN --eval 'FullForm[Hold[_:9]]'               # Hold[Optional[Blank[], 9]]
$BIN --eval 'MatchQ[f[1], f[1, x_:9]]'          # True
$BIN --eval 'MatchQ[f[1, 2], f[1, x_:9]]'       # True
$BIN --eval 'g[1, x_:9] := {1, x}; g[1]'        # {1, 9}
$BIN --eval 'g2[1, x_:9] := {1, x}; g2[1, 5]'   # {1, 5}
# regressions — bare-symbol Pattern unchanged:
$BIN --eval 'FullForm[Hold[x:_Integer]]'        # Hold[Pattern[x, Blank[Integer]]]
$BIN --eval 'FullForm[Hold[x:y]]'               # Hold[Pattern[x, y]]
$BIN --eval 'MatchQ[5, x_Integer]'              # True
# Part 2 (secondary, if shipped):
$BIN --eval 'FullForm[Hold[x_.]]'               # Hold[Optional[Pattern[x, Blank[]]]]
$BIN --eval 'Default[ff] = 0; ff[1, x_.] := {1, x}; ff[1]'   # {1, 0}
# regressions — .. / ... / decimals unchanged:
$BIN --eval 'FullForm[Hold[x_..]]'              # Hold[Repeated[Pattern[x, Blank[]]]]
$BIN --eval 'FullForm[Hold[x_...]]'             # Hold[RepeatedNull[Pattern[x, Blank[]]]]
$BIN --eval 'FullForm[Hold[1.5]]'               # a number (3/2), not dot syntax
```

## Hard rules

- Do NOT commit/branch/push/touch git config. Leave changes in the tree; Claude gates.
- NEVER edit any `mma-hol` clone or the checkpoints. Do NOT change the matcher.
- Files within Scope only; full machine access, no network; commands = file edits,
  `cargo`, release `rum-cli`. Minimal diff. Paste real verbatim output; if Part 2
  hits the stop-loss, ship Part 1 and say so.

## Report format

1. What changed — Part 1 (the lowering branch) and Part 2 (lexing) or its
   stop-loss, files + line ranges.
2. Verification — each command verbatim.
3. Verdict — `x_:v` lowers to Optional and matches/defaults correctly, bare-symbol
   Pattern unchanged, (Part 2: `x_.` works), `..`/`...`/decimal regressions clean.
4. Stop-loss / open questions.
