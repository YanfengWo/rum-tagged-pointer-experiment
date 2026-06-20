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
- [docs/proptaut-performance-audit.md](./docs/proptaut-performance-audit.md):
  performance-diagnosis method and pitfalls for `PropTaut` / `mma-hol`
  work; required before Pair/Simp or PropTaut performance investigations
- [docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md):
  staged plan for reaching WolframKernel parity or better on the supported
  `mma-hol` target slice
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

Performance investigations must stay anchored to the current acceptance root.
For `mma-hol`, that means the unmodified `Pair.wl` load or the warm public
mkPair probe that reproduces the gate, not whichever smaller probe currently
has readable profile output. Before drilling into any child function, build or
reuse a closed phase tree for the parent target: the measured child phases must
sum close to the parent wall time, the largest child must be named with a
percentage of the parent, and the same target must have a no-profile baseline.
Do not descend into a child merely because it is slow in isolation, differs from
WolframKernel, or appears high in an inclusive ranking. If a child is not shown
to dominate the current root, label it a side lane and return to the root phase
tree. The June 2026 Pair/Simp detour into one-case `EXCLUDEDMIDDLE[p]` is the
failure mode to avoid: that probe was real but was not proven to own the
hundreds-of-seconds `Pair.wl` gate.

The current performance mainline is the staged kernel rewrite plan in
[docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md).
The target is WolframKernel parity or better on the supported `mma-hol` slice,
not merely "good enough" interpreter performance. The plan is to lower the
ordinary failure-path constants through symbol identity, expression
representation, borrowed evaluator frames, matcher/dispatch plans,
conversion-chain plans, and allocator strategy before considering bytecode or
JIT work. Treat semantic failed-conversion/no-change caches as optional
rum-specific shortcuts, not as the inferred WolframKernel mechanism.

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
call-tree `Expr` representation. Pair/Simp runtime stability cuts reduced the
local user-rule-only mkPair probe to about `40.61s` and `basicSimpset[]` to
about `20.27s`. The 2026-06-19 runtime sharing cut retained an exact atomic
`Catch` tag fast path and changed `normalize_match_expr` to preserve shared
call trees when normalization is a no-op; on the unmodified clean clone,
`basic-only-pair4` improved from about `5.68-6.36s` to `2.23s`,
`pair4-stock-repeat8` from `60.76s` to `18.30s`, and the first `mkPairInj`
`CONVRULE` root from about `564s` to `307s`. The `Pair.wl` gate remains open.
Exact per-symbol/per-bloom-bit version
snapshots, skipping dirty-related call-cache entries, whole-expression exact
normal-form caching, symbol fingerprint hashing, shallow definition matcher
metadata, a one-entry call-stability MRU, and heavier LTO/codegen flag probes
were measured and rejected or left as optional local knobs rather than default
commitments. Task 2b also rejects call-cache size-cap tuning and wider
dirty-bloom saturation thresholds for the warm public mkPair shape:
miss-reason counters point mostly at absent pointer-keyed call entries, which
means the conversion chain is building fresh call trees. Copy-based
`auto/Simp.wl` prototypes are retained as diagnostic evidence only: flat
converter combination cut the first `mkPairInj` gate to about `440s`, atomic
leaf no-op caching cut it to about `253s`, and the combined prototype loaded
full `Pair.wl` in about `183s` with theorem markers true. Do not upstream or
depend on those downstream rewrites. The next Pair/Simp work should stay
rum-side: either continue small representation/sharing cuts with before/after
checks on `basic-only-pair4`, `pair4-stock-repeat8`, and the first `CONVRULE`
root, or split the remaining closed root phases before changing algorithms. A
temporary opt-in
shared-subtree stamp prototype proved that shared subtrees can hit after the
side cache is cleared, but it was reverted before commit: the uncommitted
carrier regressed the default PropTaut distribution probe from about `10.2s`
to about `11.6s` while failing to move the Pair/Simp gate. Keep the negative
result and do not restore the `SharedArgs` stamp slot unless it is truly
zero-cost when disabled.
Follow-up phase probes found that the final opened-body target can be
single-digit seconds after preheat, while basic and congruence preheats are
measurable `PropTaut` costs. Those probes are valid side evidence only. They do
not supersede the older root-level Pair evidence: dependency-preloaded
`Get["stdlib/Pair.wl"]` spent about `564s` in the Pair body versus about `40s`
in WolframKernel, and the warm public mkPair probe still times out after normal
preheat. Smaller single-lemma probes show `impCongLemma[]`, `andCongLemma[]`,
and `orCongLemma[]` are same-shaped `PropTaut` runs; no-profile
`impCongLemma[]` is `22.54s` in rum versus `7.08s` in WolframKernel; bare
``HOL`Bool`EXCLUDEDMIDDLE[p]`` is `0.969s` in rum versus `0.322s` in
WolframKernel. Keep these numbers as bounded side-lane evidence, not as the
current Pair root hotspot. A 2026-06-18 deep Pair-root split also showed that
the first `andCongLemma[]` call can appear as a `~24s` `doAndCong` block, but
later calls are microsecond/millisecond cache hits; treat it as one-time
memoized `PropTaut` cold-cache cost, not a repeated steady-state Pair/Simp
multiplier. The current Pair/Simp performance task is to continue from the
retained rum-side sharing result above, not to patch `mma-hol` or chase side
probes that do not own the current root.

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
