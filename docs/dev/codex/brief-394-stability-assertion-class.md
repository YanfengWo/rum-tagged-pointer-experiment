# brief-394 — sweep the class of interner-order-dependent assertions

brief-388 repaired the two tests that had been observed failing. The brief-392
review found five more assertions resting on the same mechanism, and the
director confirmed them. **This is a gate miss on my part**: I verified that
the two known tests were fixed and that the lane argument held, and never asked
how many other assertions sat on the same footing. The diagnosis had the same
shape — it examined the two recorded sightings because those were the
sightings.

## The mechanism, already settled

`symbol_stability_bloom_bit` (`evaluation_cache/stability.rs:18`) takes one of
128 bits from `symbol.interned_id()`, assigned by the process-global interner
that parallel tests share. `StabilityBloom::intersects` (`model/cache.rs:541`)
compares its `regular` and `generated` lanes independently. When a target and
its dependencies land in the same lane, an id collision is a legal false
positive: the proof is not cached, the direct association update declines, and
the generic renderer allocates new args.

**The stability predicate is one-sided by design.** A true proof is
authoritative; a false one may be conservative, and every consumer redoes the
work rather than trusting a negative (`association_lvalue.rs:53` anticipates a
lost proof and rebuilds). So an assertion that a proof *exists*, that a pointer
key is unchanged, or that an exact amount of work happened, is asserting
something the mechanism does not promise — unless the construction makes the
collision impossible.

brief-388's construction does that: an identity-bearing target writes only the
generated lane, an Association of `System` symbols and integer keys writes only
the regular one, so `intersects` reduces to `(0 & X) || (Y & 0)` for any
interner order.

## Goal

Close the class, not five more instances.

**Sweep first.** The review named `evaluation_cache/tests.rs:105`, `:472`,
`:499`, `association_write_index.rs:84`, and `association_write_recovery.rs:40`.
Treat that as a starting point, not the answer: search for the shape — any
assertion on proof existence, cache residency, `own_args_ptr_key` or another
pointer identity, `index_builds`/`indexed_probes` or another exact work count —
and report the complete list with `path:line`. If the review missed some, say
so.

**Then decide per assertion, and say which you chose:**

- **It is testing semantics** — the value is right, the association contains
  what it should, the definition is reachable. Keep it as it is; it does not
  depend on the fast path and needs no construction change.
- **It is testing the optimization** — that the direct path was taken, that
  args were updated in place, that the index survived. Move it to the
  lane-disjoint construction so it holds for any interner order.
- **It is testing the optimization but cannot use that construction** — say
  why. If an assertion genuinely needs ordinary symbols and also needs the fast
  path, that is a finding to report, not something to force.

**Do not simply delete the optimization assertions.** They cover a real
property that the project cares about; the point is to make them hold by
construction rather than by luck. An assertion that disappears is coverage
lost, and this brief exists because coverage was thinner than it looked.

## Verification (MANDATORY — run it, iterate to green, paste verbatim)

```
cargo fmt --check
scripts/run-bounded.py --timeout 900 -- cargo test --workspace
```

### The determinism gate

A green workspace run proves nothing here; all of these pass today. Use
brief-388's controlled interner:

1. For each assertion you classify as optimization-testing, **demonstrate it
   currently fails** under some preintern count before you change it. If you
   cannot make one fail, say so — that assertion may not actually be exposed,
   and reporting that is better than changing it on suspicion.
2. After the change, **sweep 0..256 preintern counts for every affected test**
   and require all to pass. Paste the pattern strings and summary counts.
3. Any temporary instrumentation is reverted; the worktree is clean at the end.

Two runs at the counts you happened to find are not sufficient — a changed
construction moves the bit, and passing at two points could mean the new bit
merely misses. The sweep is what separates structural from lucky.

## Scope

Files you MAY modify: the test modules named above and any others your sweep
finds, plus test-support helpers they need.

**No production behavior may change.** The implementation was found correct in
brief-387 and re-confirmed since. If you conclude an assertion cannot be made
deterministic without a production change, STOP and report — that would be a
ruling for the director, not part of this brief.

**The MAY list is a floor, not a ceiling** for purely mechanical ripple — a
test-support signature, a helper's visibility. List each under "mechanical
ripple". Anything else unlisted: STOP and report.

Out of scope: the Bloom width, the cache size cap, the interner,
`Symbol`/`SymbolText` representation, `TODO.md`, `PROGRESS.md`, `AGENTS.md`,
`docs/**`.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config.
- No new `unsafe`.
- **Do not widen the Bloom, change the cap, add a retry, or reorder interning.**
  Each lowers a probability without establishing a guarantee; the director
  rejected them by name in brief-388 and that stands.
- **Testing Policy is a hard line** (`AGENTS.md`): rum-vs-rum only.
- Timeout discipline: `scripts/run-bounded.py --timeout SECONDS`, sized for the
  sweep rather than one run.
- **Stop-loss** — report rather than improvise if: an assertion needs both
  ordinary symbols and the fast path; the sweep fails at any count; or
  determinism would require a production change.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format

1. The complete list of assertions in this class, with `path:line`, and whether
   the review's five were the whole set.
2. Per assertion: semantics or optimization, what you did, and why.
3. What changed — per file, line ranges.
4. Mechanical ripple.
5. Verification — `fmt`, workspace counts, the per-assertion pre-change failure
   demonstration, and the post-change sweep patterns.
6. Stop-loss / open questions.
