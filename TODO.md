# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-08-23, committed tree through `d97abd7`. Uncommitted
performance candidates do not count as completed until their correctness,
timing, counter, and memory gates have all been reviewed.

## Active — post-v0.1 algorithmic audit

The repository-wide O(n^2)-and-worse audit has completed its P0 tranche. The
three most recent P0 changes are:

- `c73df5c` — resumable traversal state no longer copies the remaining suffix
  on every step;
- `97873ee` — `Module` escape closure is computed with a linear worklist;
- `d97abd7` — wide UpValue candidate-symbol discovery uses an adaptive index.

The broader completed optimization line is summarized in
[PROGRESS.md](./PROGRESS.md#2026-08-22--08-23--post-v010-safe-rust-algorithmic-performance-sweep).
The remaining measured work is P1, in this order:

1. **Ordered matcher cursor.** The generic ordered, non-`Flat`, all-`Single`
   sequence matcher still rebuilds remaining pattern/value vectors. A cursor
   implementation is under evaluation, but cannot be accepted until its
   rum-vs-rum timing gate is rerun on an uncontended machine.
2. **Shared replacement plan.** `Replace` still clones or scans its rules at
   each visited node and computes subtree depth even for level specifications
   that do not need it. Share a compiled `RewriteRulePlan`; compute depth only
   for negative levels. Preserve rule order, held evaluation, suspension, and
   message behavior.
3. **Three bounded wide-input scans.** Remove repeated prefix rescans in
   `RumTokens`, index wide TOML table keys, and replace `Reap`'s linear tag
   bucketing with an adaptive index. Keep small-input paths allocation-light.

Every item remains subject to the performance gate in
[docs/testing-strategy.md](./docs/testing-strategy.md): safe Rust only, base
versus candidate timing plus counters and memory, and reject or report
regressions and space-for-time tradeoffs.

## Decision-gated or deferred

- **Custom `Sort` / `KeySort` comparisons.** Further reordering or caching can
  change comparator call order and observable side effects. This is a semantic
  decision, not an automatic performance patch.
- **Full polynomial factorization.** Deferred by user decision. The documented
  `Factor` contract remains the current rational-polynomial subset: monomial
  extraction plus recursively discovered linear factors. A substantially more
  complete algorithm requires finite-field factorization and its supporting
  polynomial machinery; do not grow the current linear-factor code piecemeal.
- **Formatter binary-chain planning (P2).** Some long binary chains still
  rebuild temporary vectors. Revisit after the P1 runtime/stdlib work, with
  formatter output frozen before changing the planner.

## Standing engineering debt

- Deep `Expr` trees constructed directly through the Rust API can overflow the
  native stack on drop. Source-created trees are depth-guarded; three iterative
  `Drop` designs regressed the hot path by 20-50% and were rejected. Keep the
  host limitation documented in
  [docs/host-embedding-and-hardening.md](./docs/host-embedding-and-hardening.md).
- Session snapshots are UTF-8 source images. A direct binary snapshot format is
  deferred until substantially larger session images justify the extra format
  and compatibility surface.
- Keep suspension-safe helper state on typed continuation values. `Monitor`
  refresh and `ValueQ` rollback remain best-effort unless their documented
  contract is deliberately promoted.
- Continue splitting formatter planner/builder/render/test modules by
  responsibility as they grow; do not create a separate string crate without a
  real ownership boundary.
