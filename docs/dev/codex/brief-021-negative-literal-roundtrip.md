# brief-021: fix non-idempotent negative-literal lowering (snapshot round-trip rot)

## Goal

A held negative numeric literal must lower to an ATOMIC negative number, so that
`FullForm`/`DumpSave`/`Get` round-trips are idempotent. Right now rum lowers a
prefix-minus over any operand to `Times[-1, operand]` unconditionally, so a
numeric literal `-1` becomes the held expression `Times[-1, 1]` instead of the
integer atom `-1`. Evaluation always collapses `Times[-1, 1] -> -1`, so this is
invisible in normal use — but in HELD contexts (a `SetDelayed` RHS, `Hold[...]`,
and the textual `DumpSave`/`Get` snapshot format) it is preserved, and because
the serialized `Times[-1, b]` re-parses to `Times[Times[-1, 1], b]`, every
snapshot round-trip wraps the `-1` one level deeper — unbounded growth and
representation rot. When done, prefix-minus of a numeric literal folds to the
negated atom, the round-trip is a fixed point, and all existing tests stay green.

## Context pointers

- Read `AGENTS.md` (operational notes, the unmodified-`mma-hol` hard line) and
  the PROGRESS entry `2026-06-22 Snapshot reliability test (dimension A)` — it
  has the full diagnosis and minimal repro for this exact bug (BUG-RT).
- Key files:
  - `crates/rum-runtime/src/runtime/lowering.rs` — lowers the `rum-syntax` tree
    to `Expr`. **The bug is at `lower_prefix_sugar`, the `PrefixOp::Minus` arm
    (currently around lines 205-207):**
    ```rust
    PrefixOp::Minus => Ok(canonical_call(
        "Times",
        vec![Expr::integer(BigInt::from(-1)), lowered],
    )),
    ```
    This wraps EVERY operand, including a numeric-literal `lowered`.
  - `crates/rum-core/src/expr.rs` — `Expr` numeric atoms (`Expr::integer`, the
    rational/real constructors, `Rational::negated`). Use these to build the
    negated atom; do NOT change the `Expr`/`Symbol`/serde shape.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. Acceptance
  is mechanical: the repro outputs below and the named tests decide.

## The fix (precise)

In `lower_prefix_sugar`, `PrefixOp::Minus`: if `lowered` is a numeric literal
atom (Integer, Rational, and Real if rum has one), produce the NEGATED numeric
atom directly (e.g. `Expr::integer(-n)`, the rational/real negation). For any
non-numeric `lowered` (a symbol, a call, a pattern, etc.) keep the existing
`Times[-1, lowered]` lowering unchanged — `-x`, `-f[y]`, `-(a+b)` must stay
`Times[-1, ...]`.

Match on the already-lowered `Expr` (so `-1`, `-3`, `-1/2` all fold). Do not
fold a non-literal. Do not "evaluate" — this is a parse-time constant fold of a
unary minus applied to a literal, which is what WolframKernel does (its
`FullForm[Hold[-1]]` is `Hold[-1]`).

