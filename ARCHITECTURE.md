# Architecture

## Mission

`rum` is a Rust implementation of a Mathematica / Wolfram-style core language.

The project's long-term goal is not merely to parse and evaluate expressions,
but to grow a reusable language kernel that can support:

- symbolic evaluation and rule-based programming
- business, industrial, and DSL-oriented execution
- CLI and REPL use
- future third-party GUI frontends
- future third-party IDE or editor integrations
- future protocol adapters such as Jupyter
- future external host communication layers that may resemble WSTP-style links

The early product is therefore the kernel, not a notebook application.

## Scope

### In scope

- symbolic expression trees
- exact integers
- parser, lowering, and diagnostics
- evaluator and rule engine
- definitions, attributes, scoping, and contexts
- embeddable runtime sessions
- host-facing request/config APIs
- terminal tools and optional protocol adapters

### Out of scope for the early phases

- full notebook or GUI products
- project-owned GUI frontends
- project-owned IDE/editor plugins
- graphics and visualization
- full CAS coverage
- broad front-end compatibility shims

## Design Goals

The architecture is intended to optimize for four things at once:

- a kernel that is useful on its own
- clear layering and maintainable module boundaries
- host-agnostic extensibility
- steady growth without turning the evaluator into an untestable monolith

That is why the project does not treat CLI, REPL, host embedding, and Jupyter
as separate products with separate cores. They are different clients of the
same runtime.

The practical implication is:

- the project should invest primarily in kernel capabilities
- adapters matter because they validate and expose those capabilities
- adapter-specific work should stay secondary unless it clearly strengthens the
  kernel boundary or proves that the kernel is ready for broader host use

## Why This Is Not Overdesign

Some of the current infrastructure was accelerated by the Jupyter workstream:

- session-oriented request APIs
- structured diagnostics and machine-readable failures
- host-driven input requests
- suspension/resume
- stable session configuration
- explicit terminal-vs-protocol separation

These should not be viewed as Jupyter leakage.

They are kernel capabilities that are also required by any serious future host:

- a GUI frontend needs structured results, diagnostics, completion, inspect,
  and resumable input
- an IDE integration needs parse/complete/inspect/is-complete style requests
- an embedding API needs stable configuration and transport-friendly outcomes
- an external protocol layer needs resumable execution and non-terminal result
  shapes

In other words, Jupyter has been a useful forcing function, but the resulting
infrastructure serves the broader architectural goal of a reusable language
kernel.

What *would* count as overdesign would be pushing protocol-specific details
into the evaluator core. That boundary is deliberately avoided.

## Dependency Adoption Policy

`rum` is not trying to minimize dependencies at all costs.

The project prefers a more selective rule:

- keep core language semantics in-project
- use mature external libraries when the problem is primarily about security,
  protocol compliance, or standard data-format handling
- avoid importing convenience dependencies that blur crate boundaries without
  materially improving correctness or maintainability

This policy is motivated by the same long-term goals that shape the rest of the
architecture:

- stability
- security
- deployability
- predictable maintenance cost

This means the architecture should continue to prefer self-implementation for:

- parser and lowering behavior
- evaluator sequencing
- definitions, scoping, and rule application
- matcher semantics
- formatter layout policy

Those parts define the language and are therefore part of the kernel's
identity.

By contrast, the architecture should be willing to adopt mature libraries for
infrastructure concerns such as:

- cryptographic signing and verification
- standard protocol serialization formats
- standard message or transport utilities when correctness depends on protocol
  compatibility rather than custom language semantics

Even when an external dependency is adopted, it should usually sit behind a
thin project-owned boundary. For example, a Jupyter signing or JSON layer
should remain a small `rum-jupyter` adapter surface rather than leaking library
types or protocol details into `rum-runtime`.

The goal is not "fewest crates". The goal is to spend custom implementation
effort where `rum`'s semantics are unique, and to rely on mature libraries
where the problem is standard, security-sensitive, or protocol-critical.

## Layering

The workspace is split by responsibility.

