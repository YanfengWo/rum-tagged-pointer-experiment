# Flat + Orderless + Optional matcher contract

This document records rum's own candidate contract for matching `Optional`
arguments under a `Flat` + `Orderless` head. It replaces a WK-derived
expectation retired as a testing-policy fossil by
[Brief 276](codex/brief-276-flat-optional-respec.md) on 2026-07-20.

## Status

**The head-wrapped single-element candidate specified below is NOT
implemented.** Only the bare form is tried today. The specification is
recorded as the intended contract for a future attempt; Brief 276's
implementation half stopped before any source edit (the signature
threading reached beyond that brief's scope). Current behavior is pinned
by `flat_orderless_optional_matches_collects_and_omits` and
`flat_orderless_optional_with_sequence_neighbors` in
`crates/rum-runtime/src/runtime/tests/pattern_matching_orderless_optional.rs`
— for example, `ReplaceList[g[1, 2], g[a_, b_:0] :> {a, b}]` yields
`{{2, 1}, {1, 2}, {g[1, 2], 0}}`, with no wrapped rows.

## Candidate contract (specification, not current behavior)

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

## Enumeration order

rum does not target WK's enumeration order. It guarantees only deterministic
ordering under rum's own matcher contract; enumeration-order conformance with
WK is a documented WON'T-FIX.