Secondary (only if trivial and in the same arm's spirit): the BinaryOp `Minus`
lowering (around line 324) builds `Plus[lhs, Times[-1, rhs]]`; for a numeric
literal `rhs` this yields a held `Times[-1, 1]`-style factor (e.g. `a - 1` ->
`Plus[a, Times[-1, 1]]`). This is STABLE under round-trip once the prefix fix
lands (the inner `-1` re-reads as an atom, so it does not grow), so it is NOT
required for idempotence. Fold it too ONLY if it is a clean one-liner reusing
the same negate helper; otherwise leave it and note it. Idempotence (no
unbounded growth) is the gate, not full canonicalization.

## Scope

- Files you MAY modify:
  - `crates/rum-runtime/src/runtime/lowering.rs` (the fix).
  - ONE test file under `crates/rum-runtime/src/runtime/` (or the existing
    lowering/parse test module) to add the regression test below. If you add a
    new module, keep it minimal and wired in normally.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` standard;
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol` representation or any public
    type/serde shape (you may CALL its constructors, not change them);
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your
    own report, `.git`, git config, `Cargo.lock`.
- Do NOT add unconditional cost to the default evaluation path. This is a
  parse/lowering-time change only; it must not touch any per-evaluation-step
  hot path.

## Deliverable

- The lowering fix in `lower_prefix_sugar`.
- A regression test asserting the held forms are atomic (see Verification).
- In your report: the verbatim before/after of the three repro commands and the
  snapshot round-trip idempotence check.

## rum / Rust pitfalls (read twice)

1. **Only fold NUMERIC LITERALS.** `-x`, `-f[x]`, `-(a+b)`, `-symbol` MUST stay
   `Times[-1, lowered]`. A too-broad fold that negates non-numbers is wrong.
2. **Do not change evaluation semantics.** Evaluated `-1`, `2 - 3`, `-x*y`, etc.
   must still evaluate to the same values; only the HELD/parsed shape of a
   literal changes. Run the full suite.
3. **Rationals/reals too.** `-1/2` should fold to the negated rational atom, not
   `Times[-1, Rational[1,2]]`. Reuse `Rational::negated` / the numeric path.
4. **Minimal diff.** No drive-by reformatting of `lowering.rs`. Touch only the
   `PrefixOp::Minus` arm (and the BinaryOp `Minus` arm only if you do the
   trivial secondary fold).
5. **Stop-loss:** if folding breaks more than a couple of tests in ways that are
   not a trivial expected-output update, STOP and report — the round-trip
   idempotence may need a different locus than you think.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
```

Then the BUG-RT repro, with the release CLI (`cargo build --release -p rum-cli`
first if needed). Paste each line's verbatim output:

```
# 1. held negative literals must be atomic now:
./target/release/rum-cli --eval 'FullForm[Hold[-1]]'            # expect "Hold[-1]"
./target/release/rum-cli --eval 'FullForm[Hold[-5]]'            # expect "Hold[-5]"
./target/release/rum-cli --eval 'FullForm[Hold[Times[-1, b]]]'  # expect "Hold[Times[-1, b]]"
./target/release/rum-cli --eval 'FullForm[Hold[-1/2]]'          # expect "Hold[Rational[-1, 2]]" (rum's rational FullForm)

# 2. negation of a NON-literal is unchanged:
./target/release/rum-cli --eval 'FullForm[Hold[-x]]'            # expect "Hold[Times[-1, x]]"

# 3. evaluation semantics unchanged:
./target/release/rum-cli --eval '{-5, 2 - 3, -x + x, -(1/2) + 1}'  # expect {-5, -1, 0, 1/2}

# 4. snapshot round-trip idempotence (the real goal):
rm -f /tmp/rt0.mx /tmp/rt1.mx /tmp/rt2.mx
./target/release/rum-cli --eval 'Sandbox`g[x_] := x - b; DumpSave["/tmp/rt0.mx", "Sandbox`"]; "ok"'
./target/release/rum-cli --eval 'Get["/tmp/rt0.mx"]; DumpSave["/tmp/rt1.mx", "Sandbox`"]; "ok"'
./target/release/rum-cli --eval 'Get["/tmp/rt1.mx"]; DumpSave["/tmp/rt2.mx", "Sandbox`"]; "ok"'
# all three files must be byte-identical for the g definition; show:
grep 'Sandbox`g' /tmp/rt0.mx /tmp/rt1.mx /tmp/rt2.mx
wc -c /tmp/rt0.mx /tmp/rt1.mx /tmp/rt2.mx
# PASS = the SetDelayed line is identical across rt0/rt1/rt2 (no growing Times[-1,1] nesting),
#        and the three byte counts are equal.
```

Loop edit -> `cargo test` -> fix until green, then rerun the repro and paste the
before/after for command 1 and command 4.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates and commits.
- Do NOT modify files outside Scope. If you think you must, STOP and say so.
- Full machine access, but touch NOTHING outside this repo, no network. Only
  commands: file edits within Scope, `cargo`, the release `rum-cli`, `python3`
  for parsing.
- Never edit any `mma-hol` file.
- Minimal diff: no drive-by reformatting/renames/comment churn.
- You MUST paste the real verbatim verification output; if something is not
  green, say so explicitly — never fake it.

## Report format (final message)

1. What changed — per file, line ranges, one-line why.
2. Verification — each command, verbatim output (test counts + the four repro
   blocks).
3. The verdict — confirm round-trip idempotence (rt0==rt1==rt2 for `g`), that
   `-x` stayed `Times[-1, x]`, and that evaluation semantics are unchanged.
4. Stop-loss / open questions — anything unsure (e.g. if the secondary BinaryOp
   fold was or was not done, and why).
