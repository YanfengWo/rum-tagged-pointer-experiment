# Difftest Coverage Goals

This directory holds executable coverage targets for `rum-difftest`.

`inventory-baseline.txt` is the checked minimum coverage floor for Lane A/B
fixtures. It is intentionally not just a captured snapshot: source records
guard the imported-vs-curated fixture mix, and category records guard the
high-risk semantic areas that should not disappear during fixture refactors.

The current category records cover these priorities:

- Lane A one-shot same-name semantics: arithmetic, lists and list-surface
  transforms, ordering, scoping,
  scoping/closure behavior, definitions and definition precedence, patterns,
  rules/traversal, structure selectors, threading, context/namespace behavior,
  associations, functional combinators, control objects, messages/attributes,
  option handling, failure/status objects, and string-language subsets.
- Lane B stateful semantics: definitions/messages, message switches and
  message-list state, mutation/update state, option-default mutation and
  option-value scopes, package/default-value behavior, scoped control-flow
  restoration, and fixture-boundary isolation for the multi-batch oracle
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

For focused audits while adding or pruning one fixture family, use the
inventory filters instead of visually scanning the full report:

```bash
cargo run -p rum-difftest -- inventory --lane lane-a --source curated-core --category threading
```

These filters also apply to `--summary-json`, `--baseline`, and
`--write-baseline`. Focused baseline comparisons only consider checked
baseline records that match the active filter scope, and fail fast if no
records match. Filtered `--write-baseline` output includes comment lines
recording the active filters. Keep the checked repository baseline generated
from the unfiltered inventory unless an intentionally scoped baseline artifact
is being created for a local review.
