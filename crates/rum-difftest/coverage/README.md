# Difftest Coverage Goals

This directory holds executable coverage targets for `rum-difftest`.

`inventory-baseline.txt` is the checked minimum coverage floor for Lane A/B
fixtures. It is intentionally not just a captured snapshot: source records
guard the imported-vs-curated fixture mix, and category records guard the
high-risk semantic areas that should not disappear during fixture refactors.

The current category records cover these priorities:

- Lane A one-shot same-name semantics: arithmetic, lists, ordering, scoping,
  definitions, patterns, rules/traversal, associations, functional
  combinators, control objects, failure/status objects, and string-language
  subsets.
- Lane B stateful semantics: definitions/messages, package/default-value
  behavior, and fixture-boundary isolation for the multi-batch oracle
  transport.

Use the baseline as a regression gate, not as a claim of exhaustive parity:

```bash
cargo run -p rum-difftest -- inventory --check-baseline --fail-on-regression
cargo run -p rum-difftest -- check
```

When fixture additions or reshuffles are deliberate, generate a candidate
baseline with:

```bash
cargo run -p rum-difftest -- inventory --write-baseline /tmp/rum-inventory-baseline.txt
```

Review the semantic category deltas before updating the checked baseline. Do
not lower category minima just to make a local deletion pass; either replace
the lost coverage or document why that semantic target is no longer active.
