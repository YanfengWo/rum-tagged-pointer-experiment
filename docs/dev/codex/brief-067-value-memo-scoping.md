# brief-067 — value-memo / hash-consing scoping (B0 diagnostic)

## Goal

Decide the go/no-go for the deferred **hash-consing / structural value-memo**
perf project by MEASURING its net effect on the CURRENT `main` binary (v8, the
32-byte `Expr` layout post briefs 053/054), not the 2026-06-22 numbers taken on
the old 64-byte layout. The 2026-06-22 shadow probe measured the *count-weighted*
prize (`hit_rate=0.2565, mismatches=0` on the bounded Pair descent) but NEVER
measured the two things that actually decide the project: (B) the *net wall-clock
effect* of a REAL short-circuiting memo with naive structural keys (i.e. value-memo
WITHOUT hash-consing), and (C) how much of any shortfall is the per-dispatch
O(expr) structural-hash key cost (the number that says how much hash-consing would
have to recover). This is a **diagnostic/scoping brief**: throwaway, env-gated
instrumentation only, NO product-code behavior change and NO change left on any
default path. Deliverable is a cost-benefit table + a verdict, with verbatim
numbers. Leave the throwaway instrumentation UNCOMMITTED in the working tree so
Claude can reproduce it during the gate (Claude reverts it after gating — do NOT
revert it yourself, and do NOT commit it).

## Context pointers

- Read `AGENTS.md` (operational notes; the unmodified-`mma-hol` hard line) and
  these `PROGRESS.md` sections FIRST:
  - `## 2026-06-22 Hash-consing / value-memo probe: 57% dispatched-call structural repeat`
  - `## 2026-06-22 Shadow value-memo VERDICT — sound for Pair, ~26% achievable, but needs the full hash-consing architecture`
    (this is the prototype you are re-running; it describes the exact shadow-memo
    shape, the `runtime-profile-basic-only-pair4` probe, and the two hard caveats).
  - `docs/representation-rewrite-plan.md` §"Out of scope" (this project is the
    explicitly-deferred WK-class lever) and the B3-refuted note (the stamp is a
    DIFFERENT lever — do not confuse them; the memo short-circuits NON-stable
    `REWRCONV→Failure` dispatches, which the stamp could never touch).
- Key files:
  - `crates/rum-runtime/src/runtime/eval_core.rs:73` `evaluate_inner_unstable` —
    the `Expr::Call { head, args }` arm at line ~85 is the single hook point
    (dispatch a call, then optionally key/store/compare/short-circuit).
  - `crates/rum-runtime/src/runtime/eval_core.rs:662` `evaluate_call` — the real
    work you are wrapping/short-circuiting.
  - `crates/rum-runtime/src/runtime/evaluation_cache.rs` `bump_evaluation_epoch*`
    and the `evaluation_epoch` field (session_state.rs:358) — the epoch you MUST
    validate memo entries against. `Do`/`Table` loop-var scopes deliberately do
    NOT bump the epoch (that is caveat 1 in PROGRESS — the Pair descent is sound
    because it has no free dynamic vars; you do NOT need to fix that here, only
    to key on `evaluation_epoch` so any epoch bump invalidates every entry).
  - `crates/rum-core/src/profiling.rs:3` (`RUM_PROFILE_REPR`, the env-gated
    cold-load `AtomicU8` pattern) — mirror this for a new `RUM_PROFILE_VALUE_MEMO`
    gate. `crates/rum-core/src/expr.rs` — `Expr`/`CallData` already `derive(Hash)`
    (lines ~469/506/570/652), so `std::hash::Hash` of a call is available for a
    structural key; a `u64` from `DefaultHasher`/`ahash` over `(head, args)` is
    your naive key.
  - `scripts/pair-simp-hotspot-probes.sh` — the bounded-descent harness. The
    probe is `runtime-profile-basic-only-pair4` (file
    `scripts/perf-probes/pair-simp/runtime-profile-basic-only-pair4.wls`); it is
    the EXACT probe the 2026-06-22 shadow memo used (`total≈397105`). It builds
    temporary instrumented Simp/Pair copies in scratch and never edits the
    canonical checkout.
