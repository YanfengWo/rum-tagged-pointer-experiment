# rum

`rum` is an experimental Rust implementation of a Mathematica / Wolfram-style
core language runtime, with an emphasis on symbolic evaluation, rules,
patterns, scoping, and embeddable rule execution for enterprise and industrial
use cases.

The project intentionally starts with the core language layer:

- expression trees and symbol handling
- exact integers via `BigInt`
- syntax and parsing
- evaluation and rule application
- scoping and contexts
- a REPL for interactive use

Out of scope for the early phases:

- notebook or GUI tooling
- computer algebra and broader math systems
- graphics and visualization
- Wolfram front end compatibility

## Workspace

The repository is organized as a Cargo workspace:

- `rum-syntax`: syntax tree and parser
- `rum-core`: shared core data structures such as `Expr` and `Symbol`
- `rum-runtime`: evaluation session and lowering from syntax into core values
- `rum-stdlib`: builtin symbol registration for the core language
- `rum-repl`: interactive shell layer
- `rum-cli`: command-line entry point

## Current status

This bootstrap commit focuses on project structure and a thin executable slice:

- exact integer atoms are already represented with `num_bigint::BigInt`
- the parser can currently handle integers, symbols, strings, and simple calls
- evaluation is intentionally minimal and mostly echoes lowered expressions
- the REPL is present now so we can evolve the runtime interactively

## Getting started

Run the REPL:

```powershell
cargo run -p rum-cli
```

Evaluate a single expression:

```powershell
cargo run -p rum-cli -- --eval "Identity[123456789012345678901234567890]"
```

Useful REPL commands:

- `:help`
- `:inspect <symbol>`
- `:quit`

## Next steps

The immediate roadmap is to move from the bootstrap runtime toward real
Mathematica-style core semantics:

1. richer parser coverage for patterns, rules, and operators
2. definition tables and evaluation control
3. pattern matching and rule application
4. scoping constructs and contexts
5. better REPL ergonomics and eventual Jupyter integration via a separate
   adapter crate
