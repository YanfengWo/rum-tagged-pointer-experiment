# Architecture

## Mission

`rum` is a Rust implementation of a Mathematica / Wolfram-style core language.
The initial target is not a notebook system or a CAS, but a programmable core
language runtime that can eventually power:

- complex rule evaluation
- business and industrial decision logic
- host-embedded DSLs
- interactive scripting through a REPL, with a later path to Jupyter

## Scope

### In scope

- symbolic expression trees
- exact integer atoms through `BigInt`
- parser and core syntax model
- evaluator and rule engine
- symbol definitions and attributes
- scoping and contexts
- embeddable runtime sessions
- REPL and future kernel adapters

### Out of scope for the early phases

- notebooks and GUI tooling
- graphics
- advanced algebra and numerical systems
- front-end compatibility layers

## Layering

The workspace is intentionally split by responsibility.

### `rum-syntax`

Owns source-facing syntax:

- parser inputs and errors
- syntax expression tree
- text-oriented concerns such as strings, symbol tokens, and call syntax

This crate should stay focused on source structure rather than evaluation.

### `rum-core`

Owns runtime-neutral core values:

- `Expr`
- `Atom`
- `Symbol`
- numeric atom representation

The most important design decision already in place is that exact integers are
first-class runtime values, not an afterthought.

### `rum-runtime`

Owns stateful execution:

- lowering from syntax into core values
- evaluation sessions
- builtin registration
- symbol inspection
- later: rule dispatch, attributes, tracing, resource limits, contexts, and
  scoping semantics

### `rum-stdlib`

Owns builtin installation for the core language. This crate should register
names and later provide the executable implementations for core builtins such
as `Set`, `SetDelayed`, `MatchQ`, `ReplaceAll`, `Module`, and `Hold*`.

### `rum-repl`

Owns interactive shell behavior. It should stay thin and work through stable
runtime APIs instead of reaching into evaluator internals directly.

That separation is important because the same session model should be reusable
for:

- terminal REPL use
- tests and golden files
- host embedding
- a future Jupyter kernel adapter

### `rum-cli`

Owns process-level concerns:

- argument parsing
- file execution
- REPL startup
- process exit codes

## Execution model direction

The bootstrap code only implements a tiny echo-style evaluator, but the target
architecture is:

1. parse source into syntax expressions
2. lower syntax into runtime `Expr` values
3. evaluate according to symbol definitions and attributes
4. apply rewrite rules and scoping semantics
5. expose traceable results to CLI, REPL, or host APIs

The current implemented evaluation sequence and the remaining semantic gaps are
tracked in `docs/runtime-evaluation-audit.md`.

## Numeric model

Early support for exact integers is required because it matters for both
language fidelity and practical rule/DSL scenarios such as:

- identifiers that exceed machine integer width
- exact arithmetic in financial or operational logic
- deterministic serialization and comparison

The current bootstrap uses `num_bigint::BigInt` in `rum-core::Atom::Integer`.
This should remain the foundation as the numeric tower expands later.

## REPL and Jupyter boundary

The REPL is treated as a first-class feature from the beginning. The project
should not assume that interactivity is a thin afterthought over a batch-only
runtime.

That means runtime APIs should evolve toward session-oriented operations such
as:

- parse
- evaluate
- inspect symbol
- emit diagnostics
- optionally expose execution traces

When a Jupyter adapter is added later, it should live in a separate crate and
translate protocol messages into those session-oriented runtime calls.

## Near-term roadmap

1. expand the parser to cover operators, rules, patterns, and placeholders
2. add proper symbol tables and definition storage
3. implement pattern matching and rule application
4. add evaluation control and scoping constructs
5. improve the REPL with history, completion, and inspection
