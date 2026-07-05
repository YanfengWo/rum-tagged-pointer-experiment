# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log).

## Release Transformation (ACTIVE)

Coordination doc: [docs/dev/release-transformation-plan.md](./docs/dev/release-transformation-plan.md).

- [x] M0 plan committed; brief-124 (differential-testing harness removed),
  brief-125 (external-engine scripts archived; microbenches now rum-only),
  brief-126 (closed-line docs + historical dev briefs archived), M1
  (DISCLAIMER/README/AGENTS/testing-strategy/TODO rewritten to the new
  policy).
- [ ] brief-127: wording scan across `crates/` and kept docs (test renames,
  comments, strings; nominative whitelist).
- [ ] brief-128: builtin reference generator from the `rum-stdlib` catalog
  (`BuiltinSpec` + limits notes) with a drift test.
- [ ] brief-129: hand-written language guide under `docs/guide/` (syntax,
  evaluation model, scoping, resource limits and sandbox, design boundaries,
  embedding quick start).
- [ ] brief-130: rustdoc pass (`rum-host` first), per-crate Cargo metadata,
  top-level housekeeping (fold `ISSUE-sandbox-profile-fs-builtins.md` into
  the entry below and remove the file).
- [ ] M2: final sweep + full gate; archive briefs 124-130; bundle cold
  backup. Publication mechanics (hosting, repo name, first tag) are a
  separate later decision.

## After the Release Cut

- [x] **Toolchain / MSRV — SETTLED (2026-07-05): stay on Rust `1.85`.** It is
  Debian stable's default rustc (zero toolchain burden for adopters and
  distro builds) and the edition-2024 floor, so `rust-version = "1.85"` is
  the public MSRV. Post-`1.85` language features are not used;
  `cargo-llvm-cov` stays at the matching `0.6.21`. If an upgrade is ever
  reconsidered, re-baseline both microbench suites in the same session.
- [ ] **Coverage rebuild** (the integration-depth gap left by removing the
  external acceptance corpus): `cargo-llvm-cov`-driven buildout prioritizing
  the host lifecycle (interrupt/cancel, wall-clock timeout, poison recovery,
  `reset()`, snapshot/restore round-trips, typed `HostValue`, capability
  sandbox, `Send`/concurrency), then error/resource-limit paths
  (recursion/iteration/parse-depth/match-depth `TerminatedEvaluation` tags,
  genuine-cycle handling), then a rum-native integration/stress corpus (deep
  recursion, large rewriting workloads, cancellation under load).
- [ ] Scoped-filesystem roots for a trusted-but-limited host profile
  (follow-up to the capability sandbox; the sandboxed profile itself is done).
- [ ] Python adapter stays frozen until a concrete downstream need; when
  reopened, bind via pyo3 to `rum-host`, not to runtime internals
  ([docs/python-adapter-plan.md](./docs/python-adapter-plan.md)).
- [ ] Downstream embedder (separate repo, own roadmap): `dun`, a
  ratatui-based terminal editor, plans `rum` as its optional heavy-compute
  plugin engine — plugins as pure computations (input snapshots in, output
  intents out) under per-plugin resource limits, cancel/timeout, clean reset,
  and the `Pure` capability sandbox. Light paths (config, themes) stay
  native; `rum-host` is pulled in only when needed.
- [ ] Document the embedding adoption cost for light hosts: `rum-host`
  dependency footprint, binary-size delta, and cold `Session` +
  `install_core` startup time.
- [ ] `vm_rhs_template/` fate (experiment, not instrumentation —
  gate/delete/keep). The profiling/debug_tools fate is settled: they stay in
  the kernel, runtime-gated, hidden from restricted hosts via the Diagnostic
  capability.

## Pre-Release Fixes (found writing the guide, brief-129)

- [ ] **Unbounded user recursion crashes the process**:
  `f[n_] := f[n + 1]; f[0]` overflows the native stack instead of tripping
  `recursion_limit` into a recoverable outcome. For a sandbox-first runtime
  this is a must-fix before the release cut (guard the evaluator re-entry
  depth against native stack exhaustion on this path).
- [ ] **`ReplaceRepeated` limit surfaces as a CLI hard error**:
  `1 //. x_ :> x + 1` reports `error: ReplaceRepeated exceeded iteration
  limit (256)` instead of returning `TerminatedEvaluation[...]` like the
  other recoverable budgets. Align it with the limits-as-values contract.

## Standing Engineering Debt

- [ ] Matcher: three known gaps in `Flat`+`Orderless`+`Optional` combinations
  (binding order for flat-wrapped singletons; `ReplaceList` under-collection
  of grouped flat matches; their compound). Low urgency, exotic corner. The
  `#[ignore]`'d tests in `tests/pattern_matching_orderless_optional.rs` are
  the target spec; a previous localized fix regressed neighboring behavior
  and was reverted under stop-loss.
- [ ] Keep the suspension-safe helper boundary from regressing; continue
  moving helper payloads onto typed continuation values when a flow cannot be
  a `Result<Expr, EvalError>` plus one ordinary suspension frame. Monitor
  display refresh and `ValueQ` rollback stay best-effort unless their
  contract is promoted.
- [ ] Binary snapshot format (deferred, low priority): session persistence is
  textual today, which is fine at current scale; a binary path (direct
  deserialize, no re-parse/re-eval) becomes worthwhile for much larger
  session images.
- [ ] Symbolic `Plus`/`Times` like-term detection is O(n²) per expression
  (bounded upside at small term counts; accepted tradeoff). Revisit only with
  profiler evidence from a real symbolic-algebra workload.
- [ ] Deep `Expr` trees built directly through the Rust API (not through the
  parser, which is depth-guarded) can overflow the stack on drop; iterative
  `Drop` designs all regressed the hot path 20-50% and were rejected.
  Documented host limitation in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
- [ ] Formatter internals: continue splitting planner/builder/render/test
  modules as the syntax surface grows; decide later whether the string
  subsystem earns a crate boundary (do not create one by default).
- [ ] REPL/CLI polish: decide whether trace viewing belongs in the REPL;
  snapshot-style regression coverage for help/inspection/message rendering.
