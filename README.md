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
- `rum-jupyter` is an early adapter layer being migrated onto `rum-host`, not
  part of the evaluator core

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
- `rum-host`: narrow Rust embedding facade over `rum-runtime` for host
  sessions, requests, structured outcomes, and resumable input
- `rum-terminal`: terminal-facing rendering for diagnostics, messages, and
  inspection
- `rum-repl`: interactive shell layer
- `rum-cli`: command-line entry point
- `rum-jupyter`: optional Jupyter-facing adapter and transport work
- `rum-difftest`: optional differential/conformance runner and fixture work

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [TODO / Roadmap](./TODO.md)
- [Progress Log](./PROGRESS.md)
- [Core-Language Support Matrix](./docs/core-language-support-matrix.md)
- [Testing Strategy](./docs/testing-strategy.md)
- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
- [Jupyter Host Migration Audit](./docs/jupyter-host-migration-audit.md)
- [`mma-hol` Smoke Driver And Snapshot Audit](./docs/mma-hol-smoke-and-snapshot.md)
- [Formatter Style Notes](./docs/formatter-style.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)
- [Jupyter Console Manual Smoke](./docs/jupyter-console-manual.md)

## Application Directions

The project is not currently maintaining detailed showcase-project plans.
Future examples should be driven by concrete downstream needs rather than by
speculative roadmap breadth.

Plausible application directions include:

- symbolic policy, validation, and rewrite layers for AI, data, or workflow
  tooling
- compiler, runtime, or DSL semantics prototypes where a Rust host owns system
  infrastructure and `rum` owns symbolic rules, diagnostics, and explanation

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

Run the minimal Rust host embedding example:

```bash
cargo run -p rum-host --example basic_embedding
```

## Current Priorities

Near-term work is concentrated in these areas:

- hardening the new `rum-host` embedding boundary and migrating `rum-jupyter`
  request handling onto that facade
- using `mma-hol` as the preferred real-project pressure test for the
  supported Wolfram-style language surface
- further tightening evaluator semantics where behavior is still somewhat ad hoc
- continuing to expand the rule / matcher language
- keeping the `CL-01` through `CL-09` mainlines documented as closed while
  promoting only concrete downstream blockers from deferred long-tail notes
  back into scheduled work
- broadening conformance, regression, and adapter-focused test coverage

`rum-jupyter` remains useful and actively maintained as a basic Notebook tech
preview and `rum-host` contract validator, but it is not the primary driver of
the core language roadmap.

## Notes For Evaluators

Two practical points are worth keeping in mind during review:

- `rum-jupyter` is intentionally optional and should be layered through
  `rum-host`; protocol details should not be treated as evaluator semantics
- some roadmap and audit documents are intentionally candid about remaining
  gaps; they are meant to make review easier, not to imply the current system
  is only a stub
