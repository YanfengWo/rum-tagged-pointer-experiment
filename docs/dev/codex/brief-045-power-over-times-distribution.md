# brief-045: Power-over-Times distribution — `(2 x)^2 -> 4 x^2`, `(a b)^2 -> a^2 b^2` (numeric-exponent scope)

## Goal

`Power` over a `Times` base does not distribute in rum: `(2 x)^2` stays
`Power[Times[2, x], 2]` instead of `4 x^2`. WK distributes in two layers (verified
this session); implement the NUMERIC-exponent slice in `eval_power`, reusing the
brief-043 `(A)` nested-power flatten and `combine_multiplicative_args` already in
the tree. When done, rum matches WK on:

```
(2 x)^2      -> 4 x^2            (6 x)^2    -> 36 x^2
(2 x)^3      -> 8 x^3            (x/2)^2    -> x^2/4
(-2 x)^3     -> -8 x^3           (2 x)^-2   -> 1/(4 x^2)
(3/2 x)^2    -> 9/4 x^2          (2 x y)^2  -> 4 x^2 y^2
(a b)^2      -> a^2 b^2          (a/b)^2    -> a^2/b^2
(2 x)^(1/2)  -> Sqrt[2] Sqrt[x]  (2 x y)^(1/2) -> Sqrt[2] Sqrt[x y]
```

## The WK rule (verified this session — implement EXACTLY)

For `Power[Times[factors...], n]`, split `factors` into a numeric product `c`
(integers/rationals) and the symbolic rest `R`:

1. **The numeric factor `c` is always lifted and raised: `c^n`** — for ANY numeric
   `n` (`(2 x)^2 -> 4·x^2`, `(2 x)^(1/2) -> 2^(1/2)·x^(1/2)`, `(3/2 x)^2 ->
   (9/4)·x^2`). In rum `c^n` is just `eval_power(c, n)`: it computes a rational
   when `n` is an integer, and stays the inert `Power[c, n]` otherwise (rum is
   rationals-only — it does NOT take roots, so `4^(1/2)` stays `Power[4, 1/2]`,
   NOT `2`; that non-simplification is a pre-existing limitation, NOT this brief's
   concern).
2. **The symbolic rest `R` distributes ONLY for INTEGER `n`:**
   - `n` integer → each symbolic factor is raised separately: `∏ f_i^n`
     (`(a b)^2 -> a^2 b^2`; `(a/b)^2 -> a^2·(b^-1)^2 -> a^2 b^-2`, which is where
     the brief-043 `(A)` nested-power flatten kicks in).
   - `n` rational but NON-integer → `R` is NOT distributed; it stays as one
     `Power[Times[R...], n]` (`(a b)^(1/2) -> Sqrt[a b]`, NOT `Sqrt[a] Sqrt[b]`;
     `(2 x y)^(1/2) -> Sqrt[2]·Sqrt[x y]`). A single-factor `R` is just
     `Power[f, n]`.
3. Combine the numeric part and the symbolic part with
   `combine_multiplicative_args(Times, [...])` (it drops a `1` coefficient, so
   `c == 1` self-cleans).

**OUT OF SCOPE (leave inert / lazy, lock as guards):**
- Non-numeric (symbolic) exponent `n`: `(2 x)^n` stays `Power[Times[2, x], n]`
  (WK gives `2^n x^n`, but symbolic exponents are a separate axis — same line we
  drew for brief-043; do NOT handle them here).
- rum not taking roots of perfect squares (`(4 x^2)^(1/2)` → rum keeps
  `Power[4, 1/2]·Power[Power[x,2],1/2]`, WK `2 Sqrt[x^2]`) — pre-existing
  rationals-only limitation, NOT this brief.

## Context pointers

- Read `AGENTS.md` and the `docs/next-session-handoff.md` top `2026-06-24` block
  (brief-043/044 landed the like-term/like-base combination + the `(A)` flatten).
- Key file/fn: `crates/rum-runtime/src/runtime/arithmetic.rs` `eval_power` (~:16).
  Its current shape: a numeric-base branch, then the `(A)` branch
  (`if let Some((base, inner_exponent)) = split_power_base_exp(&args[0])` →
  integer outer exponent → `build_power_from_base_exp`), then the fallback
  `Ok(Expr::call(head.clone(), args.to_vec()))`. **Add the Times-base branch
  AFTER the `(A)` branch and BEFORE the fallback.**
- Reuse, do NOT reimplement: `expr_as_exact_rational` (numeric test + the `c`
  product), `expr_as_signed_integer_value` (the integer-`n` test),
  `self.eval_power(&system_expr("Power"), &[..])` (for `c^n` and each `f_i^n` —
  recursion terminates: `c` is numeric, each `f_i` is a non-`Times` factor),
  `self.combine_multiplicative_args(&system_expr("Times"), &[..])`,
  `is_system_symbol_name(head, "Times")`.

## Algorithm (precise)

In `eval_power`, after the `(A)` branch, add:
```text
if args[0] is Expr::Call { head, args: factors } with is_system_symbol_name(head,"Times"):
    # only act when the exponent is numeric (integer or rational); symbolic n -> fall through to lazy
    if let Some(n_rat) = expr_as_exact_rational(&args[1]):
        # split factors
        c = product of expr_as_exact_rational(f) over numeric factors   # Rational, default 1
        R = the non-numeric factors (Vec<Expr>)
        numeric_part = self.eval_power(Power, [Expr::rational(c), args[1].clone()])
        if let Some(_int) = expr_as_signed_integer_value(&args[1]):      # integer n
            parts = vec![numeric_part];
            for f in R: parts.push(self.eval_power(Power, [f.clone(), args[1].clone()]));
            return self.combine_multiplicative_args(Times, &parts);
        else:                                                            # rational non-integer n
            sym = match R.len() {
                0 => Expr::integer(1),
                1 => self.eval_power(Power, [R[0].clone(), args[1].clone()]),
                _ => Expr::call(Times-base reconstructed, args[1] as exponent)  # Power[Times[R...], n], not distributed
            };
            return self.combine_multiplicative_args(Times, &[numeric_part, sym]);
    # else (symbolic n): fall through to the lazy fallback
```
Notes:
- `c` defaults to `Rational::one()` when there is no numeric factor; then
  `numeric_part = 1^n = 1` and `combine_*` drops it.
