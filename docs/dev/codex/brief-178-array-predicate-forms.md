# Brief 178 — Unit-3 fixes 2: array predicate test/rank forms

Implementation brief. Unit-3 fix batch 2 of the Usability Pass
(tracker `docs/dev/usability-pass.md`, unit-3 triage item 2; evidence
E4 in
`docs/dev/usability-pass/predicates-associations-findings.md`). Pure
per-builtin — NO call planning. Every currently-accepted call stays
byte-identical; currently-rejected documented forms gain meaning.

## Semantics (WL contract — read carefully)

The element `test` is applied to each LEAF element; the predicate is
`True` only if the shape condition holds AND `test[leaf]` yields `True`
(strict `TrueQ`) for every leaf. Evaluate `test[leaf]` through the
normal evaluator (as `Select` does).

IMPORTANT about `ArrayQ`'s second argument: in WL it is a DEPTH/rank
PATTERN matched against the integer depth — NOT a shape. `ArrayQ[m, 2]`
tests depth == 2; `ArrayQ[m, _]` any depth; `ArrayQ[m, 1|2]` depth 1 or
2. Do not implement it as a shape spec.

## Changes (`crates/rum-stdlib/src/predicates/arrays.rs`)

1. **`VectorQ[expr, test]`** (2-arg): `True` iff `expr` is rank-1 AND
   every element passes `test`. `VectorQ[expr]` byte-identical.
2. **`MatrixQ[expr, test]`** (2-arg): `True` iff `expr` is rank-2
   (rectangular, as today) AND every leaf passes `test`.
   `MatrixQ[expr]` byte-identical.
3. **`ArrayQ` rank pattern + element test**:
   - 2-arg `ArrayQ[expr, patt]`: today only a literal non-negative
     integer is accepted (`expr_as_non_negative_usize`, else inert).
     Change to MATCH the integer depth against `patt` via the kernel's
     pattern-match (`MatchQ`-equivalent) semantics. `ArrayQ[m, 2]` must
     stay byte-identical (integer `2` matches depth `2`); `ArrayQ[m, _]`
     → any depth; `ArrayQ[m, 1|2]` → depth 1 or 2.
   - 3-arg `ArrayQ[expr, patt, test]`: depth matches `patt` AND every
     leaf passes `test`. Extend the arity to 1..=3.
   - `ArrayQ[expr]` (1-arg) byte-identical.

Non-array first argument keeps today's behavior (`False` for
ArrayQ/1-arg-shape checks; the test forms return `False` when the
shape condition already fails, WITHOUT applying `test`). A first
argument that is genuinely symbolic (not a list) stays as today.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{VectorQ[{1, 2}, IntegerQ], VectorQ[{1, a}, IntegerQ], MatrixQ[{{1, 2}, {3, 4}}, IntegerQ], MatrixQ[{{1, 2}, {3, a}}, IntegerQ], ArrayQ[{{1, 2}, {3, 4}}, _], ArrayQ[{{1, 2}, {3, 4}}, 1 | 2], ArrayQ[{{1, 2}, {3, 4}}, 3], ArrayQ[{{1, 2}, {3, 4}}, 2, IntegerQ], ArrayQ[{{1, 2}, {3, a}}, 2, IntegerQ]}'
# defaults byte-identical:
printf '' | cargo run -q -p rum-cli -- --eval '{VectorQ[{1, 2}], VectorQ[{{1}, {2}}], MatrixQ[{{1, 2}, {3, 4}}], MatrixQ[{1, 2}], ArrayQ[{{1, 2}}], ArrayQ[{{1, 2}}, 2], ArrayQ[{{1, 2}}, 1], ArrayQ[x]}'
git status --short
```

Expected: fmt clean; workspace green (report totals); probe 1 →
`{True, False, True, False, True, True, False, True, False}`; probe 2
byte-identical to today (paste — the depth-integer forms and bare
forms must not move).

## Scope

- MAY modify: `crates/rum-stdlib/src/predicates/arrays.rs`,
  `crates/rum-stdlib/src/predicates.rs` (only if a shared helper is
  needed), `crates/rum-stdlib/src/catalog/predicates.rs`
  (descriptions), `crates/rum-stdlib/src/docgen.rs` (LIMITS),
  `crates/rum-stdlib/src/tests/**`, `crates/rum-stdlib/src/tests.rs`,
  `docs/reference/builtins.md` (regenerated if descriptions changed).
- MUST NOT touch: the operator-form matcher, other predicate files
  beyond a shared helper, other families, the guide, `TODO.md`, the
  tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`.
- Every currently-accepted call byte-identical (especially the
  integer-rank `ArrayQ[m, 2]` path); changes only give meaning to
  currently-rejected documented forms.
- `test[leaf]` must be strict `TrueQ` — a `test` that returns anything
  but literal `True` fails the predicate for that leaf.
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per file; how the `ArrayQ` depth-pattern match is
   wired (which kernel helper); the leaf-test application path.
2. Verification — all commands verbatim.
3. Stop-loss / open questions (empty if none).
