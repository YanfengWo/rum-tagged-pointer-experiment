# TODO

This file is for active or near-term work only. Completed history and
operational lessons live in [PROGRESS.md](./PROGRESS.md) (append-only log),
and release-facing changes live in [CHANGELOG.md](./CHANGELOG.md).

Status checkpoint: 2026-08-26, committed tree through `4f5b85f1`. Uncommitted
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

**Re-probed 2026-08-26, all still present.** Numbers are from the release CLI
at `c18b01f9`, each probe carrying a liveness assertion:

1. **`ValueQ` clones the whole session per call.** 4.6 us with no definitions,
   90.6 us at 2 000, 333 us at 8 000, and 1.94 ms at 32 000 — a cost that
   depends on session history, not on the question asked. Confirmed against
   the wall clock: 100 calls in a 32 000-definition session move the process
   from 0.10 s to 0.30 s. A 420x spread that ends at two milliseconds is worth
   fixing cheaply, not worth a redesign.
2. **`Part` / `Extract` materialize every association entry per call.** 200
   reads of `a[[Key[7]]]` take 0.26/0.55/1.20/2.42 s at 4k/8k/16k/32k keys, a
   strict doubling; positional `a[[3]]` has the same shape.
3. **`StringTrim` with a literal *list* leaves the fast path.** At 8 000
   characters: 342 us for `{"1", "2"}` against 28 us for the single literal
   `"1"` and 21 us for whitespace, and 1.45 ms against 77 us at 32 000. Linear,
   not quadratic — the defect is a 12x-to-18x constant, so the surveyed
   "quadratic suffix helper" reading is wrong even though the target is real.
4. **Symbol ordering allocates two `String`s per comparison**
   (`ordering_sets.rs:875`), 6.7x/7.0x/14.2x slower than integer ordering at
   4k/8k/16k. Not a mechanical swap: `Symbol::cmp` compares context then name,
   which is not the lexicographic order of the joined qualified name, so a
   repair has to argue output equivalence.
5. Confirmed by reading: REPL re-parses the whole accumulated buffer per line
   (`rum-repl/src/lib.rs:117-121`, O(L^2) on a paste), the Jupyter kernel busy
   spins a full core while a cell runs (`socket.rs:329-334`), and the gensym
   sweep never rearms after an all-live pass (`gensym_gc.rs`, default
   threshold 200 000, so reachability is low).

**Re-probe each before briefing it.** Two of this batch's items turned out to
be already fixed, and one survey finding — "`SameTest` never exits early" —
was an artifact of an invalid probe: rum's custom equivalence test is a
positional argument, not a `SameTest` option, so the option form silently
compared nothing. Findings age, and probes lie.

**Rule right-hand-side `Condition` guards are not implemented (brief-347,
diagnosed 2026-08-26).** `f[x_] := rhs /; test` is a standard idiom and rum
silently does something else: the guard is never evaluated, a false guard never
rejects the rule, and the `Condition` leaks into the result — `b[x_] := x^2 /;
x > 0` makes `b[3]` return `9 /; True` and `b[-3]` return `9 /; False`. Dedup
compares left-hand sides only, so a rule differing solely by its guard is
treated as a redefinition of the unguarded one and disappears. `Rule` and
`RuleDelayed` carry the same hole through the replacement system.

Parsing and lowering are correct; `DownValue` and `RewriteRule` simply have no
guard field, so the whole `Condition` is stored as the body and substituted as
one. The guard in *pattern* position works correctly and is the model to
follow. No existing test encodes the wrong behavior.

Every candidate-selecting lane is affected: `DownValues`, `UpValues`,
`SubValues`, `DefaultValues`, `OwnValues`, `ReplaceList`, `ReplacePart`,
string rules, and the exception-handler scan. Director's rulings on the plan:
only exact `True` accepts and anything else declines (matching the pattern-side
guard); nested guards flatten to an ordered sequence evaluated left to right;
a guard shares its body's `OptionValue` scope; identity is syntactic after
alpha-canonicalization, so `Condition[body, True]` is a distinct candidate from
an unguarded rule; `Unset` removes all guard variants of an lhs; a bare
`Condition` expression stays inert; `NValues` and `Dispatch` are out of scope.
**Guarded `OwnValues` are deliberately narrowed**: no storage-representation
change and no `symbol.rs` hot-path cost — check the guard only when the stored
value is itself a `Condition`, and record multiple guarded own-values for one
symbol as unsupported. This work precedes the `ValueQ` line below, which
depends on it: `ValueQ[b[-3]]` cannot be right until `b[-3]` evaluates right.

