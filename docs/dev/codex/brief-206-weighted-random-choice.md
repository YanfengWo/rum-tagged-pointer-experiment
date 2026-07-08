# Brief 206 — Weighted RandomChoice (unit-7 fix 3)

Implementation brief. Executes the unit-7 triage batch 3 (tracker
`docs/dev/usability-pass.md`, "Unit 7 triage"; findings gap #3 in
`docs/dev/usability-pass/character-classes-random-findings.md`). Adds WL's
weighted-choice form `RandomChoice[weights -> elements]` using EXACT
rational weights and integer arithmetic (no floating point — rum is
rationals-only). Per-dispatch generation, NOT call planning, NOT
attribute-driven planning → NO microbench A/B. Read `crates/rum-stdlib/
src/random.rs` `eval_random_choice` first (brief-205 just added the
`parse_dims` / `build_random_array` helpers there — reuse them).

## What and why

Today `RandomChoice[list(, count)]` draws uniformly. WL also accepts a
weighted spec:

- `RandomChoice[{w1, ..., wk} -> {e1, ..., ek}]` — a single draw where
  `ei` has probability `wi / sum(w)`;
- `RandomChoice[{w..} -> {e..}, n]` — a flat list of `n` weighted draws;
- `RandomChoice[{w..} -> {e..}, {dims}]` — a nested array of weighted
  draws.

rum has no floats, so weights are non-negative EXACT rationals (integers
included). Draw exactly by scaling to integer weights and drawing a
uniform integer over their sum — never a float.

The existing plain-list path (uniform draw, scalar list, and the
brief-205 `{dims}`) MUST stay byte-identical. The weighted path is entered
ONLY when the first argument is a `Rule[...]`.

## The exact algorithm (no floating point)

Given weights `w_i = p_i / q_i` (each `Rational` is already reduced with
`q_i > 0` via `rum_core::Rational`) and elements `e_i`, same non-empty
length:

1. `D = lcm(q_1, ..., q_k)` (fold with the `Integer::lcm` trait method,
   the same `Integer` trait `number_theory.rs` uses for `gcd`).
2. Integer weight `W_i = p_i * (D / q_i)` — exact, non-negative.
3. `T = sum(W_i)` (must be `> 0`).
4. Prefix sums `cum_i = W_1 + ... + W_i`, so `cum_k == T`.
5. Each draw: `r = random_bigint_below(session, &T)` (uniform in `[0, T)`,
   the existing helper), then the chosen index is
   `cum.partition_point(|c| c <= &r)` (first `cum_i > r`; well-defined
   because `cum_k == T > r`). Return `elements[idx].clone()`.

All values are `BigInt`/`Rational`; there is no `f64` anywhere.

## Change 1 — detect the Rule form and branch (`eval_random_choice`)

At the top of `eval_random_choice`, after computing `(spec, count)` from
the arity match but BEFORE `list_items(spec)`, detect a two-argument
`Rule`:

```rust
if let Expr::Call { head, args: rule_args } = spec {
    if is_system_symbol_name(head, "Rule") && rule_args.len() == 2 {
        return eval_random_choice_weighted(
            session, head_original, args, &rule_args[0], &rule_args[1], count,
        );
    }
}
```

(Use whatever binding the function already has for the ORIGINAL call head
— the `head: &Expr` parameter — for building inert returns; do not shadow
it with the Rule head. Match the existing arm's use of
`is_system_symbol_name`, imported from `kernel_api`.)

If `spec` is not a `Rule`, fall through to the existing plain-list code
UNCHANGED.

## Change 2 — `eval_random_choice_weighted`

```rust
fn eval_random_choice_weighted(
    session: &mut Session,
    head: &Expr,            // original RandomChoice call head, for inert returns
    args: &[Expr],
    weights_expr: &Expr,
    elements_expr: &Expr,
    count: Option<&Expr>,
) -> Result<Expr, EvalError> {
    // Parse the two lists; a non-list stays inert (unparseable, no message).
    let (Some(weight_items), Some(element_items)) =
        (list_items(weights_expr), list_items(elements_expr))
    else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    // Malformed spec -> message + inert (mirror the existing empty case).
    if element_items.is_empty() || weight_items.len() != element_items.len() {
        session.push_message(
            "RandomChoice", "weights",
            "weight and element lists must have equal, nonzero length",
        );
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    // Weights: non-negative exact rationals. A non-rational weight stays
    // inert (could be symbolic); a negative one is a fail-closed message.
    let mut weights = Vec::with_capacity(weight_items.len());
    for w in weight_items {
        let Some(w) = expr_as_exact_rational(w) else {
            return Ok(Expr::call(head.clone(), args.to_vec()));
        };
        if w.numerator() < &BigInt::from(0_u8) {
            session.push_message("RandomChoice", "weights", "weights must be non-negative");
            return Ok(Expr::call(head.clone(), args.to_vec()));
        }
        weights.push(w);
    }
    // Scale to integer weights via lcm of denominators; require total > 0.
    let mut denom_lcm = BigInt::from(1_u8);
    for w in &weights {
        denom_lcm = denom_lcm.lcm(w.denominator());
    }
    let mut cum = Vec::with_capacity(weights.len());
    let mut total = BigInt::from(0_u8);
    for w in &weights {
        let scaled = w.numerator() * (&denom_lcm / w.denominator());
        total += &scaled;                 // total tracks the running sum
        cum.push(total.clone());          // prefix sum
    }
    if total <= BigInt::from(0_u8) {
        session.push_message("RandomChoice", "weights", "at least one weight must be positive");
        return Ok(Expr::call(head.clone(), args.to_vec()));
    }
    let elements: Vec<Expr> = element_items.to_vec();
    let mut draw = |s: &mut Session| -> Expr {
        let r = random_bigint_below(s, &total);
        let idx = cum.partition_point(|c| c <= &r);
        elements[idx].clone()
    };
    match count {
        None => Ok(draw(session)),
        Some(count) => {
            if let Some(n) = parse_output_count(session, count)? {
                let values = (0..n).map(|_| draw(session)).collect();
                Ok(list_expr(values))
            } else if let Some(dims) = parse_dims(session, count)? {
                Ok(build_random_array(session, &dims, &mut draw))
            } else {
                Ok(Expr::call(head.clone(), args.to_vec()))
            }
        }
    }
}
```

