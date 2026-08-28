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
2. **`Part` / `Extract` materialized every association entry per call — FIXED
   `4e045135`** (briefs 357-358). Per read: positional 12.61 ms -> 1.00 us at
   32 000 keys and exactly flat across width; keyed 12.27 ms -> 10.36 us.
   Allocations per read are now width-invariant (14.03 at both 4 000 and
   32 000 keys, against 12 083 and 96 101 before).

   **Leftover, deliberate:** keyed reads are sublinear rather than flat —
   3.7x for 8x width — because a wide association does not always hold the
   read-side index under cache admission pressure. That policy (`MAX_KEYS`,
   the retained-entry budget) is a standing decision and was explicitly out of
   scope. Revisit it only with the memory tradeoff stated, since the surveyed
   "65 536-key cliff" was already judged a fossil.
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
5. REPL re-parsing, the Jupyter busy-spin, and the gensym sweep — **all FIXED**
   (`21569ddb`, `3732615a`). The REPL examines only appended text and runs the
   full check once per form rather than once per line; the kernel sleeps its
   launcher's 10 ms idle interval instead of yielding; the sweep rearms at a
   watermark, so per-batch cost stops climbing (base +106% by the third batch,
   candidate -11%).

**Batch 7 is closed.** Every surveyed item was re-probed, and each was either
fixed or recorded with its reason. Two defects found along the way outrank
what the batch was looking for, and both have their own entries above:
`Do[f[i] = i, …]` installing one raw definition, and rule right-hand-side
`Condition` guards never being evaluated.

## Next up — scheduled 2026-08-28

Ordered after re-probing everything below. Silent defects outrank loud ones,
and the instrument outranks nothing here because the instrument turned out to
be fixed already.

1. **The assignment-target normalization family.** Planned by brief-376,
   which re-probed all three defects, widened each, and returned two repair
   families rather than one or three. **Dispatch order: brief-377 (family A)
   first, then family B in three steps.**

   - **Family A — evaluated `Part` selectors.** D2 plus the `Part` branches of
     `AssociateTo`/`KeyDropFrom`: thirteen mutation heads, not eleven. The
     repair is a typed `EvaluatedPartSelectors` that assignment-target parsers
     require, so a future head cannot pass raw selectors without a compile
     error. Brief-377.
   - **Family B — definition-lhs lookup identity.** D1's fallback store, D3's
     tagged installs, and the two ordinary-`Set` defects found while probing
     (below). One attribute-aware, resumable preparation, shared by the store
     side and the lookup side.

   **The contract, settled 2026-08-28.** A definition's lhs arguments are
   prepared according to the head's hold attributes: non-held positions are
   fully evaluated, including a delayed own-value; held positions keep their
   raw symbols. **The invariant is bidirectional — the identity a definition is
   stored under and the identity a lookup constructs must come from the same
   preparation.** rum currently violates it in both directions, which is why
   family B covers both:

   - Not normalizing where it should: `AssociateTo[af[ii], …]` storing
     `af[ii]`, `TagSet`/`UpSet` storing a raw lhs, and `fD[iD] = 8` with
     `iD := 1 + 1` storing `HoldPattern[fD[1 + 1]]` so `fD[2]` cannot reach it.
   - Normalizing where it should not: with `SetAttributes[fH, HoldFirst]` and
     `xH = 5`, `fH[xH] = 9` stores `HoldPattern[fH[5]]`, so the natural
     `fH[xH]` lookup misses it. This one is the opposite direction, and
     covering only the first direction would fix three defects and hide it —
     the same shape as the `Block` frame's bare `Symbol` in the frame-roots
     line.

   `TagUnset` is in family B's ripple: if tagged definitions normalize, removal
   must use the same preparation.

   **Ruling (user, 2026-08-28): keep rum's `UpSet::nosym`.** Once `m[ww] ^= 22`
   with `ww = 5` normalizes to `m[5]`, no symbol in the lhs can serve as a tag,
   and rum already reports exactly that for `m2[7] ^= 33`. The wider WL family
   reaches a differently-shaped diagnostic there; rum keeps the more direct one
   and stays internally consistent, as it did when it kept its precise
   `ValueQ[f[2]]` answer. Design input only — no engine comparison enters the
   repository.

   **Retired by evidence: the "raw rules become reachable after `Clear`"
   argument.** Plan-376 recorded it as a reason a repair must diagnose rather
   than refuse. It describes rum's own raw-store behavior, not a semantics
   worth keeping: under the settled contract the rule is stored ready for
   lookup and is reachable immediately. The "preserve raw plus a warning"
   option, and the diagnostic question that depended on it, are both closed.
2. **The `rum-jupyter` ZeroMQ ordering flake.** Third occurrence reached, which
   is what the standing agreement said to wait for. Diagnostic brief.
3. **`ValueQ` session-clone cost** and the remaining batch-7 performance items,
   as scheduled below.