### `rum-syntax`

Owns source-facing syntax:

- lexer and parser
- syntax tree
- spans and source diagnostics
- preserved trivia
- formatter / pretty-printer

This crate should describe source structure, not runtime semantics.

### `rum-core`

Owns runtime-neutral core values:

- `Expr`
- `Atom`
- `Symbol`
- exact integer representation
- canonical string atoms

This layer exists so syntax, runtime, and future hosts can share a stable value
model without dragging evaluator state into every crate.

### `rum-runtime`

Owns the actual kernel:

- lowering from syntax into core values
- evaluation sessions
- staged call dispatch and evaluation sequencing
- definitions and rule application
- contexts and package loading
- messages, tracing, and resource limits
- scoping and held evaluation
- symbol inspection
- host-facing request/config APIs
- runtime-side suspension/resume

This crate is the center of the system. It should contain language semantics,
not terminal presentation or protocol envelopes.

### `rum-stdlib`

Owns builtin installation and future library growth.

The single source of truth for core builtin metadata now lives in
`rum-runtime`, while `rum-stdlib` remains responsible for installation and
library expansion. This keeps builtin semantics, dispatch metadata, and
registration from drifting apart.

### `rum-terminal`

Owns terminal-facing presentation:

- rendered runtime messages
- rendered diagnostics
- rendered inspection output

This crate exists so CLI/REPL output can evolve without forcing terminal text
formats into the kernel API.

### `rum-repl`

Owns interactive shell behavior:

- input loop
- multiline handling
- history
- command parsing
- shell-oriented interaction flow

The REPL should stay thin and build on stable runtime and terminal layers.

### `rum-cli`

Owns process-level concerns:

- argument parsing
- file/script entry
- REPL startup
- exit codes

Like the REPL, the CLI should use runtime request APIs and terminal rendering
instead of reaching directly into evaluator internals.

### `rum-jupyter`

Owns Jupyter-facing protocol and transport concerns:

- execute / inspect / complete / is-complete mapping
- notebook-oriented output envelopes and MIME bundles
- wire codecs
- connection-file/bootstrap handling
- channel/session/signature handling
- transport loops and backend-specific socket behavior

This crate is intentionally optional. It is a host adapter, not part of the
core evaluator.

Its architectural role is twofold:

- provide a real, broadly accessible frontend that is easy to stand up across
  platforms
- act as a capability-validation adapter for the kernel, proving that host
  requests, structured diagnostics, resumable input, transport-friendly
  results, and lifecycle handling are mature enough for serious embedding

In other words, `rum-jupyter` matters because it exercises the kernel in a
general host setting, not because notebook-specific behavior is itself the main
product.

## Architectural Principle: Kernel First, Adapters Second

The central rule of this architecture is:

`rum-runtime` should expose a host-facing kernel boundary, and every frontend
or protocol adapter should build on top of it.

That boundary now includes:

- parse/eval/inspect/complete/is-complete style requests
- structured diagnostics and failures
- transport-friendly message and trace payloads
- stable session configuration
- host-driven input requests
- suspend/resume behavior

This boundary is intentionally host-agnostic. It is used today by:

- CLI
- REPL
- tests
- `rum-jupyter`

It should also be the basis for any future:

- third-party GUI frontend
- third-party IDE/editor integration
- embedding API
- external protocol or link layer

## What Must Stay Out Of The Kernel

The following belong outside `rum-runtime`:

- Jupyter message schemas
- wire codecs
- routing ids
- channel layouts
- connection files
- signature policies
- notebook-specific MIME shaping
- backend-specific transport loops
- terminal-oriented text formatting

Those concerns are real and important, but they belong in adapters such as
`rum-terminal` and `rum-jupyter`, not in the evaluator core.

## Current Execution Model

At a high level, the kernel currently does this:

1. parse source into syntax expressions
2. lower syntax into canonical core `Expr` values
3. evaluate to a fixed point under recursion, cycle, and pass limits
4. run a staged dispatch pipeline for special forms, argument preparation,
   threading, definitions, builtins, and pure functions