- Acceptance/source clone (READ-ONLY): `/Users/fft/Developments/mma-hol-acceptance`
  (HEAD `200f13a`). Pass it as `--mma-hol` / `RUM_MMA_HOL_ROOT`. Never edit it.
- rum is a Rust reimplementation of a Wolfram-language kernel slice. Acceptance
  here is mechanical: the pasted counter totals + the wall A/B decide, not prose.

## Scope

- Files you MAY modify (THROWAWAY, uncommitted, left in the tree for Claude):
  - `crates/rum-runtime/src/runtime/eval_core.rs` — add the env-gated shadow /
    short-circuit hook in the `Call` arm and the counter plumbing.
  - `crates/rum-runtime/src/runtime/` — one NEW throwaway module file
    (e.g. `value_memo_probe.rs`) for the memo map + counters + env gate + the
    `Drop`/eprintln dump, wired in via `mod`. Prefer a single new file over
    scattering edits.
  - You MAY add a `dev-dependencies`/`dependencies` line ONLY if you need
    `ahash` and it is not already present; if `std::collections::hash_map::DefaultHasher`
    suffices, add NO dependency (preferred — this is throwaway). If you touch
    `Cargo.toml`/`Cargo.lock`, say so loudly in the report.
- Files/areas you MUST NOT touch:
  - anything under any `mma-hol` clone or the `mma-hol` standard — NEVER edit the
    acceptance standard; the probe scripts already copy into scratch.
  - `crates/rum-core/src/expr.rs` `Expr`/`Symbol`/`CallData` representation or any
    public type/serde shape.
  - `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except appending to your own
    report; `.git`, git config.
- **No default-path cost.** Every counter/hash/map operation MUST be behind the
  `RUM_PROFILE_VALUE_MEMO` env gate read once into a cold-load `AtomicU8` (the
  `RUM_PROFILE_REPR` pattern). When the flag is off, the `Call` arm must do a
  SINGLE relaxed atomic load and fall straight through to the existing
  `self.evaluate_call(head.clone(), args)` — no hashing, no map, no allocation,
  no extra branch beyond the one guarded load. Prove this with the microbench A/B
  in Verification (flag OFF must be flat vs `main`).

## Deliverable

Three measurements on `runtime-profile-basic-only-pair4` (current release binary),
plus a verdict. All from `RUM_PROFILE_VALUE_MEMO`-gated throwaway code:

- **A — reproduce the count-weighted prize (shadow mode).** In the `Call` arm,
  when the gate is on: compute the naive structural key `u64` of the dispatched
  `(head, args)` call, run the real `evaluate_call`, then on a repeat key
  (same key AND same `evaluation_epoch`) recompute-and-COMPARE the fresh result
  against the stored one. Do NOT short-circuit in this mode. Report:
  `total`, `hits`, `hit_rate`, `mismatches`, `distinct`. Expected ≈ the
  2026-06-22 row (`total≈397105, hit_rate≈0.2565, mismatches=0`); the point is to
  confirm it still holds on the 32-byte `Expr` (a materially different hit_rate or
  any `mismatches>0` is itself the finding).

- **B — net wall-clock effect of a REAL memo WITHOUT hash-consing (the decisive
  number).** Add a second gate mode that ACTUALLY short-circuits: on a hit
  (key match + epoch match) return the stored result WITHOUT calling
  `evaluate_call`; on a miss, evaluate and store. Naive structural keys (the
  O(expr) hash is paid on 100% of dispatches — exactly the no-hash-consing case).
  Measure the bounded-descent internal `AbsoluteTiming` (or the probe's own
  reported time) three ways, best-of-N (N≥3), same session/machine idle:
  1. gate OFF (baseline),
  2. gate = shadow (A mode: hash+store+compare, no short-circuit) — isolates the
     key+bookkeeping COST,
  3. gate = short-circuit (B mode) — the net effect.
  Report all three times + the ratios `(2)/(1)` (pure overhead of naive keying)
  and `(3)/(1)` (net effect of the memo). **CONFIRM the descent's final result is
  unchanged between (1) and (3)** (paste the probe's terminal result/eq check for
  both) — this is the empirical soundness check for the short-circuiting version,
  complementing A's `mismatches`.

- **C — key-cost attribution.** From B's `(2)/(1)` you already have the naive-key
  overhead as a fraction of the descent. State it explicitly as "the O(expr)
  structural-hash on 100% of dispatches costs X% of the descent wall", and the
  per-dispatch ns (`(time(2)-time(1)) / total`). This is the number hash-consing
  would have to recover (cheap pointer-identity keys) for the count-weighted 26%
  to convert to wall time.

Assemble a small table: `hit_rate | mismatches | (2)/(1) naive-key overhead |
(3)/(1) net memo effect | verdict`.

## rum / Rust pitfalls (read twice)

1. **Measure internal `AbsoluteTiming` / the probe's reported time, never raw wall
   time.** rum-cli startup contaminates wall. Compare rum-internal to rum-internal
   only (this is a within-rum A/B, no WK needed).
2. **The gate must be truly zero-cost when off** (pitfall 3 in the template): one
   relaxed `AtomicU8` load, then the unchanged existing call. No `Mutex`, no map
   touch, no hash when off. The microbench A/B (flag OFF vs `main`) is the proof.
3. **Suspension safety** (pitfall 4): the `Call` arm feeds `evaluate_call`, which
   is on the suspend/resume path. In short-circuit mode you return a stored `Expr`
   in place of a completed `evaluate_call` result — that is fine ONLY for results
   that actually completed (you store the returned `Ok(Expr)`); do NOT attempt to
   memoize across a suspension. If `evaluate_call` can return a suspension
   sentinel rather than a finished value on this path, do NOT store or short-
   circuit those — store only fully-evaluated `Ok` results and say so in the
   report. When unsure, store conservatively (fewer entries) — a missed memo
   under-counts the prize (safe) but a stored suspension is a correctness trap.
4. **Epoch validation is mandatory.** Key entries on `(structural_key,
   evaluation_epoch)` or clear the map on any epoch bump; a stale entry surviving
   an epoch bump would be the classic value-memo unsoundness. The Pair descent
   has no epoch bumps mid-descent (that is why it is sound), so this mainly guards
   the measurement from surprises — but implement it correctly regardless.
5. **One variable per measurement** (pitfall 6): B's three runs differ ONLY in the
   env var; same binary, same probe, same machine-idle. Verify no maintenance
   daemon (`mds`/`backupd`) is at CPU before timing.
6. **Stop-loss** (pitfall 7): if A's `hit_rate` reproduces but B shows the memo is
   a clear net LOSS AND C shows the key cost is the whole cause, that is a COMPLETE
   finding — STOP, report it (it means hash-consing is the required enabler and the
   go/no-go passes to the hash-consing cost, a separate brief). Do not start
   building hash-consing here. Equally, if B shows a clear net WIN with naive keys,
   STOP and report that (value-memo may be worth it even without hash-consing).

## Verification (MANDATORY — you run it; iterate to evidence)

This is a diagnostic brief: the EVIDENCE is the deliverable. Build release first,
then gather A/B/C. Paste every command and its verbatim key lines.

```
cargo build --release
cargo fmt --check
cargo test -p rum-runtime          # the throwaway module must compile clean; no behavior test expected
```

Then the descent measurements (adapt the exact flags; paste what you actually ran):

```
# baseline / shadow / short-circuit — one env var is the only difference
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 \
  --engine rum --output-dir /tmp/cdx_brief_067_base
