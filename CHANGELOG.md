# Changelog

All notable changes to this project are documented in this file. The format is
based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Pre-1.0,
minor versions may include breaking changes; the project aims to follow
[Semantic Versioning](https://semver.org/spec/v2.0.0.html) once it reaches 1.0.

## [Unreleased]

### Changed

- Completed the P0 tranche of a safe-Rust repository-wide algorithmic
  performance sweep. Wide
  evaluation, matching, replacement, collection, association, string, and
  polynomial paths now avoid numerous repeated scans, suffix copies, and
  per-step continuation clones; small-input paths retain adaptive linear tiers.
- Incremental `AppendTo`, association mutation, and positional `Part`
  assignment now use conservative in-place/COW fast paths when ownership and
  evaluator-cache invariants permit it.
- The core-language microbench measures elapsed time with the kernel's
  monotonic Rust `Instant` clock rather than differencing the host wall clock.
- Native ZeroMQ tests skip when the environment denies loopback access instead
  of requiring an out-of-sandbox rerun.

### Fixed

- Added the previously missing `list[[i]] = value` assignment semantics.
- Bounded recursive operator parsing and made uninitialized `Module` binding
  processing iterative.
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