5. return values plus structured diagnostics, messages, and optional traces

Lowering and evaluation are intentionally separated:

- lowering is responsible for syntax sugar normalization and representation
  validity
- runtime evaluation is responsible for semantic validity and execution

For details, see:

- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)

## Normalization As A First-Class Boundary

The runtime now treats normalization as an architectural concern rather than an
incidental parsing detail.

There are two explicit normalization families:

- resolved-argument normalization
  - for builtins that intentionally evaluate parts of their input before
    compiling internal config
  - examples include `Thread`, traversal config, loader calls, and attribute
    updates
- raw-form normalization
  - for builtins whose inputs are held or syntax-shaped and therefore must be
    interpreted without ordinary evaluation
  - examples include message selectors, definition targets, assignment lhs
    classification, and scoping binding lists

This distinction matters because it prevents the evaluator from silently
collapsing held/syntax-driven language constructs into ad hoc inline parsers.

## Suspension/Resume As Kernel Infrastructure

Host-driven input and resumable execution are now part of the kernel model.

That includes:

- `InputString` backed by a real runtime input facility
- saved continuation state instead of request replay
- typed internal continuation support for helper paths that do not naturally
  return `Expr`

This is another place where the design should be understood as general kernel
infrastructure, not Jupyter-specific machinery. The same model is valuable for
future GUI prompts, embedding, and protocol-driven hosts.

## Numeric Model

Exact integers are first-class kernel values, not an afterthought. That matters
for both Wolfram-style language fidelity and practical deterministic DSL work.

The current foundation is `num_bigint::BigInt` inside `rum-core::Atom::Integer`.
Future numeric expansion should build from that exact baseline.

The broader mathematical surface is intentionally a later workstream. The
project should first stabilize the kernel semantics, then add the practical
exact-arithmetic and symbolic math needed for business and industrial workloads
before considering broader CAS-style expansion.

## Compatibility Target

For named Wolfram-language surface features, the default target is semantic
compatibility rather than "close enough" resemblance.

In practice, that means:

- if `rum` ships a same-name function or form, it should aim to match Wolfram
  behavior as closely as possible
- deliberate divergence is acceptable only when it is documented explicitly
- conformance should be checked with differential tests where practical rather
  than inferred only from local intent

This compatibility target is one reason the project treats conformance testing
as a core engineering task rather than optional polish.

## String Subsystem Direction

Full Wolfram-style string functionality is important to the project's intended
commercial, industrial, and DSL-oriented use cases, so strings are treated as a
planned core-language workstream rather than a peripheral convenience feature.

The intended first implementation wave keeps responsibilities split as follows:

- `rum-syntax`: string literals and source-facing escaping
- `rum-core`: canonical string atoms
- `rum-runtime`: evaluation semantics, matching, search, and transformation
- `rum-stdlib`: builtin registration and library growth

A dedicated string crate is intentionally deferred until the subsystem is large
and stable enough to justify its own boundary.

## Code Organization As Architecture

Source organization is treated as part of the architecture, not as cleanup
after the fact.

The governing principles are:

- keep modules readable to humans
- avoid both giant files and forests of tiny files
- prefer feature-oriented subfolders once a subsystem grows
- favor boundaries that help incremental compilation
- design functions so important behavior is reachable by focused unit tests

That is why the project is steadily moving away from giant catch-all files and
toward clearer module boundaries, smaller helpers, and explicit planning or
normalization layers.

The detailed rules live in
[Code Organization Guidelines](./docs/code-organization-guidelines.md).

## Relationship To The Roadmap

This document describes structural intent and long-term rationale.

It should stay relatively stable and answer questions such as:

- what belongs in each crate?
- why does the runtime expose host-facing session APIs?
- why is `rum-jupyter` separate?
- why are some Jupyter-motivated facilities still intentionally in the kernel?

For active work, open gaps, and near-term sequencing, the source of truth is
[TODO.md](./TODO.md), not this file.