# then rerun with RUM_PROFILE_VALUE_MEMO=shadow and RUM_PROFILE_VALUE_MEMO=short (best-of-3 each)
```

Then the OFF-path zero-cost proof (the standing hot-path rule):

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_067_micro
# paste ratios for downvalue-exact-hit, if-true, pattern-hit vs main — flag OFF must be flat
```

Paste: A's counter totals; B's three times + two ratios + the two terminal-result
lines proving (1)==(3); C's overhead % and per-dispatch ns; the microbench OFF-path
rows.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the throwaway
  instrumentation UNCOMMITTED in the working tree; Claude reproduces then reverts it.
- Do NOT revert your own instrumentation and do NOT commit it — Claude needs it to
  reproduce the numbers during the gate.
- Do NOT modify files outside Scope. Never edit any `mma-hol` file.
- **No new `unsafe`** in `rum-core`/`rum-runtime` (both are zero-`unsafe` today).
  A safe `HashMap<u64, (Expr, u64)>` + `AtomicU8` gate is all this needs.
- **The OFF path is a hot path** — the microbench A/B (flag OFF vs `main`) is
  mandatory, not optional. A regression on any hot label with the flag OFF is a
  defect to fix before reporting.
- Stop-loss: a complete finding (net loss+key-cost-explains-it, OR net win) → STOP
  and report; do not start the hash-consing implementation.
