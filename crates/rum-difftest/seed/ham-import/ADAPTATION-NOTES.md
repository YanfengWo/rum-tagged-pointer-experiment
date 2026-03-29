# Ham Seed Adaptation Notes

This note explains how to reuse the imported `ham-difftest` assets when
building `rum-difftest`.

## What Is Directly Useful

### 1. Golden file structure

The imported `ham` golden format is a good starting point for `rum`:

- normal cases:
  - `input ||| expected`
- known differences:
  - `input !!! id=... | type=... | ham=... | wolfram=... | reason=...`

`rum` should adopt the same idea, even if field names or rendering details are
adjusted.

### 2. Wolfram regeneration script concept

[regen.wls](./regen.wls)
shows a practical way to regenerate golden outputs from Mathematica while
special-casing strings, lists, and rationals. The exact formatting rules will
need to be adapted to `rum`, but the workflow is reusable.

### 3. Seed correctness categories

The current imported categories are useful bootstrapping buckets:

- arithmetic
- control
- definitions
- evaluation_control
- lists
- ordering
- patterns
- scoping
- strings

These are especially useful for a first single-expression oracle lane.

### 4. Seed performance suites

The imported timing and memory suites are small but useful as starter corpora:

- timing: 20 expressions
- memory: 13 expressions

They should be treated as baseline inputs, not as final policy.

## What Is Not Directly Transferable

### 1. Output formatting

The imported expected outputs reflect `ham`'s current rendering conventions.
They are not automatically correct for `rum`.

Likely rewrite points:

- string quoting / display conventions
- list rendering details
- message formatting
- whitespace normalization policy

### 2. Coverage claims

The imported corpus is useful, but it does not cover the whole `rum` scope.

Known weak or missing areas:

- associations
- richer functional/programming surfaces
- package loading workflows
- host-facing API behavior
- protocol-level adapter behavior
- multi-step session semantics beyond single-line `;` chains

### 3. Host interaction

`ham-difftest` is primarily expression-oriented. `rum` has stronger host-facing
ambitions, especially around runtime sessions and future Rust/Python adapters.

That means:

- imported `ham` seed files are suitable for kernel semantics
- they are not sufficient for host contract testing

## Required Split For `rum`

When adapting this seed pack, keep two different test families:

### Wolfram diff

Use imported seeds here for:

- expression results
- message parity where implemented
- printed output parity where a stable rendering exists
- stateful session scripts once `rum` adds that fixture format

### `rum`-native contract tests

Do not force these into Wolfram diff:

- Rust host adapter semantics
- Python host adapter semantics
- runtime suspension and resume payloads
- Jupyter transport behavior
- any `rum`-specific embedding boundary

These should become `rum`-native regression/contract suites.

## First Curation Pass Recommendation

The first `rum`-side curation pass should:

1. preserve all imported files unchanged under `seed/ham-import/`
2. copy only clearly relevant cases into future `cases/wolfram/core/`
3. mark mismatches as `rum` divergences instead of editing the seed files
4. add a new session-script lane for multi-step semantics

## Suggested First Migration Order

1. `arithmetic.txt`
2. `patterns.txt`
3. `definitions.txt`
4. `scoping.txt`
5. `ordering.txt`
6. `lists.txt`
7. `evaluation_control.txt`
8. `strings.txt`
9. `control.txt`

Reason:

- this order follows the most semantics-sensitive kernel areas first
- it aligns well with `rum`'s current compatibility priorities
