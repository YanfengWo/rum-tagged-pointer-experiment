# Layer B — Single-allocation call-node rewrite (detailed plan)

Status: **active, greenlit 2026-06-22.** This is the detailed executable plan for
Layer B of `docs/representation-rewrite-plan.md`. It supersedes that doc's
sketch of Layer B (B0-B3); in particular **B3 (the inline normal-form stamp) is
dropped** — it was prototyped (Task-3b) and refuted (33M+ stamp hits, no Pair
movement; the carrier also cost ~12-14% on the default path).

## Why now (the decision)

The per-call dispatch/eval constant has been driven to a **hard plateau** by
Phase 0 + Layer A (A1-A4): the worst microbench labels are down ~13-17% from the
frozen baseline, A4 was a systemic win that translated to the bounded Pair
descent (`-5.8 to -7.9%`), and the latest cut (brief-015, a correct removal of
redundant per-dispatch attribute lookups) landed **at the microbench noise
floor**. Incremental WORK-shaving is exhausted: individual cuts now cost an
8-minute back-to-back A/B just to be seen above noise.

The remaining gains require fixing the **representation design flaws themselves**,
not shaving around them. The `34x` figure that previously seemed to justify (or,
when re-examined, de-justify) this work is unreliable — re-measured this session,
the heavy `basic-only-pair4` descent is `5.83x`, the same ~6x as everything
else; the `34x` was a per-failed-dispatch DIFFERENTIAL from one probe, not a
representative wall gap. So Layer B is **not** justified or blocked by the 34x.
It is justified by: (1) a genuine, counter-proven design flaw (multi-allocation
nodes), (2) the plateau leaving structural change as the only path to measurable
further gains, (3) code-quality / cache-locality / foundation value.

## The flaw being fixed (flaw 2, counter-proven)

`Expr::Call { head: SharedExpr(Arc<Expr>), args: SharedArgs(Arc<Vec<Expr>>) }`
allocates, per freshly-built call node:

- the head `Arc<Expr>` (1 heap alloc),
- the args `Arc<Vec<Expr>>` = the `Arc` box (1) **plus the `Vec`'s separate heap
  buffer (1)**,

= **up to 3 heap allocations per call node**. The `RUM_PROFILE_REPR` counters
confirm `shared_args_alloc + shared_expr_alloc ≈ 2/call` on the downvalue paths
(and the hidden Vec buffer is a third, uncounted, allocation). A4 made the
*no-change rebuild* reuse the head Arc, but genuine node CONSTRUCTION — the bulk
of the Pair descent, which builds fresh terms — still pays the full price. The
`Vec` buffer is pure waste: a call node's arity is fixed at construction, so the
`Vec`'s growth capacity and separate indirection buy nothing.

## Honest expectations

This is a constant-factor + code-quality + cache-locality fix, NOT a Pair-34x
silver bullet (no such lever exists — the gap is a uniform distributed
constant). The expected perf is a general allocator-pressure reduction across all
node construction; magnitude is **to be measured in B1**, not promised up front.
The durable wins are: fewer live allocations, contiguous args (one cache line
fetch instead of two), and a cleaner immutable node representation that removes a
class of in-place-mutation foot-guns.

## The key design insight (drives the phasing)

The current **two independent Arcs** (head, args) are exactly what lets Layer A
(A2 COW prepare, A3 candidate reuse, A4 head reuse) share head and args
*independently*. A naive single contiguous `Arc<[Expr]>` (head-at-index-0,
WolframKernel-style) would collapse that independence. Two consequences:

1. **Subtree sharing is NOT lost** under any layout — subtrees are `Expr`s placed
   into the parent's storage, and an `Expr::Call` is itself a shared `Arc`, so a
   reused subterm stays shared regardless of the parent's layout.
2. **Independent head/args reuse IS lost** under a single Arc — but it is a
   narrow case (mostly "the candidate is the call itself", already coverable by
   reusing the whole node). Still, it must be measured, not assumed away.

