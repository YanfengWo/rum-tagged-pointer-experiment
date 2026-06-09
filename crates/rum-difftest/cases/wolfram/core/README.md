# Wolfram Core Cases

This directory now holds the first `rum`-owned Lane A curated fixtures.

These files intentionally use the same first-pass golden format as the imported
seed pack:

- `input ||| expected_output`
- `input !!! id=name | type=... | reason=...`

That keeps them compatible with the current `lane-a` runner without inventing a
second fixture grammar too early.

You can run one of these files directly with:

```bash
cargo run -p rum-difftest -- lane-a \
  --oracle-kernel /path/to/WolframKernel \
  --fixture crates/rum-difftest/cases/wolfram/core/smoke_arithmetic.txt
```

Current curated files are meant to be small, stable suites that are easy to
inspect and extend. They should prefer:

- same-name Wolfram semantics
- single-expression checks
- low-surprise formatting
- compact coverage over exhaustive duplication of the imported seed corpus

The current first pass includes:

- small cross-domain smoke suites
- a focused exact-arithmetic suite covering negative integer division
  semantics, rational powers, and modular exponentiation
- a small polynomial/rational-expression suite covering renderer-supported
  `Expand` / `Together` outputs that currently avoid broader term-ordering
  differences
- association and keyed-data smoke coverage
- a definition-precedence suite promoted from resolved imported seed coverage
- dedicated loop/procedural and control-object suites
- a structure selector suite for stable `Part` / `Extract` / `ReplacePart` /
  `Depth` / `Level` coverage
- a threading suite for `Thread`, `MapThread`, and current listable-dispatch
  coverage
- a focused functional-combinator suite covering current CL-04 operator and iteration subsets
- a focused rules/patterns/replacement/traversal suite for Lane A stateless
  conformance
- a failure/status suite covering stable CL-06 confirmation and status-object checks
- focused string-language suites for surface operations, string-patterns,
  templating, and similarity helpers
