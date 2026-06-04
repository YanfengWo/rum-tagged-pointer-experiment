# Rum Difftest Handoff

This file is meant to be enough for a fresh agent to restart `rum-difftest`
work without reading older chat history.

## Goal

Build a `rum-difftest` workstream that treats same-name Wolfram features as
compatibility targets by default, while keeping `rum`'s own host/runtime/API
contracts separate from Wolfram-specific transport and embedding behavior.

The oracle side of this workstream is optional and user-owned. `rum` should
not assume a bundled Wolfram license, a project-managed oracle install, or a
single pinned Wolfram version. Instead, when a contributor has access to a
usable local Mathematica/Wolfram environment, `rum-difftest` should help them
drive that environment as an oracle.

## Core Principle

`rum-difftest` should not be one monolithic test bucket. It should be split
into separate lanes with different oracles.

For the Wolfram-backed lanes, keep these policy constraints explicit:

- the oracle command should be user-configured and may point to
  `wolframscript`, `WolframKernel`, `MathKernel`, or a thin wrapper script
- the license belongs to the user running the diff, not to the project
- generated oracle outputs are transient artifacts and should not be committed
- every diff run should regenerate fresh comparison data
- version variance is acceptable and useful because it also probes
  compatibility with users' existing Wolfram environments

And keep the exclusion boundary explicit too:

- `rum-difftest` is for oracle-comparable same-name semantics
- it is not the main home for `rum`-specific runtime/session features
- it is not the main home for suspend/resume, structured diagnostics, host
  APIs, or adapter/protocol tests
- those tests should stay in the crate-local suites that own those contracts

Current implementation status:

- a first-pass `lane-a` runner now exists
- it accepts `--oracle-kernel PATH` or `--config PATH`
- config currently supports `oracle_kernel = "/path/to/WolframKernel"`
- it drives a local kernel executable directly and stores generated oracle
  inputs/wrappers only in a temporary run directory
- it does not yet claim full message/output/top-level display parity
- a first pass of `rum`-owned curated Lane A files now exists under
  [cases/wolfram/core](./cases/wolfram/core/README.md)
- that curated set currently covers arithmetic/structures, loops/control,
  associations, functional combinators, failure/status, and multiple string
  subfamilies
- the current `lane-a` comparison is still display-sensitive, so curated files
  should prefer scalar/list/plain-call results over outputs whose main
  difference is just `rum` canonical form vs Wolfram `InputForm`

### Lane A: Single-Expression Wolfram Diff

Use a Wolfram oracle for one-shot expression checks:

- input text
- kernel-visible result
- optional messages
- optional printed output

This lane should start from the imported `ham` golden files under
[seed/ham-import/golden](./seed/ham-import/golden).

Priority areas:

- arithmetic normalization and exact numbers
- rules, patterns, traversal, replacement
- held evaluation and scoping
- definition precedence and messages
- strings once the string subsystem is mature enough

### Lane B: Stateful Session Diff

Some kernel semantics are about a sequence of evaluations, not one input line.
This lane should cover:

- assignment / redefinition order
- context evolution
- `Begin` / `End` / `BeginPackage` / `Needs`
- iterator leakage and scoping restoration
- message accumulation and clearing

Recommended format:

```text
# @name: specific-value-overrides-general-rule
>>> f[1] = 10
<<< 10
>>> f[x_] := x + 1
<<< x + 1
>>> f[1]
<<< 10
```

### Lane C: Known-Difference Ledger

Do not hide mismatches in comments. Track them as data.

The imported `ham` fixture format is a good seed:

```text
input !!! id=name | type=temporary | rum=... | wolfram=... | reason=...
```

For `rum`, this should become the divergence ledger for same-name features.
Current implementation has `divergences/known-differences.txt` plus
`rum-difftest lane-c` to validate fixture IDs against the ledger.

### Lane D: Performance Regression

Timing and memory suites should exist, but they are not primarily Wolfram
compatibility checks. Their primary oracle is `rum`'s own baseline.

The imported `ham` timing and memory suites under
[seed/ham-import/bench](./seed/ham-import/bench)
and
[seed/ham-import/mem](./seed/ham-import/mem)
are seed inputs, not final `rum` policy. Current implementation has
`rum-difftest lane-d`, which executes those seed inputs locally and reports raw
metrics. Its default `smoke` profile skips imported heavy / very-large
categories for daily validation; use `--profile full` for explicit all-input
baseline collection. It does not yet store historical baseline artifacts or
enforce thresholds.

### Lane E: Host Contract Tests

Rust/Python host adapters and runtime embedding boundaries are important, but
they are not Wolfram-diff targets.

These should be tested against `rum`'s own public contract:

- session request/result types
- structured output/messages
- suspension and resume
- host callbacks / input requests
- Rust and Python adapter semantics

### Lane F: Adapter / Protocol Tests

Jupyter and other protocols should be tested against protocol specs and `rum`'s
own regression expectations, not against Wolfram transport behavior.

## What Should Not Be Forced Into Wolfram Diff

Do not use Wolfram as the oracle for:

- Rust host API shape
- Python host API shape
- suspension payload formats
- Jupyter framing details
- any `rum`-specific host capability mechanism
- any `rum`-specific kernel extension that does not have a meaningful
  Wolfram-side oracle

The right split is:

- Wolfram diff for kernel semantics
- contract tests for `rum`'s own host and protocol surfaces

## Imported Seed Pack

The imported `ham` assets are documented in:

- [seed/ham-import/MANIFEST.md](./seed/ham-import/MANIFEST.md)
- [seed/ham-import/ADAPTATION-NOTES.md](./seed/ham-import/ADAPTATION-NOTES.md)

Raw counts:

- 9 golden correctness categories
- 166 normal correctness cases
- 3 known-difference cases
- 20 timing expressions
- 13 memory expressions

## Immediate Work Queue

1. Keep the imported `ham` corpus intact under `seed/`.
2. Continue extending the curated `cases/wolfram/core/` tree page by page from
   the matrix-guided kernel priorities.
3. Normalize output conventions further so Lane A can compare more than today's
   scalar/list/plain-call-friendly subset.
4. Continue extending Lane A/B curated fixtures from matrix-guided priorities.
5. Add richer Lane C metadata links and close criteria for each divergence.
6. Add Lane D historical baseline artifacts and threshold policy.
7. Keep Lane E/F boundaries explicit: host-contract and adapter/protocol tests
   stay in their owning crates rather than becoming Wolfram diffs.

## If This Folder Was Copied Standalone

If the receiving checkout only copied `difftest/` and not the accompanying
docs or TODO edits from another environment, that is fine. The next agent
should treat this folder as the canonical handoff package and recreate any
repository-level TODO or documentation updates locally as needed.