(Adjust to the real signatures — e.g. `random_bigint_below(&mut Session,
&BigInt) -> BigInt`, `list_items(&Expr) -> Option<&[Expr]>`. The closure
borrows `cum`, `total`, `elements`; it must be a `&mut` closure passed to
`build_random_array` as `&mut draw`. If the borrow checker objects to
`draw` being used both directly and via `&mut draw`, restructure so each
`match` arm creates its own closure, or take `&mut draw` uniformly.)

Add imports to the `use rum_runtime::kernel_api::{...}` line:
`is_system_symbol_name`, `expr_as_exact_rational`, `Integer`; and
`Rational` from `rum_core` if a type annotation needs it (mirror
`number_theory.rs:1-6`). Do NOT add anything unused.

## Change 3 — docgen Limits (MERGE, do not replace) + regen reference

`RandomChoice` (`docgen.rs:112`) — brief-205 just extended this note; MERGE
again (append, preserve the existing text incl. the `{dims}` sentence):
append ` RandomChoice[weights -> elements] draws each element with
probability proportional to its exact non-negative weight.` Then
regenerate (do NOT hand-edit):

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

Only the `RandomChoice` Limits line should change.

## Change 4 — tests (`crates/rum-stdlib/src/tests/random.rs`)

Add a test covering: deterministic concentration (`RandomChoice[{1, 0, 0}
-> {a, b, c}]` is always `a`; `RandomChoice[{0, 1, 0} -> {a, b, c}, 5]` is
`{b, b, b, b, b}` — these do not depend on the PRNG stream); rational
weights produce a valid mix drawn only from the elements
(`RandomChoice[{1/2, 1/2} -> {a, b}, 20]` — every element is `a` or `b`);
reproducibility under a fixed seed (`SeedRandom[k]` twice → `SameQ`);
nested form (`RandomChoice[{1, 1} -> {a, b}, {2, 2}]` is 2×2); fail-closed
diagnostics with a pushed message + inert return for length mismatch
(`{1, 2} -> {a, b, c}`), negative weight (`{-1, 2} -> {a, b}`), and
zero total (`{0, 0} -> {a, b}`); and that the plain path is unchanged
(`RandomChoice[{a, b, c}]` still draws a single element). Mirror the
existing test helpers in the file (`core_session`,
`core_session_with_entropy`, the message-assertion pattern).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[3]; {RandomChoice[{1, 0, 0} -> {10, 20, 30}], RandomChoice[{0, 1, 0} -> {10, 20, 30}, 5], RandomChoice[{1/2, 1/2} -> {10, 20}, 8], RandomChoice[{1, 1} -> {10, 20}, {2, 2}]}'
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[3]; a = RandomChoice[{3, 1} -> {10, 20}, 6]; SeedRandom[3]; b = RandomChoice[{3, 1} -> {10, 20}, 6]; {SameQ[a, b], RandomChoice[{a2, b2, c2}], RandomChoice[{1, 2} -> {10, 20, 30}], RandomChoice[{-1, 2} -> {10, 20}], RandomChoice[{0, 0} -> {10, 20}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals). First probe: `10`;
`{20,20,20,20,20}`; a length-8 list of only `10`/`20`; a 2×2 nested list
of `10`/`20`. Second probe: `SameQ` `True`; the plain `RandomChoice[{a2,
b2, c2}]` still returns one symbol; the three malformed weighted specs
each print a `RandomChoice::weights` message and return the inert call.

## Scope

- MAY modify: `crates/rum-stdlib/src/random.rs` (the Rule branch + the new
  weighted function + imports), `crates/rum-stdlib/src/docgen.rs` (the
  RandomChoice Limits merge), `docs/reference/builtins.md` (regenerated —
  the RandomChoice Limits line only), `crates/rum-stdlib/src/tests/
  random.rs` (the new test).
- MUST NOT touch: the plain-list `RandomChoice` behavior; `RandomInteger`/
  `RandomString`/`RandomSample`/`RandomUuid`/entropy heads;
  `parse_dims`/`build_random_array` bodies (reuse only); the `catalog/*`
  specs; the runtime attribute mirror; any capability; `TODO.md`, the
  tracker, findings files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`, NO floating point (`f32`/`f64` are forbidden — the draw is
  exact integer arithmetic over scaled weights).
- The plain-list path stays byte-identical — the weighted branch is
  entered ONLY for a two-arg `Rule` first argument. If a probe shows any
  change to `RandomChoice[list]` / `RandomChoice[list, n]` /
  `RandomChoice[list, {dims}]`, that is a defect.
- Weighted draws must be UNBIASED: uniform integer in `[0, T)` over the
  integer-scaled weights, bucketed by prefix sum. Do NOT approximate.
- Fail closed on malformed weighted specs (length mismatch, negative
  weight, zero total) with a `RandomChoice::weights` message + inert
  return; a non-list or non-rational spec stays inert without a message
  (unparseable, possibly symbolic).
- No A/B gate: per-dispatch generation. If `cargo test` surfaces a change
  to an unrelated draw sequence, STOP and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — `random.rs` (Rule branch + weighted fn + imports),
   `docgen.rs` (merge), reference regen, the test; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
