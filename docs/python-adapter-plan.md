# Python Adapter Notes (downstream project)

Decision (2026-07-06): the Python binding is a **separate downstream
repository**, not a crate in this workspace. It embeds `rum` into Python via
`pyo3` on top of `rum-host`. The repository and `v0.1.0` host boundary are now
published, but the binding remains deferred until a concrete downstream
consumer requests it. This repository does not grow Python binding code,
packaging workflows, or Python-specific examples.

This keeps the kernel workspace buildable with nothing but the declared Rust
toolchain, and it makes the binding what it architecturally is: a downstream
consumer of the public embedding boundary, like any other host.

## Standing Rule

The binding binds to `rum-host`, never directly to broad `rum-runtime`
internals. `rum`-side obligations are only: keep `rum-host` a sufficient,
documented embedding boundary, and publish the embedding adoption-cost
numbers recorded in the [embedding guide](./guide/embedding.md).

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

Reopen Python adapter work only when a concrete downstream project needs Python
embedding. At that point, also require that the binding can stay on the
documented `rum-host` contract and that a small smoke surface can be delivered
without expanding kernel scope.

If revived, start with:

- `Session.eval(text)`
- `Session.resume_with_input(text)`
- inspect / complete / is-complete only after eval/resume is stable
- rendered values and simple structured messages before any raw `Expr` interop

The old detailed Python AI policy showcase plan was removed. AI/data policy
work remains an application direction, not a project-owned example commitment.
