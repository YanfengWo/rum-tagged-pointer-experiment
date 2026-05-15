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

For development, `rum-jupyter` now also has a minimal launcher binary and a
temporary-kernelspec smoke path so we can validate real Jupyter-client
round-trips without installing a global kernelspec or keeping a long-lived
notebook service around.

### `rum-host` (planned)

`rum-host` is the planned first-class Rust embedding crate.

Its role is to sit between `rum-runtime` and future long-running Rust hosts,
providing a narrower and more stable public surface than exposing broad
runtime internals directly.

It should own:

- host-oriented session construction
- stable host-facing configuration facades
- eval / resume / inspect / complete / is-complete helpers
- transport-friendly outcome, failure, message, and input-request types
- small embedding-oriented helper layers that future adapters can share

It should not own:

- language semantics
- terminal rendering
- Jupyter protocol concerns
- broad evaluator internals
- Python-binding-specific policy

The design intent is for `rum-host` to become the primary Rust embedding
boundary, with future Python support layered on top of it rather than binding
directly to a broad `rum-runtime` surface.

### `rum-python` (planned)

`rum-python` is the planned optional Python adapter crate.

Its role is to provide a thin Python-facing layer on top of `rum-host` for
evaluation, suspend/resume, and editor-style request flows in Python
applications.

It should:

- wrap the stable `rum-host` boundary
- keep Python packaging and binding concerns isolated
- expose host-friendly Python objects rather than broad evaluator internals

It should not:

- bind directly to large portions of `rum-runtime`
- make Python the primary architectural driver for the kernel
- absorb notebook protocol behavior that belongs in `rum-jupyter`

The design intent is for Python support to remain optional and downstream of
the Rust host boundary, not a second path around it.

### `rum-difftest`

Owns differential and conformance-testing assets that are broader than any one
crate-local unit suite:

- imported seed corpora
- future `rum`-owned curated fixture trees
- fixture parsers and normalization helpers
- future user-configured Wolfram/Mathematica oracle runners
- divergence ledgers and fixture metadata
- performance baseline inputs

This crate is not part of the evaluator dependency chain. It exists so
conformance work can grow as a first-class workstream without turning
`rum-runtime` tests into one undifferentiated bucket of local regressions,
oracle checks, performance baselines, and adapter-contract assertions.

This crate is also intentionally optional and environment-gated. `rum` does
not ship a Wolfram license, a bundled oracle binary, or checked-in
oracle-generated expected outputs. If a contributor has access to a usable
Wolfram environment of their own, `rum-difftest` should help them run
differential checks against that local installation. If not, the rest of the
project should remain fully buildable and testable without it.

That boundary is useful for more than licensing. A user may diff `rum`
against an older or otherwise non-project-standard Wolfram installation, so
the oracle lane is also a compatibility probe against real preexisting user
environments rather than only a check against one pinned project version.

Just as importantly, `rum-difftest` is not the home for every kernel-facing
test. If a behavior is specific to `rum`'s own runtime model, host-facing
session APIs, suspension/resume mechanics, structured diagnostics, or any
adapter/protocol contract, its primary tests should stay near the owning crate
instead of being pulled into the oracle workstream.

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

The planned Rust realization of that boundary is documented in
[Host Adapter Plan](./docs/host-adapter-plan.md).
The planned Python layer above it is documented in
[Python Adapter Plan](./docs/python-adapter-plan.md).

## Reference Host Scenarios

The architecture can keep a small number of reference host scenarios in mind
when judging API ergonomics without turning those scenarios into product-scope
commitments.

For a future Python host, the reference scenario is AI- and data-tooling-
oriented applications that need deterministic rule evaluation, symbolic
normalization, structured diagnostics, and resumable execution around model,
tool, or workflow orchestration. The current concrete planning target is
[Python AI Policy](./examples/python-ai-policy/PLAN.md): a small offline
AI-infrastructure policy / rewrite / validation example with a thin Python
adapter and a narrow canonical request IR.

For a future Rust host, the reference scenario is a compiler/runtime-style
prototype that needs stable session lifecycles, predictable resource
boundaries, transport-friendly outcomes, and embedding-friendly request flows.
The current concrete planning target is
[Rust Wasm Prototype](./examples/rust-wasm-proto/PLAN.md): a Rust-hosted
WebAssembly validation and semantics prototype where Rust owns IR, fixtures,
memory, and host plumbing while `rum` owns symbolic validation, step rules,
traps, diagnostics, and explanation.

These scenarios are design pressure, not product definition. They should help
the project choose a clean and useful host boundary, but they should not turn
`rum` into an AI framework, a Wasm runtime, or a family of project-owned host
products.

## API Scope Boundary

The host-facing API should stay intentionally small and stable.

