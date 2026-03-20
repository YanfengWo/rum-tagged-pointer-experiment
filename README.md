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

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)
- [Roadmap](./TODO.md)

## Development environment note

The current repository lives inside a larger portable toolchain workspace.

For the current development environment, do not assume global `cargo`, `git`,
`python`, or `node` should be used directly from inside `rum`. Prefer entering
the portable environment first through the parent workspace tooling, for
example:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo test --workspace"
```

or:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli"
```

This is a temporary adaptation rule for the current local development setup,
not a permanent release requirement. Before packaging or publishing `rum`,
portable-environment wrappers, parent-relative entrypoints, and development-only
tooling assumptions should be reviewed and removed or replaced with normal
project-local release flows.

## Current status

`rum` is no longer just a bootstrap echo runtime. The current workspace now has
a usable early core-language slice:

- exact integer atoms via `num_bigint::BigInt`
- parser and lowering support for rules, patterns, slots, message names,
  tagging, Wolfram-style comments, preserved lexer trivia, source-aware parse
  tooling, and a formatter prototype
- session-based evaluation with ownvalues, upvalues, downvalues, and subvalues
- hold-aware control flow with `Hold*`, `Evaluate`, `Sequence`, `If`, `Which`,
  `Function`, `With`, `Module`, and `Block`
- contexts, package loading, and inspection builtins such as `BeginPackage`,
  `Needs`, `Get`, and `FindFile`
- runtime messages, `Check`, `Quiet`, tracing, resource limits, `Thread`, and
  `Listable`
- a working REPL with multiline input, history persistence, completion, `:parse`,
  `:format`, `:lower`, `:inspect`, and `:messages`

The source tree has also been reorganized around real Rust module boundaries so
that maintenance and incremental compilation stay practical as the runtime
grows.

## Getting started

Run the REPL:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli"
```

Evaluate a single expression:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --eval \"Identity[123456789012345678901234567890]\""
```

Inspect parse output from the CLI:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --parse \"f[1 2]\""
```

Format a single expression from the CLI:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --format \"f[x(* note *)+y]\""
```

Override the formatter width from the CLI:

```powershell
cmd /c "call ..\scripts\activate.cmd && cargo run -p rum-cli -- --format \"f[aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb]\" --line-width 60"
```

The formatter API supports configurable width through
`FormatterOptions::line_width`, and both the CLI and REPL `:format` command now
expose that width override. The current formatter rules and roadmap are
documented in `docs/formatter-style.md`.

Useful REPL commands:

- `:help`
- `:parse <expr>`  (shows syntax plus token/trivia debug output)
- `:format [--line-width N] <expr>`  (prototype canonical formatter; see `docs/formatter-style.md`)
- `:lower <expr>`
- `:complete <prefix>`
- `:history [N]`
- `:inspect <symbol>`
- `:messages <symbol>`
- `:quit`

## Next steps

The next stage is to deepen tooling and tighten the remaining semantic gaps
rather than to "make evaluation exist" from scratch:

1. formatter stabilization, richer multi-span diagnostics, and the later
   pretty-printer consolidation in `rum-syntax`
2. better REPL ergonomics such as trace-oriented debug views
3. continued evaluator cleanup based on the runtime audit and TODO roadmap
4. host-facing API stabilization and later Jupyter integration through a
   separate adapter crate
