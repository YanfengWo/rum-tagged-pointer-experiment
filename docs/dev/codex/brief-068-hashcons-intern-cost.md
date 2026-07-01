# brief-068 — hash-consing intern cost (B0 diagnostic, task-1 go/no-go decider)

## Goal

Settle the go/no-go for the deferred **hash-consing / value-memo** perf project
(task 1 of the perf line) by MEASURING the one cost brief-067 left unmeasured: the
per-construction **interning overhead** that hash-consing would impose on the
construction hot path. brief-067 proved the value-memo prize is real+sound but
BLOCKED by the O(expr) dispatch key cost, so the only way to harvest the ~26%
count-weighted (≈7% live) prize is O(1) identity keys = hash-consing — which pays
its own cost by interning every constructed expression into a global structural
table (a probe on 100% of constructions, a HOTTER path than dispatch). This brief
measures that interning cost on the SAME Pair descent as the prize (a common
denominator) so the verdict is a direct comparison: **if interning-on-every-
construction ALONE already costs ≥ the ~7% live prize, hash-consing cannot pay and
task-1 is dead** — even before adding the memo. Diagnostic/scoping brief:
throwaway, env-gated instrumentation only, NO product-code behavior change, NO
default-path cost, NO `Expr` layout change. Deliverable is a cost table + verdict
with verbatim numbers. Leave the throwaway instrumentation UNCOMMITTED for Claude
to reproduce during the gate (Claude reverts it — do NOT revert or commit it).

## The faithfulness rule (READ FIRST — the whole verdict hinges on it)

Real hash-consing computes a node's structural key in **O(arity)**: it combines
the node's head + its immediate children's ALREADY-CACHED interned ids/hashes. It
does NOT re-walk the subtree. A full recursive `expr.hash()` per construction is
**O(subtree)** and repeats exactly the mistake brief-067 measured on dispatch
(the O(expr) SipHash that cost +135%) — it OVERSTATES the interning cost by ~the
tree depth and would falsely kill the project. So:

