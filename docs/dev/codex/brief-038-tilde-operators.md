# brief-038: add the `~`-family — `~~` StringExpression and `a ~ f ~ b` infix application

## Goal

Add the two `~`-based operators (they share the `~` lexer arm, so one brief). Run
AFTER brief-036/037 (shared parser files). Found by the 2026-06-23 syntax-axis
survey.

1. **`a ~~ b`** → `StringExpression[a, b]` — an ordinary infix binary (like the
   ones in brief-036). `StringExpression` builtin exists.
2. **`a ~ f ~ b`** → `f[a, b]` — INFIX APPLICATION (WL "tie"): the thing BETWEEN
   the two single `~` is an arbitrary expression `f` that becomes the head. This
   is NOT a fixed-token binary operator; it needs a small custom parse rule.

| op | meaning | lowers to | precedence (rum scale) | assoc |
|----|---------|-----------|------------------------|-------|
| `a ~~ b`    | StringExpression | `StringExpression[a, b]` | ~35 (looser than `StringJoin`=55) | left |
| `a ~ f ~ b` | infix application | `f[a, b]`                | ~67 (tighter than `Add`=50) | left |

## WolframKernel semantics (verified — match these)

```
a ~~ b ~~ c     ==> StringExpression[a, b, c]            (flat; nested is fine if StringExpression is Flat)
a ~~ b <> c     ==> StringExpression[a, StringJoin[b, c]]   (<> tighter than ~~)
a ~ g ~ b       ==> g[a, b]
a ~ g ~ b + c   ==> Plus[g[a, b], c]                     (~g~ tighter than +)
x ~ f ~ y ~ g ~ z  ==> g[f[x, y], z]                     (left-assoc)
```

## Touch points

1. **Lexer** (`lexer.rs`): add a `~` char arm — `~~` (a `TildeTilde` /
   StringExpression token) checked BEFORE a bare `~` (a `Tilde` token). (`~` is
   currently `unexpected character`, so this is a fresh arm.)
2. **TokenKind** (`trivia.rs`): add `TildeTilde` and `Tilde` (+ `as_str` `~~` / `~`).
3. **`~~` as a normal BinaryOp** — mirror brief-036/`//` exactly: add
   `BinaryOp::StringExpression` (token `~~`, precedence ~35, left-assoc),
   `helpers.rs` mapping `TokenKind::TildeTilde => BinaryOp::StringExpression`, and
   `lowering.rs` arm → `canonical_call("StringExpression", vec![lhs, rhs])`.
4. **`a ~ f ~ b` infix application — a custom rule in the parser's infix/led
   loop** (`parser/front.rs` / wherever binary infix is driven). When the next
   token in led position is a single `Tilde` and the current left operand is
   `lhs`: parse an expression `f` (the operator — parse at a precedence that stops
   at the closing `~`), REQUIRE a closing `Tilde`, then parse the right operand
   `rhs` at the `~`-application precedence, and build the AST node for `f[lhs, rhs]`
   (a normal call/application node — the SAME node `f @ x` or juxtaposition builds,
   head `f`, args `[lhs, rhs]`). Left-associative: a following `~ g ~ z` ties onto
   the whole `f[lhs,rhs]`. If the closing `~` is missing, emit a parse error.
   - Lowering: this produces an application AST node, so `lowering.rs` already
     turns it into `Expr::call(f, [a, b])` — no new lowering arm if you emit the
     existing application/Call AST node. (If the AST has no direct "call" builder
     for this, reuse whatever `f @ x` lowers through.)

## Scope

- MAY modify: `crates/rum-syntax/src/{lexer.rs,trivia.rs,ast.rs,parser/**}`,
  `crates/rum-runtime/src/runtime/lowering.rs` + the matching test modules.
- MUST NOT touch: other operators; `crates/rum-core` shape; any `mma-hol` clone or
  checkpoints; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except this report;
  `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **`~~` before `~`** in the lexer (most-specific-first).
2. **The middle of `~ f ~` is a full expression**, not an identifier —
   `a ~ HOL`Bool`foo ~ b` must work (the head can be context-qualified or
   compound). Parse it as an expression bounded by the closing `~`.
3. **Left associativity**: `x ~ f ~ y ~ g ~ z` = `g[f[x,y], z]`. After building
   `f[x,y]`, the loop must see the next `~ g ~` and tie onto it.
4. **`~f~` builds an ordinary application node** (head = `f`, args = `[a,b]`) —
   reuse the existing Call/application AST + lowering; do NOT invent a new
   BinaryOp for it (it has no fixed operator token).
5. **Precedence is a best-estimate** — if a WK-match probe fails, adjust and rerun.
   Minimal diff.

## Verification (MANDATORY — paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'FullForm[Hold[a ~~ b]]'             # Hold[StringExpression[a, b]]
$BIN --eval 'FullForm[Hold[a ~~ b <> c]]'        # Hold[StringExpression[a, StringJoin[b, c]]]
$BIN --eval 'FullForm[Hold[a ~ g ~ b]]'          # Hold[g[a, b]]
$BIN --eval 'FullForm[Hold[a ~ g ~ b + c]]'      # Hold[Plus[g[a, b], c]]
$BIN --eval 'FullForm[Hold[x ~ f ~ y ~ g ~ z]]'  # Hold[g[f[x, y], z]]
$BIN --eval '3 ~ Plus ~ 4'                       # 7
$BIN --eval '{1, 2} ~ Join ~ {3, 4}'             # {1, 2, 3, 4}
# regression — // and other ops still fine:
$BIN --eval 'FullForm[Hold[a // f]]'             # Hold[f[a]]
```

## Hard rules

- Do NOT commit/branch/push/touch git config. NEVER edit `mma-hol`/checkpoints.
- Files within Scope only; full machine access, no network; commands = file edits,
  `cargo`, release `rum-cli`. Minimal diff. Paste real verbatim output; fix any
  WK-disagreeing probe before reporting.

## Report format

1. What changed — `~~` (the 5 standard touch points) + the `~ f ~` custom parse
   rule (file + lines), precedences used.
2. Verification — each command verbatim.
3. Verdict — `~~` and infix `~f~` parse/lower right (incl. left-assoc chain),
   regressions clean.
4. Stop-loss / open questions.