Not scheduled, and why: the call-stability-proof flake stands at one recorded
occurrence against the same three-occurrence agreement, and association keyed
reads are sublinear by a standing cache-admission decision that needs its
memory tradeoff stated before it can be reopened.

## Opened by batch 7, not yet scheduled

- **Gensyms held only by an opaque continuation frame — CLOSED 2026-08-28**
  (briefs 364-375, `d13aa27f` through `4e4c229f`). Four defects, not the two
  originally recorded: the design brief's re-probe added a matcher binding
  table losing its bindings, and a `Block` frame holding a bare `Symbol` that
  *leaked* on the next sweep — the opposite direction from the others, and
  proof that covering only `Expr` would have fixed three and hidden the
  fourth. All four are verified closed against plan-364's original probes,
  which assert the old buggy values and therefore now fail by asserting the
  defect.

  The fix is enforced rather than maintained. `SuspensionFrame: FrameRootTrace`
  makes a missing traversal a compile error, the frame queue is private so
  there is one insertion route, and a repository invariant test bans the
  deleted APIs and any handwritten traversal outside a two-entry allowlist
  (`RandomState`, `SyntaxError` — both structurally incapable of holding a
  root). Frames are `Arc`-rooted in two Session slots so the executing frame
  stays visible to a sweep, and one traversal now feeds both the root walk and
  the memory proxy, which used to charge a flat 16 bytes per frame.

  It also removed ordinary-path waste: a closure captured its state before the
  suspend/resume split, so a width-2003 `Switch` paid 63.8 KiB for a capture it
  never used. That width slope is now constant-size noise.
- **The `rum-jupyter` native ZeroMQ ordering flake has now appeared three
  times** (briefs 165, 242, and a workspace run during brief-362). Each time it
  passes in isolation — three isolated runs and two further workspace runs
  after the third occurrence were all green. The standing agreement was to
  brief it on a third occurrence; it is unrelated to the 10 ms idle sleep,
  which landed after the flake's first two appearances.