Therefore the high-value, low-risk fix is to compact the **args side only**:
`SharedArgs(Arc<Vec<Expr>>)` → `SharedArgs(Arc<[Expr]>)`. This drops 3→2 allocs
per node, makes args contiguous, and **preserves the `Expr::Call { head, args }`
variant shape** — so none of the ~572 destructure sites change and Layer A's
independent head/args COW is fully retained. The full single-node collapse is a
*separate, data-gated* decision (B2), not a prerequisite.

## Phase plan

### B0 — allocation measurement scaffold (do first)

The existing `RUM_PROFILE_REPR` counts `SharedArgs::new` calls but NOT the
underlying `Vec` buffer, so it cannot show the 3→2 win. Add a direct measure:

- A gated global-allocator shim (or a `mimalloc` stat read) that counts total
  heap allocations / bytes over a workload, behind the same cold-load
  `AtomicU8` zero-cost-when-off pattern. Report alloc count for the
  microbench tier loops and the Bool-stack load.
- Acceptance for B0: with the shim ON, `Do[Do[fExact[1],{1e5}],{10}]` shows a
  baseline allocation count we can diff against after B1.

Brief: `brief-016-allocation-counter`.

### B1 — args compaction: `SharedArgs = Arc<[Expr]>`

The brief-012 attempt stopped here because `Arc<[Expr]>` cannot be mutated in
place (`Arc::make_mut` needs `T: Clone`; `[Expr]` is unsized) and a scope audit
missed ~70 in-place mutators (`pop`/`push`/`insert`/`remove`/indexed-assign/
`iter_mut` via the `DerefMut`/`make_mut` on `SharedArgs`) across ~12 files
(`structure_ops/{sequence/insert,sequence/delete,replace/replace_part,
exact_paths,surface}`, `rewrite_builtins`, `definition_updates`,
`definition_catalog/traversal_continuation`, `parse_and_message_helpers`,
`suspension/{collections,rewrite_rules,traversal}`). Do it de-risked:

- **B1a (migrate mutators to build-then-freeze, NO type change yet):** while
  `SharedArgs` is still `Arc<Vec>`, rewrite every in-place `SharedArgs` mutation
  to: take an owned `Vec<Expr>` (via `into_vec`/`to_vec`), mutate the `Vec`, then
  reconstruct (`SharedArgs::from(vec)` / `Expr::call*`). After B1a, NOTHING
  mutates a `SharedArgs` through `DerefMut`. Gate: pure refactor,
  `cargo test --workspace` green, zero perf delta. Split into 2-3 briefs by
  directory if the diff is large; each gated independently. This is the step
  that makes the type swap safe.
  Briefs: `brief-017-mutator-migration-{structure-ops,rewrite-builtins,rest}`.
- **B1b (swap the backing):** change `SharedArgs(Arc<Vec<Expr>>)` →
  `Arc<[Expr]>`; update `new` (`Arc::from(vec)`), `into_vec` (now always
  `to_vec`), `Default`, `Deref<Target=[Expr]>`, `AsRef`, `ptr_key` (fat-pointer
  data address), `IntoIterator`; **remove `DerefMut`** (nothing needs it after
  B1a). Keep the serde derive (wire format identical: a sequence). Fix any
  remaining `.as_slice()`/`&Vec` compile fallout by moving to slices. Gate:
  `cargo test --workspace` green, `DumpSave`/`Get` snapshot round-trip
  unchanged, B0 allocation count drops on the node-construction loops, the
  primitive tier + Bool-stack load improve or hold (clean back-to-back A/B),
  Layer A COW counters (`into_vec_clone` stays 0 on the hot paths) intact.
  Brief: `brief-018-shared-args-arc-slice`.

Acceptance for B1: 3→2 allocs/node confirmed by the B0 counter; no behavior
change (workspace + snapshot green); a measurable (clean A/B) general tier
improvement or, if it lands at the noise floor like brief-015, an honest null —
either way the allocation reduction is real and the code is simpler/immutable.

### B2 — full single-allocation node (DATA-GATED, only if B1 justifies it)