- For the non-integer multi-factor `sym`, rebuild `Power[Times[R...], n]` directly
  (do NOT recurse into `eval_power` on the whole `Times[R...]` — that would loop;
  construct the inert `Power` node, then let `combine_multiplicative_args` place
  it). A single-factor `R` recurses on the bare factor (safe, not a `Times`).

## Scope
- File you MAY modify: `crates/rum-runtime/src/runtime/arithmetic.rs` (`eval_power`
  + a small private helper if you want, e.g. `partition_times_numeric`) and a test
  module under `crates/rum-runtime/src/runtime/tests/` (extend `arithmetic.rs`).
- MUST NOT touch: the brief-043/044 `combine_*` / `split_*` logic (reuse, don't
  edit); any other handler; `crates/rum-core`; any `mma-hol`/snapshot;
  `AGENTS.md`/`PROGRESS.md`/`TODO.md`/`docs/**` except this report; git.
- Do NOT implement the out-of-scope items. Do NOT add unconditional cost to the
  non-`Times`-base path (the new branch only runs when `args[0]` is a `Times`).

## rum / Rust pitfalls
1. **Match WK's two layers exactly.** Numeric `c` lifts for ANY numeric `n`;
   symbolic `R` distributes ONLY for integer `n`. `(a b)^(1/2)` MUST stay
   `Power[Times[a, b], 1/2]` (not distributed); `(a b)^2` MUST become `a^2 b^2`.
2. **rum does not take roots.** `c^n` for non-integer `n` stays inert
   (`Power[2, 1/2]`); that's correct for this brief. Don't try to simplify it.
3. **No recursion on the whole `Times` base** for the non-integer multi-factor
   case — build the inert `Power[Times[...], n]` directly or you loop.
4. **Reuse `(A)`**: `(a/b)^2` works because `(b^-1)^2` hits the nested-power
   flatten — you get it for free by recursing `eval_power` on each integer-`n`
   factor. Don't special-case it.
5. **Minimal diff**, one new branch + optional helper; don't perturb 043/044.
6. **Stop-loss**: a probe disagreeing with WK for a reason you can't localize →
   STOP and report.

## Verification (MANDATORY — run, paste verbatim)
```
cargo fmt --check
cargo test -p rum-runtime           # 690/0 + your new cases, all green
cargo build --release -p rum-cli
BIN=./target/release/rum-cli

# integer-exponent distribution (=== prints True):
$BIN --eval '(2 x)^2 === 4 x^2'
$BIN --eval '(2 x)^3 === 8 x^3'
$BIN --eval '(-2 x)^3 === -8 x^3'
$BIN --eval '(3/2 x)^2 === 9/4 x^2'
$BIN --eval '(6 x)^2 === 36 x^2'
$BIN --eval '(x/2)^2 === x^2/4'
$BIN --eval '(2 x)^-2 === 1/(4 x^2)'
$BIN --eval '(2 x y)^2 === 4 x^2 y^2'
$BIN --eval '(a b)^2 === a^2 b^2'
$BIN --eval '(a/b)^2 === a^2/b^2'
# rational-exponent: numeric lifts, symbols stay together (=== True):
$BIN --eval '(2 x)^(1/2) === 2^(1/2) x^(1/2)'
$BIN --eval '(2 x y)^(1/2) === 2^(1/2) (x y)^(1/2)'
$BIN --eval '(a b)^(1/2) === (a b)^(1/2)'      # True (self) — and check it stays one Power below

# guards — paste actual output (must NOT distribute / must stay inert):
$BIN --eval '(a b)^(1/2)'             # Power[Times[a, b], 1/2]   (symbolic, non-integer → not distributed)
$BIN --eval '(2 x)^n'                 # Power[Times[2, x], n]     (symbolic exponent → out of scope)
$BIN --eval '(4 x^2)^(1/2)'           # numeric lifted but 4^(1/2) NOT rooted — paste it
# brief-043 regressions still hold:
$BIN --eval 'x^2 x^3 === x^5'
$BIN --eval 'x - x === 0'
```
Paste everything verbatim. (The real-tower 1122 re-run is Claude's gate; you do
NOT run mma-hol.)

## Hard rules
- Do NOT git commit/branch/push. Leave changes in the working tree.
- NEVER edit any `mma-hol` clone or snapshot.
- Do NOT modify files outside Scope; do NOT implement out-of-scope items.
- Full machine access, nothing outside this repo, no network. Commands: edits in
  scope, `cargo`, release `rum-cli`.
- Minimal diff; paste real verbatim output; if a probe is not WK-correct, say so.

## Report format
1. What changed — the `eval_power` Times branch (+ helper) + the test, line
   ranges, one-line why.
2. Verification — `fmt`, test count, every `===`/guard verbatim.
3. Verdict — numeric `c` lifts for any numeric `n`; symbolic `R` distributes only
   for integer `n`; out-of-scope items inert; 043/044 unchanged.
4. Stop-loss / open questions.
