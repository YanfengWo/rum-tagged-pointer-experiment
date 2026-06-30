# brief-052: extend the memory comparator into the four-C panel (peak / retained / transient / post-cleanup)

## Goal

Upgrade `scripts/core-memory-microbench.sh` from its current three raw metrics
({`MemoryInUse` delta, `MaxMemoryUsed`, peak RSS}) into the **four-C memory
evaluation panel** that the rest of the memory line gates against. Each workload, in
both engines, must report — baseline-subtracted, in `MemoryInUse`/`MaxMemoryUsed`
PROXY bytes — the four constants plus the key derived metric, as rum-vs-WK ratios,
and the run must aggregate `C_max`/`C_p95`/`C_geomean` across labels and flag any
ratio that DRIFTS with N (drift ⇒ a complexity/order gap, not a constant factor):

- **peak** = `MaxMemoryUsed − b0` (high-water proxy → OOM-risk surrogate)
- **retained** = `MemoryInUse_after − b0` (final live representation; the existing
  `mem_delta`)
- **transient** = `peak − retained` = `MaxMemoryUsed − MemoryInUse_after`
  (mid-evaluation bloat, distinct from representation overhead)
- **post-cleanup** = `MemoryInUse_postcleanup − b0` (residual after the workload's
  results are cleared = leak / cache / intern-table growth surrogate)

plus the existing real **peak RSS** (from `/usr/bin/time -l`) kept as the OOM metric.
When done, `comparison.tsv` carries all four C's + transient as rum/WK ratios per
label, a new `aggregate.tsv` (or appended block) carries `C_max`/`C_p95`/`C_geomean`
for the headline metrics + the N-drift verdict, and `cargo fmt --check`/`cargo test
--workspace` stay green (this is script-only + at most a tiny builtin-free change —
NO runtime logic change expected).

## Context pointers

- Read `AGENTS.md` and `docs/memory-management-plan.md` (the Phase-0 instruments;
  this brief is the Phase-0.5 yardstick upgrade) + the four-C entry in `TODO.md`
  "Current Cut".
- Key files:
  - `scripts/core-memory-microbench.sh` — the comparator to extend (read it fully).
    Today: `run_workload` builds `program = "b0 = MemoryInUse[]; <expr>; b1 =
    MemoryInUse[]; mx = MaxMemoryUsed[]; Print[\"RUM_MEM\t<label>\t\", b0, b1, mx]"`,
    runs it under `/usr/bin/time -l`, parses `RUM_MEM` + peak RSS, and the trailing
    `awk` writes `comparison.tsv` with mem_delta/max_memory/rss ratios. The labels +
    `workload_expr()` (:114-129) define the workloads.
  - `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs` — `MemoryInUse`
    / `MaxMemoryUsed` proxies (already implemented; no change expected).
- rum is a Wolfram-language-kernel slice in Rust; `MemoryInUse[]`/`MaxMemoryUsed[]`
  are deterministic structural PROXIES — ALWAYS compare baseline-subtracted DELTAS,
  never absolute values (rum proxy ≠ WK real-kernel scale). Acceptance is mechanical:
  the new TSV columns + the aggregate rows, reproduced verbatim.

## Implementation (precise)

1. **Extend the per-workload program template** (`run_workload`, ~:184) to also do a
   cleanup + a post-cleanup proxy read. New template (keep the literal `RUM_MEM` tab
   protocol, just add one trailing field `bp`):
   ```
   b0 = MemoryInUse[]; <expr>; b1 = MemoryInUse[]; mx = MaxMemoryUsed[];
   <cleanup>; bp = MemoryInUse[];
   Print["RUM_MEM\t<label>\t", b0, "\t", b1, "\t", mx, "\t", bp]
   ```
   The cleanup is per-workload (a workload that binds `memGlobal` must
   `ClearAll[memGlobal]`; a workload that binds nothing uses `Null`). Add a
   `workload_cleanup()` shell function mirroring `workload_expr()`. Use only WL that
   BOTH engines accept (`ClearAll[sym]`, `Null`). Do NOT add a cache-clear builtin
   that one engine lacks; if rum has no portable cache-clear, post-cleanup = "after
   ClearAll" is the defined metric (document that in `--help`).
2. **Parse the new `bp` field** (`parse_measurement`, the `RUM_MEM` awk) and add a
   `mem_postcleanup` column to `measurements.tsv` (extend its header :111).
3. **Compute the four C's + transient in the trailing awk** (:222-265) and rewrite
   `comparison.tsv` (extend the header :112) so each label row carries, for rum and
   WK and their ratio: `retained` (= b1−b0), `peak` (= mx−b0), `transient` (= mx−b1),
   `post_cleanup` (= bp−b0), and the existing real `peak_rss`/`rss_delta`. Keep the
   `ratio()` helper's divide-by-zero guard.