Only after B1's measured result decides it is worth the cost. The collapse to
one allocation + best cache locality (`Expr::Call(Arc<[Expr]>)` head-at-0, or
`Arc<CallData{head, args}>`) requires migrating ~572 `Expr::Call { head, args }`
destructure sites to accessors, and it trades away independent head/args reuse.

- **B2-decision:** with B1 data, prototype the single-node backing on a branch
  and measure the marginal 2→1 alloc + locality win against the lost independent
  reuse (re-run the Layer A counters + tier + Pair). If the marginal win is
  small (likely, given A3's pure-alloc cut did not translate to Pair), STOP and
  bank B1 — a small prize behind a 572-site migration is a valid stop
  (representation-rewrite-plan decision point 2).
- **B2a (mechanical accessor migration, behavior-preserving):** introduce
  `Expr::head()/args()/as_call()` accessors and migrate all ~572 destructure
  sites off direct field access, while the backing is STILL two Arcs. Pure
  refactor; `cargo test --workspace` green; zero perf delta. In batches by
  crate/directory, each its own brief + gate.
- **B2b (backing swap):** swap to the single-allocation node behind the stable
  accessor interface; only rum-core + constructors change. Gate: tests +
  snapshot + allocation halves again + tier/Pair improve or hold.

## Risk register / off-ramps

- **The ~70 in-place mutators (B1a).** The audit that missed them is why B1a is
  a SEPARATE behavior-preserving step gated on `cargo test --workspace` before
  any type change. Off-ramp: if a mutator cannot be cleanly build-then-frozen,
  keep it on an owned `Vec` boundary.
- **Serde / snapshot drift (B1b/B2b).** `Arc<Vec<Expr>>`, `Arc<[Expr]>`, and a
  single-node layout all serialize as the same sequence; gate hard on the
  existing `DumpSave`/`Get` round-trip. The in-memory layout is internal.
- **`into_vec` always clones (B1b).** Post-A2 the hot tier shows `into_vec=0`, so
  this lands only on ~18 cold builtin callers; confirm no hot-tier regression in
  the B1b A/B. If a hot label regresses, STOP.
- **Layer A interaction.** B1 preserves independent head/args Arcs, so A2/A3/A4
  are intact. B2 would collapse them — that is exactly what B2-decision measures
  before committing.
- **Suspension/resume.** Arg vectors flow through suspend frames; B1a/B1b must
  keep the resume contract (gate with the input-resume tests).
- **Migration size (B2).** 572 sites is why B2a is mechanical-only and batched,
  and why B2 is data-gated rather than assumed.

## Measurement & acceptance (every step)

- `cargo fmt --check`, `cargo test --workspace`, the `DumpSave`/`Get` round-trip.
- The B0 allocation counter (direct alloc-count diff — the existing
  `RUM_PROFILE_REPR` cannot show the Vec-buffer elimination).
- Primitive tier via `scripts/core-language-microbench.sh` with a **clean
  same-session back-to-back A/B** on an idle machine (the only reliable gate near
  the noise floor — verify maintenance daemons at 0% CPU first; for a committed
  baseline, A/B against the parent commit via `git checkout <commit>^ -- <file>`,
  since `git stash` of a committed file is a no-op).
- Bounded Pair re-measure (`basic-chain-orelsec-split`) as the downstream signal.

## Decision points

1. After B0+B1: re-rank and decide whether B2 (the 572-site single-node) is worth
   it from the measured 2→1 marginal prize. A small prize is a valid stop.
2. B3 (inline stamp) stays DROPPED unless a fundamentally new mechanism (value
   memoization, not stability-stamp) is separately justified with fresh data —
   out of scope here.

## Execution model

Director (me): direction, gating, commits, the data-gated decisions. Codex:
bounded briefs (`brief-016`…), self-verified, left in the working tree. Each
phase is one or more briefs in `docs/dev/codex/`. No `mma-hol` edits; the
acceptance clone `/Users/fft/Developments/mma-hol-acceptance` stays pristine.
