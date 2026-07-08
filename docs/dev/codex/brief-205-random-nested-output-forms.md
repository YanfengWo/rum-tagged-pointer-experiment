# Brief 205 — Nested `{dims}` output for RandomInteger + RandomChoice (unit-7 fix 2)

Implementation brief. Executes the unit-7 triage batch 2 (tracker
`docs/dev/usability-pass.md`, "Unit 7 triage"; findings gap #2 in
`docs/dev/usability-pass/character-classes-random-findings.md`). This adds
the WL-documented tensor output form to two draw functions. It is a
per-dispatch generation change — NOT call planning, NOT attribute-driven
planning — so NO microbench A/B is needed. Read `crates/rum-stdlib/src/
random.rs` first (the `eval_random_integer` / `eval_random_choice`
dispatch and the `parse_output_count` / `random_integer_between` /
`random_usize_below` / `list_expr` helpers).

## Scope decision (READ — this brief is deliberately narrower than the finding)

The audit flagged `{dims}` for `RandomInteger`, `RandomChoice`, AND
`RandomString`, plus a zero-arg `RandomString[]`. This brief ships ONLY
`RandomInteger[spec, {dims}]` and `RandomChoice[list, {dims}]` — the two
unambiguous WL tensor-sampling forms whose second argument is a SAMPLE
COUNT. `RandomString`'s second argument is the string LENGTH (not a sample
count), so a `{dims}` form and the zero-arg `RandomString[]` have an
uncertain WL surface; per the unit-5 "DEFER where the WL surface is
uncertain — never guess" discipline they are OUT of this brief (recorded
as deferred in the tracker). Do NOT touch `RandomString`, `RandomSample`,
`parse_random_text_args`, or any entropy/password head.

## What and why

Today `RandomInteger[spec, count]` and `RandomChoice[list, count]` parse
`count` only as a scalar exact integer → a flat 1-D list; a list second
argument stays inert (verified: `RandomInteger[9, {2, 3}]` →
`RandomInteger[9, List[2, 3]]`). WL's tensor form
`RandomInteger[spec, {n1, n2, ..., nk}]` returns a nested k-deep array
(row-major); `RandomChoice[list, {n1, ..., nk}]` likewise. Add these,
reusing the existing per-draw primitives and the generated-element limit.

The existing scalar-count and no-count paths MUST stay byte-identical
(`RandomInteger[spec]` = bare integer; `RandomInteger[spec, n]` = flat
length-n list; same for `RandomChoice`). The `{dims}` branch is ADDED
strictly after the scalar attempt fails.

## Change 1 — two shared helpers in `random.rs`

**`parse_dims`** — parse a dimensions list, bounding allocation:

```rust
/// Parses a `{n1, n2, ...}` dimensions spec of non-negative exact
/// integers. Returns None (caller stays inert) if `expr` is not a list of
/// non-negative exact integers. Enforces the generated-element limit on
/// every prefix product so no level can allocate an unbounded number of
/// sub-arrays.
fn parse_dims(session: &Session, expr: &Expr) -> Result<Option<Vec<usize>>, EvalError> {
    let Some(items) = list_items(expr) else {
        return Ok(None);
    };
    let mut product = BigInt::from(1_u8);
    let mut dims = Vec::with_capacity(items.len());
    for item in items {
        let Some(dim) = expr_as_exact_integer(item) else {
            return Ok(None);
        };
        if dim < BigInt::from(0_u8) {
            return Ok(None);
        }
        product *= &dim;
        // Bounds the cumulative allocation at THIS level (and, being a
        // prefix product, every shallower level too) — catches a huge
        // leading dim such as {10^20, 0} where the leaf count is 0 but the
        // top level would allocate 10^20 empty sub-lists.
        session.enforce_generated_element_limit_for_big_count(&product)?;
        // A dim that does not fit usize can only occur once a prior dim was
        // 0 (else the prefix product would exceed the limit above), so that
        // subtree is never materialized; 0 is a safe stand-in there.
        dims.push(dim.to_usize().unwrap_or(0));
    }
    Ok(Some(dims))
}
```

(Confirm the exact `BigInt` ops compile against `rum_core::BigInt`;
`enforce_generated_element_limit_for_big_count` is the same helper
`parse_output_count` already calls.)

**`build_random_array`** — recursive row-major nested builder over a
per-leaf draw closure:

```rust
fn build_random_array(
    session: &mut Session,
    dims: &[usize],
    draw: &mut dyn FnMut(&mut Session) -> Expr,
) -> Expr {
    match dims.split_first() {
        None => draw(session),
        Some((&head, rest)) => {
            let items = (0..head)
                .map(|_| build_random_array(session, rest, draw))
                .collect();
            list_expr(items)
        }
    }
}
```

`dims == []` yields a bare leaf (0-D array), matching WL
`RandomInteger[spec, {}]`. A single-element `{n}` yields a flat length-n
list — identical to the scalar-`n` path.

## Change 2 — wire the `{dims}` branch into the two dispatchers

`eval_random_integer`, the `[spec, count]` arm — keep the scalar path
exactly; add the dims branch between the scalar-None and the inert
fallback:

```rust
[spec, count] => {
    let Some((min, max)) = random_integer_bounds(spec) else {
        return Ok(Expr::call(head.clone(), args.to_vec()));
    };
    if let Some(count) = parse_output_count(session, count)? {
        let values = (0..count)
            .map(|_| random_integer_between(session, &min, &max))
            .collect();
        Ok(list_expr(values))
    } else if let Some(dims) = parse_dims(session, count)? {
        Ok(build_random_array(session, &dims, &mut |s| {
            random_integer_between(s, &min, &max)
        }))
    } else {
        Ok(Expr::call(head.clone(), args.to_vec()))
    }
}
```

