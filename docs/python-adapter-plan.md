# Python Adapter Notes (downstream project)

Decision (2026-07-06): the Python binding is a **separate downstream
repository**, not a crate in this workspace. It embeds `rum` into Python via
`pyo3` on top of `rum-host`, and it starts after this repository's
publication mechanics are settled (a path dependency works earlier if
needed; wheels are self-contained either way). This repository does not grow
Python binding code, packaging workflows, or Python-specific examples.

This keeps the kernel workspace buildable with nothing but the declared Rust
toolchain, and it makes the binding what it architecturally is: a downstream
consumer of the public embedding boundary, like any other host.

## Standing Rule

The binding binds to `rum-host`, never directly to broad `rum-runtime`
internals. `rum`-side obligations are only: keep `rum-host` a sufficient,
documented embedding boundary, and publish the embedding adoption-cost
numbers tracked in `TODO.md`.

The intended stack remains:

- `rum-runtime`
  - kernel semantics and session machinery
- `rum-host`
  - stable Rust embedding contract for parse, eval, resume, inspect,
    complete, is-complete, failures, messages, and structured outputs
- optional future Python adapter
  - thin Python-facing layer on top of `rum-host`

## Frozen Non-Goals

Do not spend active roadmap time on:

- a `rum-python` crate
- `pyo3` integration
- Python packaging or wheel workflows
- Python-specific public API design
- Python examples that require maintained project-owned adapter code

Do not let Python concerns reshape `rum-runtime`.

## Revival Criteria

Reopen Python adapter work only when at least one of these is true:

- a concrete downstream project needs Python embedding
- `rum-host` has a stable enough v0.1 contract to wrap without exposing
  evaluator internals
- a small Python smoke surface can be implemented without expanding kernel
  scope

If revived, start with:

- `Session.eval(text)`
- `Session.resume_with_input(text)`
- inspect / complete / is-complete only after eval/resume is stable
- rendered values and simple structured messages before any raw `Expr` interop

The old detailed Python AI policy showcase plan was removed. AI/data policy
work remains an application direction, not a project-owned example commitment.
