# rum

`rum` is an experimental Rust implementation of a Mathematica / Wolfram-style
core language runtime.

The project is focused on the language kernel rather than a notebook system or
computer algebra stack. The current emphasis is:

- symbolic expression trees
- exact integers
- parser/lowering infrastructure
- definitions, rules, and pattern matching
- scoping and held evaluation
- embeddable session APIs
- CLI / REPL tooling
- an early optional Jupyter-facing adapter

## What A Reviewer Should Know First

`rum` is no longer a bootstrap prototype. The core language slice is already
substantial:

- `rum-syntax` has a lexer, parser, spans, source-aware diagnostics, preserved
  trivia, and a document-IR formatter
- `rum-core` and `rum-runtime` support exact integers, canonical lowering,
  sessions, contexts, package loading, ownvalues/upvalues/downvalues/subvalues,
  hold-aware evaluation, threading, tracing, and resource limits
- rules and patterns already drive `MatchQ`, `Cases`, `ReplaceAll`, and
  `ReplaceRepeated`
- `rum-cli` and `rum-repl` are usable developer tools
- `rum-jupyter` is an early adapter layer built on top of host-facing session
  APIs, not part of the evaluator core

The main thing still in flight is not "whether evaluation exists" but how much
of the intended Wolfram-style semantics has been tightened and how much surface
area still remains to be filled in.

If you are reviewing the project, the fastest reading path is:

1. [Architecture](./ARCHITECTURE.md)
2. [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
3. [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
4. [TODO / Roadmap](./TODO.md)

## Scope

Current intended scope:

- a programmable symbolic core language runtime
- embeddable rule execution for business / industrial / DSL-style workloads
- interactive use through CLI and REPL
- a future path to richer hosts such as Jupyter or GUI frontends

Explicitly out of scope for the early phases:

- notebook or GUI frontends as the primary product
- full CAS coverage
- graphics / visualization
- broad Wolfram front-end compatibility

## Workspace

The repository is a Cargo workspace with these top-level crates:

- `rum-syntax`: syntax tree, lexer, parser, diagnostics, formatter
- `rum-core`: shared runtime-neutral core values such as `Expr`, `Atom`, and
  `Symbol`
- `rum-runtime`: lowering, evaluation sessions, definitions, scoping, messages,
  matching, tracing, and host-facing request APIs
- `rum-stdlib`: builtin installation for the implemented core-language slice
- `rum-terminal`: terminal-facing rendering for diagnostics, messages, and
  inspection
- `rum-repl`: interactive shell layer
- `rum-cli`: command-line entry point
- `rum-jupyter`: optional Jupyter-facing adapter and transport work

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [TODO / Roadmap](./TODO.md)
- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
- [Formatter Style Notes](./docs/formatter-style.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)

## Development Environment

This repository currently lives inside a larger portable toolchain workspace.
For this local setup, do not assume global `cargo`, `git`, `python`, or `node`
should be used directly. Prefer the parent portable environment:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo test --workspace"
```

The same rule applies to `git`, `cargo fmt`, and other routine development
commands.

## Quick Start

Run the full test suite:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo test --workspace"
```

Run the CLI REPL:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli"
```

Evaluate a single expression:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --eval \"Identity[123]\""
```

Inspect parser output:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --parse \"f[1 2]\""
```

Format an expression:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --format \"f[x(* note *)+y]\""
```

## Current Priorities

Near-term work is concentrated in three areas:

- further tightening evaluator semantics where behavior is still somewhat ad hoc
- continuing to expand the rule / matcher language
- preparing the later full string-language workstream

`rum-jupyter` remains useful and actively maintained, but it is not the primary
driver of the core language roadmap.

## Notes For Evaluators

Two practical points are worth keeping in mind during review:

- `rum-jupyter` is intentionally optional and layered on top of host-facing
  runtime APIs; protocol details should not be treated as evaluator semantics
- some roadmap and audit documents are intentionally candid about remaining
  gaps; they are meant to make review easier, not to imply the current system
  is only a stub
