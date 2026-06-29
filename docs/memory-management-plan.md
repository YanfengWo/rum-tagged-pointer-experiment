# Memory-management optimization plan (the post-correctness mainline)

> Status: PLAN (research done 2026-06-29, no code changes yet). Goal set by the
> user: after the time/perf line reached "WK parity to within a constant factor"
> across the board, do the SAME for **memory** — get rum's memory use to within a
> constant factor of WolframKernel on the supported `mma-hol` slice. Read this
> together with `docs/representation-rewrite-plan.md` (the time line; hash-consing
> is a shared lever) and the [[snapshot-checkpointing-direction]] /
> [[rum-project-context]] memories (the deferred ~10GB Module-gensym leak).

## Why now — the forcing function

The single-process cold `run_all.wls` (the authoritative 3126/0 Strict gate)
ballooned to **50.7 GB RSS** and swap-thrashed for >2h on this 64 GB machine
(2026-06-29, PROGRESS). It is currently INFEASIBLE to run. Root cause = unbounded
Module-gensym memory growth. So memory management is now both the next mainline
AND the prerequisite for the authoritative cold gate.

## Calibration data (rum vs WK, measured 2026-06-29)

| probe | rum | WK | meaning |
| --- | --- | --- | --- |
| `Do[Module[{x}, x=i^2; x], {i,1,20000}]` `MemoryInUse` delta | **6,002,561 B** | **80 B** | the leak: WK GCs gensyms, rum does not (~75000x) |
| `ByteCount[Range[1000]]` | 66,012 | 8,296 | ~8x constant-factor representation overhead |
| `MemoryInUse[]` at startup | 36,756 (proxy) | 62,140,992 (real) | NOT absolute-comparable — use DELTAS only |
| `MaxMemoryUsed[]` | inert (NOT implemented) | 77,588,496 | rum lacks the peak-memory metric |

## Test-method status (the user's difftest question — answered)

- **difftest Lane D already has `--suite memory`** (`cargo run -p rum-difftest --
  lane-d --suite memory`), consuming `seed/ham-import/mem/memory_suite.txt`. BUT it
  is **rum-local regression tracking** (records the `MemoryInUse` proxy + node
  counts vs a baseline, `--fail-on-regression`) — it does **NOT** invoke a Wolfram
  oracle (perf/README.md: "deliberately rum-local"). The cross-engine rum/WK probe
  is `scripts/core-language-microbench.sh`, which is **timing-only today**.
- `MemoryInUse[]` / `ByteCount[]` are **structural PROXIES**
  (`debug_tools/resources/memory.rs` `current_memory_proxy_bytes` walks the Session:
  `definitions`, `attributes`, ownvalues, contexts, eval stack, … and sums
  `size_of` estimates). Deterministic and reproducible, and the proxy **DOES see the
  Module-gensym leak** (it counts `self.definitions`, memory.rs:65 — proven: the
  6 MB delta above). BUT it is **blind to the symbol-interner `&'static` leak** (it
  only walks the Session maps, not rum-core's interner). So: proxy for
  definition-level memory; **real RSS for the true total** (`ps -o rss`, the
  `scripts/guarded-test-run.sh` watchdog, or the `alloc-count` GlobalAlloc cfg).
- **Gaps to build for "within a constant factor of WK":** (1) implement
  `MaxMemoryUsed[]`; (2) a cross-engine rum-vs-WK memory comparator (the memory
  analog of `core-language-microbench.sh`): same workload, measure rum
  {`MemoryInUse` delta, RSS delta, `MaxMemoryUsed`} vs WK {same}, report the ratio;
  (3) keep Lane D `--suite memory` for rum-local regression gating.

## Diagnosis — three levers (rum's memory vs WK)

1. **Module-gensym leak — the dominant O(N) lever.** `eval_module`
   (`scoping/lexical.rs:48`) allocates a fresh symbol per local
   (`allocate_fresh_symbol` / `scoping/unique.rs`), substitutes, evaluates the body
   via `resume_module_initializers`, and **returns the result with NO scope-exit
   cleanup** of the gensym's definitions. Contrast `eval_block` (`lexical.rs:98`),
   which capture+restores (dynamic scope, no new symbols, no leak). Two layers:
   - (a) **definitions** (ownvalue/downvalue/attributes/options on the gensym) pile
     up in `self.definitions` etc. — **reclaimable** via `definition_clear.rs`
     (`remove_matching_down_values:168`; the snapshot path already does
     `Scan[Remove, Names["HOL\`*$*"]]` before DumpSave, so Remove works).
   - (b) the **interned symbol name** is `&'static` in rum-core's interner
     (`crates/rum-core/src/expr.rs:46` `symbol_interner`, `intern_text:56` interns
     into a `HashSet<&'static str>`, append-only) — **NOT reclaimable** without an
     interner redesign. This is a hard floor on the O(1) goal.
2. **Per-expression representation overhead — a ~8x constant-factor lever.**
   `Range[1000]` is a `List` of 1000 boxed `Integer` `Expr`s (Arc + enum + BigInt),
   vs WK's packed array → `ByteCount` 66 KB vs 8 KB. Packed numeric arrays /
   hash-consing reduce this.
3. **Hash-consing / value-memo — a constant-factor + sharing lever.** Structural
   dedup of shared subtrees (validated ~26% on Pair; both a time and memory win;
   deferred in `docs/representation-rewrite-plan.md`).

## The plan (measure-gated, mirrors the time line's discipline)

**Phase 0 — instruments first (no optimization). [DONE 2026-06-29, brief-048.]**
`MaxMemoryUsed[]` and `scripts/core-memory-microbench.sh` landed; the comparator
reproduces this doc's calibration (`module-leak-20000` rum 5,966,099 B vs WK -80 B;
`retain-range-*` rum/WK 8.11x–8.25x). Baseline frozen — Phase 1 gates against it.
The original plan text follows. Implement `MaxMemoryUsed[]`
(peak memory; track a high-water mark in the proxy + optionally real RSS). Build
the cross-engine memory comparator (same workload in rum + WK, report the
{proxy-delta, RSS-delta, MaxMemoryUsed} ratios). Freeze a baseline (Lane D
`--suite memory` + an RSS baseline). Headline target: the Module×20000 6 MB → near
WK's ~80 B. Gate every later phase on this comparator + Lane D no-regression +
correctness (`cargo test --workspace` + the 2855/0 corpus + `difftest check`).