`eval_random_choice`, the counted arm — keep the empty-list check and the
scalar path; add the dims branch. The leaf clones a uniformly chosen item
(`items` borrows the list, not the session, so the closure is fine):

```rust
// after: items non-empty checked, `count` is the Option<&Expr> second arg
let Some(count) = count else {
    return Ok(items[random_usize_below(session, items.len())].clone());
};
if let Some(count) = parse_output_count(session, count)? {
    let choices = (0..count)
        .map(|_| items[random_usize_below(session, items.len())].clone())
        .collect();
    Ok(list_expr(choices))
} else if let Some(dims) = parse_dims(session, count)? {
    let len = items.len();
    Ok(build_random_array(session, &dims, &mut |s| {
        items[random_usize_below(s, len)].clone()
    }))
} else {
    Ok(Expr::call(head.clone(), args.to_vec()))
}
```

Match the existing arm structure exactly; do not reorder the empty-list /
scalar checks. Draw order is row-major (outermost dimension first), the
natural recursion order — so a `{n}` dims draws the same sequence a scalar
`n` would, keeping single-axis output reproducible against the flat form
under a fixed seed.

## Change 3 — docgen Limits (MERGE, do not replace) + regen reference

The docgen `LIMITS` table is keyed by name; append to the EXISTING entries
(brief-198 gate lesson — do NOT overwrite):

- `RandomInteger` (`docgen.rs:116`): append after the existing text —
  ` RandomInteger[spec, {n1, n2, ...}] returns a nested array of those dimensions.`
- `RandomChoice` (`docgen.rs:112`): append —
  ` RandomChoice[list, {n1, n2, ...}] returns a nested array of those dimensions.`

Then regenerate (do NOT hand-edit `builtins.md`):

```
cargo run -q -p rum-stdlib --bin generate-builtin-reference
```

Only the `RandomInteger` and `RandomChoice` Limits lines should change.

## Change 4 — tests (`crates/rum-stdlib/src/tests/random.rs`)

Add a test covering: nested shape via `Dimensions` (`RandomInteger[9,
{2,3}]` has dimensions `{2,3}`; `RandomChoice[{a,b,c}, {2,2}]` has
`{2,2}`); every leaf in range / drawn from the list; reproducibility under
a fixed seed (`SeedRandom[k]` twice gives `SameQ` nested results);
byte-identical scalar path (`RandomInteger[9, 3]` still a flat length-3
list, `RandomChoice[{a,b}, 2]` still flat); the empty-axis case
(`RandomInteger[9, {0, 5}]` is `List[]`); and the element-limit fail-close
(a dims product beyond the session budget errors with the
generated-element-limit message rather than allocating — size it from the
session's configured limit; use `core_session_with_entropy`/the existing
test session helper and mirror how other limit tests in this file assert
the failure). Follow the existing patterns in the file (e.g.
`random_text_composes_with_character_class_constants`,
`core_session_with_entropy`).

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[7]; g = RandomInteger[9, {2, 3}]; c = RandomChoice[{a, b, c}, {2, 2}]; {g, Dimensions[g], c, Dimensions[c]}'
printf '' | cargo run -q -p rum-cli -- --eval 'SeedRandom[7]; a = RandomInteger[9, {2, 3}]; SeedRandom[7]; b = RandomInteger[9, {2, 3}]; {SameQ[a, b], RandomInteger[9, 3], RandomChoice[{a, b}, 2], RandomInteger[9, {0, 5}], RandomInteger[9, {}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals). First probe: `g` a
2×3 nested integer array with `Dimensions` `{2, 3}`, `c` a 2×2 array with
`{2, 2}`. Second probe: `SameQ` `True` (reproducible), the scalar forms
still flat, `{0, 5}` → `List[]`, `{}` → a bare integer.

## Scope

- MAY modify: `crates/rum-stdlib/src/random.rs` (the two helpers + the two
  dispatch arms), `crates/rum-stdlib/src/docgen.rs` (the two Limits
  merges), `docs/reference/builtins.md` (regenerated — the two Limits
  lines only), `crates/rum-stdlib/src/tests/random.rs` (the new test).
- MUST NOT touch: `RandomString`/`RandomSample`/`parse_random_text_args`,
  any entropy/password head, `eval_seed_random`/`BlockRandom`, the
  `catalog/*` specs, the runtime attribute mirror, any capability, the
  scalar/no-count code paths' behavior, `TODO.md`, the tracker, findings
  files, `Cargo.toml`.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies, no
  new `unsafe`.
- The scalar-count and no-count paths stay byte-identical — the `{dims}`
  branch is added strictly after the scalar attempt returns None. If a
  probe shows any change to `RandomInteger[spec, n]` / `RandomChoice[list,
  n]` / bare forms, that is a defect.
- Allocation is bounded by the prefix-product element-limit enforce in
  `parse_dims`; a `{dims}` beyond budget must FAIL CLOSED with the
  existing generated-element-limit message, never OOM or hang. Do NOT add
  a new limit knob.
- No A/B gate: this is per-dispatch generation, not `eval_call_prepare`.
  If `cargo test` surfaces a change to any unrelated draw sequence, STOP
  and report.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — `random.rs` (two helpers + two arms), `docgen.rs` (two
   merges), reference regen, the test; line ranges.
2. Verification — all commands verbatim (test totals + both probes +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
