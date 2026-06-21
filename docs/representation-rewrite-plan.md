# Representation Rewrite Plan

Status: proposed, 2026-06-21. This is the multi-week plan to fix the three
expression-representation design flaws that now dominate the rum/WolframKernel
gap, after the per-primitive dispatch wins (briefs 003-008) were harvested.

## Why (goal and rationale)

rum implements a strict SUBSET of the Wolfram language. A subset evaluator with
a tight representation has no structural reason to run several times slower than
WolframKernel on the shared surface, and a real chance to run FASTER on it. The
remaining gap (worst primitive ~`5.4-5.9x` after brief-008; the `Pair.wl`/`Simp`
downstream gate still `>600s`) is no longer per-primitive logic — it is three
representation decisions that tax every call. These are design flaws, not tuning
opportunities; they must be fixed.

Target: close the primitive tier to `<=2x` of WolframKernel and bring the
`Pair.wl`/`Simp` gate into a bounded budget. Stretch: beat WolframKernel on the
supported core subset, validated by measurement, not asserted.

## The three flaws (each grounded in this session's measurements)

1. **Eager owned conversion — the Arc->owned->Arc round-trip.**
   `evaluate_inner_unstable` runs `evaluate_call(head.into_owned(),
   args.into_vec())` for every call, BEFORE knowing whether the head is a
   builtin or a user definition. `into_vec()` is free only when the args `Arc`
   is unique; in any loop or repeated subterm the `Arc` is shared, so it
   deep-clones the whole args vector every evaluation and re-wraps a fresh
   `Arc` afterward. A clean `if-true` sample spends ~`13%` in
   `SharedArgs::into_vec` / `SharedExpr::into_owned` / `Vec::clone` / `to_vec` /
   `memmove` / `drop_in_place`.

2. **Two allocations per call node.**
   `Expr::Call { head: SharedExpr(Arc<Expr>), args: SharedArgs(Arc<Vec<Expr>>) }`
   is two separate `Arc`s (head, args) — two heap allocations and two refcount
   pairs per call node. HOL proof terms are deeply nested calls, so node
   construction allocation/refcount traffic is doubled.

3. **Pointer-keyed stability cache that cannot hit rebuilt trees.**
   The call-stability cache is keyed by `(head ptr, args ptr)`. Every rewrite
   builds a NEW-pointer tree, so the cache is almost entirely `miss_absent`
   (measured directly in the Pair/Simp diagnosis: ~`75M` checks, nearly all
   absent). The "is this already in normal form" verdict lives in an EXTERNAL
   side table keyed by a pointer that the next rebuild invalidates, instead of
   ON the expression. WolframKernel carries a normal-form flag with the
   expression and shares structure, so a re-encountered stable subterm
   short-circuits with no lookup and no re-walk. This same flaw is the engine
   behind the `Pair.wl` `~34x` failed-`REWRCONV` multiplier: each failed
   conversion attempt rebuilds and re-evaluates owned trees that an inline
   normal-form flag on shared structure would skip.

## Principles (non-negotiable, learned over briefs 001-008)

- **Incremental, never big-bang.** Every step compiles, keeps `cargo test
  --workspace` green, and is independently committable. A half-done
  representation change is slower than either end state; we never leave the
  tree in that state across a commit.
- **Measured every step.** Each step states its expected win and is gated by a
  measurement (isolated phase/allocation counters preferred over the noisy
  whole-run microbench A/B; see the gate discipline in
  `docs/performance-stage0-baseline.md`).
- **Correctness preserved every step.** Suspension/resume continuation shape,
  serde round-trip, the matcher, `Ord`/`Hash`/`PartialEq` semantics, and the
  evaluation-epoch invalidation contract are invariants. No behavior change is
  acceptable as a side effect of a representation change.
- **Executed via the Codex delegation model.** Each step is one bounded
  `docs/dev/codex/brief-NNN-*.md`; Claude designs and runs the authoritative
  gate. Large mechanical migrations are their own briefs, separate from the
  behavior-changing step they de-risk.
- **No mma-hol edits, ever.** The acceptance standard stays unmodified
  (`AGENTS.md` hard line).

## Phase 0 — measurement scaffold (do first)

Before changing representation, make the cost directly countable so every later
step has an objective gate.

