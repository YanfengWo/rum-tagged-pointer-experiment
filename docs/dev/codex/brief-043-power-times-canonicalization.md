# brief-043: Power/Times symbolic canonicalization — combine like-bases / like-terms (numeric-exponent/coefficient scope)

## Goal

rum's `Plus`/`Times` canonicalization folds only NUMERIC constants and never
combines symbolic like-terms / like-bases, diverging from WolframKernel:

```
rum now                                   WK (target)
x^2 x^3   Times[Power[x,2],Power[x,3]]     x^5
x x       Times[x,x]                       x^2
2 a 3 a   Times[6,a,a]                      6 a^2
x/x       Times[Power[x,-1],x]             1
x^3/x^2   Times[Power[x,3],Power[Power[x,2],-1]]   x
x+x       Plus[x,x]                        2 x
2 x+3 x   Plus[Times[2,x],Times[3,x]]      5 x
x-x       Plus[x,Times[-1,x]]              0
```

Close this with THREE related changes, all in `arithmetic.rs`, scoped to
**numeric exponents and numeric coefficients only**:

- **(A)** `eval_power`: flatten nested powers `Power[Power[b, e1], e2] -> Power[b, e1*e2]`
  when `e1` is an exact rational AND `e2` is an **Integer** (WK-safe: only an
  integer outer exponent distributes — `(x^2)^3 -> x^6`, `(x^2)^-1 -> x^-2`,
  but `(x^2)^(1/2)` must STAY `Power[x^2, 1/2]`). Needed because rum builds
  division denominators as `Power[Power[x,2], -1]`; without flattening, (B)
  cannot see the shared base.
- **(B)** `combine_multiplicative_args`: after the numeric fold, group the
  remaining factors by base and sum exponents (`x^2 · x^3 -> x^5`, `a b / b -> a`).
- **(C)** `combine_additive_args`: after the numeric fold, group the remaining
  terms by kernel and sum coefficients (`2x + 3x -> 5x`, `x - x -> 0`).

When done, rum matches WK on like-base/like-term combination for numeric
exponents/coefficients, with every existing test green and no regression on the
hot `Plus`/`Times` path.

## Context pointers

- Read `AGENTS.md` and the `docs/next-session-handoff.md` `2026-06-24 (LATER)`
  block (this gap is recorded there with the blast-radius analysis).
- **Why this is lower-risk than it looks (verified — do not re-derive):**
  `combine_additive_args`/`combine_multiplicative_args` are also called by
  `calculus.rs` (`D`) and the `polynomials/` subsystem (`Expand`/`Factor`/
  `Together`/`Coefficient`). Those subsystems **already self-combine** via their
  own polynomial representation and feed `combine_*` already-DISTINCT terms — so
  adding combination inside `combine_*` is IDEMPOTENT for them (nothing to merge).
  Confirmed: `D[x^2,x]`=`2 x`, `Expand[(x+1)^2]`=`1+2x+x^2`, `Factor[x^2-1]`,
  `Together`, `Coefficient` are all already correct in rum. The ONLY behavior that
  changes is the bare `Plus`/`Times` path — exactly the target. Their existing
  tests passing after your change is the proof; keep them green.
- **Core correctness anchor:** WK's kernel already does this combination and
  mma-hol runs correctly under WK, so making rum combine moves rum TOWARD WK.
  Implement to match WK exactly on the cases below.
- Key functions (`crates/rum-runtime/src/runtime/arithmetic.rs`):
  - `eval_power` (~:16) — add (A).
  - `combine_multiplicative_args` (~:444) — add (B). Current body: folds
    `expr_as_exact_rational` factors into `numeric`, pushes the rest verbatim,
    then `normalize_call_args_by_attributes` (sorts/flattens via Orderless).
  - `combine_additive_args` (~:411) — add (C), symmetric.
- Helpers already present: `expr_as_exact_rational` (:503, Integer+Rational),
  `expr_as_signed_integer_value` (Integer only — use for the (A) `e2`-is-Integer
  test), `Expr::rational`/`Expr::integer`/`Expr::call`, `system_expr("Power")`/
  `("Times")`, `Rational` `add`/`mul`/`zero`/`one`/`is_zero`/`is_one`.
- `Expr`'s `==` is true STRUCTURAL equality (`expr.rs:583`, with a ptr_eq fast
  path) — use it to group. Group with a `Vec<(Expr, Rational)>` + linear scan
  (factor/term counts are small; do NOT use a HashMap — `Expr`'s manual
  `PartialEq` + derived `Hash` consistency is not something to rely on here).

## WolframKernel semantics to match (verified this session)

Use `===` (SameQ) assertions so the Orderless render order does not matter
(rum's sort order differs from WK and that is a SEPARATE pre-existing thing you
must NOT touch). Each must evaluate to `True`:

