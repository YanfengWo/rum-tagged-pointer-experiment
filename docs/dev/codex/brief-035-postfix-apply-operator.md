# brief-035: implement the `//` postfix-application operator (`x // f` -> `f[x]`)

## Goal

rum's parser does not support the `//` postfix-application operator. `x // f`
(which means `f[x]` in Wolfram) fails with `error: expected expression, found /`.
This blocks `demos/Continuous.wl` (the continuous-functions stress demo, built on
the now-fully-loaded real-number tower): byte ~9185 / line 516 uses
`HOL`Bool`GEN[...] // (HOL`Bool`GEN[fV, ...])`, so `Get` aborts with
`Get::syntax` and the file defines nothing. When done, `x // f` parses to `f[x]`,
left-associative, at the correct precedence, and Continuous.wl parses. (Same
family as the earlier parser bugs — juxtaposition, slot adjacency, `\[Name]`.)

`//` is the operand-reversed, lower-precedence sibling of `@` (Apply): `f @ x`
-> `f[x]` (head = LHS); `x // f` -> `f[x]` (head = RHS). Mirror the `@`/`Apply`
machinery, reversing the operands and using a lower precedence.

## WolframKernel semantics to match (verified by probe)

```
a + b // f        ==> f[a + b]        (// binds LOOSER than +)
a = b // f        ==> a = f[b]        (// binds TIGHTER than =)
a // f /. r       ==> (f /. r)[a]     (// binds LOOSER than /. )
a // f // g       ==> g[f[a]]         (LEFT associative)
f @ a // g        ==> g[f[a]]
```
So precedence ordering (rum's scale): `Set(10) < //  < ReplaceAll(20)`. Use
**precedence 15**, **left-associative**. (`//` lowers `x // f` to `Call{head:
f, args: [x]}` = `f[x]`.)

## The FIVE touch points (mirror `@`/`BinaryOp::Apply`, reversed)

1. **Lexer token** — `crates/rum-syntax/src/lexer.rs:317` (the `'/'` arm). The
   more-specific `//.` (`:318`) and `//=` (`:322`) are already matched first;
   add a bare `//` AFTER them, e.g.:
   ```rust
   } else if self.consume_str("//") {
       TokenKind::SlashSlash
   } else if self.consume_str("/=") { ...
   ```
   (Place it after the `//.` and `//=` checks so those are not mis-split. `/.`,
   `/:`, `/;`, `/*`, `/@` ordering is unaffected.)
2. **TokenKind** — `crates/rum-syntax/src/trivia.rs`: add `SlashSlash` next to
   `SlashAt`/`ReplaceAll` (the enum + its `as_str`/display arm `"//"`, mirroring
   `Self::SlashAt => "/@"` at `:123` and `Self::ReplaceAll => "/."` at `:148`).
3. **BinaryOp** — `crates/rum-syntax/src/ast.rs`: add `BinaryOp::PostfixApply`
   to the enum (near `:227 Apply`), with `token()` -> `"//"` (the `match` at
   `:280`) and `precedence()` -> `15` (the `match` at `:324`). It must be
   LEFT-associative — confirm `is_right_associative()` returns false for it
   (don't add it to any right-assoc set).
4. **Token -> BinaryOp** — `crates/rum-syntax/src/parser/helpers.rs:176` (next to
   `TokenKind::At => Some(BinaryOp::Apply)` and `TokenKind::SlashAt =>
   Some(BinaryOp::Map)`): add `TokenKind::SlashSlash => Some(BinaryOp::PostfixApply)`.
5. **Lowering SyntaxExpr -> Expr** — `crates/rum-runtime/src/runtime/lowering.rs:330`.
   `BinaryOp::Apply => Ok(Expr::call(lowered_lhs, vec![lowered_rhs]))` builds
   `f[x]` from `f @ x` (head = lhs). Add the REVERSED case:
   ```rust
   BinaryOp::PostfixApply => Ok(Expr::call(lowered_rhs, vec![lowered_lhs])),
   ```
   (head = RHS, sole arg = LHS) so `x // f` -> `f[x]`.

If the parser's infix loop is generic over `BinaryOp` + `precedence()` (it is —
see `next_min_precedence_for_binary_op` at `front.rs:410`), no other parser edit
is needed. `//` does NOT belong in `can_follow_function_postfix` (`front.rs:419`).

## Scope

- Files you MAY modify: `crates/rum-syntax/src/lexer.rs`,
  `crates/rum-syntax/src/trivia.rs`, `crates/rum-syntax/src/ast.rs`,
  `crates/rum-syntax/src/parser/helpers.rs`,
  `crates/rum-runtime/src/runtime/lowering.rs`, plus tests under
  `crates/rum-syntax/src/parser/tests.rs` and/or a rum-runtime test module.
- MUST NOT touch: any `mma-hol` clone or the v5 checkpoints; `crates/rum-core`
  `Expr`/`Symbol`/serde shape; the other `/`-operator tokens (`/.`,`/@`,`//.`,
  `//=`,`/=`,`/:`,`/;`,`/*`) — only ADD `//`; `AGENTS.md`, `PROGRESS.md`,
  `TODO.md`, `docs/**` except this report; `.git`, git config, `Cargo.lock`.
- Do not change existing operator precedences; only add `PostfixApply = 15`.

## rum / Rust pitfalls (read twice)

1. **Token order in the lexer.** `//.` (ReplaceRepeated) and `//=` (ApplyTo)
   MUST still match before bare `//`. Adding `consume_str("//")` after them
   preserves that. Add a regression check that `a //. b`, `a //= b`, `a /. b`,
   `a // b` all still tokenize correctly.
2. **Operand reversal is the whole point.** `x // f` is `f[x]` — head is the
   RHS. Getting this backwards (`x[f]`) is the classic mistake; the test
   `FullForm[Hold[x // f]] == Hold[f[x]]` guards it.
3. **Left-associative.** `a // f // g` must be `g[f[a]]`, not `f[a // g]`.
4. **Precedence 15 is between Set(10) and ReplaceAll(20)** — verify the WK-match
   probes below; do not perturb neighbors.
5. **Minimal diff.** Five small additions; no refactor of the `/` arm or the
   binary-op loop.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'FullForm[Hold[x // f]]'        # Hold[f[x]]
$BIN --eval 'FullForm[Hold[a // f // g]]'   # Hold[g[f[a]]]
$BIN --eval 'FullForm[Hold[a + b // f]]'    # Hold[f[Plus[a, b]]]
$BIN --eval 'FullForm[Hold[a = b // f]]'    # Hold[Set[a, f[b]]]
$BIN --eval 'FullForm[Hold[a // f /. r]]'   # Hold[ReplaceAll[f, r][a]]  (i.e. (f /. r)[a])
$BIN --eval '{1, 2, 3, 4} // Total'         # 10
$BIN --eval 'Range[5] // Length'            # 5
# regressions — other slash operators unchanged:
$BIN --eval 'FullForm[Hold[a //. b]]'       # Hold[ReplaceRepeated[a, b]]
$BIN --eval 'FullForm[Hold[a /. b]]'        # Hold[ReplaceAll[a, b]]
$BIN --eval 'FullForm[Hold[f /@ x]]'        # Hold[Map[f, x]]
$BIN --eval 'FullForm[Hold[a /; b]]'        # Hold[Condition[a, b]]
```

Paste verbatim. (The real unblock — re-extend the full tower scratch and
`Get[demos/Continuous.wl]`, expecting no `Get::syntax` at the `//` — is Claude's
follow-up gate; you do NOT run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone or the v5 checkpoints.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the five touch points, files + line ranges.
2. Verification — each command, verbatim output (test counts + each eval).
3. Verdict — `x // f` parses to `f[x]`, left-assoc, precedence between Set and
   ReplaceAll, the WK-match probes pass, other `/` operators regress clean.
4. Stop-loss / open questions.