- P0.1 (brief): add opt-in counters (the env-gated cold pattern) for: owned
  conversions per call (`into_vec`/`into_owned` on a shared `Arc`), call-node
  allocations, and the call-stability `miss_absent` rate. Report them on the
  core microbench tier and on the warm `Pair.wl` mkPair shape.
- P0.2: capture the Phase-0 baseline table (primitive ratios + the new counters
  + the downstream gates: propTaut distribution, Bool load, warm mkPair) so
  every later step is a delta against a fixed point.

## Layer A — borrowed/shared dispatch (fixes flaw 1)

Stop materializing owned args eagerly; thread `SharedArgs` (or `&[Expr]`)
through dispatch and only convert to owned where a path genuinely needs it. This
is bounded and splittable, and it is a PREREQUISITE for Layer B's inline stamp
to pay off (the stamp only helps if trees are actually shared/reused).

- A1 (brief): change `evaluate_call` / `resume_call_after_head` /
  `CallDispatchState` to carry args as `SharedArgs`, reading them as `&[Expr]`
  through the stages. Defer `into_vec` to the points that already build a new
  vector. Gate: no behavior change (tests green), owned-conversion counter on
  builtin calls drops, builtin labels improve.
- A2 (brief): copy-on-write argument preparation. `prepare_arguments` returns
  the SAME `SharedArgs` when no argument changed (the common case for held and
  already-evaluated args), allocating a new vector only when an arg actually
  rewrites. This is the same COW pattern already used for lexical substitution
  and rule-RHS binding substitution (see PROGRESS history). Gate: held-builtin
  calls (`If`, `Hold`-family) stop cloning args; owned-conversion counter near
  zero for them.
- A3 (brief): the downvalue/upvalue/subvalue candidate build
  (`Expr::call(head.clone(), args.to_vec())`) reuses the shared args via
  `Expr::call_shared` / `call_with_shared_args` instead of `to_vec`, on the
  paths that still need a candidate. Gate: no behavior change; allocation
  counter on the downvalue path drops.

Acceptance for Layer A: the owned-conversion counter is near zero on the pure
builtin and held paths, the primitive tier improves, all tests green, and the
downstream gates (propTaut, Bool, warm mkPair) do not regress and ideally
improve (the conversion chain is owned-tree-churn heavy).

## Layer B — single-allocation call node + inline normal-form stamp (fixes flaws 2, 3)

The larger change; it touches `Expr` in `rum-core` and ~`572` `Expr::Call {
head, args }` destructure sites across `131` files. Do it in three de-risked
steps so the representation swap is never a 572-site big-bang.

- B0 (diagnostic brief): with the Phase-0 counters, quantify the prize — call
  allocations per node (expect ~2), the `miss_absent` rate, and how much of the
  warm mkPair / primitive tier is node allocation + absent-miss re-walks. Decide
  the node layout from data: `Arc<CallData{head, args}>` (one alloc, minimal
  churn) vs a contiguous `Arc<[Expr]>` head-at-0 layout (WolframKernel-style,
  most compact). State the chosen layout and the expected allocation drop.
- B1 (mechanical migration brief — behavior-preserving, NO perf change):
  introduce `head()` / `args()` accessors on the call and migrate the ~572
  destructure sites from `Expr::Call { head, args }` to the accessors, while the
  backing is STILL the current two-`Arc` layout. This decouples every site from
  the field layout. Gate: pure refactor — `cargo test --workspace` green, zero
  measured perf delta. (This is the step that makes B2 safe.)
- B2 (brief): swap the backing to the single-allocation node behind the now-
  stable accessor interface. Only `rum-core` and a handful of constructors
  change; the 572 sites are untouched (they use accessors). Gate: tests green,
  call-allocation counter halves, node-construction-heavy labels and warm mkPair
  improve, serde round-trip unchanged.
