# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-08-26, committed tree through `765ba51e`. Uncommitted
performance candidates do not count as completed until their correctness,
timing, counter, and memory gates have all been reviewed.

## Active — post-v0.1 algorithmic audit

The P0 tranche and all of P1 are done. Two independent re-surveys (one by each
worker) then produced a worklist of about thirty items, grouped into batches;
the 2026-08-26 entry of [PROGRESS.md](./PROGRESS.md) records what each batch
found and closed.

**Closed since the last checkpoint**, in order:

- Loop and `Module`-initializer drivers made iterative. A 20 000-iteration
  `While` terminated the process; ordinary loops are ordinary code, so this was
  a correctness defect before it was anything else.
- YAML and TOML decoders bounded, matching `FromJson`. All three codecs now
  accept exactly 256 nested collections and refuse the 257th with a message
  naming the limit and the byte offset.
- Left-associative chain lowering, five steps (plan-330). All sixteen affected
  operators overflowed the stack at 20 000 links; they now fold iteratively,
  nine of them into n-ary calls, with a 4 096-link ceiling proved on a 2 MiB
  embedding stack through the complete round trip.
- Resource budgets made symmetric (plan in briefs 336-338). Padding builtins,
  `Fibonacci`, and multi-iterator products went uncharged while their siblings
  charged the same work. A regression this introduced — sequential repetition
  charged as simultaneous residency — was found and fixed in the same batch.
- User-callback ordering (plan-339). `Sort`, `SortBy` and `ReverseSort` with a
  comparator were quadratic on every input; `KeySort` and `KeySortBy`
  overflowed the stack at 110 and 100 keys. All five now run on one iterative
  resumable merge engine.
- The microbench itself never closed stdin, so a run either finished in four
  seconds or burned its timeout and produced a header-only timings file. Every
  A/B before `dd4bdefc` was exposed to that.

**Not briefed, and why:** the surveyed "Association 4-to-5" and "65 536-key"
cliffs no longer exist. Measured across eight operations, every 4-to-5 ratio
falls between 1.02x and 1.41x and the 65 535/65 536/65 537 boundary is flat.
They were closed by `c8d34876` and `dc5f6698` earlier in this sweep; the
worklist was simply stale. Association key lookup remains a linear scan, which
is a standing decision, not a defect.

**Remaining from the survey, not yet re-probed:**

1. **`Part` / `Extract` materialize a full table** where a cursor would do.
2. **`ValueQ` clones session state** to answer a question about one symbol.
3. Scattered items: symbol-ordering allocation, REPL re-parsing, a Jupyter
   execution-time hot loop, `StringTrim`, and gensym GC.

**Re-probe each before briefing it.** Two of this batch's items turned out to
be already fixed, and one survey finding — "`SameTest` never exits early" —
was an artifact of an invalid probe: rum's custom equivalence test is a
positional argument, not a `SameTest` option, so the option form silently
compared nothing. Findings age, and probes lie.

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
- **Parentheses are transparent to comparison-chain collection.** `1 == (1 ==
  1)` is True while the `1 == True` it should reduce to stays unevaluated, so
  rum disagrees with itself. The right-hand cases are fixable — a left chain
  only extends through its lhs, so a comparison on the rhs came from explicit
  grouping — while `(a == b) == c` and `a == b == c` share one AST and cannot
  be told apart after parsing. Recorded in the lowering golden with a comment.
- **Product propagation is decided from construct nesting, not data flow.**
  `Table[Length[Table[...1000...]], {i,1,200}]` is refused although the outer
  `Table` retains only integers. Distinguishing that statically is not
  something the iteration seam can do, and the safe side of the line is the one
  that keeps `Table[Table[...]]` refused.

Every item remains subject to the performance gate in
[docs/testing-strategy.md](./docs/testing-strategy.md): safe Rust only, base
versus candidate timing plus counters and memory, and reject or report
regressions and space-for-time tradeoffs.

## Decision-gated or deferred

- **Custom `Sort` / `KeySort` comparisons — settled 2026-08-26.** The ordering
  callback schedule is unspecified: a valid strict weak order determines the
  result, while comparison count, order and direction are not promised.
  Stability is still promised, and key evaluation stays exactly once per entry
  in input order — the contract relaxed comparisons only. Anything that would
  change what a comparator's *return value* means, or how many times a *key*
  function runs, remains a separate semantic decision.
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
