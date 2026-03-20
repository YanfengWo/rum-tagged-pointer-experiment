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
- text-oriented concerns such as string literals, symbol tokens, and call syntax

This crate should stay focused on source structure rather than evaluation.

### `rum-core`

Owns runtime-neutral core values:

- `Expr`
- `Atom`
- `Symbol`
- numeric atom representation
- the canonical string atom representation shared by runtime layers

The most important design decision already in place is that exact integers are
first-class runtime values, not an afterthought.

### `rum-runtime`

Owns stateful execution:

- lowering from syntax into core values
- evaluation sessions
- staged call dispatch and evaluation sequencing
- definition storage and application
- contexts and package loading
- runtime messages, tracing, and resource limits
- symbol inspection
- scoping semantics
- string builtin semantics, matching, and text-processing behavior for the
  implemented subset

This crate now contains the evaluator core rather than only bootstrap scaffolding.

### `rum-terminal`

Owns terminal-facing presentation:

- rendered runtime messages
- rendered request failures and source diagnostics for terminal clients
- rendered inspection and message-catalog text views

This crate is intentionally presentation-only. It should depend on the stable
session/request models exposed by `rum-runtime`, but protocol-facing hosts
should consume those structured payloads directly instead of round-tripping
through terminal text.

### `rum-jupyter`

Owns protocol-adapter concerns for future notebook or kernel integration:

- protocol-neutral execute / inspect / complete request models
- adapter logic that maps those requests onto `Session` request APIs
- kernel-side result shapes such as execution counts, stream/display outputs,
  and structured failures
- a first shell-envelope layer with request ids and minimal lifecycle requests
  such as `kernel_info` and `shutdown`
- MIME-bundle and metadata-oriented result shapes for execute, inspect, and
  trace-style display payloads, without depending on terminal rendering
- a first wire-level split between shell/control replies and `IOPub` traffic,
  with busy/idle status publication, `execute_input`, and minimal interrupt /
  shutdown lifecycle handling
- more Jupyter-like reply-content shapes for execute / inspect / complete /
  kernel-info flows, plus a first `is_complete` request/response path
- an explicit wire-message encoding and transport boundary so future ZeroMQ or
  mock transports do not need to live inside the adapter itself
- first-pass inbound wire decoding, signature checks, session-id validation,
  request-channel validation, and a mock socket transport that models the
  shell / control / `IOPub` split more directly
- symmetric reply / `IOPub` decoding, configurable signature policy, routing-id
  preservation on replies, and a first event-pump layer that can consume wire
  requests and publish wire responses over the mock transport
- richer reply / `IOPub` schema details such as header dates, lifecycle-aware
  `status` / `shutdown` content, and a control-first transport loop that can
  keep draining queued requests until the mock kernel goes idle or enters
  shutdown

This crate is intentionally a placeholder for now. It should stay focused on
protocol and host-adapter concerns, not on evaluation logic or terminal
rendering.

### `rum-stdlib`

Owns builtin installation metadata for the core language. This crate centralizes
the builtin inventory, summaries, attributes, and handler registration used by
the runtime. It should stay thin and declarative rather than turning into a
second evaluator.

For the upcoming string-language workstream, `rum-stdlib` should remain the
place where string builtins are registered and described, while evaluation and
shared algorithms stay in `rum-runtime`.

### `rum-repl`

Owns interactive shell behavior. It should stay thin and work through stable
runtime APIs instead of reaching into evaluator internals directly. Terminal
formatting should come from `rum-terminal` rather than being reimplemented
inside the REPL.

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

Like the REPL, the CLI should keep using request-style runtime APIs for
machine-readable outcomes, then delegate user-facing rendering to
`rum-terminal`.

## Current execution model

The current architecture is no longer a tiny bootstrap evaluator. At a high
level it now does:

1. parse source into syntax expressions
2. lower syntax into runtime `Expr` values
3. evaluate to a fixed point under recursion, cycle, and pass limits
4. run a staged call-dispatch pipeline covering special forms, argument
   preparation, list threading, definitions, builtins, and pure functions
5. expose values, runtime messages, and optional trace data to CLI, REPL, or
   host APIs

The current implemented evaluation sequence and the remaining semantic gaps are
tracked in `docs/runtime-evaluation-audit.md`.

For lowering-specific orientation, use `docs/lowering-canonical-forms.md` for
the surface-syntax-to-core contract and validation boundary, then read
`docs/runtime-evaluation-audit.md` for the later evaluator-side semantics that
still intentionally live beyond lowering.

## Code organization

As the runtime grows, source organization is treated as an architectural concern
rather than cleanup trivia.

The current direction is:

- split code along real Rust module boundaries instead of `include!`-style text
  concatenation
- keep files small enough to review comfortably, with roughly `10k` characters
  as a maintenance target
- prefer boundaries that help both readability and incremental compilation
- avoid over-splitting into tiny files when the compile-time gain is negligible

The detailed rule set for this lives in
`docs/code-organization-guidelines.md`.

## Numeric model

Early support for exact integers is required because it matters for both
language fidelity and practical rule/DSL scenarios such as:

- identifiers that exceed machine integer width
- exact arithmetic in financial or operational logic
- deterministic serialization and comparison

The current runtime uses `num_bigint::BigInt` in `rum-core::Atom::Integer`.
This should remain the foundation as the numeric tower expands later.

## String subsystem direction

Full Wolfram-style string functionality is important to the project's intended
commercial, industrial, and DSL-oriented use cases, so it is now treated as a
planned core-language workstream rather than a peripheral convenience feature.

The intended first implementation wave is:

- keep string literal parsing, escaping, and source-facing syntax in
  `rum-syntax`
- keep canonical string atoms in `rum-core`
- implement evaluation semantics, string patterns, and search/transform
  behavior in `rum-runtime`
- register and document builtin inventory in `rum-stdlib`

The project should not introduce a dedicated `rum-string` crate yet. That
boundary should be revisited only after the builtin surface, shared helpers,
and performance constraints are concrete enough to justify a new subsystem
crate.

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

The current runtime now has an initial request-style layer for parse, lower,
evaluate, and inspect flows, with structured syntax diagnostics and
transport-friendly message/trace payloads. That layer should become the basis
for later protocol adapters instead of pushing terminal-formatted strings up
into hosts. Terminal text rendering now lives in `rum-terminal`, which gives
the CLI and REPL a shared presentation layer without forcing terminal
formatting into the host-facing API surface.

Hosts also now have a stable `SessionConfig` entrypoint for session setup,
covering resource limits, current-context defaults, context-path defaults,
package search roots, and tracing. That configuration layer should remain the
main way embedders shape a session before evaluation starts.

When a Jupyter adapter is added later, it should live in a separate crate and
translate protocol messages into those session-oriented runtime calls.
That placeholder crate now exists as `rum-jupyter`, with a first execute /
inspect / complete adapter slice built directly on the stabilized session API.

## Near-term roadmap

1. improve syntax spans, diagnostics, recovery, and comment handling
2. continue reducing ad hoc evaluator behavior through clearer metadata-driven
   sequencing
3. improve the REPL with multiline input, history, completion, and debug
   surfaces
4. stabilize host-facing session APIs for embedding and future Jupyter work
5. once the core/runtime foundation is tighter, execute the full-string
   roadmap across parsing, evaluation, text operations, string patterns, and
   unicode-sensitive behavior
