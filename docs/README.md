# rum Documentation

This directory holds `rum`'s documentation. If you are new, start with the
guide; if you are looking up a builtin, use the reference.

## Start here

- **[Language Guide](./guide/)** — the hand-written introduction to the language
  and the Rust embedding surface, taught from zero using the current build as the
  contract. Begin at [Getting Started](./guide/getting-started.md).
- **[Builtin Reference](./reference/builtins.md)** — the generated per-builtin
  contract (attributes, capability class, summary, limit notes). Regenerated from
  the standard-library catalog; do not edit by hand.

## Language and runtime contracts

- [Lowering and canonical forms](./lowering-canonical-forms.md) — how surface
  syntax lowers to canonical runtime heads.
- [Runtime evaluation audit](./runtime-evaluation-audit.md) — evaluator
  invariants and current semantic boundaries.
- [Formatter style](./formatter-style.md) — how expressions are rendered.
- [v0.1 acceptance boundary](./v0.1-acceptance-boundary.md) — the stabilization
  and public-contract boundary.

## Embedding and host

- [Host embedding and hardening](./host-embedding-and-hardening.md) — the host
  contract: interrupt/cancel, timeout, poison recovery, reset, snapshot/restore,
  capability sandbox.
- [Host adapter plan](./host-adapter-plan.md) — the embedding-adapter design.
- [Python adapter plan](./python-adapter-plan.md) — the future, downstream
  Python binding plan.

## Project process and policy

- [Testing strategy](./testing-strategy.md) — the testing policy and test
  layering (see also [`CONTRIBUTING.md`](../CONTRIBUTING.md)).
- [Code organization guidelines](./code-organization-guidelines.md) — file-size
  and module-organization rules.

## Tools

- [Jupyter console manual](./jupyter-console-manual.md) — using the optional
  Jupyter adapter.

## Internal development record

- **[dev/](./dev/)** — the transparent working history: per-line design docs and
  contracts, the Codex task briefs under [dev/codex/](./dev/codex/), the
  usability-pass ledgers, and audit findings. You do **not** need any of it to use
  or embed `rum`; it is kept public as the project's development record. See
  [dev/README.md](./dev/README.md).

## Repository-root documents

Scope, policy, and community files live at the repository root:
[README](../README.md), [DISCLAIMER](../DISCLAIMER.md), [SECURITY](../SECURITY.md),
[CONTRIBUTING](../CONTRIBUTING.md), [CODE_OF_CONDUCT](../CODE_OF_CONDUCT.md),
[CHANGELOG](../CHANGELOG.md), and [AGENTS](../AGENTS.md) (agent-facing operational
notes).
