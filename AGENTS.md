# Agent Guide

This file is the first-stop guide for coding agents working in this
repository. Keep it short and link to the canonical documents instead of
duplicating them.

`CLAUDE.md` is a separate document for the supervising Claude director (role
split, diagnosis method, the Codex task-brief workflow, and the authoritative
gate). An executing agent does not need it; follow the brief you were given
plus this guide.

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
- [docs/proptaut-performance-audit.md](./docs/proptaut-performance-audit.md):
  performance-diagnosis method and pitfalls for `PropTaut` / `mma-hol`
  work; required before Pair/Simp or PropTaut performance investigations
- [docs/representation-rewrite-plan.md](./docs/representation-rewrite-plan.md):
  **the active performance plan** (Layer A/B representation rewrite). Phase 0 +
  Layer A (A1-A4) are complete and gated as of 2026-06-22; see the
  `docs/next-session-handoff.md` top block and the `PROGRESS.md`
  `2026-06-21`/`2026-06-22` entries.
- [docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md):
  earlier staged plan (Stages 0-8); superseded by the representation-rewrite
  plan for the active line, kept for history
- [docs/performance-stage0-baseline.md](./docs/performance-stage0-baseline.md):
  canonical Stage 0 core-language microbench suite and
  `scripts/core-language-microbench.sh` runner for performance-rewrite A/B
  checks
- [docs/code-organization-guidelines.md](./docs/code-organization-guidelines.md):
  file-size and module-organization rules

`docs/next-session-handoff.md` carries an active handoff when one is being
prepared; as of 2026-06-22 its top block is the current state (Layer A complete,
the unified `34x` model, the next target). Read that block before performance
work.

## Current Downstream Lane

The active `mma-hol` pressure test now reaches the clean-clone core through
Parser/Printer path. The canonical status lives in
[PROGRESS.md](./PROGRESS.md) and [TODO.md](./TODO.md).

Use the persistent acceptance clone at
`/Users/fft/Developments/mma-hol-acceptance` (latest published `mma-hol`,
HEAD `200f13a`, the completed/launched project — a large advance past the
earlier `698e3d0` probe snapshot). This replaces the old
`/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` clone, which `/tmp` purging
decayed; keep this clone pristine (diagnostic edits only in throwaway scratch
copies, never here). Re-`git pull` it if the user says the standard moved.
Preload lower HOL files before higher layers: `ErrorUtil.wl`, `Types.wl`,
`Terms.wl`, `Kernel.wl`, `Bootstrap.wl`, `Equal.wl`, then `Bool.wl`,
`Drule.wl`, `Tactics.wl`, `Printer.wl`, and `Parser.wl` as needed.

Treat `mma-hol` as an unmodified downstream acceptance target. The task is to
make clean `mma-hol` run under rum, not to patch `mma-hol` into a different
test standard. If unmodified `mma-hol` works under WolframKernel but fails,
times out, or restores incorrectly under rum, the working assumption is a rum
semantic, performance, snapshot, CLI, or harness gap that must be debugged in
rum. Local edits to `mma-hol` are allowed only as temporary diagnostic probes
or bug reports for the `mma-hol` developer; do not present them as the rum-side
fix or the acceptance path.

This is a hard line: editing the standard to pass the standard voids the
result. Keep the canonical clean clone pristine; do diagnostic edits only in
throwaway scratch clones, never in the reference clone, and never leave a
modified reference clone behind. Two real violations to learn from, both the
same wrong-layer mistake: (1) a head-indexed dispatch + cache patch to
`auto/Simp.wl` to "cross" the Pair gate — the real fix is rum's failed-rule
dispatch cost (handoff Task 2d); (2) adding `$Packages = Union[$Packages,
holPublic]` to `tests/extend_snapshot.wls` so a restored snapshot resolves —
the real fix is rum's `DumpSave`/`Get` preserving `$Packages` across a
restore so the unmodified harness works. In both cases the WolframKernel run
of the unmodified file is the proof that the gap is rum's.

Current clean-clone smoke results are: `bool_tests.wl` `List[73, 0]`,
`drule_tests.wl` `List[43, 0]`, `tactics_tests.wl` `List[49, 0]`, and
`printer_tests.wl` + `parser_tests.wl` `List[126, 0]`. Use
`scripts/mma-hol-bounded-smoke.sh` for bounded diagnostic load/test probes. The
first stdlib/auto failure beyond Parser/Printer is still the `PropTaut` /
`Simp` path behind `Pair.wl`.

