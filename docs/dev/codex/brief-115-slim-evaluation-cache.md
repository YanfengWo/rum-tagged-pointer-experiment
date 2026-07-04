# Brief 115 — file-slimming batch 2: evaluation_cache.rs (63KB, HOT) — mechanism/instrumentation split

## Goal

Second slimming batch, first HOT file. Split
`crates/rum-runtime/src/runtime/evaluation_cache.rs` (63,261 chars, 1681
lines) into an `evaluation_cache/` module directory per the SPLITTING
PRINCIPLES (brief-114 §THE SPLITTING PRINCIPLES — read them there; they
govern every decision: by logic/function never mechanically, stable/churn
separation + sibling DAG for incremental compilation, boundaries aligned
with future refactoring, pure organization with verbatim bodies, SPLIT MAP
FIRST in the report). Zero behavior change; `cargo test --workspace` green
(currently 1301/0); every new file ≤20k chars; **microbench A/B required**
(this file's `impl Session` block runs on every evaluation call).

## Context pointers

- Read `AGENTS.md`, `docs/code-organization-guidelines.md`, and brief-114's
  principles section first.
- Measured structure of the file (verify before mapping):
  - lines ~50-186: profile TYPES (SymbolTimingProfile, Invalidation*Key/Stats,
    CallStability* types, EvaluationProfileSourceGuard) + the stability-bloom
    helpers;
  - lines ~187-524: `impl Session` — the CACHE MECHANISM (call-stability
    cache decisions, invalidation, lookups) — the HOT core;
  - lines ~525-1572: env-gated PROFILE INSTRUMENTATION (the
    `evaluation_cache_profile_*` AtomicU8 gates, counter maps behind lazy
    Mutexes on the SLOW path only, `record_*` entry points) — cold by
    construction, but its fast-outs are called FROM hot paths;
  - lines 1573+: `mod tests` (~108 lines).
- The natural axis is MECHANISM vs INSTRUMENTATION — they merely cohabit
  historically. Suggested map (verify cohesion, adjust with justification):
  `mod.rs` (facade + shared types), `stability.rs` (the impl Session cache
  mechanism + bloom helpers), `profile.rs` (gates, counters, guards,
  record_* — possibly further split if a cohesive line exists, e.g.
  counters vs source-timing), `tests.rs`. FUTURE-REFACTORING note: the
  instrumentation half is conceptually profiling-domain code — your boundary
  should make a LATER wholesale move into `runtime/profiling/` trivial, but
  do NOT do that move in this brief (one step at a time; batch 1 just
  established that directory).
- HOT-FILE RED LINES: (1) every function body VERBATIM — especially the
  `impl Session` mechanism and every disabled-path fast-out (single relaxed
  atomic load, early return); (2) no new indirection on the mechanism path;
  (3) same-crate module boundaries do not block inlining, but you still run
  the A/B to prove it.

## Microbench A/B (you run it — the exception to the usual gate split)

BEFORE any edit:

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_115_micro_base
```

After the split is green:

```
bash scripts/core-language-microbench.sh --engine rum --output-dir /tmp/cdx_brief_115_micro_new
```

Paste the per-label base-vs-new table. Judgment rules learned on this line:
sub-ms labels jitter ±10-15% on this machine after repeated bench cycles —
judge only consistent direction on cache-relevant labels (downvalue-*,
set-ownvalue, symbol-ownvalue-read, do-small); ONE suspicious-label re-run
allowed, then report. The gate will independently re-run if anything looks
off.

## Scope

- MAY modify: `crates/rum-runtime/src/runtime/evaluation_cache.rs` →
  `evaluation_cache/` (directory); `use` lines inside the new directory;
  `runtime/mod.rs` ONLY if declaration syntax demands (likely zero change).
- MUST NOT touch: any caller outside the directory (imports resolve via the
  facade unchanged), any behavior, any test content (the in-file tests move
  verbatim to `evaluation_cache/tests.rs`), `Cargo.toml`/`Cargo.lock`, any
  `mma-hol` clone, `docs/**` except your report, `.git`.

## Verification (MANDATORY — you run it; iterate to green)

```
cargo fmt --check
cargo test --workspace
ls crates/rum-runtime/src/runtime/evaluation_cache.rs 2>&1        # No such file
find crates/rum-runtime/src/runtime/evaluation_cache -name "*.rs" | while read f; do wc -c "$f"; done | sort -rn
git status --short -- crates/                                      # exactly: 1 deletion + the new directory
```

Plus the A/B table above. Every new file ≤20k chars (justify exceptions per
principle 1).

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config; changes stay in the
  working tree for Claude's gate.
- Do NOT modify files outside Scope; STOP and report if you believe you must.
- Full machine access, but nothing outside this repo, no network; only file
  edits in Scope, `cargo`, the microbench script, `grep`, `python3`.
- Never edit any `mma-hol` file. **No new `unsafe`.**
- Probe budget: the two A/B runs + at most one suspicious-label re-run.
- Minimal diff beyond the moves; paste real verbatim outputs; never fake a
  green.
- Stop-loss: any cache-relevant label consistently regressing, a boundary
  that cannot keep callers unchanged, or the same compile error twice → STOP
  and report.

## Report format (your final message)

1. THE SPLIT MAP (module → responsibility → chars; DAG; the
   future-move-to-profiling alignment note).
2. What changed — file inventory; untouched-callers evidence.
3. Verification — each command verbatim incl. the size table AND the A/B
   table with your judgment.
4. Verdict — hot-path red-line assertion; advice for batch 3
   (definition_application.rs 55KB).
5. Stop-loss / open questions (empty if none).
