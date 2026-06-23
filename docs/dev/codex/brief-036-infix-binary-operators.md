# brief-036: add 5 infix-binary operators — `^=` `^:=` `**` `//@` `|->`

## Goal

Add five missing infix-binary operators to rum's parser. Each is the SAME shape
as the just-landed `//` operator (brief-035, commit `49b081b` — use it as the
worked template): a lexer token + a `BinaryOp` variant (token + precedence +
associativity) + the token→op mapping + a `lowering.rs` arm. All five lower-target
builtins already exist in rum (`UpSet`, `UpSetDelayed`, `MapAll`, `Function`;
`NonCommutativeMultiply` is symbolic-by-design — no eval needed, just the head).
Found by the 2026-06-23 syntax-axis operator survey (PROGRESS / handoff).

| op | meaning | lowers to | precedence (rum scale) | assoc |
|----|---------|-----------|------------------------|-------|
| `a ^= b`  | UpSet         | `UpSet[a, b]`                 | 10 (= `Set`)        | right |
| `a ^:= b` | UpSetDelayed  | `UpSetDelayed[a, b]`          | 10 (= `SetDelayed`) | right |
| `a ** b`  | NonCommMult   | `NonCommutativeMultiply[a, b]`| 65 (between `Times`=60 and `Power`=70) | left |
| `f //@ x` | MapAll        | `MapAll[f, x]`                | 72 (= `Map`/`/@`)   | right |
| `x \|-> e`| Function      | `Function[x, e]`              | 14 (very loose, near `&`) | right |

rum's precedence scale (HIGHER binds TIGHTER): `CompoundExpression 3`, `TagSet 5`,
`Set/SetDelayed 10`, `// 15`, `ReplaceAll 20`, `Rule 30`, `Add 50`, `StringJoin 55`,
`Times 60`, `Power 70`, `Apply/Map 72`, `Pattern 80` (full table in `ast.rs:326`).

## WolframKernel semantics (verified — match these)

```
a ^= b        ==> UpSet[a, b]                         (a^=b binds like a=b)
a ^:= b       ==> UpSetDelayed[a, b]
a ** b * c    ==> Times[NonCommutativeMultiply[a, b], c]   (** tighter than *)
a ** b ^ c    ==> NonCommutativeMultiply[a, Power[b, c]]   (** looser than ^)
2 ** 3        ==> NonCommutativeMultiply[2, 3]        (** is SYMBOLIC — no eval, even on numbers)
f //@ x       ==> MapAll[f, x]
x |-> a + b   ==> Function[x, Plus[a, b]]             (|-> looser than +)
```

## Touch points (mirror the `//` change, commit `49b081b`)

1. **Lexer** (`crates/rum-syntax/src/lexer.rs`): add the multi-char tokens in the
   right arms, MOST-SPECIFIC FIRST:
   - `^` arm: `^:=` (UpSetDelayed) and `^=` (UpSet) BEFORE bare `^` (Power). If
     there is no `^` arm yet, add one (Power must still lex).
   - `*` arm: `**` (NonCommMult) BEFORE `*=` (TimesBy) and bare `*` (Times).
   - `/` arm (line ~317): `//@` (MapAll) BEFORE the existing `//` — order it with
     the other `//x` checks (`//.`, `//=`, `//`).
   - `|` arm: `|->` (Function) BEFORE `||` (Or) and bare `|` (Alternatives).
2. **TokenKind** (`crates/rum-syntax/src/trivia.rs`): add a token kind + `as_str`
   for each (`^=`,`^:=`,`**`,`//@`,`|->`).
3. **BinaryOp** (`crates/rum-syntax/src/ast.rs`): add `UpSet`, `UpSetDelayed`,
   `NonCommutativeMultiply`, `MapAll`, `Function` variants; fill `token()`,
   `precedence()` (values above), and add the right-assoc ones (`UpSet`,
   `UpSetDelayed`, `MapAll`, `Function`) to `is_right_associative()`. (`**` is
   left-assoc — do NOT add it there.)
4. **Token→BinaryOp** (`crates/rum-syntax/src/parser/helpers.rs:~157`): map each
   new TokenKind to its BinaryOp.
