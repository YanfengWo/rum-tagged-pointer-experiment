# Agent Guide

This file is the first-stop guide for coding agents working in this
repository. Keep it short and link to the canonical documents instead of
duplicating them.

## Project Snapshot

`rum` is an experimental Rust implementation of a Mathematica /
Wolfram-style symbolic core language runtime. The early product target is a
programmable symbolic runtime and embeddable rule engine, not a notebook UI,
full CAS, graphics system, or broad Wolfram front-end clone.

The workspace is organized as Rust crates:

- `rum-syntax`: syntax tree, lexer, parser, diagnostics, formatter
- `rum-core`: shared runtime-neutral core values
- `rum-runtime`: lowering, evaluation, definitions, scoping, messages,
  matching, tracing, and host-facing request APIs
- `rum-stdlib`: builtin installation for the implemented language slice
- `rum-host`: narrow Rust embedding facade over runtime sessions, requests,
  structured outcomes, and resumable input
- `rum-terminal`, `rum-repl`, `rum-cli`: terminal and CLI layers
- `rum-jupyter`: optional Jupyter adapter and transport work
- `rum-difftest`: optional differential/conformance runner and fixtures

## Reading Path

Before substantial work, read the smallest set needed for the task:

- [README.md](./README.md): scope, workspace, development environment
- [ARCHITECTURE.md](./ARCHITECTURE.md): architecture boundaries and design
  commitments
- [TODO.md](./TODO.md): active roadmap and technical debt
- [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md):
  current stabilization and public-contract boundary
- [docs/core-language-support-matrix.md](./docs/core-language-support-matrix.md):
  supported Wolfram-style language surface
- [docs/lowering-canonical-forms.md](./docs/lowering-canonical-forms.md):
  syntax lowering contracts
- [docs/runtime-evaluation-audit.md](./docs/runtime-evaluation-audit.md):
  evaluator invariants and current semantic boundaries
- [docs/testing-strategy.md](./docs/testing-strategy.md): test layering,
  Wolfram conformance lanes, and same-name compatibility rules
- [docs/mma-hol-smoke-and-snapshot.md](./docs/mma-hol-smoke-and-snapshot.md):
  bounded `mma-hol` smoke driver and stable snapshot gap audit
- [docs/code-organization-guidelines.md](./docs/code-organization-guidelines.md):
  file-size and module-organization rules

`docs/next-session-handoff.md` is intentionally empty unless an active handoff
is being prepared.

## Current Downstream Lane

The active `mma-hol` pressure test now reaches the clean-clone core through
Parser/Printer path. The canonical status lives in
[PROGRESS.md](./PROGRESS.md) and [TODO.md](./TODO.md).

Use the fixed clean clone at `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`
for current probes unless the user asks to refresh from the live sibling tree.
Preload lower HOL files before higher layers: `ErrorUtil.wl`, `Types.wl`,
`Terms.wl`, `Kernel.wl`, `Bootstrap.wl`, `Equal.wl`, then `Bool.wl`,
`Drule.wl`, `Tactics.wl`, `Printer.wl`, and `Parser.wl` as needed.

Current clean-clone smoke results are: `bool_tests.wl` `List[73, 0]`,
`drule_tests.wl` `List[43, 0]`, `tactics_tests.wl` `List[49, 0]`, and
`printer_tests.wl` + `parser_tests.wl` `List[126, 0]`. Use
`scripts/mma-hol-bounded-smoke.sh` for bounded diagnostic load/test probes. The
first stdlib/auto failure beyond Parser/Printer is still the `PropTaut` /
`Simp` path behind `Pair.wl`.

Stable snapshot partial validation is in place on the same clone:
`tests/build_snapshot.wls Parser` writes a Stable `bootstrap.mx`,
`tests/run_fast.wls parser` exits `0` with visible parser logs, and
`tests/extend_snapshot.wls auto/PropTaut.wl` re-dumps quietly after the
`$Packages` restore cleanup. Use
`scripts/mma-hol-bounded-smoke.sh --mode snapshot` for this lane. `SelectFirst`,
loaded-file `Print` propagation through `Get`, and CLI print preservation
before `Exit[n]` are implemented for this workflow. Full `build_snapshot.wls`
remains blocked by the `Pair.wl` / `Simp` performance gate. The fixed probe
clone has the `extend_snapshot.wls` `$Packages` cleanup; carry that patch
forward before replacing it with a refreshed live-tree clone.

