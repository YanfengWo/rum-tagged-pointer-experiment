# Next Session Handoff: Simp-Descent Loop Diagnosis (Task 2c), then Representation Work

> **2026-07-01 (LATEST — read THIS first). The host-embedding hardening arc is DONE (no
> active line — pick one; OPEN directions at the bottom of this block).** This session
> closed TWO lines: the pattern-matching line (briefs 058 + axis measurements — see the
> matcher block further below) AND the host-embedding hardening arc (briefs 059-066, logged
> here newest-context below). Tree clean, all committed on `main`. Bottom line for a new
> session: Tier-1 + Tier-2-core + the rum-jupyter real interrupt all shipped and gated;
> **OPEN next = a showcase project (sandbox + WASM enablers ready), the deferred perf line
> (hash-consing ~26% on Pair, bytecode VM), or lower-value jupyter stubs.** The arc log:
> host-embedding Tier-1 hardening (`docs/host-embedding-and-hardening.md`), as one-lever
> briefs. **brief-059 DONE** (`e5b13b9`) — interrupt/cancel + per-eval wall-clock timeout
> via ONE amortized
> poll point in the eval loop (`if self.interrupt_armed` gates a `#[cold]` poll fn,
> every 1024 steps checks an `Arc<AtomicBool>` cancel flag + a deadline; `armed`
> recomputed per top-level eval from `Arc::strong_count>1` OR timeout-set, so
> `cancel_handle(&self)` needs no `&mut`). New `HostCancelHandle`/`cancel_handle()`/
> `eval_with_timeout()`/`HostFailureCode::Cancelled`; only the Arc crosses threads.
> Gate: workspace 712/0 + rum-host 7/0 (real cross-thread cancel ~60ms + 200ms timeout +
> positive), microbench loop-empty flat-to-faster at scale 1/10/20 (no per-step cost).
> **brief-060 DONE** (`4e532a8`): catch_unwind panic boundary + session poison (host-layer
> only, zero hot-path; uniform guard → read-only methods became `&mut self`, accepted;
> does NOT catch stack-overflow aborts — those are the parser/matcher depth guards).
> **brief-061 DONE** (`0423bb4`): structured/typed `HostValue` (`.text` unchanged + a new
> `HostTyped` enum Integer/BigInteger/Rational/Symbol/Str/List/Other, depth-capped 64;
> rum-host gained a direct rum-core dep to name Expr, no lock change). **Memory budget —
> RESOLVED as a host-side pattern (user decision 2026-07-01), no new core/host code**: the
> structural proxy is an O(state) sweep + interner-blind, RSS would break the WASM-clean
> core — so the cap is imposed host-side by a watchdog thread that reads process RSS and
> reuses the brief-059 `cancel_handle()` to `.cancel()` on breach (true RSS bound, core
> stays WASM-clean, already enabled by 059). Documented with a code example in
> `docs/host-embedding-and-hardening.md` §2 Tier-1; a native-only rum-host convenience
> wrapper was considered and deliberately not added (keep rum-host platform/thread-free).
> **The Tier-1 host-hardening line is COMPLETE** (interrupt/cancel, time budget, panic
> boundary, structured `HostValue` shipped; memory budget documented). **TIER-2 STARTED:
> brief-062 DONE** (`1f5b1b0`) — `Session`/`HostSession` is now `Send` (movable to a worker
> thread; the only blocker was the suspension-frame trait object, fixed with `+ Send`, no
> unsafe, zero `!Send` captures) and intentionally `!Sync` (single-threaded mutable state;
> run one per thread). Proven by a static assert + an 8-thread move/isolation test.
> **rum-jupyter upgrades UNLOCKED by this round (a dedicated post-Tier-2 brief):** the
> `interrupt` handler is today a STUB (sets a flag, does NOT stop a running eval) and the
> transport is a single-threaded poll loop where a blocking execute can't even receive the
> interrupt — 059 cancel + 062 Send + 060 panic boundary now supply the primitives for a
> REAL interrupt + panic-resilient kernel (execute-on-worker + interrupt→cancel +
> poison-restart). See `docs/host-embedding-and-hardening.md` §2 "Downstream".
> **TIER-2 CORE NOW COMPLETE:** 063 reset/clear (`0bde8b7`, also the poison-recovery path),
> 064 whole-session introspection (`afad4de`, `definitions()`/`contexts()`), 065 session
> snapshot `restore_from_bytes` (`3d4a895`, in-memory buffer, no fs — the WASM enabler;
> reuses `render_saved_symbols`, TEXT image). So the host-hardening line (Tier-1 + Tier-2
> core) is DONE. **OPEN next: the jupyter real-interrupt brief** (execute-on-worker +
> interrupt→cancel + poison-restart — all primitives now shipped; the biggest remaining
> user-visible win) OR a showcase project (sandbox + WASM enablers ready) OR the deferred
> perf line (hash-consing ~26% on Pair, bytecode VM). See `PROGRESS.md` 2026-07-01 host
> entry + memory `host-hardening-tier1.md`. The matcher / pattern-matching blocks below are
> the previous (now-closed) line.
>
> **UPDATE — jupyter REAL INTERRUPT DONE (brief-066, `4c7771f`), the host-arc payoff.** The
> stub `interrupt` is now real: execute runs on a WORKER thread (brief-062 `Send` session
> moves in; brief-059 `cancel_handle` stays on the main pump), the pump defers Shell while
> busy + keeps polling Control + worker completion, `interrupt_request` → `cancel()` stops
> the cell, a panic → error reply + `reset()` (060/063) keeps the kernel alive. State machine
> in `adapter/worker.rs`; session always returns (worker `catch_unwind`), join only after
> `is_finished()` (no deadlock), Shell deferred not dropped, wire protocol unchanged. Hard
> gate: workspace 712/0 + rum-jupyter 80/0, 2 interrupt tests (pump + native ZeroMQ, ~8ms),
> **15× stability reruns clean** (10 me + 5 Codex). So the whole host-hardening arc (Tier-1 +
> Tier-2 core + jupyter real-interrupt) is DONE. **OPEN next: lower-value jupyter stubs
> (history/comm_info/debug), a showcase project (sandbox + WASM enablers ready), or the
> deferred perf line (hash-consing ~26% on Pair, bytecode VM).**

> **2026-07-01. Matcher backtracking stack-overflow
> CRASH fixed (brief-058, `a2d5ee6`); this is a host-resilience / Tier-1 brick. Tree
> clean, committed on `main`.** A SECOND reachable crash, distinct from the
> 2026-06-30 deep-nesting one: a tiny `Orderless` + `BlankNullSequence` input
> stack-overflowed the host — `SetAttributes[f,Orderless]; MatchQ[f@@Range[12],
> f[a___,b___,c___]/;False]` SIGABRT'd (n=14 for 2-seq). Found while picking up the
> deferred TODO "pattern-matching axis (b)" after the user asked how WK handles the
> NP-hard `Orderless`+sequence (AC-matching) blowup. **Measured both engines:** rum's
> backtracking was naive `k^n` enumeration with NO bound, and the un-TCO'd `index+1`
> recursion made native stack DEPTH grow ~with WORK → ~`2^n` clone-heavy frames
> overflow the 8 MB stack. **WK was probed too and is NOT smart** — it also does naive
> `k^n` (no multiset dedup, no condition-aware pruning; `$RecursionLimit=50` doesn't
> trip its matcher → iterative/bounded-stack), it just degrades gracefully instead of
> crashing. So the fix copies WK's RESILIENCE, not a (nonexistent) clever algorithm:
> a `match_depth` guard (`MAX_MATCH_DEPTH=512`, safe `with_match_depth_guard`, no
> unsafe) → graceful `TerminatedEvaluation["PatternMatchRecursionLimit"]` (new
> `EvalError::MatchRecursionLimitExceeded`, own specific tag). A depth guard subsumes
> a work counter here (depth≈work → first over-limit path Errs and aborts fast).
> **Gate:** `cargo test --workspace` 712/0; 5 CLI probes (adversarial→Terminated not
> crash, normal→True); microbench no regression (guard is OFF the common non-Orderless
> path); mma-hol v8 cp42 heavy-matcher subset 943/0 (arith+num+rat+simp 818 +
> proptaut+meson 125), zero limit leakage → 512 refuses no legitimate match. See
> `PROGRESS.md` 2026-07-01 + memory `wk-ac-matching-is-naive-enumeration.md`. **The
> perf half (smart pruning / iterative matcher rewrite) is NOT worth pursuing** —
> NP-hard, no general solution, WK lacks it, HOL doesn't trigger it. **Axis (a) also
> MEASURED + CLOSED 2026-07-01** (no code): per-symbol DownValues-count over the full
> cp42 tower = 1451 symbols, MAX 6 on any one (`rnumMul`), 96.3% have exactly 1, only
> 12 have ≥3 → NO long tail, so a discrimination-net/term-index has nothing to optimize;
> the linear-scan dispatch is already optimal for HOL. **So the whole pattern-matching
> line is now closed** (per-match cost already optimized, axis (b) resilience fixed,
> axis (a) a measured non-issue). Other OPEN lines unchanged from the 2026-06-30 block
> below (host-embedding Tier-1 continues —
> interrupt/cancel, session time/memory budget, structured `HostValue`, `catch_unwind`;
> this matcher fix is one of its bricks).

