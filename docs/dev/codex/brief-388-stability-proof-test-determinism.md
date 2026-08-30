# brief-388 — make the two stability-proof tests deterministic by construction

brief-387 diagnosed this and its findings are accepted; do not re-derive them.
Summary of what was settled:

- All three recorded sightings share one cause. `symbol_stability_bloom_bit`
  takes one of 128 bits from `symbol.interned_id()`, assigned by the
  process-global interner that parallel tests share. In both failures the
  target's rewritten-dirty bit and ``System`Rule``'s dependency bit are both
  `0x2000`. A Bloom overlap yields `MissDirty`, the proof is not cached, the
  direct association update declines, and the generic renderer allocates new
  args — one test sees the missing proof, the other the moved pointer key and
  rebuilt index.
- **The failure is deterministic under a controlled interner**: 36 unrelated
  symbols interned before the direct test, or 142 before the indexed one, fail
  every run.
- **The contract is one-sided.** `true`/`Some(proof)` is authoritative;
  `false`/`None` may be conservative. Every consumer redoes the work rather
  than trusting a negative, and `association_lvalue.rs:53` anticipates a lost
  proof in a comment and rebuilds. **The implementation is correct.**

**Director's ruling: pointer and index continuity are an opportunistic
optimization, not a guaranteed invariant.** Guaranteeing them would need exact
dependency disambiguation on hot-path machinery and buys nothing semantically,
since the fallback produces the same result. The repair belongs in the tests.

## Goal

Both tests become deterministic **by construction** — not by relaxing what they
check, and not by lowering a collision probability.

`StabilityBloom` has two lanes (`regular`, `generated`) and `intersects`
compares them independently (`model/cache.rs:541`). A target whose dirty lane
is structurally disjoint from its proof's dependency lane therefore **cannot**
collide, whatever ids the interner hands out. brief-387 proposed exactly this:
an identity-bearing (generated-lane) target holding an Association of regular
`System` symbols and integers.

That is the shape to build. Confirm the lane split holds for the construction
you choose — read `symbol_uses_generated_stability_bloom`
(`evaluation_cache/stability.rs:36`) rather than assuming which symbols land in
which lane.

**Split the coverage rather than shrinking it:**

- The **semantic** assertions — the value is right, the definition is right,
  the association contains what it should — stay on ordinary symbols where they
  are today. They do not depend on the fast path and must not be weakened.
- The **optimization** assertions — a valid proof exists, `own_args_ptr_key` is
  unchanged, `index_builds` does not grow — move to the collision-free
  construction, where they hold deterministically.

Neither test may simply drop an assertion. If you conclude one genuinely cannot
be made deterministic, STOP and report which and why.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The determinism gate — this is the real one

A green workspace run proves nothing here; both tests pass in a green run
today. The gate is brief-387's controlled interner:

1. Reproduce the failures on the **current** tree at the two known counts (36
   for the direct test, 142 for the indexed one) and paste the output. This
   also re-calibrates the harness: a preintern probe that cannot make the old
   tests fail cannot demonstrate anything about the new ones.
2. After the change, **sweep a wide range of preintern counts** — at least
   0..256 for both tests — and require every run to pass. Paste the pattern
   string and the summary counts.

Two runs at 36 and 142 are **not** sufficient after the change. Changing which
symbol a test uses moves its bit; passing at the two previously-failing counts
could just mean the new bit happens to miss `0x2000`, which is the
probability-lowering non-repair this brief exists to avoid. **The sweep is what
distinguishes "structurally impossible" from "currently lucky".** If any count
in the sweep fails, the construction is not disjoint and you should report that
rather than search for a luckier symbol.

State plainly, in one sentence, why a collision is impossible for your
construction — naming the two lanes involved.

## Also in scope

`association_lvalue.rs:56` says the recovery re-seeds "in exactly the cases a
seeded proof would allow". brief-387 found that a rewritten-dirty Bloom
collision is an explicit exception. Correct that comment to state the exception.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/evaluation_cache/tests.rs`
- `crates/rum-runtime/src/runtime/tests/association_write_index.rs`
- `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs` — **the
  comment only**; no behavior change
- test-support modules those two tests need

**No production behavior may change.** This is the brief's central constraint:
the implementation was found correct, and a repair that alters invalidation,
the Bloom, the cache, or the interner is out of scope. If you believe a test
cannot be made deterministic without a production change, STOP and report.

**The MAY list is a floor, not a ceiling** for purely mechanical ripple — a
test-support signature, a new helper's re-export. List each under "mechanical
ripple". Anything else unlisted: STOP and report.

Out of scope: `Symbol`/`SymbolText` representation (brief-387 flagged a cached
hash as needing its own ruling; it is not needed under this repair), the Bloom
width, the cache size cap, `TODO.md`, `PROGRESS.md`, `AGENTS.md`, `docs/**`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **Do not widen the Bloom, change the cap, add a retry, or reorder interning
  to dodge the collision.** Each lowers a probability without establishing a
  guarantee; the director rejected them explicitly.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized from a
  measured run of the same shape. The sweep is many short runs; size for the
  sweep, not for one run.
- Any temporary instrumentation used for the sweep must be reverted; the
  worktree must be clean at the end, and say so.
- **Stop-loss** — report rather than improvise if: a lane-disjoint construction
  cannot express what a test needs to check; the sweep shows a failure at any
  count; or determinism would require a production change.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The construction — what makes the two lanes disjoint, in one sentence, with
   the evidence from `symbol_uses_generated_stability_bloom`.
2. What changed — per file, line ranges, one-line why, and which assertions
   moved to the deterministic construction versus stayed on ordinary symbols.
3. Mechanical ripple.
4. Verification — `fmt`, workspace counts, the before-change reproduction at 36
   and 142, and the after-change sweep pattern over 0..256 for both tests.
5. Stop-loss / open questions.