- **First occurrence of a call-stability-proof flake, recorded not explained
  away.** One `cargo test -p rum-runtime` run during brief-370 failed
  `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  on its `has_valid_call_stability_proof_for_key` assertion
  (`evaluation_cache/tests.rs:374`). The worker attributed it to the 128-bit
  stability Bloom's sensitivity to parallel symbol-interning order. It did not
  reproduce in thirteen director attempts — eight of the test alone, five of
  the full 854-test parallel suite — so the attribution is neither confirmed
  nor refuted. brief-370 changed neither `evaluation_cache` nor
  `association_lvalue`, so it is pre-existing. Same standing agreement as the
  Jupyter flake: brief it on a third occurrence.

**Re-probe each before briefing it.** Two of this batch's items turned out to
be already fixed, and one survey finding — "`SameTest` never exits early" —
was an artifact of an invalid probe: rum's custom equivalence test is a
positional argument, not a `SameTest` option, so the option form silently
compared nothing. Findings age, and probes lie.

**Rule right-hand-side `Condition` guards — COMPLETE 2026-08-26 (briefs
347-354, `5ab6afc5` through `eb926bff`).** All eight candidate-selecting lanes
evaluate their guards: `DownValues`, `UpValues`, `SubValues`, `DefaultValues`,
generic rewrite rules, `ReplacePart`, string rules, and `OwnValues`. One
substrate, one accept rule (`True` accepts, anything else declines), and a
decline is indistinguishable from the rule not matching. Per-lane ordering
contracts are documented in `docs/guide/patterns-and-rules.md`; a single test
exercises a guarded rule in all eight lanes so a future change cannot break
one silently. The unguarded path takes no new branch, allocation or clone in
any lane.

Known limitation, deliberate: one symbol keeps a single own-value slot, so
multiple guarded own-values do not coexist — a later definition replaces the
earlier one.

**The line's own lesson, worth more than the feature.** After six green gates
a guarded `DownValue` still stopped enforcing its guard the moment anything
bumped the attribute revision — any `SetAttributes` in the session, or a
loader-snapshot restore such as `ValueQ`. Every gate had asserted the guard
*before* the trigger and never after one, and the whole-line regression test
had the same shape, so none of them could see it. Brief-354 fixed it
structurally, by putting the barrier in the single rematch API rather than
patching another call site, and the regression test now asserts after a bump.
**For a feature that caches or compiles anything derived from a definition,
the test has to act after the invalidation, not only before it.**

The original diagnosis follows.

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

**`ValueQ` — COMPLETE 2026-08-26 (briefs 345, 346, 355, 356; `b85569c2`
and `e67715f1`).** Both tiers landed. The trial no longer commits side
effects: PRNG state is restored, the host print sink is suppressed for its
duration, and `FileWrite`/`Process` are refused, with restoration on every
exit path including panic. Proven-stable targets — `ValueQ[undefinedSymbol]`
above all — answer without evaluating at all: 332.6 us to 0.76 us per call at
8 000 definitions, and no longer scaling with session history. rum's precise
answers are unchanged throughout (`f[1] = 3; ValueQ[f[2]]` is still `False`),
with one deliberate exception: `ValueQ[DumpSave[path, x]]` now answers `False`
because the refused capability fails the trial.

The reasoning that got there follows.

**`ValueQ`: precision and freedom from side effects are the hard
requirements; not evaluating is not — user ruling 2026-08-26.** The first
ruling was "answer by lookup, never evaluate", but brief-346 proved the three
cannot hold together: deciding whether a `Condition` or `PatternTest` rule
applies *requires* evaluating user code, so a non-evaluating answer must be
imprecise on the very common `f[x_?NumericQ]` and `f[x_] /; test` shapes.
Since not-evaluating was only ever a means to the other two, it is the one
that gives way.

The shape is therefore two-tiered:

1. **Provable shapes answer by lookup, without evaluating.** rum's precise
   answers are kept — `f[1] = 3; ValueQ[f[2]]` stays `False`. A coarser "does
   the head have any definition" rule was considered and rejected as too
   surprising.
2. **Undecidable shapes are evaluated in an effect-suppressed environment**,
   so the answer stays exact while nothing escapes the trial.

Tier 2 comes first: it closes the contract breaches below without changing any
answer. Tier 1 is then pure performance.

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

**The same matrix exposed three defects that have nothing to do with
iterators** — all reproduce with a plain global symbol. **Re-probed
2026-08-28 against the release CLI at `65c03870`: all three are still
present**, each probe carrying a liveness assertion and a working control.
They are one family — the assignment target is not normalized before the
definition is installed — but by three different mechanisms, so whether
they are one repair or three is a question for the design brief.

- **`AssociateTo`/`KeyDropFrom` write back to the raw target.** `ii = 2;
  af[2] = <|"a" -> 1|>; AssociateTo[af[ii], "k" -> 7]` reads `af[2]`
  correctly, applies the update correctly, then installs the result under a
  second definition `HoldPattern[af[ii]] :> <|"a" -> 1, "k" -> 7|>` and
  leaves `af[2]` untouched. Silent — no message. `KeyDropFrom` behaves the
  same way. Mechanism: the fallback path evaluates `target_expr` to read
  the current value but hands the *unevaluated* `target_expr` to
  `store_association_lvalue_update`
  (`crates/rum-stdlib/src/associations/mutation.rs:196-225`).
- **The compound assignment heads do not evaluate a `Part` selector.**
  `kk = 1; lst[[kk]] += 5` reports `Part::pkspec1` and changes nothing,
  while `lst[[kk]] = 99` works and `lst[[1]] += 5` works — a clean
  three-point control that isolates the defect to compound-assignment ×
  symbol-selector. Confirmed on nine heads (`AddTo`, `SubtractFrom`,
  `TimesBy`, `DivideBy`, `Increment`, `Decrement`, `PreIncrement`,
  `PreDecrement`, `AppendTo`). Loud, so it ranks below the silent ones.
  **The repair model is one call site**: `part_assignment.rs:228` passes
  `evaluated_selectors` to
  `exact_position_paths_from_part_target_selector_exprs`, while
  `definition_updates.rs:389` passes the raw `selector_exprs` to the same
  function.
- **`TagSet`/`UpSet` never normalize lhs arguments, and the definition they
  install is unreachable.** Worse than recorded: `tg = 5; h /: h[tag1, tg] =
  7` stores `HoldPattern[h[tag1, tg]] :> 7`, so `h[tag1, 5]` does not match
  and neither does `h[tag1, tg]` — the lhs evaluates to `h[tag1, 5]` before
  lookup, so the definition can never fire from any input. Same for the true
  up-value form (`vv = 5; g /: k[g, vv] = 11` leaves `k[g, 5]` unevaluated)
  and the tagless form (`ww = 5; m[ww] ^= 22`). A clean symbol works
  (`m3[qq] ^= 44` then `m3[qq]` is 44), and `m2[7] ^= 33` correctly reports
  `UpSet::nosym`. Silent in every failing case. Whether the lhs *should*
  normalize is still the standing contract question — but "writes a
  definition that nothing can reach, without a message" is a defect under
  either answer, so decide the contract, then fix at least the silence.

**The microbench scale ceiling — FIXED `d9c713e1` (brief-344), verified
2026-08-28.** Both halves are closed: the harness has headroom, and it fails
loudly when it does not. `--scale 5` at the default `--reps 7` now completes
and writes 112 measured labels with an empty stderr, and the scale is proved
live — 100 of the 112 labels carry exactly 5x the scale-1 iteration count,
the other 12 being fixed-width labels that scale does not touch. The script
now exits non-zero on both failure shapes (`core-language-microbench.sh:1084`
for a non-zero engine exit, `:1090` for zero measurement rows), so the
header-only `timings.tsv` can no longer pass for a run. The record above of a
scale-1 ceiling was written before brief-344 and was never retired.

Note for whoever re-checks this: `--reps 1` is not a valid probe of the
ceiling. The repetition loop is the layer that nests around each label's own
loop, so lowering it hides the very defect being tested.

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