- Minimal diff: no drive-by reformatting/renames outside the throwaway hook.
- Paste REAL verbatim output. If a run did not produce evidence, say so — never fake.

## Report format (your final message)

1. What changed — the throwaway hook + module, per file with line ranges and the
   env-gate mode it adds (explicitly: "throwaway, uncommitted, for Claude to revert").
2. Verification — each command with verbatim output (test counts, the A counters,
   the B three-times + ratios + the two terminal-result lines, C's numbers, the
   microbench OFF-path rows).
3. The finding / verdict — the cost-benefit table + which of the three go/no-go
   branches the numbers land in (net win w/o hash-consing / prize real but blocked
   by key cost → hash-consing is the enabler / prize too small → kill task-1).
4. Stop-loss / open questions — where you stopped, anything unsure (e.g. whether
   any dispatched result on this path is a suspension sentinel you excluded).

---

## Outcome (Claude gate, 2026-07-01) — DONE. Verdict: naive value-memo is a net loss; hash-consing is the required enabler.

Codex ran the throwaway instrumentation over three rounds (self-corrected the
counter granularity to dispatched-calls-only in v3); Claude gated by reproducing
the base-vs-short A/B independently and reverted the throwaway code (nothing
committed but this doc + the PROGRESS/handoff notes). Scope clean (only
`eval_core.rs`+`mod.rs`+the new throwaway module were touched; no `Expr`/serde/
mma-hol). OFF-path microbench flat vs `main` (if-true/downvalue ratios ~0.95-0.96).

**Measured (v3, `runtime-profile-basic-only-pair4`, current 32-byte `Expr`):**

| metric | value | reading |
|---|---|---|
| shadow `hit_rate` | **0.2565**, `mismatches=0`, `distinct=160448` | count-weighted 26% prize + soundness reproduce the 2026-06-22 row EXACTLY on the new layout |
| shadow/base wall | **~2.36x** | the O(expr) recursive-SipHash key on 100% of dispatches alone adds ~+135% |
| short/base wall | **~1.72x** (Codex AbsoluteTiming); Claude repro 133s/87s = 1.53x wall | a REAL short-circuiting memo with naive keys is a decisive NET LOSS |
| short real hit_rate | **0.066-0.074** | short-circuiting removes the downstream repeats it feeds on, so the achievable prize is ~4x below the shadow ceiling |
| terminal result | `is_thm=True changed=False` for base==short | short-circuit version is empirically sound too |

**Verdict = branch 2 of the brief's three:** the prize is real and sound but
BLOCKED by the key cost. Naive (non-hash-consed) value-memo is off the table.
Converting the 26% ceiling to wall time requires O(1) identity keys =
hash-consing, whose OWN cost (a global intern on 100% of *constructions* — a
hotter path than dispatch) is NOT measured here and could itself eat the
(already-shrunken ~7% live) prize. Full hash-consing remains the
`representation-rewrite-plan.md` §"Out of scope" largest-commitment lever.

**Recommendation:** do NOT start the hash-consing build on this evidence. Either
(i) a tighter follow-up that measures the O(1)-key ceiling AND the interning cost
before committing, or (ii) pivot to task-2 (bytecode VM), which attacks the same
distributed per-call constant more directly. Scope decision is the user's.