- B3 (the inline normal-form stamp) — **REFUTED as the Pair/34x lever; do NOT
  build it on that expectation.** This step was already prototyped (Task-3b,
  2026-06-13, `RUM_CALL_STABILITY_INLINE_STAMP=1`): the stamp carried the
  stable verdict correctly (unit litmus passed) and **hit `33M+` times** on warm
  mkPair, **yet both variants still timed out at the same `268M+`
  stability-check scale**. That refutes the "redundant re-evaluation of shared
  stable subterms" hypothesis: if the cost were re-checking shared stable
  subterms, the stamp would have moved it — it did not, so the Pair cost is
  **genuine work in the conversion descent**, not shareable redundancy. The
  stamp carrier (a `Mutex<Option<..>>` on every `SharedArgs`) also imposed a
  ~`12-14%` default-path regression (the reverted-carrier hard line in
  `CLAUDE.md`). Conclusion: the `34x` Pair gate is the per-failed-dispatch
  CONSTANT in the descent (handoff Task 2d), i.e. the same per-call dispatch/eval
  constant the primitive tier measures — NOT a cache/sharing/stamp problem. An
  inline stamp could only ever be a marginal play on the general constant, and
  only if made truly zero-cost when off; it is not greenlit.

Acceptance for Layer B: the **per-call dispatch/eval constant** (the real `34x`
lever, via compounding over the descent) and the node-allocation pressure come
down, primitive tier and `Pair.wl`/warm mkPair materially down, all tests green,
serde + suspension invariants intact. The stamp/absent-miss framing is dropped.

## Sequencing and why A precedes B

A before B is not arbitrary: Layer B's inline stamp only pays off when subtrees
are actually shared and reused, which Layer A (shared dispatch + COW arg
preparation) establishes. Doing B first would put a stamp on trees that the
eager owned conversion immediately rebuilds — reproducing the absent-miss
behavior the stamp is meant to kill. Within B, B1 (mechanical accessor
migration) must precede B2 (backing swap) so the swap is not a 572-site edit.

## Measurement and acceptance

- Primitive tier: `scripts/core-language-microbench.sh` (min-of-N, both engines)
  for the ratio; isolated per-phase timers and the Phase-0 allocation/absent-miss
  counters for the authoritative per-step delta (the whole-run A/B is too noisy
  to attribute single-digit-percent moves — see brief-007/008 gate notes).
- Downstream gates: propTaut distribution, Bool-stack load, and the warm
  `Pair.wl` mkPair shape, re-measured after each layer. The representation work
  is justified largely BY the downstream gates (the Pair `34x` multiplier is
  owned-tree churn in the conversion chain), so these are first-class acceptance
  signals, not afterthoughts.
- Every step: `cargo fmt --check`, `cargo test --workspace`, `rum-difftest
  check`, and an unmodified-`mma-hol` smoke remain green.

## Risk register and off-ramps

- **Serde / snapshot format drift (B2).** The textual `DumpSave` shape must be
  preserved; the node layout is an internal detail. Gate B2 with the existing
  snapshot round-trip test. Off-ramp: keep the serialized shape identical even
  if the in-memory layout changes.
- **Suspension/resume continuation shape (A1).** The dispatch state carries
  resume frames; changing args to shared must keep the suspend/resume contract.
  Gate with the session-request resume tests. Off-ramp: keep `CallDispatchState`
  owning a `SharedArgs` (still `Send`/`Sync` via `Arc`).
- **Stamp correctness (B3).** A stale-true stamp would skip needed
  re-evaluation — a correctness bug, not a perf one. Gate hard with the
  epoch-invalidation regressions and a deliberate "definition added between
  reuses" test. Off-ramp: if exact invalidation is too costly, the stamp can be
  conservative (validate against the global epoch only), which still beats the
  pointer-keyed absent-miss.
- **Layer not paying off.** If A's counters show the owned churn was already
  cheap in practice, or B2's allocation halving does not move the tier, STOP at
  that step and bank what landed — each step is independently valuable. Do not
  push to B3 on faith; B0's data decides.

## Decision points

1. After Phase 0 + Layer A: re-rank. If the tier is already near `2x` and the
   downstream gates moved, Layer B may be optional or deferable.
2. After B0 (the diagnostic): the measured allocation/absent-miss prize decides
   whether B2/B3 are worth the 572-site migration. A small prize is a valid
   stop.
3. After B3: re-measure the stretch goal (beat WolframKernel on the core
   subset). Whether to chase the last fraction (WK-class hash-consing / value
   memoization — a separate, larger effort) is a fresh decision with fresh data.

## Out of scope (explicitly deferred)

Full structural hash-consing / value-level memoization of arbitrary expressions
(beyond the inline stamp on shared call nodes) is the largest possible
commitment and is NOT in this plan. It becomes a candidate only if, after Layer
B, measurement shows the remaining gap is dominated by re-evaluation of
logically-identical-but-unshared subterms that the inline stamp cannot reach.
