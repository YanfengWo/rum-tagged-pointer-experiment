# Brief 190 — Commonest[list,n] + Quantile[list,{q..}] (unit-5 fix 4a)

Implementation brief. Executes part of unit-5 triage fix-batch "cheap
flexibility" (tracker `docs/dev/usability-pass.md`, unit-5 section;
evidence E5 in `docs/dev/usability-pass/math-findings.md`). Two
WL-documented extra-argument forms for stats aggregates, both
folk-cheap (sort / map over the existing scalar path). No new algorithm,
no attributes, no special values. Default 1-/2-arg forms stay
byte-identical.

Both changes are in `crates/rum-stdlib/src/arithmetic/stats.rs`.

## Change 1 — `Commonest[list, n]`

`eval_commonest` (`stats.rs:38`) currently accepts ONE argument and
returns every element tied for the maximum count. WL also documents
`Commonest[list, n]` = the `n` elements with the largest number of
occurrences, most-frequent first.

- Accept a second argument `n` (arity 1..2; keep the existing arity
  message for other counts). When `n` is a concrete NON-NEGATIVE exact
  integer: sort the existing `counts` frequency classes by count
  DESCENDING with a STABLE sort (so equal counts keep first-appearance
  order — `counts` is already built in first-appearance order), take the
  first `min(n, counts.len())`, and return them as a `List`.
  `Commonest[list, 0]` → `{}`.
- A non-integer / negative / symbolic `n` leaves the call inert
  (unchanged fallback) — do NOT invent a message.
- The one-argument form is UNCHANGED (all max-count elements, same
  order).

Acceptance: `Commonest[{a, b, a, c, b, a}, 2]` → `{a, b}` (a:3, b:2);
`Commonest[{a, b, a, c, b, a}, 1]` → `{a}`;
`Commonest[{a, b, a, c, b, a}, 10]` → `{a, b, c}`;
`Commonest[{a, b, a, c, b, a}, 0]` → `{}`;
`Commonest[{a, b, a, c, b, a}]` → `{a}` (one-arg unchanged).

## Change 2 — `Quantile[list, {q1, q2, ...}]`

`eval_quantile` (`stats.rs:70`) currently takes a SCALAR `q`. WL also
documents a LIST of quantile parameters: `Quantile[list, {q1, ...}]` =
`{Quantile[list, q1], ...}`.

- When the second argument is a `List`, map the EXISTING scalar quantile
  computation (the `nearest_rank_index` nearest-rank selection) over each
  element and return a `List` of the results. Each `qi` is validated
  exactly as the scalar path does (exact rational in `[0, 1]`); if ANY
  `qi` is out of range or not an exact rational, fail closed with the
  SAME `Quantile::range` message and return the inert call (do not return
  a partial list). The empty-input-list check (`Quantile::empty`) stays
  as today.
- A scalar second argument is UNCHANGED.

Acceptance: `Quantile[{1, 2, 3}, {0, 1/2, 1}]` → `{1, 2, 3}`;
`Quantile[{1, 2, 3}, 1/2]` → `2` (scalar unchanged);
`Quantile[{1, 2, 3}, {1/2, 5}]` → `Quantile::range` + inert;
`Quantile[{}, {1/2}]` → `Quantile::empty` + inert.

## Tests

Add tests in `crates/rum-stdlib/src/tests/exact_reduction_shapes.rs`
(where the Commonest/Quantile/Median tests live):
- Commonest: the five acceptance rows above.
- Quantile: the four acceptance rows above.
- Keep the existing Commonest/Quantile/Median tests passing unchanged.

## Verification (MANDATORY — run all, paste verbatim)

```
cargo fmt --check
cargo test --workspace 2>&1 | grep -E '^test result' | sort | uniq -c
printf '' | cargo run -q -p rum-cli -- --eval '{Commonest[{a,b,a,c,b,a},2], Commonest[{a,b,a,c,b,a},1], Commonest[{a,b,a,c,b,a},10], Commonest[{a,b,a,c,b,a},0], Commonest[{a,b,a,c,b,a}], Quantile[{1,2,3},{0,1/2,1}], Quantile[{1,2,3},1/2], Quantile[{1,2,3},{1/2,5}]}'
git status --short
```

Expected: fmt clean; workspace green (report totals — new tests added);
`Commonest` rows → `{a,b}, {a}, {a,b,c}, {}, {a}`; `Quantile` rows →
`{1,2,3}, 2`, and `Quantile[{1,2,3},{1/2,5}]` prints `Quantile::range`
and stays inert.

## Scope

- MAY modify: `crates/rum-stdlib/src/arithmetic/stats.rs` (Commonest +
  Quantile), and `crates/rum-stdlib/src/tests/exact_reduction_shapes.rs`
  (new tests).
- MUST NOT touch: `Median`, `Mean`, `nearest_rank_index` internals
  (reuse as-is), any other builtin; `Total`/`PowerMod` (separate
  briefs); the guide, `docs/reference/builtins.md`, `TODO.md`, the
  tracker, findings files, `Cargo.toml` files.

## Hard rules

- Do NOT `git commit`, branch, push. No network, no new dependencies,
  no new `unsafe`. Preserve any suspend/resume shape you touch.
- Compat: the 1-arg `Commonest` and the scalar `Quantile` forms are
  byte-identical to today. New forms are folk-cheap (sort / map over the
  existing path) — no new algorithm.
- No A/B gate needed (not an eval/dispatch/matcher hot path).
- Minimal diff. Stop-loss: same failure twice → STOP and report.
- Paste real verbatim verification output; never claim green without it.

## Report format

1. What changed — per function, line ranges.
2. Verification — all commands verbatim (test totals + the probe +
   `git status --short`).
3. Stop-loss / open questions (empty if none).