**`ValueQ` is being rebuilt as a lookup — director's ruling 2026-08-26.**
It will answer without evaluating its argument at all, and it keeps rum's
precise answer: `f[1] = 3; ValueQ[f[2]]` stays `False`, because no definition
can apply to `f[2]`. A coarser "does the head have any definition" rule was
considered and rejected as too surprising. This ruling supersedes the repair
comparison in brief-345: an answer that never evaluates has no side effects to
roll back and no session to clone, so both the contract breaches below and the
per-call cost above disappear with it rather than being traded off.

The three breaches are what the ruling closes (brief-345, all reproduced by
the director). The generated reference — normative under the testing policy —
says `ValueQ` tests "by trial evaluation without committing side effects".
Session-local state does roll back (`ValueQ[yy = 7]` leaves `yy` undefined),
but three effects escape the trial:

- **The deterministic PRNG advances.** `SeedRandom[123]; ValueQ[RandomInteger[
  1000000]]` consumes the first draw, so the next draw returns what should
  have been the second. `Capability::Pure` cannot serve as an effect proof
  here — `RandomInteger` and `SeedRandom` are deliberately classified `Pure`.
- **`Print` reaches the host sink.** Output streams immediately when a sink is
  installed; the existing rollback test only covers the buffered no-sink case.
- **`FileWrite` commits.** `ValueQ[DumpSave[path, x]]` writes the file, and no
  in-memory snapshot can undo it.

The open question is the accuracy of the lookup, not the contract. rum already
has a machine that answers "would evaluating this change it" — the evaluation
stability check the evaluator uses to skip stable expressions — but it is
deliberately conservative, and its conservative direction ("might change")
becomes a wrong `True` here. How precise it is, which shapes need more, and
what an undecidable shape should answer are the questions to settle.

**Found while re-probing, and more serious than the list above: iterator
variables were invisible on an assignment lhs.** `Do[f[i] = i, {i, 1, 3}]`
installed one `HoldPattern[f[i]] :> 3` and said nothing — the standard table
fill and memoization idiom. Fixed in `4f5b85f1` (briefs 342-343): a clean
iterator symbol lives in a dynamic own-value frame, ordinary evaluation
resolves definitions then that frame, and the held-lhs normalizer stopped
after definitions. It now takes the same third step in the same order. The
`Do[With[{k = i}, f[k] = k], …]` workaround in the test suite is gone.

**The same matrix exposed two defects that have nothing to do with
iterators** — both reproduce with a plain global symbol, neither is fixed:

- **`AssociateTo`/`KeyDropFrom` write back to the raw target.** `ii = 2;
  AssociateTo[af[ii], "k" -> 7]` reads `af[2]` and then installs a third
  definition `HoldPattern[af[ii]] :> <|"k" -> 7|>`, leaving `af[2]` empty.
  Silent.
- **None of the eleven compound assignment heads evaluate a `Part`
  selector.** `kk = 1; lst[[kk]] += 5` reports `Part::pkspec1` and changes
  nothing, while `lst[[kk]] = 99` works. Loud, so it ranks below the silent
  one.
- `TagSet`/`UpSet` never normalize lhs arguments either (`h /: h[tag1, tg] = 7`
  stores `h[tag1, tg]` with `tg = 5`). That one is a standing contract
  question, not a regression — decide it before touching it.

**The microbench cannot run above `--scale 1`.** `--scale 5` dies on
`evaluation exceeded iteration work limit (100000)` and writes a
header-only `timings.tsv`, while the script still exits 0 with a warning —
the same silent-empty shape as the stdin defect fixed in `dd4bdefc`. The
cause is the deliberately conservative product propagation recorded below:
the harness nests its repetition loop around each label's own loop. Two
things are wrong and both matter, because this is the instrument every A/B
depends on: the harness has no headroom above scale 1, and it fails
quietly when it has none.

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
