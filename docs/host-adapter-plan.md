# Host Adapter Plan

Status: the initial plan and Jupyter migration are complete. This file retains
the design rationale; the current API contract is `rum-host` rustdoc and the
[embedding guide](./guide/embedding.md).

## Goal

`rum-host` is the first-class Rust embedding crate for `rum`.

Its job is to expose a narrow, stable, host-oriented API on top of
`rum-runtime` so backend services, rule engines, and DSL-oriented applications
can embed the kernel without depending directly on evaluator internals.

This crate is the reference host boundary for:

- long-running Rust services
- rule-engine style embedding
- DSL execution backends
- future Python bindings layered through a thinner FFI/binding boundary

## Non-Goals

`rum-host` should not:

- replace `rum-runtime` as the home of language semantics
- expose matcher/planner/lowering internals as public stable contract
- absorb terminal formatting, CLI concerns, or Jupyter protocol details
- become a kitchen-sink framework for every possible host scenario
- force Python-specific design concerns back into `rum-runtime`

## Relationship To Existing Crates

- `rum-runtime`
  - owns kernel semantics, sessions, request handling, and structured outcomes
- `rum-host`
  - owns the narrow public embedding surface built on top of `rum-runtime`
- `rum-jupyter`
  - remains an optional protocol adapter and consumes `rum-host` for kernel
    request handling; its Notebook behavior shipped in the `v0.1` surface
- future Python adapter
  - should wrap `rum-host` rather than binding directly to broad
    `rum-runtime` internals

## Crate Boundary

The crate is intentionally narrow and dependency-light.

It should provide:

- session construction
- stable configuration entrypoints
- request submission helpers
- suspend/resume helpers
- transport-friendly result facades
- small adapter-facing helper types

It should avoid re-exporting broad internal modules from `rum-runtime`.

## Initial Public-Surface Proposal (historical)

The initial proposal was intentionally small:

- `HostSessionBuilder`
- `HostSession`
- `HostConfig`
- `HostParseRequest`
- `HostParseResponse`
- `HostEvalRequest`
- `HostEvalOutcome`
- `HostFailure`
- `HostInputRequest`
- `HostInputSuspension`
- `HostStructuredOutput`
- `HostCompletionRequest`
- `HostCompletionResponse`
- `HostInspectRequest`
- `HostInspectResponse`
- `HostIsCompleteRequest`
- `HostIsCompleteResponse`

That surface covered these flows:

- `parse`
- `eval`
- `resume_with_input`
- `inspect`
- `complete`
- `is_complete`

The delivered surface also includes:

- session lifecycle helpers
- resource-limit configuration
- message/print/output accessors

It should not expose:

- raw definition tables
- matcher IR
- planner internals
- ad hoc access to internal continuation payloads
- mutable evaluator state that bypasses request APIs

## Data Shape Guidance

The stable host-facing types should prefer transport-friendly facades over
direct exposure of all internal runtime details.

Initial policy:

- expose rendered values and structured outcomes directly
- expose diagnostics/messages/prints in stable host-friendly structs
- keep raw/internal evaluator details behind `rum-runtime`
- do not expose raw `Expr` in the first `rum-host` contract
- revisit selective `Expr` exposure only if a real Rust host scenario shows
  that rendered/facade values are insufficient

The bias should be toward a smaller stable contract first.

## Initial Implementation

The first crate wave is intentionally a thin wrapper over the existing
`rum-runtime` request APIs:

- `HostSessionBuilder` builds a configured runtime session and installs the
  current core builtin set by default.
- `HostSession::parse`, `eval`, `resume_with_input`, `inspect`, `complete`,
  and `is_complete` are the supported request entrypoints.
- `HostEvalOutcome` distinguishes success, input suspension, and failure
  without treating host input as an ordinary error.
- `HostFailure` uses stable coarse failure codes plus rendered messages rather
  than exposing every internal evaluator error type as public API.
- `HostStructuredOutput` is a compact facade for result, print, message, and
  trace outputs that downstream adapters can map into their own transports.

The crate currently avoids ownership of terminal rendering, notebook MIME
bundles, Jupyter wire schemas, and Python binding policy.

## Relationship To Python

Python adapter work remains a separate, frozen downstream direction. Publication
and the Rust host boundary are complete, but no concrete consumer currently
justifies binding or packaging work in this repository.

If it is revived later, Python wraps the existing `rum-host` contract; it does
not open a preliminary kernel or host-surface expansion line.

Until a concrete downstream need appears, do not spend roadmap time on Python
API shape, packaging, or examples. This keeps Python ergonomics possible later
without letting binding details reshape core runtime boundaries too early.

## Delivered Validation Strategy

The first validation wave for `rum-host` includes:

- adapter-focused Rust tests for eval flows through the public facade
- suspend/resume tests through the host facade
- structured outcome, failure-code, message, print, parse, inspect,
  completion, and completeness coverage
- host-backed `rum-jupyter` request handling, keeping Jupyter protocol,
  transport, and MIME details in `rum-jupyter`
- one small example embedding binary or example module, with the current
  `crates/rum-host/examples/basic_embedding.rs` serving as the minimal smoke
  example; broader showcase work should be driven by concrete external
  pressure from a large downstream package or workload

## Delivered Build Order (historical)

1. define the stable host-facing contract in documentation
2. identify which existing `rum-runtime` types are already close enough to wrap
3. create a small `rum-host` crate with builders and eval/resume flow
4. add host-focused regression tests
5. expand to inspect/complete/is-complete
6. migrate `rum-jupyter` request handling onto `rum-host`
7. add one documented real-Notebook smoke/deploy path
8. keep Python frozen unless a concrete downstream need reopens it

Items 1 through 7 are complete. The facade model is split into request,
outcome, and inspection modules; public contract tests and the minimal Rust
embedding example are present; `rum-cli`, `rum-repl`, and `rum-jupyter` consume
the facade; and the documented Notebook smoke/deploy path shipped in the
`v0.1.0` boundary. Future surface growth requires concrete downstream pressure.

## Jupyter Migration Target

The earlier "reuse later" stance was superseded by the `v0.1` boundary:
`rum-jupyter` routes kernel-facing request handling through `rum-host`, and the
documented real-Notebook deploy path and frontend smoke are complete.

The migration moved these responsibilities behind `rum-host`:

- session construction and core-builtin installation
- eval and stdin resume
- inspect, complete, and is-complete
- stable success, input-suspension, failure, message, print, trace, and
  structured-output facades
- public adapter-facing result and failure shapes

These responsibilities remain in `rum-jupyter`:

- Jupyter request/response models
- ZeroMQ/native transport and heartbeat handling
- connection-file parsing, bootstrap, and kernelspec/deploy scripts
- channel/session/signature handling
- wire codecs and validation
- IOPub lifecycle envelopes and MIME bundle construction
- graceful handling of unsupported Jupyter messages

## Deferred Design Questions

- should `rum-host` eventually expose a carefully bounded raw-expression
  escape hatch, or keep all expression data behind rendered/facade values?
- which additional structured outputs should be guaranteed stable in v1 of
  `rum-host` beyond result, print, message, and trace?
