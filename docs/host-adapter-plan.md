# Host Adapter Plan

## Goal

`rum-host` is the planned first-class Rust embedding crate for `rum`.

Its job is to expose a narrow, stable, host-oriented API on top of
`rum-runtime` so backend services, rule engines, and DSL-oriented applications
can embed the kernel without depending directly on evaluator internals.

This crate is meant to become the reference host boundary for:

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
  - remains an optional protocol adapter and should eventually reuse
    `rum-host`-level pieces where that reduces duplication
- future Python adapter
  - should wrap `rum-host` rather than binding directly to broad
    `rum-runtime` internals

## Crate Boundary

The planned crate should be optional and dependency-light.

It should provide:

- session construction
- stable configuration entrypoints
- request submission helpers
- suspend/resume helpers
- transport-friendly result facades
- small adapter-facing helper types

It should avoid re-exporting broad internal modules from `rum-runtime`.

## Proposed Public Surface

The first-pass public surface should stay intentionally small:

- `HostSessionBuilder`
- `HostSession`
- `HostConfig`
- `HostEvalRequest`
- `HostEvalOutcome`
- `HostFailure`
- `HostInputRequest`
- `HostStructuredOutput`

That surface should cover these flows:

- `eval`
- `resume_with_input`
- `inspect`
- `complete`
- `is_complete`

The public surface may also include:

- session lifecycle helpers
- resource-limit configuration
- message/print/output accessors

It should not initially expose:

- raw definition tables
- matcher IR
- planner internals
- ad hoc access to internal continuation payloads
- mutable evaluator state that bypasses request APIs

## Data Shape Guidance

The stable host-facing types should prefer transport-friendly facades over
direct exposure of all internal runtime details.

Default guidance:

- expose rendered values and structured outcomes directly
- expose diagnostics/messages/prints in stable host-friendly structs
- keep raw/internal evaluator details behind `rum-runtime`
- only expose `Expr` directly where it materially improves real embedding
  scenarios without freezing too much internal representation

The bias should be toward a smaller stable contract first.

## Relationship To Python

Python should come after `rum-host`, not before it.

The planned order is:

1. stabilize the Rust host contract in `rum-host`
2. validate it through Rust-side embedding examples and tests
3. add a thin `pyo3`-based Python adapter on top of that contract

That keeps Python ergonomics important without letting Python binding details
reshape core runtime boundaries too early.

## Validation Strategy

The first validation wave for `rum-host` should include:

- adapter-focused Rust tests for session lifecycle and eval flows
- suspend/resume tests through the host facade
- structured outcome and message coverage
- one small example embedding binary or example module, with
  [Rust Wasm Prototype](../examples/rust-wasm-proto/PLAN.md) as the current
  long-range showcase candidate once the host boundary is stable enough
- selective reuse by `rum-jupyter` only after the surface proves useful

## Suggested Build Order

1. define the stable host-facing contract in documentation
2. identify which existing `rum-runtime` types are already close enough to wrap
3. create a small `rum-host` crate with builders and eval/resume flow only
4. add host-focused regression tests
5. expand to inspect/complete/is-complete
6. only then plan Python on top

## Initial Open Questions

- how much `Expr` exposure is worth preserving in the stable host API?
- should host config be a wrapper around `SessionConfig` or an independent
  facade that converts inward?
- which structured outputs should be guaranteed stable in v1 of `rum-host`?
- when should `rum-jupyter` begin reusing `rum-host` helpers versus continuing
  to call `rum-runtime` directly?
