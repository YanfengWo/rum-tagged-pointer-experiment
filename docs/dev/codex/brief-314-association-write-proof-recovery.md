# brief-314 — recover the Association direct-write proof after a cache clear

## Goal

Incremental Association writes fall off a cliff — not a curve — once a session
crosses the call-stability cache's entry cap. Writing 16 000 keys takes 1.5 s;
writing 20 000 takes **55 s**. Make the direct-write path recover its proof
after the cache is cleared, so the cliff becomes a bounded, amortized cost.

This is Step A of the design in brief-313. **Step B (reusing the read-side
lookup index to remove the remaining quadratic) is a separate brief — do not
start it here.** Fixing the cliff first is deliberate: an index whose benefit a
cache clear can erase is worth little.

## Measured today (release CLI, verified by the director — do not re-derive)

`a = <||>; Do[a[i] = i, {i, 1, n}]`:

| n | time |
|---|---|
| 8000 | 0.36 s |
| 16000 | 1.52 s |
| **20000** | **54.8 s** |

Going from 16 000 to 20 000 is 1.25× the work for **36×** the time; pure
quadratic growth would predict 1.56×. This is a regime change, and it is silent
— no message, no warning.

The mechanism is confirmed by a controlled override:

| n = 20000 | time |
|---|---|
| default cap (16384) | 59.9 s |
| `RUM_CALL_STABILITY_CACHE_MAX_ENTRIES=10000000` | **2.8 s** |

21× apart, so the cap's clear is the cause, not a coincidence of size.

## The mechanism

1. Every direct write builds a fresh `Rule[key, rhs]` and calls
   `cached_expr_stability_with_bloom` on it
   (`crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs:41`).
   That is a new pointer key each time, so it misses and **inserts an entry for
   a single-use rule that is dead the moment the write finishes**. Net +1 live
   entry per write.
2. Crossing `CALL_STABILITY_CACHE_MAX_ENTRIES`
   (`evaluation_cache/stability.rs:12`, `1 << 14` = 16384) triggers
   `clear_call_stability_cache_for_size_cap` (`:361`), which wipes the whole
   cache — including the proof the direct-write path needs.
3. `take_call_stability_proof_for_key` (`association_lvalue.rs:53`) then fails,
   the write falls back to the slow path, and **the slow path never re-seeds the
   proof** (`association_data.rs:646-653` stores the value and bumps the epoch,
   nothing more). So the direct path is disabled *permanently* for that
   Association, and every subsequent write pays parse + clone + render, O(n)
   each.

Confirm each of these three steps yourself before changing anything; a
counter-based reading is better evidence than a timing here.

## The fix

When `take_call_stability_proof_for_key` returns `None`, recompute the
certificate on the spot rather than giving up: call the same check the evaluator
would have used to seed it — `association_data_stability_with_bloom`
(`eval_core/stability.rs:113`) — on the current args. If the data is stable,
synthesize the proof, proceed with the direct write, and reinsert it so
subsequent writes hit again. If it is not stable, fall back exactly as today.

This is behaviour-preserving by construction: the recomputation is the *same*
check whose result the evaluator would have cached, so a write takes the direct
path in exactly the cases it does now, and the fallback is untouched.

Cost shape: O(n) once per cache clear, not per write. Say explicitly in the
report how many recomputations a 20 000-key loop performs — if it is more than
a handful, the fix is in the wrong place and you should stop and report.

Two constraints on the implementation:

- **Do not raise or remove the cap**, and do not change the clear policy. The
  cap exists to bound memory; this brief makes the write path survive a clear,
  it does not argue about when clears happen.
- **Do not stop inserting the per-write rule entry** as a shortcut. That would
  change what the stability cache contains for every other consumer. If you
  believe the dead single-use entry is itself worth eliminating, report it as a
  finding — it may well be a better fix, but it is a different change with a
  different blast radius, and the director will decide.

`association_data_stability_with_bloom` is a private `fn` in `eval_core`; a
visibility widening to `pub(in crate::runtime)` is expected and acceptable.

## Scope

Files you MAY modify:

- `crates/rum-runtime/src/runtime/kernel_data/association_lvalue.rs`
- `crates/rum-runtime/src/runtime/eval_core/stability.rs` (visibility only)
- `crates/rum-runtime/src/runtime/evaluation_cache/stability.rs` (only if the
  proof reinsertion genuinely requires it — say why)
- test modules under `crates/rum-runtime/src/runtime/tests/`

Out of scope: everything else. Specifically not the read-side
`model/cache.rs` index (that is Step B), not `association_data.rs`'s slow path,
and not `rum-stdlib`.

Do not add unconditional cost to the default evaluation path.

## Deliverable

- The proof recovery, with the recomputation count per loop reported.
- A test that writes past the cap (20 000 keys is past it; keep the test's debug
  runtime tolerable and report it) and asserts the result, so the cliff cannot
  return unnoticed.
- Counter evidence that writes past the cap still take the direct path — an
  allocation or mutation-path counter is the right instrument, since "in place
  or copied" is a yes/no question with a load-immune answer.
- Before/after timings at n = 8000 / 16000 / 20000 / 32000.

## Verification (MANDATORY — run it, paste verbatim)

```
cargo fmt --check
cargo test --workspace
```

Then, with a fresh `cargo build --release -p rum-cli` — feed stdin from
`/dev/null`, the CLI waits for input otherwise:

```
python3 - <<'PY'
for n in (8000, 16000, 20000, 32000):
    open("/tmp/b314-%d.rum" % n, "w").write(
        'a = <||>;\nPrint["n=%d t=", AbsoluteTiming[Do[a[i] = i, {i, 1, %d}]], " len=", Length[a]];\n'
        % (n, n))
PY
for n in 8000 16000 20000 32000; do ./target/release/rum-cli /tmp/b314-$n.rum < /dev/null; done
```

Each must report the correct `len=`, and the 16000→20000 step must no longer
jump by more than the curve predicts. Also re-run the controlled override at
n = 20000 and show the gap between capped and uncapped has closed.

Then the microbench A/B: this is the evaluation write path, so run
`scripts/core-language-microbench.sh` on base and candidate and paste the
association-related labels plus same-tier neighbors, with a plain statement of
whether the machine was quiet.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; the director gates and commits.
- Do NOT modify files outside Scope. If you believe you must, STOP and report.
- Touch nothing outside this repo; no network. Commands: file edits in Scope,
  `cargo`, the release `rum-cli`, the microbench scripts, and `python3`.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own documented behavior
  only; no other vendors' engines, no oracle harnesses.
- **No new `unsafe`.** `rum-core` and `rum-runtime` are ZERO-`unsafe`.
- **Ownership is the trap in this area.** `plan-291` removed an extra strong
  holder that made `Arc::make_mut` copy instead of mutating in place. Do not
  introduce one. If your change holds anything across the mutation window, stop
  and report it.
- Minimal diff; no drive-by reformatting.
- Paste real verbatim output. If a run did not reach green, say so.

## Report format (your final message)

1. What changed — per function, with line ranges.
2. Your confirmation of the three-step mechanism, with the evidence you used.
3. Verification — each command with verbatim output: test counts, the timing
   table, the recomputation count, the counter evidence, and the A/B rows.
4. Stop-loss / open questions — including whether you think the dead per-write
   cache entry should be eliminated separately.