The PropTaut performance lane has been reopened by external review and a
follow-up same-bit stability-invalidation experiment. The retained current
frontier is the PropTaut distribution probe at about `12.73-12.88s` in release
versus the local WolframKernel internal timing of about `3.80s`;
`proptaut_tests.wl` is green at about `239s`, but `stdlib/Pair.wl` still
exceeds a `600s` load budget and remains the next downstream gate. Do not treat
the earlier cross-engine `<2x` target as an active commitment.

Keep performance work probe-first: identify the hot path, make the smallest
targeted change, then rerun the same timing smoke to reject negative
optimizations. Retained work includes symbol interning with dense ids,
id-hashed definition/attribute/binding maps, dependency-aware stability-cache
invalidation with a rewritten dirty bloom for repeated same-bit writes,
mimalloc in `rum-cli`, a message-symbol existence index, and the shared
call-tree `Expr` representation. Pair/Simp runtime stability cuts now reduce
the local user-rule-only mkPair probe to about `40.61s` and `basicSimpset[]`
to about `20.27s`, but the public warm mkPair probe still times out at `240s`
and the `Pair.wl` gate remains open.
Exact per-symbol/per-bloom-bit version
snapshots, skipping dirty-related call-cache entries, whole-expression exact
normal-form caching, symbol fingerprint hashing, shallow definition matcher
metadata, a one-entry call-stability MRU, and heavier LTO/codegen flag probes
were measured and rejected or left as optional local knobs rather than default
commitments. The next evidence-backed frontier is larger representation work:
single-allocation call nodes, inline evaluated/stable stamps, and borrowed
argument preparation.

For long-running probes or commands that may spawn children, do not start them
bare. Use `scripts/run-bounded.py --timeout SECONDS -- <command> ...` or a
repo script that delegates to it. The wrapper starts the command in a new
process group and kills the whole group on timeout or interruption, which
avoids orphaned `wolframscript`, `WolframKernel`, server, or cargo child
processes that later require manual cleanup.

## Development Commands

Use the local Rust toolchain declared by the workspace. The current expected
Rust version is `1.85.x`.

Common checks:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo test --workspace
```

Use focused tests while iterating, then run the broader relevant suite before
committing. For runtime semantics work, `cargo test -p rum-runtime` is the
default minimum.

## Wolfram Oracle Policy

For semantic spot checks against Wolfram behavior, prefer a local kernel binary
such as:

```bash
/Applications/Wolfram.app/Contents/MacOS/WolframKernel -noprompt
```

Do not treat MCP-wrapped or connector-hosted Mathematica sessions as the
baseline oracle. They are useful for tiny checks but may impose security,
time, or memory limits.

Do not commit oracle-generated outputs. If differential fixtures are needed,
put them through the `rum-difftest` workflow and document deliberate
divergences.

Sandboxed agent shells may not see loopback proxies or network configuration
that works in a normal shell. Networked `cargo` commands or local oracle
commands may need explicit unsandboxed execution approval.

## Code Organization Rules

Follow [docs/code-organization-guidelines.md](./docs/code-organization-guidelines.md).
The practical thresholds are:

- under `10k` chars: preferred
- `10k` to `20k`: warning range
- `20k` to `35k`: split-plan range
- over `35k`: architecture debt unless explicitly documented

Prefer real module boundaries over arbitrary cuts. Split by responsibility:
syntax vs lowering, evaluation vs matching, planning/normalization vs
rendering/mutation, transport vs codec vs backend, and so on.

When editing a file already over threshold, avoid making it worse unless the
change is urgent and the split plan is recorded in `TODO.md`.

## Testing And Conformance

Follow [docs/testing-strategy.md](./docs/testing-strategy.md). In short,
choose the lowest layer that proves the contract under test.

Keep tests near the crate that owns the behavior:

- `rum-runtime` tests own runtime/session semantics and same-name language
  behavior.
- `rum-difftest` owns Wolfram-oracle-comparable fixture lanes and imported
  seed packs.
- Adapter/protocol behavior should stay in the adapter crate tests, not in
  Wolfram diff fixtures.

For same-name Wolfram-language features, aim for Wolfram-compatible semantics
unless the support matrix or divergence docs say otherwise.

## Working Rules

- Do not edit `WLBuiltin/` unless explicitly asked; it is intentionally outside
  the active roadmap.
- Keep `TODO.md` and `docs/core-language-support-matrix.md` aligned when a
  feature moves from planned to implemented or when a limitation is discovered.
- Do not check in generated build artifacts, oracle transcripts, or temporary
  probe files.
- Preserve user changes in a dirty worktree. Never revert unrelated edits just
  to make a patch easier.
- Prefer small, reviewable commits with messages that state the semantic or
  organizational change.
