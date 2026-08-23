# rum

`rum` is an embeddable symbolic term-rewriting language and runtime written in
Rust, built sandbox-first: sessions run under caller-set resource limits
(recursion, iteration, parse depth, elapsed-time timeout), support interrupt and
cancel, can be reset, snapshotted, and restored, and enforce a per-session
capability policy so default sandboxed evaluation is deterministic and a host
can evaluate untrusted scripts safely.

The surface syntax belongs to the Wolfram Language family of notations; the
evaluator implements `rum`'s own documented semantics. See
[DISCLAIMER.md](./DISCLAIMER.md) for scope, trademarks, and project history —
`rum` is an independent project and makes no compatibility claims.

The project focuses on the language kernel rather than a notebook system or a
computer-algebra stack:

- symbolic expression trees with exact rational arithmetic
- definitions, rules, pattern matching, and rewriting
- scoping and held evaluation
- parser / lowering / formatter infrastructure
- embeddable session APIs with structured host I/O
- CLI / REPL tooling and an optional Jupyter adapter

## Status

The core language slice is substantial and tested:

- `rum-syntax` has a lexer, parser, spans, source-aware diagnostics, preserved
  trivia, and a document-IR formatter
- `rum-core` and `rum-runtime` support exact integers and rationals, canonical
  lowering, sessions, contexts, package loading,
  ownvalues/upvalues/downvalues/subvalues, hold-aware evaluation, threading,
  tracing, and resource limits
- rules and patterns drive `MatchQ`, `Cases`, `DeleteCases`, `ReplaceAll`,
  `ReplaceRepeated`, `Replace`, `ReplaceList`, and `ReplaceAt`
- `rum-stdlib` installs the builtin catalog: arithmetic, number theory,
  combinatorics, structure and list operations, associations, functional
  operators, strings, predicates, logic, and control flow
- `rum-host` is the Rust embedding boundary: typed outcomes, interrupt/cancel,
  monotonic elapsed-time timeout, panic isolation with session poisoning and recovery,
  `reset()`, snapshot / restore-from-bytes, and a sandbox capability policy
  (`HostConfig::sandboxed()`)
- `rum-cli` and `rum-repl` are usable developer tools; `rum-jupyter` is an
  optional host-backed adapter

## Scope

Current intended scope:

- a programmable symbolic core-language runtime
- embeddable rule execution for business / industrial / DSL-style workloads
- sandboxed evaluation of untrusted scripts inside Rust hosts
- interactive use through CLI and REPL

Explicitly out of scope:

- notebook or GUI frontends as the primary product
- full CAS coverage; floating-point numerics (exact rationals only, by design)
- graphics / visualization
- compatibility with, or implementation of, other vendors' products and
  protocols

## Sealed Packages — real encapsulation, one mode

Where `rum` diverges from its notation-family origins, it does so
deliberately, for capabilities the convention cannot offer. The flagship
example is `DefinePackage`, the sealed package form.

In the notation family's convention, context-based privacy is a naming
convention: a fully qualified name always resolves, so a "private" symbol
stops accidents, not callers who spell the name. `rum` owns its kernel, so
`DefinePackage` makes the private tier a real boundary: every undeclared
symbol a source unit binds is minted with a fresh internal identity and the
unit body is rewritten to reference it. The name is never entered in the
resolution tables — spelling it from outside, directly or via `Symbol` /
`ToExpression` / string concatenation, yields a distinct ordinary symbol
with no path to the sealed binding. Introspection is capability-gated:
sandboxed sessions see no trace that a private even exists, while trusted
sessions keep a full debug view.

Encapsulation this strong usually costs persistence — a boundary held by
unforgeable identities cannot be written into a text image, so systems in
this position historically split into a "strict" mode (real boundary, state
that cannot be checkpointed) and a "stable" mode (checkpointable state,
boundary demoted back to convention). `rum` refuses the split: a snapshot
records each sealed package as its registration form and restore re-runs
the registration, minting fresh identities; sealed values that leaked into
ordinary variables are re-linked to those fresh identities by a
restore-only marker, and a snapshot fails loudly rather than emit a sealed
reference it cannot re-link. One mode carries both properties: a boundary
stronger than the convention, and checkpoint/restore that keeps it sealed
across every round trip.

See the language guide's Sealed Packages section for the full contract.

## Workspace

The repository is a Cargo workspace with these crates:

- `rum-syntax`: syntax tree, lexer, parser, diagnostics, formatter
- `rum-core`: shared runtime-neutral core values such as `Expr`, `Atom`, and
  `Symbol`
- `rum-runtime`: lowering, evaluation sessions, definitions, scoping, messages,
  matching, tracing, and host-facing request APIs
- `rum-stdlib`: the standard library — every builtin family's implementation
  and catalog specs, installed into sessions via `install_core`
- `rum-host`: narrow Rust embedding facade over `rum-runtime` for host
  sessions, requests, structured outcomes, and resumable input
- `rum-terminal`: terminal-facing rendering for diagnostics, messages, and
  inspection
- `rum-repl`: interactive shell layer
- `rum-cli`: command-line entry point
- `rum-jupyter`: optional Jupyter-facing adapter and transport work

## Documentation

- [Full documentation index](./docs/README.md)
- [Language guide](./docs/guide/)
- [Generated builtin reference](./docs/reference/builtins.md)
- [Disclaimer, Trademarks, and Project History](./DISCLAIMER.md)
- [Architecture](./ARCHITECTURE.md)
- [TODO / Roadmap](./TODO.md)
- [Progress Log](./PROGRESS.md)
- [Testing Strategy](./docs/testing-strategy.md)
- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
- [v0.1 Acceptance Boundary](./docs/v0.1-acceptance-boundary.md)
- [Host Adapter Plan](./docs/host-adapter-plan.md)
- [Host Embedding and Hardening](./docs/host-embedding-and-hardening.md)
- [Formatter Style Notes](./docs/formatter-style.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)
- [Jupyter Console Manual](./docs/jupyter-console-manual.md)

The [language guide](./docs/guide/) and generated
[builtin reference](./docs/reference/builtins.md) are part of the current
documentation contract.

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

## Development

- The workspace targets Rust `1.85` (edition 2024); build and test with
  `cargo fmt --check` and `cargo test --workspace`.
- The contribution workflow, the permanent testing policy, and the hot-path A/B
  rule are in [CONTRIBUTING.md](./CONTRIBUTING.md). The helper scripts are
  documented in [scripts/README.md](./scripts/README.md), and agent-facing
  operational notes are in [AGENTS.md](./AGENTS.md).

## Direction

The `v0.1.0` host and language boundary is published. Current maintenance work
is a measured, safe-Rust audit of avoidable superlinear algorithms in the
runtime and standard library. The P0 tranche is complete; the remaining P1
items and deliberately deferred work are tracked in [TODO.md](./TODO.md).

Broader polynomial factorization is intentionally deferred until the project
is ready to add the finite-field polynomial machinery needed for a complete
algorithm. Downstream Python bindings and showcase applications remain future
projects on top of the stable `rum-host` facade rather than kernel work.

## Contributing, security, and license

- Contributions are welcome under the terms in [CONTRIBUTING.md](./CONTRIBUTING.md);
  participation is governed by the [Code of Conduct](./CODE_OF_CONDUCT.md).
- To report a security issue, and for the sandbox threat model, see
  [SECURITY.md](./SECURITY.md).
- Release notes are in [CHANGELOG.md](./CHANGELOG.md).
- `rum` is MIT-licensed ([LICENSE](./LICENSE)); scope, trademarks, and project
  history are in [DISCLAIMER.md](./DISCLAIMER.md).