```
x x === x^2
x^2 x^3 === x^5
x^2 x === x^3
a b / b === a
x x x / x === x^2
2 a 3 a === 6 a^2
x/x === 1
x^2 y x === x^3 y
(x^2)^3 === x^6
(x^2)^-1 === x^-2
x^3/x^2 === x
x^2/x^3 === x^-1
x + x === 2 x
2 x + 3 x === 5 x
x - x === 0
3 x - x === 2 x
x + 2 x - 3 x === 0
a + b + a === 2 a + b
x + x^2 + x === 2 x + x^2
```

Plus the bare scalar reductions (assert the literal result): `x - x` -> `0`,
`x/x` -> `1`, `x + 2 x - 3 x` -> `0`.

## Algorithm (precise)

### (A) `eval_power` nested-power flatten
When `args == [Power[b, e1], e2]` with `e1 = expr_as_exact_rational(...)` AND
`e2 = expr_as_signed_integer_value(...)` (Integer): compute `e3 = e1 * e2`
(Rational) and return the canonical power of `b` with exponent `e3`:
`e3 == 0` -> `Expr::integer(1)` (note: `b` is non-numeric here, so `b^0 -> 1`
is correct; a numeric `Power[0,0]` never reaches this arm — guard with the
existing `base.is_zero()` check if needed), `e3 == 1` -> `b`, else
`Power[b, Expr::rational(e3)]`. Do NOT flatten when `e2` is non-integer
(`(x^2)^(1/2)` stays `Power[Power[x,2], 1/2]`).

### (B) `combine_multiplicative_args` like-base merge
After the numeric fold, before `normalize_*`, transform the non-numeric
`combined` factors:
1. `split_base_exp(f) -> (Expr base, Rational exp)`:
   - `f == Power[b, e]` with `e = expr_as_exact_rational` -> recurse on `b`:
     `let (bb, ee) = split_base_exp(b); (bb, ee * e)` (so a nested
     `Power[Power[x,2],-1]` that slipped past (A) still collapses to `(x, -2)`).
   - else -> `(f.clone(), Rational::one())`.
2. Group into `Vec<(Expr, Rational)>` by `base ==`, summing `exp`.
3. Rebuild each group: `exp == 0` -> drop (it contributes 1); `exp == 1` -> push
   `base`; else -> push `Power[base, Expr::rational(exp)]`.
4. Hand the rebuilt factors (+ the numeric coefficient as today) to
   `normalize_call_args_by_attributes`, with the SAME empty/singleton handling as
   now.

### (C) `combine_additive_args` like-term merge
After the numeric fold, before `normalize_*`, transform the non-numeric
`combined` terms:
1. `split_coeff_kernel(t) -> (Rational coeff, Expr kernel)`:
   - `t == Times[...]`: scan its args for the (unique, already-folded) numeric
     factor -> `coeff`; the remaining factors are the kernel — rebuild as the
     single remaining factor, or `Times[rest...]` preserving order (the args are
     already Orderless-sorted, so equal kernels stay structurally equal). No
     numeric factor -> `coeff = 1`.
   - else -> `(Rational::one(), t.clone())`.
2. Group into `Vec<(Expr kernel, Rational coeff)>` by `kernel ==`, summing
   `coeff`.
3. Rebuild each group: `coeff == 0` -> drop (this is what makes `x - x -> 0`);
   `coeff == 1` -> push `kernel`; else -> rebuild via
   `self.combine_multiplicative_args(&system_expr("Times"), &[Expr::rational(coeff), kernel])`
   (safe: that path never calls back into `combine_additive_args`).
4. Hand the rebuilt terms (+ numeric constant as today) to `normalize_*`, same
   empty/singleton handling (empty -> `0`).

## Scope

- File you MAY modify: `crates/rum-runtime/src/runtime/arithmetic.rs` (the three
  fns + new private `split_*` helpers) and a test module under
  `crates/rum-runtime/src/runtime/tests/` (extend `arithmetic.rs` there).
- **OUT OF SCOPE — do NOT implement, leave rum's current output as guards:**
  - Symbolic-exponent combination: `x^a x^b -> x^(a+b)`, `(x^a)^2 -> x^(2a)`.
    Only NUMERIC exponents/coefficients in this brief.
  - Power-over-Times distribution: `(a b)^2 -> a^2 b^2`, `(2 x)^2 -> 4 x^2`.
    Leave `(2 x)^2` as `Power[Times[2, x], 2]` (a SEPARATE gap).
  - rum's Orderless SORT order (differs from WK) — do not touch; that is why the
    table uses `===`, not literal render comparison.
- MUST NOT touch: any other file; `BuiltinHandler`s; the evaluator/dispatch
  outside these three fns; `calculus.rs`/`polynomials/` (they call `combine_*`
  but you change only `combine_*` internals); any `mma-hol` clone or snapshot
  checkpoints; `crates/rum-core` `Expr`/`Symbol`/serde shape; `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**` except this report; `.git`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Match WK exactly on the listed edges.** `(x^2)^(1/2)` must NOT collapse
   (non-integer outer exponent); `x - x` MUST become `0` (coeff 0 dropped);
   `x/x` MUST become `1` (exp 0 dropped). A near-miss here is a NEW divergence —
   worse than the current honest gap.
