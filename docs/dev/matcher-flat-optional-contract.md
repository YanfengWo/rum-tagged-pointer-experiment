# Flat + Orderless + Optional matcher contract

This document records rum's own candidate contract for matching `Optional`
arguments under a `Flat` + `Orderless` head. It replaces a WK-derived
expectation retired as a testing-policy fossil by
[Brief 276](codex/brief-276-flat-optional-respec.md) on 2026-07-20.

## Status

**The head-wrapped single-element candidate specified below is implemented
as of Brief 279.** Current behavior is pinned by
`flat_orderless_optional_matches_collects_and_omits` and
`flat_orderless_optional_with_sequence_neighbors` in
`crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`.

## Candidate contract

For a `Flat` pattern head without `OneIdentity`, a plain `Blank`-style
pattern argument (`MatcherArg::Single`) matched against a grouped selection
of exactly one value has two candidate forms, in this order:

1. the bare element;
2. the element wrapped in the flat head, such as `g[1]` for the bare element
   `1`.

The wrapped candidate is tried immediately after the bare candidate and
before any grouping that consumes more elements. This order applies to both
plain `Flat` matching and `Flat` + `Orderless` candidate generation. Heads
with `OneIdentity` retain the bare-only path.

For first-match operations such as `MatchQ` and `/.`, a successful bare
candidate and the rest of the pattern win before the wrapped candidate is
considered. For collect-all operations such as `ReplaceList`, successful
bare and wrapped matches are distinct rows, with the wrapped row immediately
following its bare row before the next consume length is tried.

## Optional omission is collected in full (all `Flat` heads)

Brief 279 also repaired a separate, pre-existing defect that the work above
exposed: the `Flat` collect-all path's `Optional`-omission arm was
first-match-shaped, so it kept at most ONE omission match and silently
dropped the rest.

Unlike the wrapped-candidate contract above, this repair is **not** gated on
`OneIdentity` — it applies to every `Flat` head, **including the built-ins
`Plus` and `Times`**, which therefore recover previously dropped rows:

```
ReplaceList[a + b, Plus[x_, y___, z_:0] :> {x, {y}, z}]
  before:  {{b, {}, a}, {a, {}, b}, {a, {b}, 0}}
  after:   … additionally {b, {a}, 0} and {Plus[a, b], {}, 0}
```

The tell that the old behavior was wrong: under `Orderless` it collected
`{a, {b}, 0}` but not its mirror `{b, {a}, 0}`. No first-match (`MatchQ`,
`/.`) result changes; rows are only ever added. Pinned by
`flat_optional_omission_collects_every_match_for_builtin_heads`.

## Candidate-set size is not pruned

A pattern with n single-value slots offers a 2^n bare/wrapped cross product,
so `ReplaceList` row counts grow accordingly on `Flat`-without-`OneIdentity`
heads. This is accepted (user ruling, 2026-07-20): correctness on small
examples is the requirement, efficiency is not a consideration, and WL itself
applies no heuristic pruning here either. Do not add pruning to shrink these
counts.

## Enumeration order

rum does not target WK's enumeration order. It guarantees only deterministic
ordering under rum's own matcher contract; enumeration-order conformance with
WK is a documented WON'T-FIX.