- **Model R (realistic — DRIVES the verdict):** on each Call-node construction,
  compute an **O(arity)** key = hash of (head identity proxy, each immediate
  arg's identity proxy). An "identity proxy" is: the `Arc` pointer address for a
  shared child (`SharedExpr`/`SharedArgs`/a `Call`'s args `Arc`), or the atom
  value for an atom. Do NOT recurse into children. Then probe a thread-local
  intern table (lookup; insert-on-miss). This models what hash-consing actually
  pays per construction.
- **Model U (naive upper bound — for contrast only):** full recursive
  `expr.hash()` (O(subtree)) + the same table probe. Report it to bracket the
  range, but the go/no-go is Model R.
- **Known bias to state, do not fix:** the prototype's pointer-proxy ids will NOT
  dedup structurally-equal-but-freshly-Arc'd children, so almost every
  construction is a table MISS+insert. That makes R a *conservative upper bound of
  the realistic per-op cost* (real hash-consing has hits = lookup-only, no insert,
  and saves an allocation by returning the shared node). If even this conservative
  R is below the prize, the real cost is lower still — a clean "headroom exists"
  signal. If conservative R already exceeds the prize, that is decisive against.

## Context pointers

- Read `AGENTS.md` (operational notes; unmodified-`mma-hol` hard line) and:
  - `PROGRESS.md` `## 2026-07-01 Perf line reopened — brief-067 value-memo scoping`
    (the prize + the key-cost finding this brief follows up) and the two
    `## 2026-06-22` hash-consing/value-memo sections (the original caveat 2:
    "capturing the 26% REQUIRES cheap keys → hash-consing … interning overhead
    must not eat the win" — this brief measures that overhead).
  - `docs/dev/codex/brief-067-value-memo-scoping.md` Outcome section (the verdict
    this extends) and `docs/representation-rewrite-plan.md` §"Out of scope" (full
    hash-consing = the largest-commitment lever; this brief decides whether it is
    worth designing).
- Key files:
  - `crates/rum-core/src/expr.rs:716` `Expr::call(head, args: Vec<Expr>)`,
    `:723` `Expr::call_shared(head: SharedExpr, args: SharedArgs)`, `:727`
    `Expr::call_with_shared_args` — the three Call constructors. The single
    construction chokepoint that fires on ~100% of Call nodes is where the args
    `Arc` is built; `crates/rum-core/src/profiling.rs` `record_shared_args_alloc()`
    / `record_shared_expr_alloc()` (the existing `RUM_PROFILE_REPR` `AtomicU64`
    counters, brief-016) already sit on that path — reuse the construction COUNT
    from these for the per-op-ns math, and hook the intern probe at the same
    chokepoint so it fires once per constructed Call node.
  - `crates/rum-core/src/profiling.rs` (119 lines) — the cold-load env-gated
    `AtomicU8`/`AtomicU64` pattern (`RUM_PROFILE_REPR`) to MIRROR for the new
    `RUM_PROFILE_INTERN_COST` gate.
  - `scripts/pair-simp-hotspot-probes.sh` — the Pair-descent harness; probe
    `runtime-profile-basic-only-pair4` (the exact one brief-067 used, base wall
    ~57-90s). This is the primary denominator (same workload as the prize).
  - `scripts/core-language-microbench.sh` — construction-heavy labels
    (`list-literal-small`, `list10`, and the nested/`Range` labels) for the clean
    per-construction ns and the OFF-path no-regression proof.
- Acceptance/source clone (READ-ONLY): `/Users/fft/Developments/mma-hol-acceptance`
  (HEAD `200f13a`). Pass as `--mma-hol` / `RUM_MMA_HOL_ROOT`. Never edit it.

## Scope

- Files you MAY modify (THROWAWAY, uncommitted, left for Claude to revert):
  - `crates/rum-core/src/expr.rs` — the intern-probe hook in the Call
    construction chokepoint (or the three constructors), env-gated.
  - `crates/rum-core/src/profiling.rs` (or a NEW throwaway
    `crates/rum-core/src/intern_cost_probe.rs` wired via `mod`) — the intern
    table + counters + the `RUM_PROFILE_INTERN_COST` gate + a dump.
  - Prefer NO dependency; `std::collections::HashMap` +
    `std::collections::hash_map::DefaultHasher` suffice. If you add any
    dep/`Cargo.toml`/`Cargo.lock` line, say so loudly.
- **You MUST NOT change the `Expr`/`Symbol`/`CallData` layout or any public
  type/serde shape.** No new field on any node (that is the very thing we are
  avoiding). The intern table is EXTERNAL (a side table keyed by the proxy key);
  it does not alter the node.
- Files/areas you MUST NOT touch: any `mma-hol` clone/standard (the probe scripts
  copy into scratch); `AGENTS.md`, `PROGRESS.md`, `TODO.md`, `docs/**` except your
  own report; `.git`, git config.
- **No default-path cost.** Every hash/table/counter op MUST be behind the
  `RUM_PROFILE_INTERN_COST` env gate read once into a cold-load `AtomicU8` (the
  `RUM_PROFILE_REPR` pattern). Flag OFF ⇒ a SINGLE relaxed atomic load then the
  unchanged constructor — no hash, no table, no allocation, no extra branch. Prove
  it with the microbench OFF A/B.

## Deliverable

On `runtime-profile-basic-only-pair4` (primary, same denominator as the prize) and
the construction-heavy microbench labels, current release binary:

- **P1 — the decisive Pair-descent comparison.** Best-of-N (N≥3), machine idle,
  one env var the only difference:
  1. `RUM_PROFILE_INTERN_COST` OFF (baseline),
  2. `=arity` (Model R — O(arity) key + table probe on every construction),
  3. `=subtree` (Model U — O(subtree) `expr.hash()` + table probe).
  Report the three internal `AbsoluteTiming`s + ratios `R/base` and `U/base`.
  **`R/base − 1` is the interning overhead as a fraction of the descent** — the
  number to compare against the ~7% live prize (and the ~26% count ceiling).
  Also report the construction count on the descent (from the REPR counters) and
  the per-construction ns for R and U (`(time − base)/constructions`).

- **P2 — clean per-construction ns + OFF-path proof.** Run
  `core-language-microbench.sh` for R, U, and OFF. For the construction-heavy
  labels report the per-op ns (R and U) and confirm the OFF run is flat vs a clean
  `main` binary (the zero-cost-when-off proof).

- **Assemble the table:** `model | Pair R/base ratio | overhead % of descent |
  per-construction ns | vs ~7% live prize | verdict`.

## rum / Rust pitfalls (read twice)

1. **O(arity), not O(subtree), for Model R** — see the faithfulness rule above.
   Getting this wrong is the one way this brief produces a false verdict.
2. **Measure internal `AbsoluteTiming` / the probe's reported time, never raw
   wall** (rum-cli startup contaminates wall). Within-rum A/B; no WK needed.
3. **The gate must be truly zero-cost when off** — one relaxed `AtomicU8` load,
   then the unchanged constructor. The microbench OFF A/B is the proof.
4. **No `Expr` layout change** — the intern table is a side table; do not add a
   cached-id/hash field to any node (that would be starting the implementation,
   and it changes the layout this project must not perturb here).
5. **One variable per measurement** — the three P1 runs differ ONLY in the env
   var; same binary, same probe, machine idle (check `mds`/`backupd` at 0% first).
6. **Stop-loss** — if R/base clearly exceeds the prize (interning alone eats it),
   that is a COMPLETE decisive finding: STOP, report "task-1 dead", do NOT start
   building hash-consing or the integrated memo. If R/base is comfortably below
   the prize, STOP and report "headroom exists → recommend the integrated
   intern+O(1)-key-memo confirm as the next brief" — do NOT build it here.

## Verification (MANDATORY — you run it; iterate to evidence)

Diagnostic brief: the EVIDENCE is the deliverable. Build release first.

```
cargo build --release
cargo fmt --check
cargo test -p rum-core         # the throwaway hook must compile clean; no behavior test expected
```

Pair descent (adapt flags; paste what you ran), e.g.:

```
RUM_MMA_HOL_ROOT=/Users/fft/Developments/mma-hol-acceptance \
  bash scripts/pair-simp-hotspot-probes.sh --only runtime-profile-basic-only-pair4 \
  --engine rum --timeout 300 --output-dir /tmp/cdx_brief_068_base
# rerun with RUM_PROFILE_INTERN_COST=arity and =subtree (best-of-3 each)
```

Microbench per-op ns + OFF proof:

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_068_micro
# paste construction-label rows for arity/subtree/off; off must be flat vs main
```

Paste: P1 three times + two ratios + construction count + per-op ns; P2 per-op ns
+ the OFF-path rows.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave the throwaway
  instrumentation UNCOMMITTED; Claude reproduces then reverts it. Do NOT revert it
  yourself and do NOT commit it.
- Do NOT modify files outside Scope. Never edit any `mma-hol` file. No `Expr`
  layout/field change.
- **No new `unsafe`** in `rum-core`/`rum-runtime` (both zero-`unsafe` today). A
  safe `HashMap` + `AtomicU8`/`AtomicU64` is all this needs.
- **The OFF path is a hot path (constructors)** — the microbench OFF A/B (flag OFF
  vs `main`) is mandatory. Any regression with the flag OFF is a defect to fix
  before reporting.
- Stop-loss: a decisive finding (R eats the prize, OR R has clear headroom) → STOP
  and report; do not start the implementation or the integrated experiment.
- Minimal diff: no drive-by reformatting/renames outside the throwaway hook.
- Paste REAL verbatim output. If a run did not produce evidence, say so — never fake.

## Report format (your final message)

1. What changed — the throwaway hook + table, per file with line ranges and the
   env-gate modes it adds ("throwaway, uncommitted, for Claude to revert"); confirm
   NO `Expr` layout change.
2. Verification — each command with verbatim output (test counts, P1 three times +
   ratios + construction count + per-op ns, P2 rows, the OFF-path rows).
3. The finding / verdict — the cost table + which branch: "task-1 dead (interning
   ≥ prize)" or "headroom exists (interning < prize) → recommend integrated confirm".
4. Stop-loss / open questions — where you stopped, anything unsure (e.g. whether
   the O(arity) proxy captured all Call constructions, or table-miss bias notes).

---

## Outcome (Claude gate, 2026-07-01) — DONE. Verdict: MARGINAL/conditional, not a clean "headroom" — the enabler taxes 100% of constructions and only pays back on descents.

Codex built a faithful throwaway probe (verified: `arity_intern_key` hashes head +
`for arg in args` O(arity) via `ptr_key()` proxies, no subtree recursion; hooked in
all three `Expr::call*` constructors; no `Expr` layout change; OFF-path microbench
flat, off/main ~1.0). Claude reverted it (nothing shipped but this doc + PROGRESS/
handoff). Codex's headline was "headroom exists"; Claude's independent reproduction
found the picture is more cautionary, from a denominator distinction:

**Measured on `runtime-profile-basic-only-pair4`:**

| lens | number | reading |
|---|---|---|
| Codex, DESCENT SEGMENT (internal `AbsoluteTiming`) | arity/base = **0.98** (≈ noise), subtree/base = **2.68x** | on the descent segment the O(arity) intern cost is absorbed by dispatch/eval work; O(subtree) is disastrous (confirms the faithfulness rule mattered) |
| Claude, FULL RUN (wall, base 94/95s → arity 106/107s) | arity/base ≈ **1.12x** (tight, reproducible, NOT noise) | the enabler adds ~+12% over the whole load+descent |
| Codex, construction-heavy microbench (per-op) | arity **+14% to +100%** (`table-iterator-pair-list` 121→241 ns) | the raw per-construction tax is real on construction-dominated code |

**Reconciliation:** the descent-segment ≈noise and the full-run +12% are both true —
the delta lives in the NON-descent load, where the counters show **26.1M of the 27.2M
constructions** happen (avg_arity 1.88). The intern tax is only ~free when amortized
against descent dispatch work; on construction/load-heavy work it is a real 12-100%
tax with NO offsetting memo benefit (the value-memo prize is descent-only). Caveat
inflating the full-run number: the prototype does not dedup (pointer-proxy ids →
98.4% table MISS → a 27M-entry HashMap with poor cache behavior); real hash-consing
dedups to a far smaller table (the descent has only ~160K distinct dispatched
structures), so the true at-scale per-op cost is lower than this prototype's — but
it is NOT zero, and it applies engine-wide.

**Verdict = conditional/marginal, NOT a clean go.** Task-1's economics: a ~7% live
descent prize (brief-067) vs a global construction tax that is ≈noise on descents
but +12-100% elsewhere. Net depends on the deployment workload mix and on the real
(deduped) table's per-op cost — neither is settled by a cost-only probe. This can
only be closed by the INTEGRATED experiment: a real (deduping) hash-cons prototype
whose O(1) shared identities key the short-circuiting value-memo, measured as NET
wall on the descent AND on a construction/load-heavy workload. **Recommendation:
this is the go/no-go decider brief IF task-1 is pursued — but given the prize is
modest (~7%) and the enabler now has a measured pervasive tax, the honest read is
task-1 is marginal; pivoting to task-2 (bytecode VM), which attacks the same per-call
constant with no global construction tax, is at least as attractive.** Scope decision
is the user's.
