# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Pre-1.0,
minor versions may include breaking changes; the project aims to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it reaches 1.0.

## [Unreleased]

### Changed

- Sorting with a user comparator is `n log n`. `Sort`, `SortBy` and
  `ReverseSort` built a full all-pairs table — 639 200 comparisons at n=800 —
  and now run on an iterative, stable, resumable merge engine that stays within
  `n*ceil(log2 n)`. Ties keep their input order.
- The ordering-callback schedule is documented as unspecified: a valid strict
  weak order determines the result, while comparison count, order and direction
  are not promised. Key functions still run exactly once per entry, in input
  order.
- Arithmetic, composition and variadic chains of three or more operands lower
  to one n-ary call rather than nested binary ones: `a + b - c + d` is
  `Plus[a, b, Times[-1, c], d]`, and `.`, `<>`, `|` and `~~` flatten likewise.
  Explicitly written canonical calls stay opaque, so `Plus[a,b] + c` remains
  `Plus[Plus[a, b], c]`.
- Left-associative operator chains are bounded at 4 096 links, reported with a
  dedicated diagnostic. The limit is enforced for programmatically built syntax
  trees as well as for parsed source.

- Completed the P0 tranche of a safe-Rust repository-wide algorithmic
  performance sweep. Wide
  evaluation, matching, replacement, collection, association, string, and
  polynomial paths now avoid numerous repeated scans, suffix copies, and
  per-step continuation clones; small-input paths retain adaptive linear tiers.
- Ordered, non-`Flat` multi-argument matching now advances all-`Single`
  pattern/value cursors in place instead of rebuilding the remaining vectors
  at every argument, including across suspended matcher conditions.
- Incremental `AppendTo`, association mutation, and positional `Part`
  assignment now use conservative in-place/COW fast paths when ownership and
  evaluator-cache invariants permit it.
- The core-language microbench measures elapsed time with the kernel's
  monotonic Rust `Instant` clock rather than differencing the host wall clock.
- Native ZeroMQ tests skip when the environment denies loopback access instead
  of requiring an out-of-sandbox rerun.
- Loading a script is linear in its length. The splitter re-parsed the whole
  accumulated text once per line while a form stayed unterminated, so a
  10 000-line data literal took 7.75 s and the same data on one line took
  0.01 s; it now lexes once and drives the parser form by form, taking 0.01 s.
  Each form also carries the syntax tree that parse produced, so evaluation no
  longer parses it a second time.
- A further round of complexity reductions on wide inputs, each measured
  before and after: `ToToml` and `FromToml` no longer rescan or rebuild per
  entry; `StringReplace`, `StringCases` and `StringApply` no longer copy their
  remaining input and accumulated output per element; `KeyTake`, `KeyDrop` and
  `KeyDropFrom` index their key membership tests; `ReplacePart` merges exact
  positional rules into a single pass; `Together` merges addends pairwise
  instead of renormalizing after every one; `Reap` indexes its tag buckets; and
  UpValue dispatch discovers candidates only at the positions where a stored
  tag can live, which is what made installing many UpValues on one symbol
  quadratic.

### Fixed

- `While`, `For`, `Until` and `Module` bindings with initializers no longer
  consume a native stack frame per iteration or binding. A 20 000-iteration
  loop terminated the process; loops now run iteratively and an unbounded loop
  reaches an existing deadline or interrupt instead of aborting.
- Sixteen left-associative operators no longer exhaust the native stack on long
  chains. The parser's recursion guard never saw them, because such chains are
  parsed iteratively and the depth was deferred into the syntax tree.
- `FromYaml` and `FromToml` refuse over-deep documents with a message instead
  of exhausting the native stack, matching `FromJson`. All three accept 256
  nested collections and refuse the 257th.
- `KeySort` and `KeySortBy` with a custom ordering no longer overflow the stack
  — previously at 110 and 100 keys respectively — and no longer clone the whole
  sort state per comparison.
- `StringPadLeft`, `StringPadRight`, `DecimalString`, numeric `StringIndent`,
  `StringExpandTabs` and `Fibonacci` now charge their work against the existing
  resource budgets, as their siblings already did.
- Multi-iterator `Table`, `Do`, `Sum` and `Product` charge the product of their
  iterator counts, so iterators that each pass the limit can no longer multiply
  past it. Counts that come from a variable or from an enclosing iterator are
  included; constructs that discard their inner results are not charged for
  sequential repetition.
- Comparison-chain lowering no longer recurses, so long comparison chains no
  longer overflow the stack while being lowered.

- Added the previously missing `list[[i]] = value` assignment semantics.
- Bounded recursive operator parsing and made uninitialized `Module` binding
  processing iterative. The parser's single depth limit is now three separate
  caps — delimiter nesting, operator chains, and `;`-separated statements —
  because one number could not serve all three: statement count is not nesting
  depth. `;` chains are collected iteratively in both the parser and lowering,
  and a form holding too many statements reports its own error naming the limit
  and suggesting the fix (splitting the line), rather than a nesting message.
- Wide inputs no longer abort the process. `Replace`, `ReplaceAll`, `Cases`,
  `Count`, `Position`, `DeleteCases`, `ReplaceList`, `Reap`, `StringReplace`,
  `StringCases`, `StringApply`, and `UpValue`/`SubValue` dispatch each walked
  their input by recursing once per element, rule, or definition, so a flat list
  of a few thousand entries overflowed the native stack. All of them now
  iterate, with suspension and resume preserved; the boundary is the documented
  resource limit, which refuses gracefully.
- Building an association one key at a time no longer falls off a performance
  cliff. Writes stability-checked a freshly built rule on every store, so the
  cache filled with single-use entries; crossing its cap wiped the proof the
  direct write path needs, and the slow path never re-seeded it, disabling the
  fast path permanently. Twenty thousand keys took 55 seconds and thirty-two
  thousand took over eleven minutes.
- Replaced wide symbolic `Plus`/`Times` like-term linear scans with an adaptive
  group index; the former unconditional quadratic wide path is no longer
  current.

## [0.1.0] — 2026-07-21

Initial public source release. `rum` is an embeddable, sandbox-first symbolic
term-rewriting language and runtime written in Rust.

### Language and runtime
- Symbolic expression trees with exact rational arithmetic (no floating point).
- Definitions, rules, and pattern matching, including Orderless/Flat/OneIdentity
  and optional/default patterns.
- Scoping and held evaluation, sealed packages (real, kernel-enforced
  encapsulation), lazy sequences, and structured exceptions.
- A folk-scope standard library of 642 builtins; see
  [`docs/reference/builtins.md`](./docs/reference/builtins.md).

### Embedding and sandbox
- `rum-host`: a Rust embedding facade with structured outcomes, resumable input,
  a bidirectional typed value bridge, and named capability profiles
  (`sandboxed` / `hermetic` / `scripting`).
- Per-session capability policy, always-on resource limits, opt-in request
  memory ceiling and monotonic elapsed-time timeout, crash-only per-request isolation,
  interrupt/cancel, reset, and snapshot/restore. See [`SECURITY.md`](./SECURITY.md).

### Tooling
- `rum-cli` and `rum-repl` terminal front ends, and an optional `rum-jupyter`
  adapter.

[0.1.0]: the initial public snapshot; this project has no prior public release.
