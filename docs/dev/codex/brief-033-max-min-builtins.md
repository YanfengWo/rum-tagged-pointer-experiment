# brief-033: implement `Max` and `Min` — missing variadic numeric System builtins

## Goal

rum lacks both `Max` and `Min` (`Names["System`Max"]` is `{}`; `Max[3,5]` stays
inert `Max[3, 5]`). A symbol-axis conformance scan (mma-hol call-form System
builtins MINUS rum's `Names["System`*"]`) found these are the ONLY two System
builtins mma-hol invokes unqualified that rum is missing. mma-hol relies on them
on the real-arith load path: `auto/RealArith.wl`'s
`rnumNe[m_Integer, n_Integer] := ... lo = Min[m, n]; hi = Max[m, n];
loT = rnumTm[lo]; hiT = rnumTm[hi]` (and `stdlib/Real/Real/CompactSet.wl`).
With `Max`/`Min` inert, `rnumTm[Min[m,n]]` is malformed — the same latent
failure family as the just-fixed `Numerator` (brief-032) and `Nothing`
(brief-030). When done, `Max`/`Min` are System builtins computing the numeric
maximum/minimum of exact numbers, matching WolframKernel on the cases mma-hol
uses.

## Context pointers

- Read `AGENTS.md` and PROGRESS `2026-06-23`. This is a PROACTIVE fix found by a
  symbol-table cross-check, not yet the live frontier (the immediate Seq blocker
  after Numerator is a separate `Sort`-comparator/Association bug Claude is
  diagnosing); `Max`/`Min` bite slightly deeper (the `rnumNe` integer path).
- `Max`/`Min` are variadic numeric builtins — mirror the existing
  `eval_gcd`/`eval_lcm` shape (`crates/rum-runtime/src/runtime/arithmetic.rs:251`
  and `:272`): collect arg values, fold, fall back to the inert `head`-call if
  any arg is non-numeric.
- Number extraction helpers already exist in `arithmetic.rs`:
  `expr_as_exact_rational` (`:416`, accepts Integer AND Rational — verify it
  returns `Some` for both) and `expr_as_exact_integer` (`:409`). Compare values
  as `Rational` (an integer is a rational with denominator 1), then return the
  extremum **as the original `Expr`** (preserve Integer-vs-Rational: `Max[3,5]`
  must return `5` the Integer, `Max[3/2,1]` must return `3/2`).
- The FOUR touch points (mirror `Denominator`/`GCD`):
  1. `crates/rum-runtime/src/runtime/metadata.rs:268` (`enum BuiltinHandler`,
     near `Denominator,`) — add `Max,` and `Min,`.
  2. `crates/rum-runtime/src/runtime/arithmetic.rs` — add `eval_max` and
     `eval_min` next to `eval_gcd`.
  3. `crates/rum-runtime/src/runtime/eval_dispatch/surface.rs:25` — add
     `BuiltinHandler::Max => self.eval_max(head, args),` and the `Min` arm.
  4. `crates/rum-runtime/src/runtime/builtin_catalog/numeric.rs` (near the
     `"GCD"`/`"Denominator"` specs) — register `"Max"` and `"Min"`.

## WolframKernel semantics to match (verified)

```
Max[3,5]=5   Min[3,5]=3   Max[3]=3   Min[3]=3
Max[1,5,2]=5   Max[3/2,1]=3/2   Min[3/2,1]=1
Max[{3,5},2]=5   (List args are flattened)
Max[]=-Infinity   Min[]=Infinity
```

Required behavior for `eval_max` (and `Min` symmetric):
1. **Flatten `List` arguments** (recursively) into a flat sequence of operands,
   matching WK's `Max[{3,5},2] == 5`. (mma-hol's hot calls are scalar
   `Max[m,n]`, but flattening is cheap and correct.)