The project should expose enough public surface for:

- creating and configuring sessions
- submitting kernel requests
- receiving structured values, diagnostics, messages, and suspension outcomes
- building adapters or extensions on top of that boundary

It should not aim to expose every evaluator internal, normalization helper,
matcher detail, or experimental runtime abstraction as part of the public API.

The architectural goal is to make extension possible without making internals
part of the public contract. If contributors want richer Python, Rust, GUI,
IDE, or protocol integrations, they should be able to build them on top of a
narrow kernel boundary or in optional extension crates rather than by expanding
`rum-runtime` into a kitchen-sink host framework.

The current plan is to realize that boundary through a dedicated `rum-host`
crate rather than by making `rum-runtime` itself the long-term embedding API.

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

## Conformance Strategy

Conformance work should not be treated as one monolithic test bucket. Different
questions need different oracles.

The intended split is:

- Lane A: single-expression Wolfram diff for one-shot kernel semantics
- Lane B: stateful session diff for multi-step evaluation behavior
- Lane C: a known-difference ledger tracked as data rather than hidden in
  comments
- Lane D: performance regression suites whose oracle is `rum`'s own baseline,
  not Wolfram
- Lane E: host-contract tests for runtime session APIs, suspend/resume, and
  structured outcomes
- Lane F: adapter/protocol tests for Jupyter and future protocol layers

`rum-difftest` should own the imported seed packs, future curated fixture
directories, and the runners/helpers for the Wolfram-diff and divergence-ledger
lanes. By contrast, host-contract and adapter/protocol tests should stay near
the crates whose public contracts they validate.

For the Wolfram-oracle lanes, the runner policy should stay explicit:

- differential execution is optional and uses the contributor's own
  Wolfram/Mathematica installation and license
- the runner should accept a user-configured local oracle command such as
  `wolframscript`, `WolframKernel`, or `MathKernel`
- oracle-generated result files are transient run artifacts, not checked-in
  repository assets
- each oracle run should regenerate fresh comparison material rather than rely
  on project-owned committed snapshots
- version variance is expected and useful; a diff run can validate both `rum`
  semantics and compatibility with the user's existing Wolfram environment

The corresponding negative rule should also stay explicit:

- same-name kernel semantics that a Wolfram oracle can judge belong in
  `rum-difftest`
- `rum`-specific kernel extensions, runtime inspection helpers, suspend/resume
  flows, host-facing request/response contracts, and adapter/protocol behavior
  do not
- those `rum`-native tests should remain in crate-local unit, regression, and
  integration suites where the owning contract is defined

That split preserves the project's main architectural boundary:

- Wolfram is the oracle for same-name kernel semantics where practical
- `rum` itself is the oracle for host APIs, suspension payloads, and protocol
  behavior

## Core Language Boundary For This Project Phase

For the current project phase, `rum` defines its practical "core language"
scope by the union of two official Wolfram documentation navigation groups:

- the `Core Language & Structure` pages currently used as scope anchors:
  - `List`
  - `Expressions`
  - `Associations`
  - `Variables & Functions`
  - `Rules & Patterns`
  - `Functional Programming`
  - `Procedural Programming`
  - `String Manipulation`
  - `Package Development`
  - `Tuning & Debugging`
- the `Language Overview` pages currently used as scope anchors:
  - `Symbolic Expressions`
  - `Lists & Expression Manipulation`
  - `Associations`
  - `Functional Operations`
  - `Pattern Matching`
  - `Rules & Transformations`
  - `Definitions & Assignments`
  - `Logic & Tests`
  - `Scoping & Modularity`
  - `Procedural Programming`
  - `Error & Exception Handling`
  - `String Manipulation`

This definition includes both:

- same-name functions explicitly called out by those pages
- language mechanisms those pages treat as foundational, such as symbolic
  expressions, held evaluation, attributes, pattern matching, scoping,
  contexts, packages, messages, and debugging-oriented runtime behavior

This definition does not automatically pull in every adjacent area linked from
those pages. When a page drifts into clearly non-kernel domains such as
front-end-only behavior, notebook UI, graphics, web/network integration,
external-process or filesystem operations, compiler/parallel tooling, or
numerical-algorithm monitoring features, those remain out of scope unless the
project explicitly adopts them later.

The project should therefore treat these page sets as:

- the public-facing definition of the current core-language target
- the source of truth for support-matrix and conformance-planning work
- the default place where any unimplemented, partial, or deliberately divergent
  same-name feature must be recorded explicitly

The working support matrix for that scope lives in
[Core Language Support Matrix](./docs/core-language-support-matrix.md).

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
[TODO.md](./TODO.md). For page-driven language coverage, status, and divergence
tracking, use
[Core Language Support Matrix](./docs/core-language-support-matrix.md).
