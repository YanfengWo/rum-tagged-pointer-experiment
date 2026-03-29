# Python Adapter Plan

## Goal

The planned Python adapter is an optional project-owned binding layer for
embedding `rum` in Python applications.

Its job is to expose a small, Python-friendly API for:

- evaluation
- suspend/resume with host-provided input
- inspect/complete/is-complete style editor flows
- structured results, messages, and prints

It should target practical Python scenarios such as:

- AI and tool-orchestration workflows
- data and notebook-adjacent scripting
- lightweight DSL execution from Python hosts
- test and automation harnesses

## Primary Architectural Rule

Python should bind to `rum-host`, not directly to broad `rum-runtime`
internals.

The intended stack is:

- `rum-runtime`
  - kernel semantics and session machinery
- `rum-host`
  - stable Rust embedding contract
- planned Python adapter
  - thin Python-facing layer on top of `rum-host`

This keeps Python ergonomics important without allowing Python binding details
to reshape the evaluator core.

## Non-Goals

The Python adapter should not:

- become the canonical public API for all hosts
- expose large slices of evaluator internals directly into Python
- force Python-specific object models into `rum-runtime`
- absorb notebook protocol behavior that belongs in `rum-jupyter`
- turn `rum` into a Python-first framework

## Planned Crate Boundary

The adapter will likely live in its own optional crate, tentatively named
`rum-python`.

That crate should:

- depend on `rum-host`
- use `pyo3`
- keep Python packaging/build concerns isolated from kernel crates
- expose a small set of Python classes and conversion helpers

It should not:

- bypass `rum-host`
- directly re-export arbitrary `rum-runtime` modules
- leak `pyo3` concerns into unrelated workspace crates

## Proposed Python Surface

The first-pass Python-facing API should stay intentionally small.

Suggested objects:

- `SessionBuilder`
- `Session`
- `EvalOutcome`
- `EvalFailure`
- `InputRequest`
- `StructuredOutput`
- `RuntimeMessage`

Suggested methods:

- `Session.eval(text: str) -> EvalOutcome`
- `Session.resume_with_input(text: str) -> EvalOutcome`
- `Session.inspect(text: str, cursor: int | None = None) -> ...`
- `Session.complete(text: str, cursor: int | None = None) -> ...`
- `Session.is_complete(text: str) -> ...`

The surface may later include:

- builder-based session configuration
- convenience conversion helpers
- explicit lifecycle helpers for long-running sessions

## Data Shape Guidance

The first Python API should bias toward predictable host-friendly wrappers
rather than exposing every internal Rust type.

Suggested approach:

- values exposed as canonical text first
- messages exposed as simple Python objects with stable fields
- prints exposed as strings
- structured outputs exposed as simple wrapper objects or dictionaries
- suspension exposed as a dedicated `InputRequest` object

`Expr` should only be exposed into Python if a later real use case justifies
the contract cost.

## Error Model

The Python adapter should distinguish:

- successful evaluation outcomes
- structured evaluation failures
- Python binding misuse

Suggested default policy:

- normal kernel issues stay inside `EvalOutcome` / `EvalFailure`
- Python-side misuse raises Python exceptions
- suspend/resume remains a normal kernel flow, not a Python exception path

## Packaging Direction

Packaging should be postponed until the API shape is stable enough to justify
it.

When that time comes, the likely direction is:

- `pyo3`
- `maturin` or a similarly standard build workflow
- optional wheels only after the Rust host contract proves stable

That keeps wheel/distribution concerns out of the current kernel-first phase.

## Validation Strategy

The first validation wave should include:

- smoke tests for session creation and repeated eval
- suspend/resume tests through Python
- completion/inspect/is-complete smoke coverage
- message/print/result shape assertions
- one small example integration for notebook-adjacent or tool-oriented usage

## Suggested Build Order

1. finish stabilizing the `rum-host` contract
2. identify the exact host-facing Rust types Python should wrap
3. create a minimal `rum-python` crate with `SessionBuilder`, `Session`, and
   eval/resume only
4. add Python smoke tests
5. expand to inspect/complete/is-complete
6. revisit packaging and distribution only after that

## Initial Open Questions

- should Python results default to canonical text only, or include richer
  structured fields from day one?
- how much of `StructuredOutput` should be surfaced as Python-native
  dictionaries versus typed wrapper classes?
- should the first adapter expose any direct `Expr` interop at all?
- when should `rum-jupyter` and the Python adapter share helper code through
  `rum-host`-level abstractions?
