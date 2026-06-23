# brief-032: implement `Numerator` — the missing partner of `Denominator`

## Goal

rum has `Denominator` but NOT `Numerator`. `Numerator[1/2]` stays UNEVALUATED
(`Numerator` resolves to `Global`Numerator` / a package-`Private` context, never
`System`Numerator`; `Names["System`Numerator"]` is `{}`), while `Denominator[3/4]`
correctly returns `4`. This is the current correctness-mainline blocker at
`stdlib/Real/Seq.wl`: `auto/RealArith.wl`'s `positiveScalarTerm[1/2]` computes
`p = Numerator[1/2]`, but in rum that is the inert symbol-application
`HOL`Auto`RealArith`Private`Numerator[1/2]`, so `rnumTm[Private`Numerator[1/2]]`
is not a real-typed HOL term, and `mkComb[realMulConst[], <that>]` raises
`mkComb: argument type mismatch` (`fun`/`real`) — diagnosed by a two-engine diff
(WK has every intermediate `real`-typed; the divergence is rum's missing
`Numerator`). When done, `Numerator` is a System builtin mirroring `Denominator`
and the `scalarTerm`/`positiveScalarTerm` rational path produces well-typed
terms. (Same bug family as `Nothing`/brief-030: a missing System builtin falls
through to a non-System context and stays inert.)

## Context pointers

- Read `AGENTS.md` and PROGRESS `2026-06-23` (the Real/Seq frontier; brief-030
  `Nothing` was the prior layer; brief-031 diagnosed THIS layer).
- `Numerator` is a pure mirror of the existing `Denominator`. The FOUR exact
  touch points (mirror each):
  1. `crates/rum-runtime/src/runtime/metadata.rs:268` — `enum BuiltinHandler`
     has `Denominator,`; add `Numerator,` (alongside it).
  2. `crates/rum-runtime/src/runtime/arithmetic.rs:174` — `eval_denominator`;
     add `eval_numerator` right next to it (see body below).
  3. `crates/rum-runtime/src/runtime/eval_dispatch/surface.rs:25` —
     `BuiltinHandler::Denominator => self.eval_denominator(head, args),`; add
     `BuiltinHandler::Numerator => self.eval_numerator(head, args),`.
  4. `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs:64` — the
     `BuiltinSpec::ordinary("Denominator", ..., BuiltinHandler::Denominator)`
     block; add the `"Numerator"` spec.
- `Rational::numerator()` already exists: `crates/rum-core/src/expr.rs:312`
  (partner of `denominator()` at :316).
- THE ONE ASYMMETRY vs `Denominator`: for an **Integer**, `Denominator` returns
  `1` but `Numerator` returns the integer ITSELF (`Numerator[5] == 5`,
  `Numerator[-5] == -5`). Get this right.

## The new function (mirror of `eval_denominator`)

`eval_denominator` is:
```rust
match &args[0] {
    Expr::Atom(Atom::Integer(_)) => Ok(Expr::integer(1_i32.into())),
    Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.denominator().clone())),
    _ => Ok(Expr::call(head.clone(), args.to_vec())),
}
```
`eval_numerator` is the same shape with the Integer and Rational arms changed:
```rust
match &args[0] {
    Expr::Atom(Atom::Integer(value)) => Ok(Expr::integer(value.clone())),
    Expr::Atom(Atom::Rational(value)) => Ok(Expr::integer(value.numerator().clone())),
    _ => Ok(Expr::call(head.clone(), args.to_vec())),
}
```
(Keep the same `args.len() != 1` guard with
`self.invalid_call_with_exact_arg_count("Numerator", args, 1)`. Match the exact
binding/borrow idiom the surrounding arithmetic builtins use — verify the
`Atom::Integer` arm binds the value the same way the other `eval_*` numeric
helpers do; if `Expr::integer` wants an owned `BigInt`, `value.clone()` is right.)

## Scope

- Files you MAY modify: the four files above, plus a test module under
  `crates/rum-runtime/src/runtime/tests/`.
- MUST NOT touch: any `mma-hol` clone or the v5 checkpoints; `crates/rum-core`
  `Expr`/`Symbol`/serde shape (the `numerator()` accessor already exists — only
  CALL it); `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except this report;
  `.git`, git config, `Cargo.lock`.
- No default-path cost concern (a new builtin arm only runs when `Numerator` is
  called).

## rum / Rust pitfalls (read twice)

1. **Integer arm asymmetry.** `Numerator[Integer n] = n`, NOT `1`. This is the
   single way the mirror differs from `Denominator`.
2. **Sign lives in the numerator.** rum's `Rational` normalizes sign into the
   numerator, so `Numerator[-3/4]` is `-3` via `value.numerator().clone()` with
   no extra handling — confirm with the test, don't special-case.
3. **Non-number stays inert.** `Numerator[x]` (symbolic), `Numerator[a+b]` →
   return the unevaluated `head`-call exactly as `Denominator` does. Do NOT try
   to implement WL's full symbolic `Numerator` (factor extraction) — match
   `Denominator`'s surface only; that is all the number tower needs.
4. **Registration must make it System.** After registering, `Context[Numerator]`
   MUST be `"System`"` and `Names["System`Numerator"]` MUST be `{Numerator}`
   (this is the actual bug — it currently resolves to `Global``). The
   `BuiltinSpec` registration is what puts it in `System``; verify it.
5. **Minimal diff.** Four small edits + one test. No drive-by changes.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'Numerator[1/2]'                 # 1
$BIN --eval 'Numerator[3/4]'                 # 3
$BIN --eval 'Numerator[-3/4]'                # -3
$BIN --eval 'Numerator[5]'                   # 5
$BIN --eval 'Numerator[-5]'                  # -5
$BIN --eval 'Numerator[0]'                   # 0
$BIN --eval 'Context[Numerator]'             # System`
$BIN --eval 'Names["System`Numerator"]'      # {Numerator}
$BIN --eval 'FullForm[Numerator[x]]'         # Numerator[x]  (inert, symbolic)
# regression — Denominator still right:
$BIN --eval 'Denominator[3/4]'               # 4
$BIN --eval 'Denominator[5]'                 # 1
```

Paste verbatim. (The real unblock — restore v5 `cp34`, re-extend RealArith with
the new binary, `extend_snapshot stdlib/Real/Seq.wl`, expecting the `mkComb`
error to be GONE — is Claude's follow-up gate; you do NOT run it.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- NEVER write to/edit any `mma-hol` clone or the v5 checkpoints.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.

## Report format (final message)

1. What changed — the four edits + the test, files + line ranges.
2. Verification — each command, verbatim output (test counts + each eval).
3. Verdict — `Numerator` is a System builtin mirroring `Denominator`, Integer
   returns itself, sign/inert/regression all correct, tests green.
4. Stop-loss / open questions.
