# brief-387 — the call-stability-proof flake (diagnostic)

Two flakes have been recorded in `TODO.md`, both in the association write path,
both asserting an identity-shaped property, both green in isolation:

- `association_lvalue_direct_update_keeps_args_identity_and_reinserts_proof`
  (`evaluation_cache/tests.rs:374`), on
  `has_valid_call_stability_proof_for_key` — **twice**, during brief-370 and
  again during brief-385.
- `association_write_index::repeated_replacement_writes_stay_indexed`
  (`association_write_index.rs:146`), on `own_args_ptr_key` and lookup
  counters — once, during brief-379, never resolved as flake-or-fixed.

**Scope: NONE — diagnostic only.** No source change. Reading, grepping, and
throwaway probes are in scope; leave no probe files in the repository. The
deliverable is evidence and a recommendation.

## What the director has already established

Do not re-derive this; check it and build on it.

**The Bloom bit comes from the global interner.**
`symbol_stability_bloom_bit` (`evaluation_cache/stability.rs:18`):

```rust
let mix = u64::from(symbol.interned_id()).wrapping_mul(0x9e3779b97f4a7c15) ^ identity;
1u128 << (mix >> 57)
```

`interned_id()` is assigned by `static SYMBOL_INTERNER`
(`rum-core/src/expr.rs:48`), a process-global. Test binaries run tests in
parallel threads sharing that interner, so which id a symbol receives depends
on what other tests interned first. The bit is one of **128**, so by the
birthday bound a dozen or so live symbols already give a substantial collision
probability, and `StabilityBloom::intersects` (`model/cache.rs:541`) treats any
shared bit as a conflict.

**The cache key is not the suspect.** `CallStabilityCacheKey::new`
(`model/cache.rs:495`) is built from `ptr_key()` of head and args, and the
entry owns both (`CallStabilityCacheEntry.head/args`, `model/cache.rs:506`), so
the pointers cannot be recycled while the entry lives. The director initially
suspected this and was wrong; the earlier attribution to the Bloom looks right.

**There is a size cap that clears the whole cache**, not an LRU eviction:
`call_stability_cache_max_entries()` (`stability.rs:371`) and
`clear_call_stability_cache_for_size_cap()` (`:361`).

**A possible coupling to the second flake.** `CallStabilityCacheEntry` holds
`SharedExpr`/`SharedArgs` strongly. `repeated_replacement_writes_stay_indexed`
asserts `own_args_ptr_key` is unchanged — i.e. that the write updated in place
rather than copying — and in-place update depends on the `Arc` not being
shared. If a stability entry happens to hold that same `SharedArgs`, the
refcount is above one and the write copies instead, moving the pointer key.
Whether such an entry is present depends on Bloom-driven invalidation, which
depends on the interner order. **That chain is a hypothesis, not a finding.**

## The question this brief must answer first

**Is the implementation wrong, or is the test asserting something the
implementation never promised?**

A Bloom filter has false positives by construction. If the contract of the
stability proof is "may conservatively report not-stable", then
`assert!(has_valid_call_stability_proof_for_key(...))` is asserting a property
the mechanism does not guarantee, and the defect is in the test — the same
shape as the ZeroMQ flake closed in brief-384, where the poller was correct and
the assertion demanded a race outcome.

If instead the contract is "stable under these specific conditions, exactly",
then a false positive that flips the answer is an implementation defect.

Settle this from the code and its comments — what the proof is *for*, who
consumes it, and what a false "not stable" costs at each consumer. State the
answer before recommending anything. **Both outcomes are acceptable findings;
do not assume the implementation is at fault because a test failed.**

## Experiments

Reproduction is **not** a precondition for this brief. The ZeroMQ flake did not
reproduce in the sandbox at all while reproducing at ~70% on the director's
host, and gating work on a reproduction made that brief unsatisfiable. Report
what you can reproduce; where you cannot, argue from the mechanism and say so.

**E1 — make the flake deterministic by controlling the interner.** If the
mechanism is right, the failure is a function of which ids the test's symbols
receive. Intern a controlled number of unrelated symbols before the test body
and sweep that number. If the failure rate depends on it — better, if some
value fails every time — the flake becomes a deterministic reproduction, which
is worth more than any statistic. Report the sweep.

**E2 — widen or neutralize the Bloom.** Temporarily widen the bit space, or
make `intersects` always report a conflict (maximally conservative) or never
(no invalidation), and observe both tests. This separates "Bloom false positive"
from "something else entirely". These are throwaway edits for measurement; the
repository must be clean when you finish.

**E3 — measure the refcount at the moment of failure.** For
`repeated_replacement_writes_stay_indexed`, instrument the write path to report
`Arc::strong_count` for the target `SharedArgs` when `own_args_ptr_key` moves,
and identify every holder. This directly tests the coupling hypothesis: if a
stability-cache entry is among the holders, the two flakes share a cause; if
the count is one and the key still moves, they do not.

**E4 — the third-sighting question.** `TODO.md` treats these as two flakes at
two and one sightings against a three-sighting agreement. If E3 shows one
cause, the threshold is already met and the standing agreement is satisfied.
State which it is; the director needs that answer to schedule the repair.

## What to deliver

1. **The contract answer** — is the proof allowed to report conservatively?
   With the evidence from code and consumers.
2. **E1-E3 results**, verbatim, including anything that did not reproduce.
3. **Same cause or two** — argued, with E3's holder list as evidence.
4. **Repair options, with a recommendation, not an implementation.** Cover at
   least: widening the bit space; deriving the bit from something stable across
   runs rather than the interning order; having the stability cache hold weak
   references so it cannot block an in-place update; and — if the contract
   answer says so — relaxing the assertions instead. For each, say what it
   costs on the ordinary evaluation path, since this is hot-path machinery.
5. **Open questions** needing a director or user ruling.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the repository
  clean; E2's edits are temporary and must be reverted.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only; no other
  vendor's engine, no oracle harness.
- Any timing or repetition run uses `scripts/run-bounded.py --timeout SECONDS`,
  sized from a measured run of the same shape. Note that a release-mode
  repetition run is a different instrument from the debug test gate — that is
  what exposed the ZeroMQ flake at 70% when debug showed it three times in
  three months.
- **Do not recommend a fix that makes the race less likely.** A larger cap, a
  retry, a sleep, or a wider Bloom that merely lowers the collision rate are
  not repairs unless you can state the resulting guarantee precisely. If the
  honest answer is "the mechanism is probabilistic and the test must accept
  that", say so.
- **Stop-loss** — report rather than improvise if: the contract question cannot
  be settled from the code; E1 shows the failure is independent of interner
  order, refuting the whole hypothesis; or a repair option would require
  changing `Symbol` representation.
- Paste real verbatim output. If an experiment did not run or did not
  reproduce, say so explicitly.

## Report format

1. The contract answer, with evidence.
2. E1 — the interner sweep.
3. E2 — Bloom widened/neutralized.
4. E3 — refcount and holders at failure.
5. Same cause or two, and the sighting count that follows.
6. Repair options with costs, and your recommendation.
7. Stop-loss / open questions.