5. **Lowering** (`crates/rum-runtime/src/runtime/lowering.rs:~290`): add an arm
   per op. `^=`/`^:=`/`**`/`//@` → `canonical_call("UpSet"/"UpSetDelayed"/
   "NonCommutativeMultiply"/"MapAll", vec![lhs, rhs])`. `|->` →
   `Expr::call(function_head, vec![lhs, rhs])` matching how rum builds `Function`
   from the `&` postfix (grep `BinaryOp`/`PostfixOp::Function` lowering — produce
   the SAME `Function[var, body]` shape, 2-arg).

## Scope

- MAY modify: `crates/rum-syntax/src/{lexer.rs,trivia.rs,ast.rs,parser/helpers.rs,
  parser/tests.rs}`, `crates/rum-runtime/src/runtime/lowering.rs` +
  `…/tests/lowering_golden.rs`, and a rum-runtime test module.
- MUST NOT touch: other operators' tokens/precedence; `crates/rum-core` shape;
  any `mma-hol` clone or v5/v6 checkpoints; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except this report; `.git`, git config, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Lexer order: most-specific-first.** `^:=` before `^=` before `^`; `**`
   before `*=`/`*`; `//@` before `//`; `|->` before `||`/`|`. A regression test
   must show `a ^ b`, `a * b`, `a *= b`, `a // f`, `a || b`, `a | b` still lex
   right.
2. **`**` is symbolic.** Do NOT give it eval semantics; `2 ** 3` stays
   `NonCommutativeMultiply[2, 3]`. Just the parser + head.
3. **Associativity matters.** `**` LEFT; the other four RIGHT (verify the chain
   probes). Precedence values are best-estimates from the WL relative order —
   if a WK-match probe below fails, adjust the value and rerun.
4. **`|->` lowering = `Function[var, body]`** (2-arg Function), the same node the
   `&`/`#` machinery builds — reuse it, don't invent a new shape.
5. **Minimal diff**, mirror `//`.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-syntax
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'FullForm[Hold[a ^= b]]'         # Hold[UpSet[a, b]]
$BIN --eval 'FullForm[Hold[a ^:= b]]'        # Hold[UpSetDelayed[a, b]]
$BIN --eval 'FullForm[Hold[a ** b * c]]'     # Hold[Times[NonCommutativeMultiply[a, b], c]]
$BIN --eval 'FullForm[Hold[a ** b ^ c]]'     # Hold[NonCommutativeMultiply[a, Power[b, c]]]
$BIN --eval 'NonCommutativeMultiply[2, 3]'   # NonCommutativeMultiply[2, 3]  (inert, symbolic)
$BIN --eval 'FullForm[Hold[f //@ x]]'        # Hold[MapAll[f, x]]
$BIN --eval 'FullForm[Hold[x |-> a + b]]'    # Hold[Function[x, Plus[a, b]]]
$BIN --eval '(x |-> x + 1)[5]'               # 6
# regressions:
$BIN --eval 'FullForm[Hold[a ^ b]]'          # Hold[Power[a, b]]
$BIN --eval 'FullForm[Hold[a * b]]'          # Hold[Times[a, b]]
$BIN --eval 'FullForm[Hold[a // f]]'         # Hold[f[a]]
$BIN --eval 'FullForm[Hold[a || b]]'         # Hold[Or[a, b]]
$BIN --eval 'FullForm[Hold[a | b]]'          # Hold[Alternatives[a, b]]
$BIN --eval 'FullForm[Hold[f /@ x]]'         # Hold[Map[f, x]]
```

## Hard rules

- Do NOT commit/branch/push/touch git config. Leave changes in the tree; Claude gates.
- NEVER edit any `mma-hol` clone or the checkpoints. Minimal diff.
- Full machine access, nothing outside the repo, no network. Commands: file edits
  within Scope, `cargo`, the release `rum-cli`.
- Paste real verbatim output; if a probe disagrees with WK, fix the precedence and
  rerun — do not ship a wrong grouping.

## Report format

1. What changed — per op, the 5 touch points + final precedence/assoc used.
2. Verification — each command verbatim (test counts + each probe).
3. Verdict — five operators parse + lower correctly, regressions clean.
4. Stop-loss / open questions.
