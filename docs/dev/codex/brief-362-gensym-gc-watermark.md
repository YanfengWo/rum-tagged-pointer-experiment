# brief-362 — the gensym sweep never rearms after an all-live pass

## Goal

When every candidate gensym is still reachable, the sweep finds nothing dead,
returns without shrinking the candidate set, and the very next `Module` crosses
the threshold again — so every subsequent `Module` pays a full mark.

Measured on `21569ddb` with `RUM_GENSYM_GC_THRESHOLD=100`, three consecutive
batches of 200 `Module` calls whose gensyms are all retained:

```text
candidates 600 after the third batch
batch 1: 4.18 ms    batch 2: 9.36 ms    batch 3: 15.22 ms
```

Per-batch cost grows linearly with the retained candidate count, so a session
that holds its gensyms pays quadratically overall.

The default threshold is 200 000, so reaching this needs a long-lived session
that retains that many generated symbols — low reachability, but once reached
it never recovers.

`crates/rum-runtime/src/runtime/scoping/gensym_gc.rs:53` is the return that
leaves the candidate set untouched; `scoping/lexical/module.rs:245` is the
trigger.

## Director's ruling

Use a **watermark**: after a sweep that reclaims nothing, the next sweep does
not run until the candidate set has grown by another full threshold. So an
all-live sweep over N candidates rearms at N + threshold.

This bounds both sides, and both bounds must be stated in the report:

- **Work**: consecutive `Module` calls no longer each trigger a full mark.
- **Retention**: a candidate that becomes unreachable is reclaimed after at
  most one additional threshold's worth of new candidates — not indefinitely.

A partial sweep — one that reclaims some but not all — should rearm from the
post-sweep candidate count by the same rule. Do not add a `Module`-count
cooldown as well; one mechanism, not two.

## Deliverable

- The watermark, in the session's candidate-tracking state, reset correctly
  when the candidate set shrinks for any other reason (`ClearAll`, symbol
  removal, snapshot restore — enumerate what can shrink it).
- The env override `RUM_GENSYM_GC_THRESHOLD` keeps working and interacts with
  the watermark in the obvious way.
- No change to what the sweep considers live. Every root enumerated by
  `for_each_root_expr`, every suspended outcome, and every module activation
  stays live exactly as before.

### Tests

Use the env threshold to make this testable at small sizes.

- Counter test: with all candidates live, N consecutive `Module` calls perform
  **one** full sweep, not N. Assert the sweep count directly.
- Reclamation bound: after the roots holding candidates are dropped, the
  candidates are reclaimed within one additional threshold of new candidates —
  assert they are gone, and assert they were still present just before that
  point, so the bound is pinned from both sides.
- A partial sweep rearms from the post-sweep count.
- The candidate set shrinking by other means resets the watermark correctly —
  one test per mechanism you enumerated.
- Liveness unchanged: a gensym escaping through a definition, through a
  suspended computation, and through a nested module activation all survive a
  sweep, as they do today.
- Saturating arithmetic: a threshold at `usize::MAX` or a candidate count near
  it does not overflow.

### A/B

`--scale 5`, same-binary noise floor **first, three runs each way, reported as
the spread**. Labels:

```
module-local            module-chain-escape-small    module-chain-escape-large
block-local             with-local                   do-small
matcher-single-tiny     symbol-ownvalue-read
```

Add a stable label for repeated `Module` calls under a low env threshold with
all gensyms retained — that is the row that should stop growing. Report it
across two batch counts to show the per-batch cost is now flat rather than
climbing.

## Scope

- Files you MAY modify: `runtime/scoping/gensym_gc.rs`,
  `runtime/scoping/lexical/module.rs`, `runtime/model/session.rs`,
  `runtime/session/session_state.rs`, runtime tests,
  `scripts/core-language-microbench.sh`.
- **Mechanical ripple is in scope but must be reported**, each file justified.
- **The general accept/stop criterion**: *any change to which gensyms are
  considered live is a stop.* This brief changes when the sweep runs, never
  what it decides.
- MUST NOT touch: `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`,
  `Cargo.lock`, `Expr`/`Symbol` representation.

## rum / Rust pitfalls (read twice)

1. **Candidate tracking is advisory and rollback-sensitive** — a snapshot
   restore can move the candidate set underneath the watermark. Handle it and
   test it.
2. **Use saturating arithmetic** for the watermark.
3. **Do not weaken liveness to reclaim sooner.** Retention delay is the price
   of this repair and it is bounded; missing a root is a correctness defect.
4. **No default-path cost**: `Module` is a hot path, so the watermark check
   must be a comparison against a field, nothing more.
5. **Counters before timings** — the sweep count is the primary evidence.
6. **Stop-loss is real.** If the same step fails twice for the same reason,
   STOP and report.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test -p rum-runtime
cargo test -p rum-stdlib
cargo test --workspace
```

Then the A/B as described.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director runs the authoritative gate and commits.
- Touch nothing outside this repo; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented
  behavior only.
- **No new `unsafe`**; no new dependencies.
- Bounded runs via `scripts/run-bounded.py --timeout SECONDS -- <command>`.
- Minimal diff. Paste real verbatim output; if a run did not reach green, say
  so.

## Report format (your final message)

1. What changed — per file.
2. The two bounds, stated explicitly: what the watermark saves in work, and
   the worst-case reclamation delay it costs.
3. The enumeration of everything that can shrink the candidate set, and how
   each resets the watermark.
4. Verification — each command with verbatim output.
5. The sweep-count and reclamation-bound test evidence.
6. The A/B — counters first, then the retained-Module label across two batch
   counts, with a three-run-each-way noise floor and the controls named.
7. Stop-loss / open questions.