**Phase 1 — Module-gensym DEFINITION GC (the O(N)→O(1) win, lever 1a).** On Module
exit, for each gensym it created, remove its definitions — but ONLY when it is
provably dead: it does NOT appear in the `result` AND the body did NOT write it into
any global definition. Conservatively retain anything that escaped to a global
(correct but un-reclaimed, = today). This safely captures the dominant
"Module-in-a-loop using only its own locals" leak. SOUNDNESS IS THE RED LINE — a
gensym can escape via a global write (`Module[{x}, x=5; glob=...x...; Null]` returns
a result without `x$N` yet `glob` references it), so "not in result" alone is
UNSAFE; track "wrote a global during the body" as the escape signal. Optional
completeness follow-up: a root-set mark-sweep (roots = all non-gensym definition
RHSs + eval stack + bindings) triggered at a gensym-count threshold, to reclaim
escaped-but-now-dead gensyms.

**Phase 2 — symbol-interner reclamation (close the residual O(N), lever 1b).** Give
the interner an un-intern path for dead gensyms (`&'static` HashSet → a reclaimable
arena or refcounted entries). Deeper change; it closes the name-string leak Phase 1
leaves behind, approaching WK's O(1).

**Phase 3 — representation constant-factor (levers 2 & 3).** Packed numeric arrays
for dense `List`s; hash-consing for structural sharing. The biggest project; last,
and only if the constant factor after Phases 1-2 still misses the target.

## Key file pointers

- `eval_module` / `eval_block`: `crates/rum-runtime/src/runtime/scoping/lexical.rs`
  (:48 / :98). Fresh-symbol alloc + `register_unique_generated_expr`:
  `crates/rum-runtime/src/runtime/scoping/unique.rs` (:170).
- Memory proxy + builtins: `crates/rum-runtime/src/runtime/debug_tools/resources/memory.rs`
  (`eval_byte_count:4`, `eval_memory_in_use:18`, `current_memory_proxy_bytes`).
  Builtin catalog: `crates/rum-runtime/src/runtime/builtin_catalog/debug.rs`
  (`ByteCount:62`, `MemoryConstrained:96`, `MemoryInUse:102`; `MaxMemoryUsed` ABSENT).
- Symbol interner (`&'static`, append-only): `crates/rum-core/src/expr.rs`
  (`symbol_interner:46`, `intern_text:56`, `intern_symbol_text:79`, `interned_id:164`).
- Definition removal: `crates/rum-runtime/src/runtime/definition_clear.rs`
  (`remove_matching_down_values:168`), `removed_symbols` tracking.
- difftest Lane D memory: `crates/rum-difftest/src/lane_d.rs`,
  `crates/rum-difftest/perf/README.md`, `seed/ham-import/mem/memory_suite.txt`.
- Real-RSS tooling: `scripts/guarded-test-run.sh` (watchdog),
  `crates/rum-cli/src/alloc_count.rs` (GlobalAlloc cfg), `RUM_PROFILE_REPR`.
- Cross-engine TIMING comparator to mirror for memory:
  `scripts/core-language-microbench.sh`.
- WK baseline: `wolframscript` at `/usr/local/bin/wolframscript`; always compare
  DELTAS, never absolute `MemoryInUse[]` (rum proxy ≠ WK real-kernel scale).

## Recommended start

Phase 0 — without a memory yardstick there is no "constant factor" to claim. First
Codex brief: implement `MaxMemoryUsed[]` (WK-conformant peak tracking) + the
cross-engine memory comparator script. Then Phase 1 (the Module-gensym definition
GC) is the big, well-understood, high-value win.