> **2026-06-30 (read after the 2026-07-01 block above). Two lines
> CLOSED this session: (1) correctness — rum reproduces the README's 3126/0 exactly;
> (2) the safe-Rust MEMORY LINE is CONCLUDED. Tree clean, all committed (HEAD on
> `main`).** Session arc, newest first:
>
> - **Memory line CONCLUDED for safe Rust.** Phase 0 instruments (`MaxMemoryUsed[]` +
>   `scripts/core-memory-microbench.sh`, briefs 048) + **Phase 0.5 four-C panel**
>   (brief-052: peak/retained/transient/post-cleanup; **KEY: only `retained_ratio` is
>   a clean cross-engine constant factor — peak/transient/post-cleanup/RSS WK columns
>   are confounded by WK's sticky startup high-water, read them RUM-LOCAL**) + Phase
>   1/1b Module-gensym definition GC (briefs 049/050, done 06-29; 6 MB→bounded, cold
>   `run_all` completes at 12 GB) + **Phase 3 per-node `Expr` compression rungs 1-3**
>   (briefs 053/054: box `Rational` + `Integer` small-int hybrid `IntValue` + `Symbol`
>   `NonZeroU64` niche-pack → `size_of::<Expr>()` **64→32**, dense-list `retained`
>   **8.3x→~4.0x**, AND kernel 8-18% FASTER + alloc ~200x↓ on dense ints; gated
>   `cargo test --workspace` 710/0 + mma-hol exact-arith corpus 886/0 byte-identical).
>   **Phase 2 (interner name residual) CONCLUDED as a deliberate bounded tradeoff**
>   (~235 B/gensym, per-eval bounded by `iteration_work_limit`, non-blocking; closing
>   needs unsafe or a perf-regression on the `Copy`/`&'static` `Symbol`, or a
>   prevent-at-source spike). Further is OUT of safe-Rust-without-regression scope:
>   per-node <32 B needs unsafe tagged-pointer/NaN-boxing; dense-collection parity
>   needs packed arrays (a numeric-kernel technique assessed a poor fit for symbolic
>   rationals-only rum). See `docs/memory-management-plan.md`.
> - **`TerminatedEvaluation` resource-limit tags made specific** (`63efadd`):
>   IterationWork/IntegerWork/GeneratedElement get distinct tags (was generic
>   `"ResourceLimit"`) so a user knows which budget to raise. (Found while measuring
>   Phase 2: the limit failure mode is already graceful + messaged, not a bug.)
> - **Deep-nesting stack overflow — RESOLVED.** A ~5000-deep nested literal SIGABRT'd.
>   **Parser depth guard shipped** (brief-055, `2a23d5e`: `MAX_PARSE_DEPTH`=128 →
>   graceful `parse nesting exceeded limit 128`; corpus loads under 128). This closes
>   every REACHABLE crash (WL paths depth-capped by parser + `recursion_limit` 1024 +
>   resource limits). The iterative `Drop` (for a deep `Expr` built directly via the
>   Rust API — host-only, pathological, NOT WL-reachable) was tried 3 ways (briefs
>   056/057: unsafe `ManuallyDrop`, safe `Option<Arc>`, safe sentinel) — **all
>   regressed the eval hot path 20-50%**, REJECTED; documented as a known host limit
>   in `docs/host-embedding-and-hardening.md`.
> - **PROCESS (important for future briefs): two standing rules added** to
>   `docs/dev/codex/TEMPLATE.md` + `AGENTS.md` (`294d356`): (1) NO new `unsafe` in
>   `rum-core`/`rum-runtime` (verified ZERO-unsafe; safe equivalents exist); (2)
>   hot-path changes (eval/dispatch/matcher, `Expr`/`Shared*` construct/`Drop`,
>   candidate lookup) MUST be gated with a `core-language-microbench` A/B. Root cause
>   of brief-055's miss = under-constrained brief; the rules self-corrected 056/057.
>
> **OPEN directions (pick a new line):** (a) **host embedding hardening** — now the
> natural mainline (mma-hol is through); Tier-1 in `docs/host-embedding-and-hardening.md`
> (interrupt/cancel, session time/memory budget, structured `HostValue`, `catch_unwind`
> boundary), then a showcase (sandbox + WASM-ready are the kept enablers; the specific
> project is deliberately undecided — impression-vs-utility weighed, nothing chosen).
> (b) deferred perf line (hash-consing/value-memo ~26% on Pair; bytecode VM). (c) small
> conformance: genuine-cycle → `TerminatedEvaluation[IterationLimit]` not
> `General::cycle`. (d) push mma-hol to more demos/chapters. The
> `docs/host-embedding-and-hardening.md` capability survey + tiered hardening map is
> the entry point for (a).
>
> The 2026-06-30 arith block below is still accurate for the correctness fix details;
> the blocks under it are older history.

> **2026-06-30 — the 2855-vs-3126 gap was a REAL correctness bug, now
> FIXED (brief-051). It corrects the "NOT failures, a counting
> reconciliation" framing in the 2026-06-29 block below — that was WRONG.** The
> previous session FOUND that `arith_tests.wl` truncates (rum 50, the rest silently
> skipped) but never persisted it AND the handoff recorded the opposite ("a
> coverage-counting question"). Re-derived + fixed this session. **Root cause:** rum
> raised a fatal `General::cycle` / `EvaluationCycleDetected` on the benign
> self-fixed-point `Set[a, a]` (ownvalue `a :> a`), where WK returns the symbol.
> Because a fatal eval error inside `Get` silently aborts the rest of the file,
> `arith_tests.wl` stopped at line 399 `nnfConv = HOL\`Auto\`Arith\`nnfConv` (a
> PUBLIC self-alias — the bare LHS resolves through `$ContextPath` to the same
> symbol → `Set[s,s]`), running 41/140 groups = 50/271 assertions, no FAIL, outer
> `Scan` continued → invisible. Per-file `:::`-vs-`runTests` count proved **arith is
> the ONLY truncated file** → this one bug = **221 of the 271** gap. **Fix:**
> `eval_core.rs` `evaluate_symbol` short-circuits when own_value/delayed_own_value
> RHS === the symbol (return it, no re-eval); genuine cycles (`d:=e;e:=d`) untouched.
> 3 older `self:=self`-as-error-trigger tests re-pointed at a genuine mutual cycle;
> new regressions lock the WK table. **GATE (verbatim):** `cargo fmt --check` clean,
> `cargo test --workspace` green (rum_runtime 710/0), 4 rum-cli probes == WK,
> **arith_tests.wl on v8 cp21 50→271/0/159 groups = WK gold exactly.**
> **GAP NOW FULLY CLOSED — rum reproduces 3126/0 exactly.** Authoritative fast
> confirmation (no 113-min cold run needed): real `tests/run_fast.wls` (Stable) in a
> scratch acceptance clone with v8 `cp42` as `bootstrap.mx`, new binary, =
> **3055/0** over all 38 post-bootstrap files (self-consistent), + `kernel_tests`
> cold **71/0** = **3126/0, 0 failures = the README number**. The only per-file
> change vs the old cold run is arith (+221); the apparent "residual ~50" was an OLD
> binary `$pass` undercount (old cold log printed 2905 `ok` but summary said 2855) —
> the new run is internally consistent, so it evaporates. (run_all cold/Strict vs
> run_fast Stable = different LOAD path, identical TEST content.) WK-conformance
> footnote: a genuine cycle in WK = `TerminatedEvaluation[IterationLimit]`, not a
> hard error — rum's `General::cycle` on REAL cycles is a separate future gap.
> Committed; tree clean. **NEXT options** (the correctness/count line is now DONE):
> Phase 2 interner reclamation / Phase 3 representation / a fresh perf line, plus the
> optional genuine-cycle→IterationLimit conformance.

> **2026-06-29 — MEMORY LINE Phases 0/1/1b DONE; the authoritative cold
> `run_all` BLOCKER is RESOLVED.** Briefs 048 (instruments `MaxMemoryUsed[]` +
> `scripts/core-memory-microbench.sh`, `9140fd4`), 049 (Module-gensym DEFINITION GC,
> leaf-only, `57603f8`), 050 (root-set mark-sweep GC for nested/recursive modules,
> `ab5d003`). The single-process cold Strict `run_all.wls` from the acceptance-clone
> source — previously INFEASIBLE (OOM'd to 50 GB / watchdog-killed at 41 GB) — now
> **COMPLETES under `scripts/guarded-test-run.sh` in ~113 min, peak RSS 12.3 GB
> (median 6.6 GB, bounded, no thrash), all 39 test files incl. the 4 heavy provers,
> = 2855 passed / 0 failed.** The mark-sweep is correctness-neutral (2855/0 ==
> the pre-memory-line clean count: run_fast 2784 + kernel_tests 71) AND bounded the
> memory. **OPEN: the 2855-vs-3126 reconciliation.** rum runs 2855 assertions
> consistently (Stable run_fast AND cold Strict); the README's 3126 is a separate
> upstream count (already "disputed 3124-vs-3126" in mma-hol `docs/dev/PLAN.md`) —
> likely WK expanding generated/`quantNestPat` assertions more granularly OR a
> different mma-hol commit. NOT failures; a coverage-counting question to reconcile
> against WK if desired. (Fast confirmation path for that, found 2026-06-30:
> `Get` the loaded-tower checkpoint and call the fn directly — e.g.
> `rum-cli --eval 'Get[".../rum-snapshot-checkpoints-latest/cp21-auto_Arith_wl.mx"];
> HOL\`Auto\`Arith\`nnfConv[...]'` — SECONDS, never cold-load; see
> [[feedback-test-fix-from-checkpoint]].)
> **v8 SNAPSHOT (2026-06-30):** the Stable snapshot tower was rebuilt with the
> memory-fixed binary → `rum-snapshot-checkpoints-v8/` (cp00…cp42, all ok), and is
> **byte-identical to v7** across all 29 checkpoints (GC correctness-neutral).
> `rum-snapshot-checkpoints-latest` → v8. Future rebuilds: `scripts/rebuild-snapshot.sh`
> (HEAD-tagged, one call). **NEXT options:** (1) reconcile 2855-vs-3126 (cross-count
> a few files rum-vs-WK); (2) Phase 2 interner `&'static` reclamation (the ~100 B/
> gensym RSS residual — now a small lever, no longer a blocker); (3) Phase 3
> representation (the ~8x `ByteCount` factor); (4) a fresh perf line. Tree clean,
> all committed.

> **2026-06-29 — ACTIVE LINE is now MEMORY MANAGEMENT. Read
> `docs/memory-management-plan.md` FIRST.** User direction: after the time line hit
> "WK parity to a constant factor" across the board, do the SAME for memory. Research
> is DONE (no code yet); the plan doc has everything. Headline calibration:
> `Do[Module[{x}, x=i^2; x], {i,1,20000}]` grows `MemoryInUse` by **6,002,561 B in
> rum vs 80 B in WK** — the Module-gensym leak (WK GCs gensyms, rum never does),
> which is ALSO why the single-process cold `run_all` OOM'd to 50 GB. `ByteCount[
> Range[1000]]` is 66 KB vs 8 KB (~8x dense-representation constant factor).
> **difftest answer:** Lane D HAS `--suite memory` but it is rum-LOCAL regression
> (no WK oracle); `MemoryInUse`/`ByteCount` are deterministic structural PROXIES that
> SEE the definition leak but are BLIND to the interner `&'static` leak (use RSS for
> the true total); `MaxMemoryUsed[]` is NOT implemented. **Plan:** Phase 0 instruments
> (`MaxMemoryUsed[]` + a cross-engine memory comparator mirroring
> `core-language-microbench.sh`), Phase 1 Module-gensym DEFINITION GC (sound:
> reclaim a gensym only when it is not in the result AND not written to a global —
> escape via global write is the trap), Phase 2 interner reclamation, Phase 3
> packed-array/hash-consing. **START at Phase 0** (no yardstick → no "constant
> factor"). Key files in the plan doc (`eval_module` lexical.rs:48, memory proxy
> debug_tools/resources/memory.rs, interner core/expr.rs:46). Tree clean, all
> committed.

> **2026-06-29 — the v7 batch-A 50-FAIL is CLOSED (brief-046, `7b0e4cd`). Read
> THIS FIRST.** All 50 (num 12 + rat 23 + set 15) were ONE bug, not shape
> rigidity: rum hard-holds the `MatchQ` pattern argument and, in
> `resolve_matcher_pattern_expr_owned`, used an all-or-nothing gate that left the
> WHOLE argument unevaluated when it contained any pattern syntax — so the
> `quantNestPat[...]` test helper never expanded (num/rat) and `const[pair[[2]],_]`
> kept its `Part` unevaluated (set). WK does not Hold the pattern arg; the fix
> makes the resolver recurse (evaluate ordinary subexprs, keep pattern constructs
> held, dispatch ordinary-headed pattern-bearing nodes via
> `resume_call_from_prepared_args`). Fixes the whole matcher family (`Cases`/`Count`/
> `Position`/`DeleteCases`/`ReplacePart`/`Catch`-tag) at one chokepoint. **Gate
> (verbatim): `cargo test -p rum-runtime` 692/0; batch-A 50→0 (set 189/0, num+rat
> 375/0); NO regression — real-tower 1122/0, remaining non-heavy batch-A 1063/0.**
> Method note (logged in PROGRESS + [[diagnostic-tool-trust]]): I used `MatchQ` —
> the buggy subject — as my own probe and hit two artifacts (`\:2200`→`":2200"` not
> `∀`; an `ab=HOL\`Terms\`abs` ownvalue indirection unevaluated in the held
> pattern); calibrate every expected value against plain `wolframscript`.
>
> **So v7 batch-A is now effectively 2490/0 (all non-heavy).** AND **batch-B heavy
> provers were then run on v7+046 for the first time — ALL GREEN**: meson 44/0
> (122s), proptaut 81/0 (465s), simp 46/0 (109s), meson_proveprop 69/0 (836s, was
> historically a 300s TIMEOUT — now completes). batch-B total 240/0; the old
> "known pre-existing failures" are GONE.
>
> **CLEAN count (the per-pattern subtotals above OVERLAP — do NOT sum them):**
> authoritative single `run_fast` (all 38 post-bootstrap files, Stable from cp42,
> no overlap) = **2784/0**, plus `kernel_tests` cold = **71/0**, = **2855/0 verified,
> zero failures on v7+046**. The README's `3126/0` is a DIFFERENT runner (cold
> Strict `run_all.wls` from source, +kernel_tests) and is internally disputed in
> mma-hol `docs/dev/PLAN.md` (3126 vs graduation-commit 3124). The ~271 residual is
> Stable-vs-Strict + the un-reproduced cold count, NOT failures.
>
> **BLOCKER FOUND — single-process cold `run_all.wls` is INFEASIBLE on this 64GB
> machine (this is WHY the cold Strict authoritative gate keeps getting deferred).**
> It ballooned to **50.7 GB RSS** at 2h20m, exhausted RAM + 27GB swap, swap-thrashed
> (only ~50% CPU — half its time in page-fault I/O); killed. Root cause = rum's
> unbounded Module-gensym live growth (the deferred GC, far worse than "~10GB" at
> full single-process corpus scale). Reproducing the authoritative 3126-comparable
> cold number needs the Module-gensym GC first (or a bounded segmented cold pass,
> but upper-tower TEST execution still needs the whole tower resident → same wall).
>
> **NEXT options (correctness verified 2855/0; pick a new line):**
> (1) **Module-gensym GC (deferred SAFE perf/memory work)** — now has a concrete
> forcing function: it is the prerequisite for the authoritative cold `run_all`
> 3126-reconciliation, and the ~10GB→50GB leak is the dominant memory cost
> ([[snapshot-checkpointing-direction]], [[rum-project-context]]). (2) Other
> deferred perf line — heavy-prover 046-vs-045 A/B, then hash-consing/value-memo +
> bytecode VM. (3) An OPTIONAL v8 cold rebuild (`scripts/snapshot-checkpoint-chain.sh`)
> to refresh the snapshot with the 046 binary — LOW urgency (046 is call-time, does
> NOT change theorem construction; 50→0 + batch-B verified on the existing v7 cp42).
> (4) Push the correctness mainline further (more demos / next mma-hol chapters).
> Tree clean, all committed.

