# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-08-24, committed tree through `e1dc9f62`. Uncommitted
performance candidates do not count as completed until their correctness,
timing, counter, and memory gates have all been reviewed.

## Active — post-v0.1 algorithmic audit

The P0 tranche and most of P1 are done; the 2026-08-24 entry of
[PROGRESS.md](./PROGRESS.md) records the second sweep in full. Two classes of
defect were closed along the way that the original audit had not named: wide
inputs aborting the process on native stack exhaustion, and a cache-clear cliff
that made incremental association writes 40x slower past a threshold.

**Remaining measured work, in this order:**

1. **`RumTokens` byte-to-character offsets.** `byte_to_char_offset`
   (`crates/rum-stdlib/src/strings/tokenize.rs:302`) counts characters from the
   start of the source for every span, twice per token, so tokenizing is
   superlinear: 0.0495 / 0.1263 / 0.3734 s at 25k / 50k / 90k tokens — 1.8x the
   input for 2.96x the time. Spans are produced in ascending order, so one
   monotonic cursor answers all of them in a single pass with no allocation.
   This was listed in the original P1 item but not done in the second sweep.
2. **Shared replacement plan for `Replace`.** `ReplaceAll` compiles its rules
   once per traversal and indexes exact rules; `Replace`, `ReplaceAt` and
   `ReplaceList` still try each rule at each visited node, so cost scales with
   the rule count: 0.0026 / 0.0046 / 0.0093 s for 50 / 100 / 200 rules over a
   500-element list. Share the compiled plan and reuse the exact-rule index.
   Preserve rule order, held evaluation, suspension, and message behavior.
   Note the depth half of the original item is **not** a complexity reduction:
   subtree depth is computed unconditionally, but tree depth is bounded by
   `DEFAULT_EVALUATION_PASS_LIMIT` (256), so it is a bounded constant factor.

**Deferred, with evidence, from the second sweep:**

- **Error spans are computed per form but rendered against the whole file.** A
  script `q = 1\nw = 1 ! 2` reports `2:1` when the offending `!` is at `2:7`,
  and the `Get` path shows byte offsets with no line/column at all. This is a
  real pre-existing defect, deliberately excluded from the splitter work
  because fixing it changes user-visible output and would have voided that
  change's byte-identical acceptance criterion.
- **A trailing-operator form still re-parses per line.** The linear splitter
  drives the parser form by form, but a form whose delimiters are balanced and
  whose expression is unfinished (`1 +\n2 +\n3`) cannot be predicted from
  balance alone. Such forms are short in practice; recorded rather than hidden.
- **Pattern- and key-driven `ReplacePart` rules stay sequential.** Merging them
  into one pass would reorder user-visible evaluation or change association-key
  semantics. Their outer walk is iterative, so wide rule lists no longer
  consume a native frame per rule.

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
- The same-expression evaluation-cycle guard can reject recursive definitions
  whose visible call shape repeats while other session state progresses. Relax
  it only with an explicit progress model that preserves genuine-cycle
  detection.
- Continue splitting formatter planner/builder/render/test modules by
  responsibility as they grow; do not create a separate string crate without a
  real ownership boundary.
