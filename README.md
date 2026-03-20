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
- `rum-jupyter`: early protocol-adapter crate for execute/inspect/complete/is-complete kernel flows and wire-level message shaping
- `rum-runtime`: evaluation session and lowering from syntax into core values
- `rum-stdlib`: builtin symbol registration for the core language
- `rum-terminal`: terminal-facing rendering for diagnostics, runtime messages, and inspection output
- `rum-repl`: interactive shell layer
- `rum-cli`: command-line entry point

## Documentation

- [Architecture](./ARCHITECTURE.md)
- [Lowering Canonical Forms](./docs/lowering-canonical-forms.md) for the syntax-to-`Expr` contract, canonical heads, and lowering/runtime validation boundary
- [Runtime Evaluation Audit](./docs/runtime-evaluation-audit.md) for evaluator sequencing and the remaining semantic cleanup work
- [Formatter Style](./docs/formatter-style.md)
- [Code Organization Guidelines](./docs/code-organization-guidelines.md)
- [Roadmap](./TODO.md)

If you are orienting around lowering specifically, the recommended reading path
is: `Architecture -> Lowering Canonical Forms -> Runtime Evaluation Audit ->
TODO`.

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
  tooling, and a trivia-aware document-IR formatter
- session-based evaluation with ownvalues, upvalues, downvalues, and subvalues
- hold-aware control flow with `Hold*`, `Evaluate`, `Sequence`, `If`, `Which`,
  `Function`, `With`, `Module`, and `Block`
- contexts, package loading, and inspection builtins such as `BeginPackage`,
  `Needs`, `Get`, and `FindFile`
- runtime messages, `Check`, `Quiet`, tracing, resource limits, `Thread`, and
  `Listable`
- a working REPL with multiline input, history persistence, completion, `:parse`,
  `:format`, `:lower`, `:inspect`, and `:messages`
- early host-facing request APIs for parse/lower/eval/inspect, with structured
  syntax diagnostics and transport-friendly message/trace payloads
- a stable `SessionConfig` surface for host-side setup of resource limits,
  context defaults, package search paths, and tracing behavior
- a separate terminal-presentation crate so CLI/REPL rendering no longer lives
  inside the protocol-facing session API layer
- an early `rum-jupyter` adapter that maps execute/inspect/complete-style
  kernel requests onto the session API without depending on terminal text
- a first execute-output model in `rum-jupyter` that separates text/plain result
  payloads, stderr-style runtime-message streams, and structured error outputs
- MIME-bundle-backed execute/display outputs and a first shell-envelope layer
  with request ids plus `kernel_info` / `shutdown` lifecycle handling
- inspect-side MIME bundles plus a more Jupyter-like header / parent-header /
  channel envelope, so notebook-facing adapters can stay structured end to end
- a first wire-level split between shell replies and `IOPub` events, including
  `status`, `execute_input`, and output publication plus minimal
  interrupt/shutdown lifecycle handling
- more Jupyter-like reply contents for execute/inspect/complete/kernel-info,
  a first `is_complete` flow, and an explicit wire-message / transport boundary
  for reply plus `IOPub` publication
- inbound wire decoding for signed request messages plus first-pass
  session-id/channel validation and a mock socket transport that splits shell /
  control / `IOPub` traffic
- symmetric inbound decoding for reply / `IOPub` messages, configurable
  signature policy, reply routing-id preservation, and a first mock-transport
  event pump that can execute wire requests end to end
- more Jupyter-like lifecycle details in reply / `IOPub` metadata and content,
  including header dates, lifecycle-aware `status` / `shutdown` payloads, and
  a control-first transport loop that can drain queued requests until idle or
  stop early once shutdown is requested
- a first protocol-side `stdin` flow with `allow_stdin`, `input_request` /
  `input_reply`, pending-input tracking in the adapter, and stdin-aware mock
  event-pump ordering while the kernel is waiting for frontend input
- tighter completion cursor semantics so notebook-facing completion can replace
  the full symbol span even when the cursor sits in the middle of an identifier
- a first Jupyter connection-file model/parser plus bootstrap config that can
  derive endpoint URIs, signature policy, and wire-codec setup without
  hard-coding notebook launch details inside the adapter
- an explicit transport-backend boundary, with the current mock transport
  living behind the same bootstrap/layout seam that a future ZeroMQ backend can
  reuse
- a first local socket-style backend plus frontend handles, so execute/stdin
  flows can be exercised in a more frontend-like way without directly mutating
  mock transport queues
- frontend-style end-to-end execute/stdin round-trip coverage on top of that
  local backend, plus bootstrap-driven kernel usernames in reply / `IOPub`
  headers and a first heartbeat echo path
- a first ZeroMQ-facing `socket/session/poller` seam, including explicit
  ROUTER / PUB / REP socket roles, bind-mode-aware session config, and a
  local socket driver that proves transport replacement can happen beneath the
  adapter rather than inside it
- a first ZeroMQ-shaped transport backend behind that seam, with multipart
  ROUTER / PUB / REP frame translation, bind/connect-aware socket setup, and
  fake-socket regression coverage that exercises execute and heartbeat flows
  without teaching the adapter about a concrete socket library
- a first concrete native-Rust ZeroMQ backend behind that same seam, currently
  powered by the `zeromq` crate and covered by real loopback execute /
  heartbeat round-trip tests
- a real runtime-side input path through `InputString`, so host or notebook
  clients can satisfy input requests without falling back to terminal-only
  behavior
- true request suspension/resume for host-driven input, so execute flows can
  continue from saved continuation state instead of restarting after each
  `input_reply`
- a first typed internal continuation/driver bridge for non-`Expr` helper
  paths, already used by delayed message-template rendering so stdin-driven
  executions can resume cleanly through runtime message generation

The source tree has also been reorganized around real Rust module boundaries so
that maintenance and incremental compilation stay practical as the runtime
grows.

String literals and string atoms are already part of the implemented
core-language slice, but full Wolfram-style string functionality is now treated
as an explicit post-foundation roadmap item. For the first implementation wave,
that work is expected to stay distributed across `rum-syntax`, `rum-core`,
`rum-runtime`, and `rum-stdlib`; a dedicated string crate should only be
introduced later if the subsystem becomes large and stable enough to justify a
new boundary.

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
   cleanup of remaining pretty-printer fit and planning gaps in `rum-syntax`
2. better REPL ergonomics such as trace-oriented debug views
3. continued evaluator cleanup based on the runtime audit and TODO roadmap
4. continued host-facing API hardening and deeper Jupyter/kernel compatibility
   through the separate adapter crate
   this now includes both the first ZeroMQ-shaped seam and a first concrete
   native-Rust backend behind it, with future work focused on transport
   hardening and notebook compatibility
5. after the core/runtime foundation is tighter, a dedicated full-string
   roadmap covering construction, slicing, search, replacement, string
   patterns, and unicode-sensitive behavior