> **2026-06-24 (EVENING) — v7 COLD REBUILD (045 binary) + AUTHORITATIVE batch-A
> recheck (read THIS FIRST).** After the Power/Times line (brief-043/044/045)
> landed, the full checkpoint chain was rebuilt COLD with the 045 binary →
> `rum-snapshot-checkpoints-v7/` (cp00…cp42 incl. demos/Continuous, all 28 extend
> steps ok, ~68min, `/tmp/v7_cold_build.log`). **v7 SUPERSEDES v6** — v6's lower
> checkpoints were built by a PRE-041 binary, so any test restored on them shows
> STALE theorem shapes. This actually mattered: a mixed cp26+Real/* snapshot showed
> `arith` 0-FAIL, but v7 (clean 045) shows arith/num DO fail. 045 cold-build times
> ≈ v6 (Mul 212/208s, Seq 481/476s) — no macro perf regression; real suite still
> **1122/0** under v7.
>
> **Authoritative batch-A (v7, non-heavy stdlib+engine via run_fast patterns):
> 2440 passed / 50 failed.** Per-file (FAIL belongs to the file finishing right
> after it): **num_tests 12** (arithmetic shapes `(a+b)+c=(a+c)+b`, `d|(x+d)=d|x`,
> `m≤n⇒m+p≤n+p`), **rat_tests 23** (`ratAdd`/`ratMul`/`ratLe`/`ratCanon`/`gcd`/
> `exDiv`/`bezout` shapes), **set_tests 15** (10 `X is a defining equation`:
> IN/SUBSET/UNION/INTER/DIFF/EMPTY/UNIV/POW/IMAGE/PREIMAGE; 5 `X = <lambda>`:
> BALL/BEX/INJ/SURJ/BIJ). **real_arith now 0 FAIL** — the original batch-A's
> auto-RealArith ground-domain shape fails are GONE. The original 2026-06-23 "50
> all shape in arith/rat/auto-RealArith" is a DIFFERENT 50: 045 changed shapes via
> the arith/rat decision procedures (parseLin/buildLin use WL `Plus`/`Times`/`Power`
> for linear terms), so an earlier "043-045 is orthogonal to the shape fails" guess
> was WRONG — v7 is the real baseline. These are shape/structure RIGIDITY on
> TRUE-and-used theorems (the 1122/0 real tower sits on them), not correctness bugs.
>
> **NEXT: diagnose the v7 50, two classes.** (1) 35 arithmetic/rat SHAPE fails
> (`MatchQ[concl[thm], hardcoded-WK-pattern]` divergence — 2nd-arg pattern with HOL
> `abs[bvar[0,_],…]` Blanks matches differently inline vs pre-bound; source-level
> matcher/eval diagnosis). (2) 15 set defining-equation / lambda fails (separate
> class — look at set_tests' defining-equation + lambda checks). Batch B heavy
> provers (meson/proptaut/simp) NOT yet run on v7. v7 scratch snapshot for probing:
> `rum-snapshot-checkpoints-v7/scratch.*/mma-hol/bootstrap.mx`.

> **2026-06-24 (LATER) — ATTRIBUTE-AXIS triage RESOLVED + fixes landed (read FIRST;
> supersedes the scan block just below).** The attribute-axis findings are now
> fully triaged against WK (behavioral probes, not just the `Attributes[]` diff —
> [[diagnostic-tool-trust]]), and the confirmed bugs are fixed.
>
> **DONE + committed:** brief-041 (`b7840a7`) — dropped FullForm's spurious
> `HoldAll` (`FullForm[1+1]` now `"2"` not `"Plus[1, 1]"`) and added `Listable` to
> 12 number-theory builtins (Abs/Round/Factorial/Factorial2/Denominator/Numerator/
> Mod/Quotient/PowerMod/GCD/LCM/CoprimeQ) so they thread. `cargo test -p
> rum-runtime` 688/0; all 12 threaded forms + FullForm verified vs WK. One test
> that encoded the OLD buggy `FullForm[1+2]→"Plus[1, 2]"` was corrected to WK's
> `"3"`. Also `75e8f48` — recorded the Chinese-working-language discipline as a
> hard line in `CLAUDE.md` (mirrors [[working-language-chinese]]).
>
> **KEY CORRECTION — `Evaluate` is NOT broken (the scan/handoff misdiagnosed it).**
> "`Hold[Evaluate[1+1]]` stays unevaluated" was a FULLFORM ARTIFACT: inspecting it
> with `FullForm[...]` (which held the whole thing, bug above) rendered the
> un-fired input. `Hold[Evaluate[1+1]] === Hold[2]` is `True` — Evaluate works.
> Left untouched. Textbook [[diagnostic-tool-trust]] (the instrument was the bug).
>
> **TRIAGE CLOSED — the rest of the attribute diff is BENIGN or low-value (verified,
> deliberately left):**
> - **EXTRA-Hold (Catch/Until/D/ReleaseHold/Once/Stack/OptionValue/BlockRandom/
>   MessageName/Definition/DownValues/… ~32 symbols) = ALL BENIGN.** rum hardcodes
>   the real hold behavior in the evaluator, not via the attr table, so behavior
>   MATCHES WK even though `Attributes[]` differs. Probed: `ReleaseHold[1+1]`=2,
>   `Catch[Throw[1,1+1],1+1]`=1, `D[1+1,x]`=0 — all == WK. Do NOT "fix" these by
>   editing attributes; that risks breaking the hardcoded behavior for zero gain.
> - **EXTRA-Listable on `TextWords`/`ToCharacterCode` = BENIGN** — behavior matches
>   WK exactly (`ToCharacterCode[{"A","B"}]`={{65},{66}} in both). Leave (removing
>   would break the match).
> - **`Power` MISSING Listable = the one REAL remaining gap** (`Power[{2,3},2]`
>   inert in rum, `{4,9}` in WK; `Power[{a,b},2]`→`{a^2,b^2}`). → **brief-042
>   DONE + committed** — Power threads (`{4,9}`, `{a^2,b^2}`, `Power[2,{2,3}]`=`{4,8}`),
>   all 16 scalar/symbolic/canonicalization guards byte-identical,
>   `cargo test -p rum-runtime` 689/0.
> - **`Divide`/`Subtract`/`ToUpperCase` already THREAD** correctly via lowering
>   (Times/Plus Listable / handler maps) — only the `Attributes[]` report differs
>   (cosmetic). Left as-is.
> - **`Range`/`Characters`/`ToExpression`/`Coefficient`/`Factor`/`Together`/
>   `Attributes` = low-value real Listable gaps** (HOL doesn't use them; `Attributes`
>   also has a HoldFirst-vs-Listable conflict). Deferred indefinitely unless a
>   downstream need appears.
> - Noted separately (NOT Listable, NOT this triage): rum's `Power`/`Times`
>   canonicalization is weaker than WK — `x^2 x^3` stays `Times[Power[x,2],Power[x,3]]`
>   (WK `x^5`), `x^2/x` not simplified, `Power[0,0]` inert (WK `Indeterminate`).
>   Pre-existing, doesn't block HOL; a separate future item, locked as guards in
>   brief-042 so the Listable change doesn't perturb them.
>
> **DONE — Power/Times SYMBOLIC canonicalization (brief-043 + brief-044).** rum now
> combines symbolic like-terms/like-bases like WK (numeric-exponent/coefficient
> scope): `x^2 x^3 -> x^5`, `x-x -> 0`, `x^3/x^2 -> x`, `2 a 3 a -> 6 a^2`. THREE
> changes in `arithmetic.rs`: (A) `eval_power` nested-power flatten
> `(x^2)^-1 -> x^-2` (integer outer exponent only — `(x^2)^(1/2)` stays
> `Sqrt[x^2]`), (B) `combine_multiplicative_args` like-base merge, (C)
> `combine_additive_args` like-term merge. Gate GREEN: `cargo test -p rum-runtime`
> 690/0, real-tower **1122/0** (rebuilt with the new binary AND re-run under the
> 044 binary — behavior frozen), 19 `===` probes + out-of-scope guards. Blast
> radius confirmed small: `Expand`/`Factor`/`Together`/`D`/`Coefficient` already
> self-combine, their tests unchanged. **Perf:** brief-044 took the cheap win (no-
> merge fast path + direct `Times[coeff,...]` build, no recursive
> `combine_multiplicative_args`): symbolic `Plus` 2.7x->1.96x, `Times` 2.0x->1.75x
> vs pre-043. Remaining micro regression is the O(n^2) like-term DETECTION
> (intrinsic to merging) — ACCEPTED tradeoff: real-tower (small term counts) shows
> NO macro regression (extend even slightly faster, since combination cuts
> downstream term counts); deferred to TODO "Performance And Representation".
> Still OUT OF SCOPE for the symbolic-`Power` line: symbolic-exponent combination
> `x^a x^b -> x^(a+b)` and `(2 x)^n -> 2^n x^n` (numeric `c` lifts, but symbolic
> distribution needs an INTEGER exponent — a separate axis, not yet done).
>
> **DONE — Power-over-Times distribution (brief-045).** `eval_power` now distributes
> a `Times` base for numeric exponents: the numeric product `c^n` lifts for ANY
> numeric `n`, symbolic factors distribute only for INTEGER `n`
> (`(2 x)^2 -> 4 x^2`, `(2 x)^-2 -> 1/(4 x^2)`, `(2/3 x)^2 -> 4/9 x^2`,
> `(a b)^2 -> a^2 b^2`, `(a/b)^2 -> a^2/b^2` via the brief-043 (A) flatten,
> `((a b)^2)^2 -> a^4 b^4`; but `(a b)^(1/2)` stays `Sqrt[a b]`, and rum keeps
> `Power[4,1/2]` un-rooted — rationals-only, a pre-existing limitation). Gate GREEN:
> `cargo test -p rum-runtime` 691/0, real-tower **1122/0** (045 binary), 20
> `===`/edge probes + the symbolic-exponent / non-integer guards. The Power/Times
> WK-conformance line (brief-043/044/045) is COMPLETE for the numeric-exponent
> scope.
>
> **NEXT:** the SEPARATE 50 batch-A `FAIL shape` — inline
> `MatchQ[concl[thm], quantNestPat]` divergence, narrowed to the 2nd-arg pattern
> (HOL `abs[bvar[0,_],…]` Blanks) matching differently INLINE vs pre-bound; needs
> source-level matcher/eval diagnosis (NOT an attribute issue). The last open item
> from the attribute-axis session.

> **2026-06-24 — ATTRIBUTE-AXIS conformance scan (3rd axis) found a batch of bugs.**
> Diffing `Attributes` for every System symbol in rum∩WK (recipe in
> [[coverage-conformance-direction]]; drop Protected/ReadProtected/Locked noise —
> rum doesn't Protect builtins): 34 rum-EXTRA + 78 rum-MISSING eval-attr mismatches.
> TRIAGE (behaviorally verified): **rum-EXTRA = real bugs** — `FullForm +HoldAll`
> (`FullForm[1+1]`→`Plus[1,1]` not `2` — this BURNED ~15 diagnosis probes), `Evaluate
> +HoldAll` (`Hold[Evaluate[1+1]]` stays unevaluated — Evaluate is broken!), + ~32
> more EXTRA-Hold*/Listable to triage. **MISSING Listable (23, e.g. Abs) = real bugs**
> (`Abs[{-1,-2}]` stays inert, no threading). **MISSING Hold*/SequenceHold (~40) =
> mostly BENIGN** — rum hardcodes hold in the evaluator not via the attr table
> (`Set` lacks HoldFirst attr yet `aa=5;aa=10;aa`→10 works); confirm each before
> touching. NEXT: a fix brief for the CONFIRMED attribute bugs (remove spurious
> EXTRA Hold* like FullForm/Evaluate after per-symbol WK check; add Listable where
> WK has it). SEPARATE still-open bug: the 50 batch-A `FAIL shape` (the inline
> `MatchQ[concl[thm], quantNestPat-pattern]` divergence) is NOT an attribute issue
> (MatchQ attrs match WK) — narrowed to the 2nd-arg pattern (containing HOL
> `abs[bvar[0,_],…]` Blanks) matching differently when evaluated INLINE vs
> pre-bound; needs source-level matcher/eval diagnosis. LESSON: the diagnostic
> tools are themselves under test ([[diagnostic-tool-trust]]).


> **2026-06-23 (LATE) — CURRENT STATUS (read THIS FIRST; supersedes ALL below,
> including the earlier 2026-06-23 block).**
> **MILESTONE: the full real-number tower LOADS 100% (41/41)** — the 88%/Real-Seq
> wall is gone. `core → … → ℤ → ℚ → ℝ → {Seq, SeqAux, Compact, Connected, Topology,
> CompactSet}` all load clean. **And `demos/Continuous.wl` (the NEXT-chapter
> continuous-functions demo — IVT/EVT/compact-image capstones, OUTSIDE the tower)
> also loads + its theorems verify in rum.** 4 bugs fixed this push (briefs
> 032-035, all gated+committed): **#6 `Numerator`** (`33f1795`, missing builtin,
> mirror of `Denominator`); **#7 `Max`/`Min`** (`f2d3906`, missing variadic numeric
> builtins — found PROACTIVELY by the symbol-axis scan, not yet bitten); **#8 2-arg
> `Sort[list, p]`** (`dca37c4`, the ordering-function form was a no-op stub →
> `SPEC`/`Missing[KeyAbsent]` at Seq; implemented general, NOT a canonical-order
> special-case per [[prefer-general-wl-semantics]]); **#9 `//` postfix operator**
> (`49b081b`, `x // f → f[x]`; the whole apply family `@ @@ @@@ /@ @* /*` existed
> but `//` was missing — found via the Continuous stress test + a syntax-axis
> operator survey).
>
> **METHOD that worked (record for reuse): two conformance axes + demo stress test.**
> (a) SYMBOL axis — mma-hol call-form System builtins (perl word-boundary extract)
> ∩ WK `Names["System`*"]` − rum `Names["System`*"]` → the COMPLETE missing-builtin
> set was exactly `{Max, Min}` (proactively fixed). (b) SYNTAX axis — feed the WL
> operator inventory through rum's parser (`FullForm[Hold[…]]`) → the complete
> missing-operator inventory (fix on demand, no more surprises): `~f~` infix-apply,
> `n!`/`n!!` Factorial, `a..`/`a...` Repeated, `x_.` Optional, `a^=`/`a^:=` UpSet,
> `|->`, `a~~b` StringExpression, `f'` Derivative, `a.b` Dot, `a**b`, `//@` MapAll,
> `%`/`>>`/`<<` (interactive — irrelevant to loading). The functional forms of
> `! ' . %` (Factorial/Derivative/Dot/Out) already work; only the operator sugar is
> missing. Continuous.wl used NONE of these except `//` (cross-referenced), which is
> why it loaded after just #9. The bug TAXONOMY: "missing builtin" (Nothing/Numerator
> /Max/Min) → symbol scan catches; "present-but-incomplete" (2-arg Sort) and "missing
> syntax" (`//`) → scan does NOT catch, need demo/patch-and-trace.
>
> **DONE: authoritative full-chain COLD rebuild GREEN →
> `/Users/fft/Developments/rum-snapshot-checkpoints-v6/`** (28 checkpoints
> cp00…cp42, ~65 min, log `/tmp/cold_rebuild_v6.log`). All modules cold-build clean
> from the Simp base with the FIXED binary to the bottom — this AUTHORITATIVELY
> confirms the 100% load (not just the fast Stable extend) AND REFUTES the
> lower-checkpoint taint worry (cp00–cp34 rebuilt fresh, still reach the bottom).
> Caveat: still Stable-mode extends, not the maximal Strict `run_all` (a further
> optional rung). **`demos/Continuous.wl` is now MODULE #42** of the chain:
> extended onto cp41 → `cp42-demos_Continuous_wl.mx`, AND `scripts/snapshot-
> checkpoint-chain.sh` now auto-appends it (rum-side `extra_modules` default,
> env-overridable `RUM_SNAPSHOT_EXTRA_MODULES`; the pristine clone's
> `build_snapshot.wls` is NEVER edited) so a future v7 cold-builds all 42. v6
> supersedes v5. **DONE since: the syntax-axis 🟢🟡 operator gaps — briefs 036
> (`^= ^:= ** //@ |->`, `af9eb57`), 037 (`! !! ' .. ...` + `Factorial2` builtin,
> `df833d6`), 038 (`~~`, `~f~` matchfix, `c5e5bfb`) — all gated+committed
> (rum-syntax 143/0, rum-runtime 684/0; re-survey confirms all parse). Since then:
> the 缺省模式 / Optional bug is FIXED (`x_:v`->Optional + `x_.` syntax, brief-039,
> `f92d873` — it was a PARSER bug, matcher was fine). **Dot product DONE (brief-040,
> `daaa221`)**: the `.` operator + the `Dot` linear-algebra builtin — gated (cargo
> rum-syntax 145/0 + rum-runtime 686/0; WK-match probes pass: `Dot[{1,2,3},{4,5,6}]`
> =32, mat·vec={17,39}, symbolic `{a,b}.{c,d}`=`a c+b d`, `a.b*c`->`Times[Dot[a,b],c]`;
> decimals/`..`/`x_.` regressions clean). REMAINING 🔴: `>>`/`<<` (Put/Get — special
> unquoted-filename lexing; Get exists, Put missing) — the last syntax-axis gap, and
> the lowest value (I/O, irrelevant to HOL).
> **BEHAVIOR VERIFIED (not just loads): the full Real-tower test suite passes
> 1122/0 under rum** (`tests/run_fast.wls real` on the v6 cp42 snapshot — real_*,
> real_seq/seqaux/compact/compactset/connected/topology/minmax/abs/complete/arith
> _tests.wl). So this session's Real tower (4 bugs + operators) is correct at the
> behavior level, no semantic bugs.
> **Batch-A suites (stdlib + engine, FIRST-EVER rum run): 1577 passed / 50 failed.**
> All 50 are `FAIL shape` (`MatchQ[concl[thm], hardcoded-WK-shaped comb[...]]` ->
> False) in arith / rat / auto-RealArith-ground-domain. **CONFIRMED PRE-EXISTING +
> benign, NOT a this-session regression**: the pre-session binary `62e12c7` on v5
> cp26 gives the SAME rat shape failures (154/23). Root: rum builds these theorems
> with a conclusion STRUCTURE that differs from the tests' rigid WK-shaped patterns;
> the theorems are TRUE and USED (the Real tower 1122/0 sits on rat), so this is
> shape-test rigidity, not a correctness failure (patterns are pure comb/const/_,
> untouched by this session's syntax; concl/matcher/thm-rep not modified). DEFERRED
> LOW-PRI: relax the shape tests OR understand rum's theorem-construction shape
> divergence vs WK (theorems are correct, so low value).
> NEXT TEST STEPS: (1) run batch B HEAVY provers
> meson/proptaut/simp — slow + some KNOWN pre-existing failures, run separately;
> (2) THEN the user-queued **Strict-mode cold-start full test** (`run_all.wls`,
> not run_all_stable — the maximal-rigor authoritative gate, time-consuming). Also
> a DELIBERATE DESIGN DECISION (NOT a gap to fix): rum is RATIONALS-ONLY — no
> machine/arbitrary-precision `Real` type. Decimal literals `1.5`/`.5`/`1.5e3`
> parse but lower to EXACT rationals/integers (`1.5`→`3/2`, `0.1+0.2`→`3/10`,
> `Names["System\`Real"]`={}). Rationale (user): rum's purpose isn't numerics and
> WL's arbitrary-precision float arithmetic is too heavy — rationals-only sidesteps
> it. Fine for HOL (exact reals); a settled divergence from WL for general numeric
> work. Do NOT propose adding a Real atom. ([[rum-rationals-only-by-design]])
> SNAPSHOT SIZE (2026-06-23, user noticed WK bootstrap.mx=1.5M vs rum's 5M): NOT
> a representation/density problem — rum's `.mx` is **verbose UTF-8 WL-SOURCE TEXT**
> (`Set[...]` expressions, symbol/context names spelled out, no interning), while
> WK's is compact interned BINARY. Proof: rum's 4.8M `.mx` gzips 17.6x to ~0.3M
> (< WK's 1.5M), so rum's actual info content is comparable/smaller; the gap is
> pure text-encoding redundancy. rum's `DumpSave` is implemented AS TEXT (shares
> `eval_persist_package_state` with `Save`; the `true` flag only toggles
> attributes, not binary) — so rum's `DumpSave` is effectively `Save`, the real
> binary `.mx` is NOT implemented, and the `.mx` extension is misleading. BUT
> MEASURED (don't assume): restore is FAST — `Get` of the full 4.8M snapshot ≈ 1.0s
> (cp00 450K→0.08s, cp34 2.9M→0.60s, cp42 4.8M→1.04s, ~0.2s/MB) because it stores
> pre-computed definitions (re-assign, NOT re-prove). DEV-EFFICIENCY IMPACT ≈ 0
> (cold rebuild's ~41 restores ≈ ~40s of ~65min = ~1%; module loads dominate). So
> the 3.3x size is DISK-ONLY; a binary DumpSave saves space, not time — LOW priority,
> NOT a perf win. Result semantics are correct (snapshots round-trip faithfully);
> this is a format/completeness gap, not a correctness/result-semantic bug.
> DECIDED (user): a real binary serialization IS worthwhile eventually (WK
> fidelity + larger-scale apps — at a 150G-class cache it's a ~30-100x feasibility
> flip, text ~9h/OOM vs binary minutes), but LOW priority now; recorded as a
> deferred backlog item in `TODO.md` "Performance And Representation". **NEXT:
> more demos/test stages, the 🔴 operators (`.` Dot, `>>`/`<<` I/O), or the deferred
> perf line.**
>
> ---
>
> **2026-06-23 — earlier status (SUPERSEDED by the block above; kept for the resume
> recipe + method notes).**
> Active line: the **correctness mainline** — drive the UNMODIFIED acceptance
> standard `/Users/fft/Developments/mma-hol-acceptance` (HEAD `200f13a`) load to
> "the bottom" (user's goal: the last theorem verifies, OR a hard time/space wall
> on this machine). NOT the perf/representation line (that is paused at a clean
> stop; see the 2026-06-22 block below + PROGRESS). Tree clean, all committed.
> Full log: PROGRESS.md `2026-06-23` entries. Driver:
> `scripts/snapshot-checkpoint-chain.sh` (per-module named checkpoints + manifest,
> stops at the first stderr error). Briefs 021-030 in `docs/dev/codex/`.
>
> **Progress: the full ℤ→ℚ→ℝ number tower + real-number construction LOADS — 88%
> (36/41 modules).** Authoritative checkpoints: `rum-snapshot-checkpoints-v5/`
> (cp00…cp35 = core … auto/RealArith). 5 rum bugs fixed this push, each found by a
> rebuild and advancing the frontier: (1) juxtaposition `a b`/`g #` (`67a3bdc`),
> (2) `#` slot adjacency `# 6` (`df4b368`), (3) `\[Name]` named chars (`9bcf231`),
> (4) **Function ignores extra args** — the eval-semantics one that unblocked the
> standard `Catch[e,tag,handler]` idiom and the whole Farkas-refute retry
> (`7d04aab`), (5) **`Nothing`** missing builtin (`35a6a82`). Plus the harness
> blind-spot fix (stderr error detection, `ca6312f`) and deterministic preamble.
>
> **CURRENT FRONTIER: `stdlib/Real/Seq.wl` (index 36) — MULTI-LAYERED.** `Nothing`
> fixed the `realarith-norm: term is not a real linear expression` layer; the NEXT
> error is `mkComb: argument type mismatch` (tag `term`) in
> `proveRealConclUnderFacts → realLinNormConv → normLinRAux → rMulComm →
> buildLinRAny` (types `fun`/`real`) — a deeper real-arithmetic proof-reconstruction
> issue. Likely another rum eval bug (WK loads mma-hol). Remaining after Seq:
> SeqAux, Compact, Connected, Topology, CompactSet.
>
> **HOW TO RESUME the Real/Seq dive (fast iteration base):** the `/tmp` scratch is
> transient — rebuild it: `rsync -a --exclude=.git --exclude=*.mx
> /Users/fft/Developments/mma-hol-acceptance/ <scratch>/`, then
> `cp rum-snapshot-checkpoints-v5/cp34-stdlib_Real_MinMax_wl.mx <scratch>/bootstrap.mx`,
> `extend_snapshot auto/RealArith.wl` (re-load RealArith with the CURRENT binary so
> `Nothing`/etc. resolve right — the BLAST-RADIUS lesson: an old checkpoint that
> STORES a definition using a changed symbol is tainted; cp35's RealArith baked the
> old `Private`Nothing`), then `extend_snapshot stdlib/Real/Seq.wl` reproduces the
> `mkComb` error (~4s). Diagnose by patching the relevant `HOL`Auto`RealArith`
> function on that checkpoint (e.g. `realAtomNormConv`, `buildLinRAny`, `rMulComm`)
> to Print intermediates + Throw — runs in seconds, no full reload. Method proven
> in PROGRESS `2026-06-23`.
>
> **Method/lessons (all in memory):** test a fix from the checkpoint just before
> the EARLIEST file it taints, not a full rebuild ([[feedback-test-fix-from-checkpoint]]);
> the corpus-load line is complemented by a planned symbol+syntax conformance/difftest
> pass ([[coverage-conformance-direction]]); deferred SAFE perf after correctness:
> hash-consing/value-memo (~26% sound on Pair, also a memory win) + Module-gensym GC
> (the ~10GB live leak); and a `#![forbid(unsafe_code)]` guardrail for rum-core/
> rum-runtime (the only `unsafe` is the off-by-default alloc-count GlobalAlloc).

> **2026-06-22 — CURRENT STATUS (supersedes the Task-2c/2d framing below).**
> The full running log is `PROGRESS.md` (read the `2026-06-21`/`2026-06-22`
> entries); the active plan is `docs/representation-rewrite-plan.md` (Layer A/B),
> NOT the older `performance-kernel-rewrite-plan.md`. Everything is committed
> (clean tree). Brief packages 009-014 are in `docs/dev/codex/`.
>
> **Done this session (all gated + committed):** Phase 0 repr-cost counters
> (`RUM_PROFILE_REPR`, zero-cost when off) + frozen baseline; **Layer A complete
> (A1+A2+A3+A4)** — A1 thread `SharedArgs` through dispatch, A2 copy-on-write
> `prepare_arguments`, A3 reuse `SharedArgs` in the down/up/sub-value candidate
> build, A4 thread the head as `SharedExpr`. Cumulative vs the frozen Phase-0
> baseline: worst microbench labels down ~13-17% (`downvalue-exact-hit 5.74 ->
> ~4.76x`, `if-true 5.96 -> ~5.15`, `pattern-hit 5.47 -> ~4.56`); A4 is a
> SYSTEMIC win (every call has a head).
>
> **The Task-2d framing below is SUPERSEDED.** The `~34x` was NOT a separate
> "failed-REWRCONV dispatch defect" distinct from the ~3x constant. Re-diagnosed
> this session (bounded two-engine `basic-chain-orelsec-split` + a faithful
> fresh-term failing-dispatch profile): the Pair `34x` is the SAME distributed
> per-call dispatch/eval constant the primitive microbench measures, compounded
> over the deep descent and amplified ~1.7x by the WL control combinators
> (`ORELSEC`/`REPEATC`/`TRYCONV`, curried down/sub-value functions). The failing
> MATCH itself is only ~5% of the failing dispatch; the cost is the per-call
> machinery (fixed-point evaluate loop ~2x/call, ~4 stability checks/call,
> prepare, dispatch glue). **Proof it is one lever, not two:** A4 (a per-call
> head-clone elimination, measured on the fast microbench) translated to the
> bounded Pair raw failing-dispatch `-5.8 to -7.9%` in a clean back-to-back A/B.
>
> **Dead-ends eliminated this session (do not re-attempt as the Pair lever):**
> (1) the inline normal-form stamp — already prototyped Task-3b, hits 33M+ but
> does not move Pair (refuted; `docs/representation-rewrite-plan.md` B3 marked
> refuted); (2) `SharedArgs = Arc<[Expr]>` node compaction — hit ~70 hidden
> in-place mutators, deferred (moderate flaw-2 prize); (3) Module frame setup —
> `module-local` is only `1.58x`, not the lever; (4) A3-style pure candidate
> allocation cuts — real on the microbench but did NOT translate to Pair (only
> WORK reductions like A4's head clone translate).
>
> **Next target (UPDATED 2026-06-22 — incremental cuts hit the plateau; Layer B
> greenlit):** the per-call constant is now a hard plateau — brief-015 (a correct
> redundant-lookup removal) landed at the microbench noise floor, confirming
> further WORK-shaving has steeply diminishing returns. Per the user's decision,
> the structural **Layer B (single-allocation call-node rewrite)** was the active
> line; it is now **CONCLUSIVELY CLOSED — all of flaw 2's single-alloc layouts
> tested and refuted with one understood cause** (see PROGRESS `2026-06-22`):
> `Arc<[Expr]>` (B1b: fat pointer grows `Expr`), `Arc<CallData>` (indirection +
> alloc reuse-minimized to `-30`), `triomphe::ThinArc` (no alloc-count saving —
> the construction pipeline always builds a transient `Vec` first, so any inline
> rep does the same 2 allocs + adds an element-move + worse peak memory). The
> current `Arc<Vec>` (thin pointer, zero-copy-move construction) is genuinely
> well-matched to rum's reuse-heavy workload — flaw 2 is a REASONABLE DESIGN, not
> a defect; Layer A's reuse already neutralizes its practical cost. **DO NOT
> re-attempt single-alloc node layouts.** flaw 3 (cache absent-miss) likewise
> closed: stamp refuted (Task-3b) + churn-reduction rejected (interleaved A/B —
> the cache earns its keep). **The representation-rewrite arc is CLOSED with
> understanding:** Layer A (flaw 1) was the only profitable lever (worst labels
> `-13-17%`, A4 systemic, translated to Pair). KEPT: Phase 0, Layer A (A1-A4), B0
> (alloc-count cfg tooling), B1a (immutable `SharedArgs`). The remaining WK gap is
> the distributed per-call dispatch/eval constant — a hard plateau with no single
> hotspot. **NEXT direction is open** (the representation/perf line is at a clean
> stopping point): either a different line (v0.1 stabilization per `TODO.md`), or
> a much larger separate project (evaluator bytecode/JIT), or accept the current
> state. Use the INTERLEAVED A/B + control-group-difference method for any future
> small-effect timing gate (sequential A/B sign-flips ~few% effects — see the
> methodology note in PROGRESS and [[profiling-method-for-rum]]).
>
> **Measurement discipline (learned the hard way this session):** cross-run
> microbench/Pair ratios are unreliable when anything else shares the CPU (a
> concurrent probe, or MacBook nighttime maintenance `mds`/`backupd`). Gate with
> the machine-independent `RUM_PROFILE_REPR` counters PLUS a clean same-session
> back-to-back A/B (nothing else running). For an uncommitted change the A/B can
> stash it; **once committed, A/B against the parent commit via `git checkout
> <commit>^ -- <file>` (a `git stash` of a committed file is a no-op).** Verify
> the machine is idle first (maintenance daemons at 0% CPU); `if-true` /
> `block-local` stay intrinsically jittery (~6-9%) even idle, so use the A/B,
> not a single run, for them.
>
> **2026-06-22 CODA — hash-consing/value-memo validated, then DEFERRED; pivot to
> correctness mainline.** After the representation arc closed, probed the one
> remaining WK-class lever (strategy #3, structural value memoization). A shadow
> value-memo (measure-only, recompute-and-COMPARE) on the bounded Pair descent:
> `total=397105 hits=101851 hit_rate=0.2565 mismatches=0`. Verdict (PROGRESS
> `2026-06-22 Shadow value-memo VERDICT`): SOUND on Pair (mismatches=0), ~26%
> achievable hit rate — the first sound positive structural lever, and a
> different lever than the refuted stamp (it short-circuits NON-stable
> `REWRCONV→Failure`). BUT confirmed to be the deferred WK-class project, not an
> incremental cut: (1) the epoch does NOT cover dynamic scopes —
> `Do[Plus[i,1],{i,1,N}]` gives mismatches=N (loop-var bindings don't bump the
> epoch), so a production memo needs dynamic-scope-aware invalidation; (2)
> capturing the 26% needs cheap pointer keys = hash-consing, whose own
> per-construction interning cost must not eat the win. **Validated, documented,
> DEFERRED — go/no-go is a scope decision, not a measurement question.**
>
> **DIRECTION DECISION (2026-06-22, user): pivot to the v0.1 correctness/
> stability mainline.** Performance has reached diminishing returns (only
> remaining lever is the large/uncertain hash-consing bet). New main line: run
> the UNMODIFIED acceptance standard `/Users/fft/Developments/mma-hol-acceptance`
> (HEAD `200f13a`, the full completed project — 61 smoke stages, far beyond the
> 2026-06-13 validation which only reached Parser/Printer+proptaut against the
> older `51bbe9e`) through `scripts/mma-hol-bounded-smoke.sh` to find the CURRENT
> first failing/timing-out boundary. The whole stdlib (Sum, Option, Set, Num,
> Arith, List, Finite, FTA, Int, Rat, Real/*) and several test stages are
> NEVER-validated-against-rum fresh frontier. Goal: separate genuine CORRECTNESS
> failures from PERFORMANCE timeouts, and surface whether a concentrated timeout
> exists (a better-targeted optimization than the distributed per-call constant).
> HARD LINE: the acceptance clone stays pristine (read-only); diagnostic edits
> only in throwaway scratch copies.

Process correction, 2026-06-17: recent probes drifted away from this handoff's
root target. The added `PropTaut` / one-case
``HOL`Bool`EXCLUDEDMIDDLE[p]`` probes are valid side evidence, but they were
not proven to dominate the current `Pair.wl` / warm public mkPair gate. Do not
continue drilling them as the main line. Resume from the root performance
question below: build a no-profile closed phase tree for unmodified `Pair.wl`
or the warm public mkPair reproducer, verify child phases sum close to the
parent wall time, then descend only into the largest child. Record root target,
parent segment, child percentage, profile overhead ratio, and logs at every
descent. This rule exists because the investigation lost the current-root
invariant and followed readable local profile output instead of the dominant
hundreds-of-seconds gate.

Current focus: **Task 2d** — fix rum's failed-rule-dispatch cost in the kernel,
not in `Simp.wl`. The diagnosis below (Task 2c + secondary splits) is correct
and excellent; the prototyped fix is at the wrong layer. See the review block
immediately below, then Task 2d.

Task-2c/patch review, 2026-06-13 (external): the root cause is correctly found
and the loop-count diagnosis is clean (rum and WK take identical
`simpFixpointConv` counts → not a fixpoint divergence; the earlier
origin-tag/`===` hypothesis is correctly ruled out). The multiplier is
**basic-rule failed dispatch**: `17240` `REWRCONV` calls, all match failures,
over the 19 basic rules on a no-op descent. That much is solid.

The problem with the patch is the **layer**, and it shows up as the symptoms
you noticed (snapshot blowup, `simp_tests` still `208s`, can't run stock
mma-hol):

- The prototyped fix (head-indexed dispatch + `buildBaseConv` cache) edits the
  downstream `auto/Simp.wl`, i.e. it **forks mma-hol**. The entire point of rum
  is to run the *same* HOL source WolframKernel runs, faster. A Simp.wl fork
  concedes that rum cannot, and rum still can't run an unmodified mma-hol.
- It **masks rather than fixes** the real defect. Use the developer's own
  numbers on the identical-entry-count three-lambda RHS (`431` entries in every
  rule set): adding the basic rules costs rum `+91.2s` (`22.5s → 113.7s`) but
  costs WolframKernel only `+2.7s` (`7.8s → 10.5s`). Same `17240` failed
  dispatches in both engines, but rum pays ~`5.3ms` per failed `REWRCONV`
  versus WK's ~`0.16ms` — a **~34x** gap on this path, against the ~3x general
  constant. WK proves the algorithm (try 19 rules per subterm, fail fast) is
  cheap; rum makes it catastrophic. Head-indexing "works" only by cutting the
  dispatch *count* so rum's ~34x per-dispatch slowness is hit fewer times. The
  defect is in rum's execution of a failed conversion dispatch, and it stays
  unfixed — every other dispatch-heavy workload still pays it.
- The snapshot breakage (`402MB` private-context dumps, the generated-symbol
  cleanup dance) is a *direct consequence* of fixing this in `Simp.wl`: the WL
  cache and its `Module` locals land in the symbol table. A rum-kernel fix
  touches no HOL symbol and the snapshot lane is never perturbed.

Keep the WL experiments as diagnosis — they prove that reducing dispatch count
closes the gate, which confirms the dispatch path is the cost. Do not commit or
upstream them as the fix. The fix is Task 2d.

Earlier tasks (Catch fix, Pair/Simp multiplier hunt, stability-check amplifier,
substitution-sharing audit and stamp prototype) are done and kept below as
history. The Task-3b carrier was correctly reverted.

Post-RHS-lambda split update, 2026-06-13: the single-term split has reduced
the hot shape to no-op descent over the rewritten `mkPair` RHS lambda, and the
RHS lambda has now been split. Each remaining binder roughly doubles
`simpFixpointConv` entries: full opened body `29.616s` / `53` entries, one
lambda `40.590s` / `107`, two lambdas `63.673s` / `215`, three lambdas
`113.656s` / `431` in rum; WolframKernel takes `7.640s` on the body and
`10.535s` on the three-lambda shape with identical entry counts. Rule-set
splits on the three-lambda shape are `user` only `22.492s`, `basic` only on
the user-generated RHS `94.288s`, and `userBasic` `113.656s`, so the public
multiplier is basic-rule failed dispatch layered over the no-op descent.
Scratch `Drule.wl` counters show `17240` `REWRCONV` calls on the three-lambda
shape, all match failures. Failure payload removal, sentinel throw, no-throw
failure, and fast const-head rejection inside `REWRCONV` did not materially
move the body time. A scratch indexed `buildBaseConv` reduced body
`REWRCONV` calls from `2120` to `122` but regressed without caching because
`100` indexed builds cost `27.877s`; adding exact `buildBaseConv` cache moved
the body to `21.293s` (`51` hits / `2` misses) and the three-lambda RHS to
`24.452s` (`429` hits / `2` misses, `976` failed `REWRCONV` calls). Next
evidence-backed optimization: design a real cached/compiled base-conversion
dispatch keyed by rules/context/depth and head-indexed rule candidates. Do not
ship the scratch WL patches directly; they are diagnostic.

Post-clean-baseConv experiment update, 2026-06-13: a fresh scratch clone at
`/private/tmp/rum-mma-hol-baseconv-exp.HtBwlQ/mma-hol` tested only the minimal
`auto/Simp.wl` optimization direction: head-indexed rule dispatch plus exact
`buildBaseConv` cache. This crosses the previous Pair gate. Key rum release
results: RHS three-lambda `userBasic` `27.618s` (`430` hits / `2` misses),
public `pair2Same` `34.932s` (`1509` / `2`), `pair3Same` `34.394s`,
`pair4Same` `36.681s`, `CONVRULE[simpConv[{mkPairDefThm}], REFL[pair2Same]]`
`53.047s`, cold load through `stdlib/Pair.wl` `85.537s` with `pairInjThm`
valid, `pair_tests.wl` green `42/0`, and `simp_tests.wl` green `46/0`.
WolframKernel also runs the patched `pair_tests.wl` green. A cache-only clone
was negative: RHS three-lambda still `115.585s` and public `pair2Same` timed
out at `180s`, so head-indexed candidate pruning is required. Snapshot caveat
is now diagnosed: full private-context `DumpSave` saved `Module` temporary
symbols such as `baseConv$...` / `xEq$...` and inflated
`HOL`Auto`Simp`Private`` to `402MB`. Explicitly removing
`generated = Names["HOL`*$*"]; Scan[Remove, generated];` before `DumpSave`
and restoring intentional Simp globals makes a filtered through-Pair snapshot
complete in `114.3s`, size `500KB`; restoring it takes `0.1s` and
`pair_tests.wl` remains green `42/0`. Real snapshot tooling should include a
careful generated-symbol cleanup phase before dumping.

Date: 2026-06-12. Initial diagnosis was written against commit `f06c23c`;
Task 1 has since been completed and the current status is recorded below.
This note hands the current `mma-hol` frontier to the next development
session. Read it together with the two PROGRESS entries "Representation
Optimization Pass (External)" and "Block Scope Invalidation Fix (External)",
and with `docs/proptaut-performance-audit.md` for the diagnosis method.

Post-Task-1 update, 2026-06-12: the `Catch` tag-form evaluation defect below
has been fixed in `rum-runtime`. The bounded clean-clone `test/meson_tests`
smoke now reports `List[44, 0]` in `79s` on `mma-hol` HEAD `698e3d0` (the
handoff's `43` expected count was stale for the current test tree).
`test/meson_proveprop_tests` was re-probed afterward and timed out at `300s`
without a stdout summary. Continue from Task 2 unless Task 1 regresses.

Post-Task-2 update, 2026-06-12: the Pair/Simp multiplier hunt has a written
verdict in [PROGRESS.md](../PROGRESS.md). It pointed first at a targeted
`HOL`Auto`Simp`` base-conversion experiment before any broad representation
work: warm public `simpConv[{mkPairDefThm}]` timed out in rum at `360s`
versus WolframKernel `23.10866s`, while user-rule-only mkPair simplification
was about `4.1x` and therefore not the multiplier.

Post-runtime-stability update, 2026-06-12: the targeted base-conversion memo
experiment did not close the gate, but conservative runtime stability cuts
reduced user-rule-only mkPair simplification from about `52.14s` to `40.61s`
and `basicSimpset[]` from `27.07s` to `20.27s`. The public warm mkPair /
`Pair.wl` gate remains open: the current warm public mkPair probe still times
out at `240s`. Do not keep iterating narrow stability tweaks without a new
probe.

Review update, 2026-06-12 (external): Tasks 1 and 2 are verified complete
(independent re-runs of the Catch repro, `meson_tests` `List[44, 0]`, the
full workspace suite, and `rum-difftest check`). The stability cuts also
moved the general lanes — PropTaut distribution `13.2s -> 10.0-10.2s`
(~2.7x), Bool-stack load `0.78s -> 0.54s` (~2.8x).

Post-Task-2b update, 2026-06-13: the stability-check amplifier diagnosis is
complete and recorded in [PROGRESS.md](../PROGRESS.md). Call-cache size-cap
thrash was rejected by a 16x cap A/B, and widening the dirty-bloom saturation
threshold to 128 was much worse. Miss-reason counters point mostly at absent
pointer-keyed call entries, so the remaining gate is fresh call-tree churn from
the Simp/Drule conversion chain. Task 3 is no longer blocked by an unnamed
cache mechanism, but a downstream `Simp` algorithm experiment remains a
reasonable alternative before large representation work.

Task-2b review, 2026-06-13 (external): verdict confirmed, with one gap and
one sharpening that change how Task 3 must be scoped — read Task 3 below
before starting it.

- Gap (does not change the verdict): the `1<<7`-bit dirty bloom is one bit
  per symbol, so in a write-heavy Simp phase ~96/128 bits are dirty and
  almost any cached entry is rejected by false bloom overlap. The
  `33M+ call_stability_rewritten_dirty_skips` are mostly this false overlap,
  and exact per-symbol write versions (which the original Task 2b note asked
  to re-measure on this shape) were never tried here — only the coarser
  wider-bloom knob was. It does not matter: `absent` misses (`67M+`, fresh
  trees with no entry at all) dominate the `dirty` misses (`33M+`), and
  fresh trees are immune to *any* dirty-tracking quality, so exact versions
  would be a modest win, never a gate-closer. The verdict stands.
- Sharpening (verified first-hand on a `fib[27]` baseline:
  `call_stability_miss_absent == evaluate_entries`, call-stability hit rate
  `0%`): the dominant cost is not Simp-specific. rum rebuilds a new-pointer
  tree on every rewrite, so the pointer-keyed call cache structurally cannot
  hit on freshly built trees; WolframKernel avoids the equivalent
  re-evaluation through expression sharing. The ~15x on warm mkPair versus
  the ~3x systemic constant is this redundant re-evaluation of logically
  equivalent subterms, which no cache *tuning* can remove.

Post-Task-3a update, 2026-06-13: the substitution-sharing audit is recorded
in [PROGRESS.md](../PROGRESS.md). Rule RHS instantiation does preserve shared
call subtrees for repeated bound values, unchanged sibling arguments, and
`Sequence` splices; `Expr` clones keep the shared head/argument-vector arcs.
Parent RHS calls are still rebuilt with fresh top-level pointers, so the Task
3 design constraint stands: an evaluated/stable stamp must travel on reused
shared subtrees, and the next prototype should prove that it reduces absent
call-stability misses on a workload with real shared subterms before broad
representation rollout.

Post-Task-3b update, 2026-06-13: an opt-in shared-subtree stamp prototype is
now available behind `RUM_CALL_STABILITY_INLINE_STAMP=1` and recorded in
[PROGRESS.md](../PROGRESS.md). It stores call-stability verdicts on the
existing shared argument-vector object, preserves the textual expression shape,
and uses the existing generation/dirty-bloom invalidation logic. Unit litmus
tests prove that a shared subtree can hit after the side cache is cleared, and
that a later write to the subtree head invalidates the stamp. A small rule-RHS
probe shows `call_stability_stamp_hits`.

Post-Task-3b A/B update, 2026-06-13: do not promote the stamp path to the
default runtime. The release warm public mkPair probe was run with and without
`RUM_CALL_STABILITY_INLINE_STAMP=1`. Profiled `240s` runs showed real stamp
activity (`call_stability_stamp_hits=33M+`,
`call_stability_stamp_stores=16M+`) but both variants timed out and reached the
same `268M+` stability-check scale. No-profile `360s` runs also timed out in
both variants. A follow-up split corrected a measurement trap: timeout logs can
lose earlier stdout `Print` output, so absence of `RUM_PROBE preheat_basic=` in
the full timeout log was not proof that preheat failed. Dedicated probes show
public `basicSimpset[]` still completes normally (`24.40s` standalone,
`22.68s` after mkPair setup) and the congruence preheat is `0.0018s`. The next
performance step is therefore to split the final public
`CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]` conversion itself.

Post-final-conversion split, 2026-06-13: the first split is done. A false-start
probe applied conversions to the theorem object; ignore that. The corrected
probes use `targetTm = concl[lhsAppXY]`. `buildBaseConv` takes `0.0048s` for 20
rules, and `TRYCONV[REPEATC[baseConv]][targetTm]` takes `0.079s` and leaves the
term unchanged. `descendOnce[allRules, {}, depth][targetTm]` times out at
`240s`; splitting the ordinary `comb` branch shows both
`simpFixpointConv[allRules, {}, depth][targetTm[[1]]]` and the same call on
`targetTm[[2]]` time out at `240s`. Next split: one `mkPair` application term,
comparing user rule only, basic rules only, and smaller partial applications.

Task-3a/3b review, 2026-06-13 (external): the audit and prototype were done in
the right order and the prototype is a genuinely useful **negative result**,
but it has a regression that must be corrected before any of this is committed,
and it changes how Task 3 should be scoped. Two action items, then the next
diagnosis below.

1. **Correct the default-path regression before committing (do not ship the
   carrier as-is).** The stamp prototype puts a `Mutex<Option<CallStabilityStamp>>`
   on every `SharedArgs`, and that slot is allocated and initialized on every
   call-node construction **unconditionally** — the `RUM_CALL_STABILITY_INLINE_STAMP`
   flag only gates whether the stamp is *consulted*, not whether the carrier is
   *paid for*. A clean same-session A/B (stash the uncommitted `crates/` changes,
   rebuild, measure; restore, rebuild, measure) puts the default-path PropTaut
   distribution probe at `~11.6s` with the changes versus `~10.2s` without —
   a ~12-14% regression that every workload pays, for a feature that is off by
   default and does not close the gate. The PROGRESS note "do not promote the
   stamp path to the default runtime" addresses the flag, not this structural
   cost. Fix options: revert the carrier entirely and keep only the diagnostic
   findings plus the opt-in counters (those are correctly cold-gated and cheap),
   or make the slot truly zero-cost when off (compile-time `cfg`/feature gate, or
   drop the `Mutex` — the per-session evaluator is single-threaded, so the real
   cost is the larger allocation, not the lock). Recommendation: revert the
   carrier; its value is already fully captured by the negative result below.
2. **Re-scope Task 3: its Pair justification is now refuted.** The prototype is
   exactly the cheap probe of Task 3's core mechanism, and it came back negative
   for the gate: the stamp carries the verdict correctly (litmus passes; warm
   mkPair shows `call_stability_stamp_hits=33M+`) yet the warm probe still times
   out at `240s` at the same `268M+` stability-check scale. This **refutes the
   earlier "redundant re-evaluation of shared subterms" hypothesis** — if the
   cost were re-checking shared stable subterms, the stamp would have moved it.
   It did not, so the Pair cost is not shareable redundancy; it is genuine work
   in the conversion descent. The single-allocation-node + inline-stamp rework
   may still be worth doing for the general ~2.7x constant, but it should no
   longer be greenlit on the expectation of closing Pair. Pursue the descent
   diagnosis below first.

## Where things stand

| Gate | Status |
| --- | --- |
| PropTaut distribution probe | committed HEAD `~10.2s` vs WolframKernel `3.80s` (~2.7x). The Task-3b carrier that regressed this to `~11.6s` has been reverted |
| Bool-stack load | `0.54s` vs `0.19s` (~2.8x) at committed HEAD |
| `proptaut_tests.wl` | green, ~`238s` |
| `meson_tests.wl` | green after Task 1: `List[44, 0]` in `79s` on clean `mma-hol` HEAD `698e3d0` |
| `meson_proveprop_tests.wl` | re-probed after Task 1; timeout at `300s` without a stdout summary |
| `stdlib/Pair.wl` load (stock mma-hol) | still exceeds bounded budgets; root cause is Task 2d (rum's ~34x failed-rule-dispatch cost), not a representation gap |
| `stdlib/Pair.wl` load (forked Simp.wl) | crosses the gate in a scratch clone (`85.5s` cold, `pair_tests 42/0`) but the fix is a downstream fork that masks the rum hotspot and breaks snapshots — not a shippable fix; see Task 2d |
| warm public mkPair conversion (stock) | times out; loop counts identical to WK (not looping), per-step cost is the gate |
| `cargo test --workspace`, `rum-difftest check` | pass |

The probe-visible cache-invalidation lane is closed: Do/Table iterator
Blocks no longer wipe the stability caches and matcher revisions per
iteration (`6df7bfe`), and the re-access probe is flat in expression size.
Constant-factor tuning of the current representation has reached
diminishing returns; the items below are ordered by expected value.

## Task 1 (completed): `Catch` does not evaluate its tag form

All 9 `meson_tests.wl` failures are the same defect. Minimal repro:

```wolfram
tag = "$t$"; Catch[Throw[42, tag], tag]
```

WolframKernel returns `42` (the second argument of `Catch` is evaluated,
then used as the pattern; `tag` evaluates to the string, which matches the
thrown tag). rum reports an uncaught throw because the tag form stays held
as the symbol `tag` and is matched as a pattern against the thrown string.
Literal-string forms (`Catch[Throw[42, "$t$"], "$t$"]`) and genuine
patterns (`Catch[..., _String]`) already behave correctly.

This is the same normalization issue fixed earlier for `MatchQ` /
`MemberQ` style calls (see PROGRESS "CL-Y Promoted Compatibility Fixes":
explicit pattern syntax stays held, non-pattern expressions are evaluated
before compiling an exact matcher). Apply the same treatment to the
`Catch` tag position, mirroring whatever `normalize_match_expr` path that
fix used. `Meson.wl` hits it through `mesonAttemptTag = "$mesonAttempt$"`
used as `Catch[..., mesonAttemptTag]` (Meson.wl lines 544-651).

Completion: a runtime regression test now covers the symbol-valued tag repro
(both the catching and the deliberately non-matching case), bounded
`test/meson_tests` reports `List[44, 0]` on the current clean clone, and
`test/meson_proveprop_tests` has been re-probed. Its current timeout is no
longer polluted by this bug.

Follow-up oracle check: WolframKernel does not sandbox nontrivial tag-form
side effects in the tested exact-tag path. `Print` is visible and assignments
inside the tag form remain visible after both matching and nonmatching nested
`Catch` cases, so this is no longer tracked as separate rollback debt.

## Task 2 (completed diagnosis): Pair/Simp multiplier hunt

`stdlib/Pair.wl` still exceeds `600s` with every retained optimization in
place, while the overall evaluator constant is only ~3.4x. If the load
were constant-bound it would finish in roughly `3.4x` of WolframKernel's
time; verify what WolframKernel actually needs for the same
dependency-preloaded `Get["stdlib/Pair.wl"]` (internal `AbsoluteTiming`,
not wall time). If Wolfram needs only seconds, rum is doing work Wolfram
does not do — an algorithmic multiplier, not a constant — and no amount of
representation tuning will close it.

Method (count, do not sample — see the audit doc):

1. Get the Wolfram reference timing for the same preloaded `Pair.wl` load.
2. Bisect the load cost by phase: `basicSimpset[]`, the first
   `simpConv`/`CONVRULE` use, and the `mkPairInjThm` construction were the
   expensive stages in the 2026-06-11 triage; re-measure each in both
   engines.
3. Add a cheap opt-in counter (pattern: `RUM_PROFILE_EVALUATION_CACHE` in
   `evaluation_cache.rs`) counting definition rule applications and
   `evaluate` entries; compare counts between a fast stage and a slow
   stage, and against a logical-work estimate. A stage whose rum count is
   orders of magnitude above the estimate is the multiplier; find which
   downvalue/loop generates it before changing any representation.
4. Only if counts match Wolfram-plausible work and the stage is still
   slow does the cost go back into the constant-factor bucket (Task 3).

Completion: the PROGRESS entry "Pair/Simp Multiplier Diagnosis" records the
verdict and counter numbers. The concrete construct is the public
`simpConv` / `withBasic` base-conversion path, especially repeated
`buildBaseConv` / nested `ORELSEC` / `TRYCONV` failure dispatch inside
`simpFixpointConv` and `descendOnce` over the mkPair term.

Follow-up runtime work reduced the constant factor for the same probes but
did not remove the multiplier. The retained cuts and rejected
base-conversion memo experiment are recorded in PROGRESS under
"Pair/Simp Runtime Stability Cuts".

## Task 2b (completed diagnosis): explain the stability-check amplifier

The Task 2 diagnosis localized the multiplier but did not explain it, and
the unexplained signal is precise: the warm public mkPair probe reaches
**`536M+` stability checks while `evaluate_entries` and
`down_value_applications` do not advance proportionally past the preheat
background**. Something performs enormous amounts of stability checking
without corresponding evaluation. WolframKernel runs the same logical
algorithm in `23.1s`; the general rum constant elsewhere is now only
~2.7-4x. Until the mechanism is named, neither a downstream `Simp`
algorithm change nor the Task 3 representation rework is clearly the right
spend — a third, much cheaper fix may exist. Do not start Task 3 before
this verdict is written.

Probe: the warm public mkPair shape from the Task 2 entry (preload through
`auto/Simp.wl`, precompute `basicSimpset[]`, `impCongLemma[]`,
`andCongLemma[]`, `orCongLemma[]`, then time
`CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]`), bounded at `120-180s` —
counters and samples from a bounded window are sufficient; the probe does
not need to complete.

Three candidate mechanisms, each cheap to confirm or kill. Instrument
first, conclude second:

1. **Call-stability cache size-cap thrash.** The pointer-keyed cache is
   capped at `CALL_STABILITY_CACHE_MAX_ENTRIES = 1 << 14` and is cleared
   wholesale on overflow (`evaluation_cache.rs`). The runtime stability
   cuts added data constructors, hold forms, and status objects to the
   cacheable set, so the Simp working set (large persistent simpset
   structures plus HOL terms) may now exceed `16k` live stable subtrees.
   If so the cache clears itself in a loop: insert past the cap, clear,
   re-walk everything, repeat — exactly "many checks, few evaluations".
   Confirm: add an opt-in counter that distinguishes size-cap clears from
   generation bumps and report both for the bounded warm window. Cheap
   falsification A/B: raise the cap `16x` and re-time the warm probe; a
   large move confirms, no move kills.
2. **Dirty-bloom saturation generation bumps.** Every `popcount > 96`
   saturation bumps the call-cache generation, killing all entries. A
   write-heavy Simp phase saturates every ~100 distinct written symbols,
   so long-lived stable trees may be re-walked after every ~100 writes
   forever. Confirm from the same run: compare `call_generation_bumps`
   against `symbol_invalidations` for the warm window, and correlate with
   `call_stability_misses`. If this dominates, the fix direction is finer
   invalidation for long-lived entries (exact per-symbol write versions
   were rejected once on PropTaut, but the cost profile here is different
   — re-measure on THIS shape before re-rejecting), a wider bloom, or
   exempting entries whose bloom does not intersect the saturating write
   set.
3. **Genuinely fresh trees from conversion-chain rebuilding.** If the
   checks hit mostly never-seen pointer keys (high
   `call_stability_misses` with low hit rate and no clear/bump
   correlation), then `simpFixpointConv` / `descendOnce` really does
   rebuild the `withBasic` chain per subterm and the cost is real work.
   Then the choice is the downstream algorithmic discussion or Task 3,
   as already recorded.

Attribution to callers complements the counters: sample the warm window
and run
`scripts/sample-attribute-callers.py <sample> cached_expr_stability_with_bloom`
to see which evaluator paths drive the checks (fixed-point pre-checks vs
uncached-walk recursion vs argument preparation).

Completion: the PROGRESS entry "Pair/Simp Stability-Check Amplifier Diagnosis"
names mechanism 3 as dominant for the current warm window. In the default
120s miss-reason probe, `call_stability_miss_absent=67M+` dominates
`call_stability_miss_dirty=33M+` and
`call_stability_miss_generation=4M+`; in the 180s windows, 16x cache capacity
does not move the major counts, while a dirty-bloom threshold of 128 explodes
misses and rewritten-dirty skips to `1B+`. The end-state target for the lane
stays the same: warm public mkPair completes inside a bounded budget and the
`Pair.wl` load enters the constant-bound regime (roughly `4x` of
WolframKernel's `37.7s`, i.e. about `150s`).

## Task 2c (completed diagnosis): is the Simp descent looping? — No; it is per-step cost

This is the focus for the next session. The Task 3b stamp result rules out
"shareable redundant re-evaluation" as the Pair cost, and the conversion split
narrowed the timeout to recursive descent over `mkPair` application terms
(`descendOnce` → `simpFixpointConv`, with both `comb` halves independently
timing out). The decisive next question is whether `simpFixpointConv`'s
fixpoint loop **runs the same number of steps in rum and WolframKernel** (so the
gap is the ~3-4x per-step constant) or **runs far more steps in rum** (a
non-converging fixpoint — a different, usually cheaper, class of bug).

The enabling fact: `simpFixpointConv`, `descendOnce`, and `buildBaseConv` are
**WL-level functions in `auto/Simp.wl`, not kernel builtins**, so they can be
instrumented identically in both engines. This finally makes the
multiplier-vs-constant question at the descent level *directly measurable*
rather than inferred from rum-side counters. Read `simpFixpointConv` (around
`Simp.wl:446`): its `While[True, ...]` loop breaks on `newRhs === rhs`
(fixpoint reached) or `KeyExistsQ[seen, newRhs]` (cycle), and it rebuilds
`buildBaseConv` once per entry.

Completion, 2026-06-13: the two-engine loop-count probe was run on a scratch
instrumented `Simp.wl` copy at
`/private/tmp/rum-mma-hol-simp-loop.sSsJv1/mma-hol`. The completing
user-rule-only probe
`simpFixpointConv[{mkPairDefThm}, {}, $simpDepthLimit][concl[lhsAppXY]]`
reported identical counts in rum release and WolframKernel: `3565` entries,
`3573` while iterations, max `seen` size `3`, `3565` fixpoint breaks, and `0`
cycle breaks. Timings were `40.977489205s` in rum release and `12.837901s` in
WolframKernel, about `3.2x`. This completing shape is therefore not looping in
rum and does not support the `newRhs === rhs` / origin-annotation
non-convergence hypothesis. The next split should focus on rule-set and term
shape for one `mkPair` application; if the full public ruleset is suspected of
a separate loop, use explicit in-run snapshots rather than killed-process
stdout.

Plan:

1. In a scratch copy of `Simp.wl`, add global counters: total `simpFixpointConv`
   entries, total `While` iterations across all entries, and the max `seen`-set
   size. Print them after a probe. Apply the identical edit to the WolframKernel
   run.
2. Run a **completing** probe in both engines — user-rule-only mkPair finishes
   (~`12.8s` WK, ~`40-52s` rum), so it gives comparable counts — and compare the
   three numbers.
   - **Counts roughly equal between engines** → rum's cost is the per-step
     constant; the full-ruleset timeout is just more of the same. The lane then
     legitimately belongs to the general evaluator constant (Task 3) or a
     downstream `Simp` change, and Task 3's payoff estimate applies.
   - **rum's counts much higher (10x+)** → the fixpoint is not converging in
     rum: `newRhs === rhs` keeps returning False where WK returns True, so the
     loop spins on logically-equivalent but structurally-different terms. This
     is the high-value, usually-cheap case.
3. If the loop is spinning, find the spurious structural difference: dump the
   first few rum `newRhs`/`rhs` pairs where `newRhs =!= rhs` but the terms are
   alpha/semantically equal, and diff their `FullForm`. **Prime suspect: HOL
   origin annotations.** `stripOrigin` was already a hot per-symbol downvalue in
   the Task 2b counters; if rum's rewrite output carries origin tags that WK
   strips (or normalizes differently), every `===` comparison in the fixpoint
   loop fails and the descent never settles. Other candidates to rule out:
   `Plus`/`And` argument ordering, association internal form, and beta/`betaSubst`
   output shape (`betaSubst`/`vsubstImpl` were also hot). The fix for any of
   these is a local normalization so `===` matches WK, not a representation
   rework.
4. Independently, confirm the breadth vs depth shape: the developer's observation
   that splitting `targetTm` in half leaves **both** halves at the full `240s`
   timeout is itself evidence against a simple linear descent (a linear descent
   would roughly halve). Either each half independently spins (supports step 2's
   non-convergence case) or the descent is super-linear in term size (supports a
   beta-expansion term-size blowup — measure intermediate term size across the
   descent to tell these apart).

Acceptance: a PROGRESS entry stating, with the two-engine loop counts, whether
the Simp descent is constant-bound or looping in rum, and — if looping — the
named structural difference that breaks `===`, with a minimal repro. Only after
this verdict should the lane commit to either a local normalization fix, a
downstream `Simp` change, or the Task 3 representation rework.

The developer's own proposed split (one `mkPair` application term: user rule
only vs basic rules only vs smaller partial applications) is a reasonable
secondary cut, but the two-engine loop count above is more decisive and should
come first — it distinguishes "looping" from "slow" in one measurement instead
of bisecting rule sets.

## Task 2d (current focus): fix rum's per-failed-dispatch cost in the kernel

Task 2c is answered (not looping; per-step constant) and the secondary split
localized the multiplier to **basic-rule failed dispatch** — but at a ~34x
rum-vs-WK ratio on that specific path, not the ~3x general constant (see the
review block at the top for the arithmetic). That ~34x is a rum kernel hotspot
in how a *failed* conversion dispatch is executed. Fix it there so stock
mma-hol runs fast and the snapshot lane is untouched. Do not fork `Simp.wl`.

The unit to profile is **one failed `REWRCONV` dispatch**. Its HOL body
(`Drule.wl:292`) is small: `concl`, a `MatchQ` shape check, two `Part`s,
`tryTermMatch[lhs, t, ...]`, then on `$matchFail` a `convFail[...]`. The
developer already ruled out the obvious suspects at the WL layer: removing the
`Failure` payload, a no-throw sentinel, and a fast const-head pre-check inside
`REWRCONV` each failed to move wall time, and `tryTermMatch` depth is not it
(the fast head check caught `1998/2120` failures with no wall-time change). So
the ~5.3ms is *not* in the parts that a WL edit can see — which is exactly why
this must be profiled in rum, not in `Simp.wl`.

Concrete steps:

1. Build the smallest reproducing probe that does many failed dispatches and
   completes: the `basic`-only descent on `pair4Same` is ideal — `8.739s`,
   `61` entries, `Changed=False`, all dispatch failures. It is a few seconds,
   so it samples cleanly.
2. Sample it and reduce with the existing tools:
   `scripts/sample-self-time.py` then
   `scripts/sample-attribute-callers.py <sample> <hot leaf>`. The question is
   what rum executes per failed `REWRCONV` that WK does not. Likely suspects,
   in order of how cheaply each is confirmed:
   - **Per-attempt conversion-combinator construction.** `buildBaseConv`
     builds `combineSimpConvs[Join[ctxConvs, ruleConvs]]` and each
     `simpRuleConv[#, prover]` closes over `prover = makeProver[...]`. If rum
     re-evaluates/reconstructs these `Module`/`Function` closures per subterm
     (the caching experiment helping is circumstantial evidence it does), the
     cost is closure/`Module` setup, not matching. Check whether `eval_module`
     / closure construction dominates the failed-dispatch self time.
   - **`prover` recursion on conditional rules.** If any basic rule is
     conditional, trying it invokes `prover`, which may recursively simplify
     hypotheses even when the rule ultimately fails. Check the call tree under
     a failed `REWRCONV` for re-entrant `simpFixpointConv` / prover calls.
   - **Owned-tree churn per attempt.** `concl`, `Part`, and `tryTermMatch`
     each clone HOL subterms; on a ~34x path even modest per-call cloning
     compounds. This overlaps the general representation lane (Task 3) but may
     have a cheap local win (borrow instead of clone in the match-and-fail
     path).
3. Counter cross-check: add an opt-in counter for `eval_module` entries (or
   closure constructions) per failed `REWRCONV`, run the `basic`-only
   `pair4Same` probe, and compare against the `61` entries. If closures are
   rebuilt thousands of times for `61` entries, that is the multiplier and the
   fix is to build the conversion once per `simpFixpointConv` entry (or per
   rule set) inside the kernel's handling, not per subterm.

Acceptance: a PROGRESS entry naming what rum spends the ~5.3ms-per-failed-
`REWRCONV` on, with the self-time/attribution evidence, and a rum-side change
that brings the `basic`-only `pair4Same` descent and the `userBasic` mkPair
shapes down without any edit to `auto/Simp.wl`. Target: stock (unmodified)
mma-hol `pair_tests.wl` green and `stdlib/Pair.wl` load inside a bounded budget.

If the profile shows the ~5.3ms is irreducible without the call-node
representation change (e.g. it is dominated by per-attempt owned-tree cloning
that only borrowed traversal fixes), then and only then does this merge into
Task 3 — and a Simp.wl improvement *upstreamed to mma-hol* (not a private rum
fork) becomes the pragmatic fallback. Decide that from the profile, not before.

## Task 2e (snapshot lane): make rum preserve `$Packages` across DumpSave restore

A snapshot of the Pair lane failed under rum, and the scratch experiment
"fixed" it by editing the standard's harness — `tests/extend_snapshot.wls` in
the canonical clone was modified to add
`$Packages = Union[$Packages, holPublic]` after restoring a `DumpSave`. That
edit has been reverted; the canonical clone is pristine again. The harness edit
is not the fix (it changes the acceptance standard — see `AGENTS.md`).

The real gap is rum-side: `DumpSave` does not persist `$Packages`, so after
`Get`-restoring a snapshot, `BeginPackage` dependency checks in the unmodified
harness do not know the restored packages are already loaded. WolframKernel
runs the unmodified `extend_snapshot.wls` correctly, so this is a rum
`DumpSave`/`Get` round-trip gap. Fix: have rum's snapshot save/restore preserve
(or reconstruct on restore) `$Packages` for the saved public contexts, so the
unmodified harness resolves short names and dependency state without any edit.
Note the separate, legitimate observation already in PROGRESS: rum's full
private-context `DumpSave` also serializes generated `Module` locals
(`baseConv$…`, `xEq$…`) and inflates the dump (~`402MB`); that is a real rum
snapshot-hygiene bug to fix too (exclude generated/temporary symbols), but it
is independent of the `$Packages` restore gap and must also be fixed in rum,
not by pre-cleaning the standard's scripts.

Acceptance: an unmodified-`mma-hol` snapshot build/extend through `Pair`
completes and restores correctly under rum (`pair_tests.wl` green after
restore), with no edits to any file under the `mma-hol` tree.

## Task 3 (large, gated behind Task 2d): representation rework

> Re-scoping note (Task 3b review): the inline-stamp prototype already probed
> this lane's core mechanism and did **not** close the Pair gate, so do not start
> Task 3 expecting to close Pair. It may still be worth doing for the general
> ~2.7x constant. Run Task 2c first; if that verdict is "looping," a local
> normalization fix likely closes Pair far more cheaply than this rework.

Plan, in order: single-allocation call node (head+args in one `Arc`),
which halves per-node allocation/refcount traffic and enables an inline
per-node evaluated/stable stamp replacing the pointer-keyed
call-stability side cache; then borrowed argument preparation so shared
argument vectors are not cloned per evaluation pass. Multi-day,
cross-cutting.

**Load-bearing design constraint from the Task 2b review — get this
right or the mkPair gate will not close.** The Pair/Simp gate is
dominated by `absent` call-stability misses: rum rebuilds a new-pointer
tree on every rewrite, so a pointer-keyed cache (the current side cache,
and equally a stamp that lives only on a freshly allocated node) can
never hit on those trees. An inline evaluated/stable stamp therefore
only helps if it **travels on the shared subtree `Arc`s reused through
substitution**, not on the freshly allocated parent node. Concretely:
when a rule RHS is instantiated, the bound subtrees are reused by `Arc`
clone; their stamps must survive into the new tree so that an
already-evaluated subterm is recognized as evaluated even though its new
parent has a fresh pointer. If the rework only stamps whole freshly
built nodes, it will reproduce the `absent`-miss behavior and leave the
~15x mkPair gap intact while still paying for the rewrite.

A litmus test for the design before committing to it: the `fib[27]`
baseline currently shows `call_stability_miss_absent == evaluate_entries`
and a `0%` call-stability hit rate (every rewrite is a fresh tree). After
the stamp change, a memoization-free recursive workload that re-encounters
structurally shared evaluated subterms should show those subterms
short-circuiting — i.e. the absent-miss count should fall well below the
evaluate count on a workload with real subterm sharing. If it does not,
the stamp is not propagating through substitution and the design needs
revisiting before the larger rollout.

If the propagation requirement turns out to need structural sharing /
hash-consing of subterms beyond the sharing already confirmed in Task 3a and
the existing-`SharedArgs` stamp prototype from Task 3b, that is the real shape
of the work, and it is larger than "add a stamp field". The warm mkPair A/B
did come back weak: inline stamps hit on small shared-subtree litmus tests and
in the profiled warm run, but they do not move the public gate. Before deeper
representation work, split the final public `simpConv[{mkPairDefThm}]`
conversion and check whether a downstream `Simp` algorithm change is the
cheaper route. The first split now points at recursive descent over single
`mkPair` application terms, not at `buildBaseConv` or top-level rewriting.

## Methodology guardrails (lessons already paid for)

- **Decompose composite probes before attributing their cost.** The
  interleaved-write probe was attributed to the dirty-bloom same-bit
  fallback; one decomposition command showed the cost was entirely the
  Do-iterator Block wrapper (`Do[e2;, {100}]` 2.3us/iter vs
  `Do[e2;, {i, 100}]` 428us/iter with no writes at all). Two rounds of
  invalidation-scheme work optimized a path the probe never exercised.
- **Estimate the expected win from the profile before keeping a change**,
  and reject probes whose target does not appear in attribution output.
- **One variable per A/B**, internal `AbsoluteTiming` against internal
  `AbsoluteTiming`, and never benchmark while another heavy process (or a
  second rum probe) shares the machine.
- Profile reduction tools: `scripts/sample-self-time.py`,
  `scripts/sample-attribute-callers.py`; cache counters:
  `RUM_PROFILE_EVALUATION_CACHE=1`.

## Known environment notes

- `Do[..., {10000}]`-scale loops can overflow the CLI stack and
  `Range[10^6]` exceeds the generated-element budget; scale probes
  accordingly or raise limits explicitly in the probe session.
- The acceptance `mma-hol` clone is now the persistent
  `/Users/fft/Developments/mma-hol-acceptance` (latest published HEAD
  `200f13a`, the completed project — a large advance past `698e3d0`; older
  PROGRESS/audit entries that cite the decayed
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` are historical records of
  what was measured then, not live paths). `wolframscript` is at
  `/usr/local/bin/wolframscript` (~3.2s startup —
  never compare wall times).
