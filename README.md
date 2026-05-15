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
- rules and patterns already drive `MatchQ`, `Cases`, `DeleteCases`,
  `ReplaceAll`, `ReplaceRepeated`, and current first passes of `Replace`,
  `ReplaceList`, and `ReplaceAt`
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
- `rum-difftest`: optional differential/conformance runner and fixture work

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [TODO / Roadmap](./TODO.md)
- [Core-Language Support Matrix](./docs/core-language-support-matrix.md)
- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
- [Formatter Style Notes](./docs/formatter-style.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)
- [Jupyter Console Manual Smoke](./docs/jupyter-console-manual.md)

## Future Example Plans

The `examples/` directory currently contains planning documents for future
showcase projects. These are not active implementation targets yet; they should
wait until the core runtime and host-adapter APIs are stable enough.

- [Python AI Policy](./examples/python-ai-policy/PLAN.md): a future
  Python-hosted AI-infrastructure policy / rewrite / validation example built
  around a narrow canonical request IR and thin project adapters.
- [Rust Wasm Prototype](./examples/rust-wasm-proto/PLAN.md): a future
  Rust-hosted WebAssembly validation and semantics prototype where Rust owns
  host infrastructure and `rum` owns symbolic validation, step rules,
  diagnostics, and explanation.

## Development Environment

Primary day-to-day development should happen in a native local workspace for
the current machine, using the locally installed toolchain directly (`git`,
`cargo`, `rustc`, etc.).

The current workspace expects Rust `1.85.x` as declared in `Cargo.toml`.

On Windows, prefer a real WSL checkout over the old `C:\...` / `/mnt/c/...`
portable-wrapper flow. On macOS and Linux, use the normal local checkout and
native toolchain directly.

```bash
cargo test --workspace
```

The same rule applies to `git`, `cargo fmt`, and other routine commands.

For Wolfram-oracle checks, prefer a local kernel binary such as
`WolframKernel` or `MathKernel`. Connector-based or MCP-wrapped Mathematica
instances can still be useful for tiny spot checks, but they may impose extra
security, time, or memory limits and should not be treated as the primary
semantic oracle.

When running through sandboxed agent tooling, loopback proxies such as
`127.0.0.1:10808` may be unavailable even when they work fine in a normal
shell. Networked `cargo` or oracle commands may therefore need unsandboxed
execution on this machine.

## Quick Start

Run the full test suite:

```bash
cargo test --workspace
```

Run the CLI REPL:

```bash
cargo run -p rum-cli
```

Evaluate a single expression:

```bash
cargo run -p rum-cli -- --eval "Identity[123]"
```

Inspect parser output:

```bash
cargo run -p rum-cli -- --parse "f[1 2]"
```

Format an expression:

```bash
cargo run -p rum-cli -- --format "f[x(* note *)+y]"
```

## Current Priorities

Near-term work is concentrated in four areas:

- further tightening evaluator semantics where behavior is still somewhat ad hoc
- continuing to expand the rule / matcher language
- finishing the active `Wave 2` backlog, with emphasis on the remaining
  `CL-05` keyed-data long tail plus any follow-on matcher / traversal cleanup
- splitting oversized runtime and adapter modules back into healthier boundaries

`rum-jupyter` remains useful and actively maintained, but it is not the primary
driver of the core language roadmap.

## Notes For Evaluators

Two practical points are worth keeping in mind during review:

- `rum-jupyter` is intentionally optional and layered on top of host-facing
  runtime APIs; protocol details should not be treated as evaluator semantics
- some roadmap and audit documents are intentionally candid about remaining
  gaps; they are meant to make review easier, not to imply the current system
  is only a stub