2. If the flattened operand list is **empty**: `Max[] -> -Infinity`,
   `Min[] -> Infinity`. rum HAS `Infinity` (`Names["System`Infinity"]` is
   `{Infinity}`); `-Infinity` prints as `Times[-1, Infinity]` — construct it the
   way rum builds a negated symbol (or `Expr::call(Times, [Expr::integer(-1),
   Infinity])`). If this construction is awkward, returning the inert call for
   the EMPTY case only is an acceptable documented limitation (mma-hol never
   calls them with zero args) — but the 1+-arg cases are MANDATORY.
3. If **every** flattened operand is an exact number (Integer/Rational via
   `expr_as_exact_rational`): return the operand with the maximum (resp. minimum)
   value, **as its original `Expr`** (do not coerce Integer to Rational).
4. If **any** flattened operand is non-numeric: return the inert
   `head`-call over the ORIGINAL args (do NOT attempt WK's partial symbolic
   `Max[3, x]` sort — mma-hol's path is always fully numeric).

## Scope

- Files you MAY modify: the four files above, plus a test module under
  `crates/rum-runtime/src/runtime/tests/` (extend the arithmetic tests).
- MUST NOT touch: any `mma-hol` clone or the v5 checkpoints; `crates/rum-core`
  `Expr`/`Symbol`/serde shape; `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`
  except this report; `.git`, git config, `Cargo.lock`.
- New builtin arms only run when `Max`/`Min` are called — no default-path cost.

## rum / Rust pitfalls (read twice)

1. **Preserve the original `Expr` type.** Return the winning operand itself
   (`Max[3,5]` -> Integer `5`, `Max[3/2,1]` -> Rational `3/2`), not a rebuilt
   rational. Compare BY rational value, return the original node.
2. **Integers are rationals.** `expr_as_exact_rational` should accept Integer;
   confirm, else handle Integer and Rational both. A mixed `Max[3/2, 1]` must
   compare correctly across the two.
3. **Empty-args is the ONLY case allowed to degrade.** 1+ numeric args MUST
   evaluate. Don't let an Infinity-construction snag block the whole builtin —
   if needed, ship the numeric path + inert-empty and note it.
4. **Non-numeric falls back inert**, exactly like `eval_gcd`'s
   `return Ok(Expr::call(head.clone(), args.to_vec()))`.
5. **Minimal diff.** Mirror `eval_gcd`; no drive-by changes.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

$BIN --eval 'Max[3, 5]'                 # 5
$BIN --eval 'Min[3, 5]'                 # 3
$BIN --eval 'Max[3]'                    # 3
$BIN --eval 'Max[1, 5, 2]'             # 5
$BIN --eval 'Max[3/2, 1]'             # 3/2
$BIN --eval 'Min[3/2, 1]'             # 1
$BIN --eval 'Max[-3, -5]'             # -3
$BIN --eval 'Max[{3, 5}, 2]'         # 5   (list flatten)
$BIN --eval 'Min[{4, 1}, 2]'         # 1
$BIN --eval 'Context[Max]'           # System`
$BIN --eval 'Names["System`Min"]'    # {Min}
$BIN --eval 'FullForm[Max[x, 3]]'    # inert (Max[x, 3]) — non-numeric stays
$BIN --eval 'Max[]'                   # -Infinity  (or inert if documented)
$BIN --eval 'Min[]'                   # Infinity   (or inert if documented)
# regression:
$BIN --eval 'GCD[12, 18]'            # 6
```

Paste verbatim. (The real unblock — re-extend RealArith + extend Seq on the v5
cp34 base, checking the `rnumNe`/Max-Min path no longer mis-builds — is Claude's
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

1. What changed — the four edits + the test, files + line ranges, and how the
   empty-args case was handled.
2. Verification — each command, verbatim output (test counts + each eval).
3. Verdict — `Max`/`Min` are System builtins, numeric extrema correct across
   Integer/Rational, list flatten works, non-numeric inert, regression green.
4. Stop-loss / open questions.