2. **Hot path — watch allocation/cost.** `Times`/`Plus` are the most-evaluated
   builtins and sit under the whole dispatch-constant lane. The grouping is
   `O(n^2)` over small `n`; do not add per-call heap churn beyond the rebuild.
   Reuse `Arc`/borrowed traversal where you can; the microbench must not regress
   (see Verification).
3. **Idempotence for the subsystems.** `Expand`/`Factor`/`Together`/`D`/
   `Coefficient` feed already-distinct terms; your change must leave their tests
   byte-identical. If a `calculus`/`polynomials` test changes, STOP and report —
   it means your grouping is altering already-combined input (a bug).
4. **No infinite recursion.** (C)'s rebuild may call `combine_multiplicative_args`
   (fine — it never calls back into `combine_additive_args`). (A)/(B)'s
   `split_base_exp` recursion terminates (each step strips one `Power` layer).
5. **Minimal diff.** Only these three fns + helpers + the test. No reordering,
   no doc rewrites elsewhere.
6. **Stop-loss.** Same failure twice, an existing test you can't keep green, or
   a microbench regression you can't remove → STOP and report.

## Verification (MANDATORY — run, iterate to green, paste verbatim)

```
cargo fmt --check
cargo test -p rum-runtime            # new test + ALL existing (calculus/polynomials) green
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# every one must print True (=== assertions, order-independent):
$BIN --eval 'x x === x^2'
$BIN --eval 'x^2 x^3 === x^5'
$BIN --eval 'x^2 x === x^3'
$BIN --eval 'a b / b === a'
$BIN --eval 'x x x / x === x^2'
$BIN --eval '2 a 3 a === 6 a^2'
$BIN --eval 'x/x === 1'
$BIN --eval 'x^2 y x === x^3 y'
$BIN --eval '(x^2)^3 === x^6'
$BIN --eval '(x^2)^-1 === x^-2'
$BIN --eval 'x^3/x^2 === x'
$BIN --eval 'x^2/x^3 === x^-1'
$BIN --eval 'x + x === 2 x'
$BIN --eval '2 x + 3 x === 5 x'
$BIN --eval 'x - x === 0'
$BIN --eval '3 x - x === 2 x'
$BIN --eval 'x + 2 x - 3 x === 0'
$BIN --eval 'a + b + a === 2 a + b'
$BIN --eval 'x + x^2 + x === 2 x + x^2'

# OUT-OF-SCOPE guards — must stay as rum is now (paste actual output):
$BIN --eval '(x^2)^(1/2)'      # must NOT be x (non-integer outer exp); paste it
$BIN --eval '(2 x)^2'          # Power[Times[2, x], 2]  (no distribution)
$BIN --eval 'Power[0,0]'       # Power[0, 0]
$BIN --eval 'x^a'              # Power[x, a]  (symbolic exponent untouched)
$BIN --eval '2^10'             # 1024

# microbench (paste ratios.tsv; confirm arithmetic/dispatch labels do not regress):
bash scripts/core-language-microbench.sh --engine both --output-dir /tmp/cdx_brief_043_micro
```

Add a `power_times_combine_like_bases_and_terms` test asserting the `===` cases
above resolve to `True` (use `eval_to_string(&mut session, "x x === x^2") == "True"`
etc.) plus the bare-scalar reductions (`x - x` -> `0`, `x/x` -> `1`).

Paste all output verbatim. (The real-tower 1122 re-run + batch suites are
Claude's follow-up gate — you do NOT run mma-hol.)

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude runs the authoritative gate (incl. the real-tower rebuild)
  and commits.
- NEVER edit any `mma-hol` clone or the snapshot checkpoints.
- Do NOT modify files outside Scope. Do NOT implement the out-of-scope items.
- Full machine access, touch NOTHING outside this repo, no network. Commands:
  file edits within Scope, `cargo`, the release `rum-cli`, the microbench script.
- Minimal diff. You MUST paste real verbatim output; if not green, say so.
- Stop-loss: a broken existing test or an unremovable microbench regression →
  STOP and report, do not push through.

## Report format (final message)

1. What changed — (A)/(B)/(C) per fn + the `split_*` helpers + the test, files +
   line ranges, one-line why each.
2. Verification — every command verbatim: `fmt`, test counts (call out any
   calculus/polynomials test you confirmed still green), each `===` -> True, the
   out-of-scope guard outputs, and the microbench ratios for arithmetic/dispatch
   labels with a before/after note.
3. Verdict — like-base/like-term combination matches WK for numeric exponents/
   coefficients; out-of-scope items unchanged; subsystem tests green; no
   microbench regression.
4. Stop-loss / open questions (empty if none).
