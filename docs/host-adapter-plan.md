# Host Adapter Plan

## Goal

`rum-host` is the first-class Rust embedding crate for `rum`.

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

The initial public surface stays intentionally small:

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

That surface should cover these flows:

- `parse`
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

Python adapter work is frozen for the current stabilization phase.

If it is revived later, Python should come after `rum-host`, not before it.
The order would be:

1. continue hardening the Rust host contract in `rum-host`
2. validate it through Rust-side embedding examples and tests
3. only then decide whether a thin Python adapter is worth adding on top of
   that contract

Until a concrete downstream need appears, do not spend roadmap time on Python
API shape, packaging, or examples. This keeps Python ergonomics possible later
without letting binding details reshape core runtime boundaries too early.

## Validation Strategy

The first validation wave for `rum-host` includes:

- adapter-focused Rust tests for eval flows through the public facade
- suspend/resume tests through the host facade
- structured outcome, failure-code, message, print, parse, inspect,
  completion, and completeness coverage
- one small example embedding binary or example module, with the current
  `crates/rum-host/examples/basic_embedding.rs` serving as the minimal smoke
  example; broader showcase work should be driven by concrete external
  pressure such as `mma-hol`
- selective reuse by `rum-jupyter` only after the surface proves useful

## Suggested Build Order

1. define the stable host-facing contract in documentation
2. identify which existing `rum-runtime` types are already close enough to wrap
3. create a small `rum-host` crate with builders and eval/resume flow
4. add host-focused regression tests
5. expand to inspect/complete/is-complete
6. keep Python frozen unless a concrete downstream need reopens it

Items 1 through 5 now have an initial implementation. The first hardening pass
has also split the `rum-host` facade model into request, outcome, and
inspection modules, added public adapter-contract regression coverage, and
landed a minimal Rust embedding example. The next work should validate the
surface through richer examples and downstream adapter pressure, not expand the
public surface by default.

## Jupyter Reuse Assessment

`rum-jupyter` should not be force-migrated to `rum-host` immediately. It
currently owns notebook-specific MIME bundles, IOPub lifecycle behavior,
message routing, signatures, and transport frames, all of which are outside
`rum-host`.

The likely reuse points are narrower:

- cursor-symbol extraction for inspect/complete requests
- conversion from runtime request outcomes into stable host-level success,
  input-suspension, and failure facades
- future shared examples or smoke tests that exercise host request semantics

The first `rum-host` crate should prove these seams before `rum-jupyter` takes
a dependency on it.

## Initial Open Questions

- should `rum-host` eventually expose a carefully bounded raw-expression
  escape hatch, or keep all expression data behind rendered/facade values?
- which additional structured outputs should be guaranteed stable in v1 of
  `rum-host` beyond result, print, message, and trace?
- when should `rum-jupyter` begin reusing `rum-host` helpers versus continuing
  to call `rum-runtime` directly?
