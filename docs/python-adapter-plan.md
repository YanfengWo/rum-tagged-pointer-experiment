# Frozen Python Adapter Notes

Project-owned Python adapter work is frozen for the current stabilization
phase.

Python remains a plausible future host for `rum`, especially for AI, data,
automation, and notebook-adjacent workflows. It is not an active roadmap item,
and the repository should not grow Python binding code, packaging workflows, or
Python-specific examples until a concrete downstream need justifies reopening
the work.

## Standing Rule

If Python support is revived later, it should bind to `rum-host`, not directly
to broad `rum-runtime` internals.

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