Stable snapshot partial validation is in place on the same clone:
`tests/build_snapshot.wls Parser` writes a Stable `bootstrap.mx`,
`tests/run_fast.wls parser` exits `0` with visible parser logs, and
`tests/extend_snapshot.wls auto/PropTaut.wl` re-dumps quietly on the current
rum textual `DumpSave` restore path, which now auto-registers restored public
package contexts in `$Packages` and `$ContextPath`. This snapshot lane was
also revalidated on a refreshed unmodified clone at
`/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` (HEAD `51bbe9e`). Use
`scripts/mma-hol-bounded-smoke.sh --mode snapshot` for this lane. `SelectFirst`,
loaded-file `Print` propagation through `Get`, and CLI print preservation
before `Exit[n]` are implemented for this workflow. Full `build_snapshot.wls`
remains blocked by the `Pair.wl` / `Simp` performance gate. Do not carry the
old `extend_snapshot.wls` `$Packages` workaround into upstream `mma-hol`; it
masked a rum restore divergence that is now fixed in runtime.

The Jupyter harness has an isolated `jupyter_home/` path through
`scripts/rum-jupyter-home.sh`. Use `init` to install the local venv and
kernelspec, `kernel-smoke` for real `jupyter_client`
lifecycle/stdin/protocol/unsupported-message keepalive coverage, and
`browser-smoke` for JupyterLab plus headless Chromium notebook execution.
Unsupported keepalive means empty `comm_info_reply`, ignored comm open/msg/close,
explicit unsupported `debug_reply`, and ignored unknown frontend probes; it does
not implement widgets/comm state or the debug protocol. The real Jupyter and
native ZeroMQ smokes need loopback-port access outside the Codex sandbox, but
their files remain under `jupyter_home/`. Robust interrupt/cancel and rich
display behavior remain outside the current v0.1 contract.

The current performance mainline is the staged kernel rewrite plan in
[docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md).
The target is WolframKernel parity or better on the supported `mma-hol` slice,
not merely "good enough" interpreter performance. The current route is
bottom-up: measure the smallest shared Wolfram-language statements in rum and
WolframKernel, optimize the first primitive with a clear `4-5x` ratio gap, and
only then move to larger `mma-hol` statement groups. Use
`scripts/core-language-microbench.sh` as the active Stage 0 entry point.
Pair/Simp probes remain downstream acceptance checks, not the default place to
discover the next optimization.

Do not descend into `Pair.wl` / `Simp` phase trees while lower-tier core labels
still show unexplained constant-factor gaps. If a real `mma-hol` statement
exposes a new cost, reduce it back to a core microbench label or add a new
minimal label before changing algorithms. Treat semantic
failed-conversion/no-change caches as optional rum-specific shortcuts, not as
the inferred WolframKernel mechanism.

The PropTaut performance lane has been reopened by external review and a
follow-up same-bit stability-invalidation experiment. The retained current
frontier is the PropTaut distribution probe at about `12.73-12.88s` in release
versus the local WolframKernel internal timing of about `3.80s`;
`proptaut_tests.wl` is green at about `239s`, but `stdlib/Pair.wl` still
exceeds a `600s` load budget and remains the next downstream gate. Do not treat
the earlier cross-engine `<2x` target as an active commitment.

Keep performance work probe-first but bottom-up: identify the minimal hot
primitive, make the smallest targeted change, then rerun the same
microbenchmark label and neighboring labels to reject negative optimizations.
Retained work includes symbol interning with dense ids, id-hashed
definition/attribute/binding maps, dependency-aware stability-cache
invalidation with a rewritten dirty bloom for repeated same-bit writes,
mimalloc in `rum-cli`, a message-symbol existence index, the shared call-tree
`Expr` representation, and the exact atomic `Catch` tag fast path. Historical
Pair/Simp probes remain useful acceptance evidence and negative-result history,
but they no longer define the next optimization by default.

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

**Safe Rust is a hard convention.** `rum-core` and `rum-runtime` are
ZERO-`unsafe` (verified; the only `unsafe` in the repo is the off-by-default
alloc-count `GlobalAlloc` in `rum-cli`). Do not introduce `unsafe` into the
kernel crates — there is almost always a safe equivalent at the same cost
(e.g. `Option<Arc<…>>` niche-packs to `Arc<…>`'s size and gives a safe `.take()`
for a manual `Drop`). If you believe `unsafe` is unavoidable, STOP and surface
it for a decision rather than shipping it.

**Hot-path changes are measured, not asserted.** A change touching an evaluation
hot path (eval/dispatch/matcher, `Expr`/`SharedExpr`/`SharedArgs` construction or
`Drop`, candidate-value lookup) must be gated with a
`scripts/core-language-microbench.sh` base-vs-new A/B — a "no default-path cost"
intention is not evidence; an O(N) cost hidden inside a "fast path" only shows up
when measured.

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