4. **Aggregate + drift check.** Emit `aggregate.tsv` (or a clearly-delimited block):
   for the headline ratio columns (`retained` and `peak`), compute across all
   non-baseline labels `C_max`, `C_p95`, `C_geomean` (geometric mean of the per-label
   ratios). AND a drift verdict on the `retain-range` N-sweep: compare the `retained`
   ratio at the smallest vs largest N; if `|ratio_bigN − ratio_smallN| / ratio_smallN
   > 0.15`, print `DRIFT (ratio not constant — suspect order gap)`, else `STABLE`.
5. **Add workloads** so the panel exercises all four C's and the drift axis. Keep the
   existing `baseline`, `module-leak-20000`, `retain-range-1000`, `retain-range-10000`;
   add:
   - `retain-range-100000` (a third N point for a real drift check),
   - `transient-total-200000` = `Total[Range[200000]]` (a high-transient, ~zero-retained
     workload: builds a big transient list, returns one integer → `transient` ≫
     `retained`; the metric that should expose mid-eval bloat).
   Wire each into `labels`, `workload_expr()`, and `workload_cleanup()`.
6. **Allocation-traffic (the 4th ChatGPT C) — OPTIONAL, rum-LOCAL, document don't
   force.** WK has no portable total-allocated-bytes metric, so this is NOT a rum/WK
   ratio. IF the `alloc-count` GlobalAlloc cfg (`crates/rum-cli/src/alloc_count.rs`,
   env `RUM_PROFILE_ALLOC`) can be invoked cleanly with the EXISTING binary/env (no
   second build required), add a rum-only `alloc_bytes` column; otherwise add the
   column as empty and document in `--help` that allocation-traffic is a follow-up
   needing the `alloc-count` build. Do NOT add a second mandatory binary build to the
   default path.

## Scope

- Files you MAY modify: `scripts/core-memory-microbench.sh` ONLY (plus its `--help`).
  If — and only if — a genuinely missing portable metric forces a tiny runtime
  helper, STOP and report instead (do not silently add a builtin).
- MUST NOT touch: any `mma-hol` clone; `crates/**` (this is script-only — the proxies
  already exist); `crates/rum-core/src/expr.rs`; `AGENTS.md`/`PROGRESS.md`/`TODO.md`/
  `docs/**` except appending to your own report; `.git`, `Cargo.lock`.
- No default-path cost (script-only; nothing runs in the evaluator hot path).

## Deliverable

- The extended `core-memory-microbench.sh` (post-cleanup measurement + four-C/transient
  columns + aggregate + drift verdict + the 2 new workloads + optional rum-local
  alloc column).
- A `--engine both` run's `comparison.tsv` + `aggregate.tsv` pasted verbatim in the
  report.

## Verification (MANDATORY — run, paste verbatim)

```
cargo fmt --check
cargo test --workspace
bash scripts/core-memory-microbench.sh --engine both --output-dir /tmp/cdx_brief_052
```

Then paste verbatim:
- the full `comparison.tsv` (header + every label row),
- the full `aggregate.tsv` (or the aggregate block) incl. the `C_max`/`C_p95`/
  `C_geomean` rows and the N-drift `STABLE`/`DRIFT` verdict,
- confirm `module-leak-20000` `post_cleanup` is now SMALL (the Phase-1 gensym GC
  reclaims it; this is a sanity check that the post-cleanup metric works), and
  `transient-total-200000` shows `transient ≫ retained`.

If `cargo test --workspace` is not green (it should be untouched — this is
script-only), STOP and report; do not edit `crates/**` to chase it.

## Hard rules

- Do NOT `git commit`, branch, push, or touch git config. Leave changes in the
  working tree; Claude gates + commits.
- Do NOT modify files outside Scope. If a metric truly needs a runtime change, STOP
  and write that in the report.
- Full machine access, repo only, no network. Commands: edits to the script,
  `cargo fmt`/`cargo test`, the microbench script, `wolframscript` (via the script),
  `python3` only if needed for parsing.
- Never edit any `mma-hol` file.
- Stop-loss: if the WK side errors on a workload (e.g. a cleanup form WK rejects),
  fix the form to be portable; if it errors twice for the same reason, STOP and
  report the exact WK stderr.
- Minimal diff: no drive-by reformatting of the unaffected parts of the script.
- Paste REAL verbatim output; if a run did not complete, say so — never fabricate
  numbers.

## Report format (your final message)

1. What changed — the script sections touched, with line ranges + one-line why.
2. Verification — each command + verbatim output (the two TSVs in full, the fmt/test
   result lines).
3. The finding — confirm post_cleanup is small for module-leak (GC works) and
   transient ≫ retained for the transient workload; note any DRIFT verdict.
4. Stop-loss / open questions (empty if none).
