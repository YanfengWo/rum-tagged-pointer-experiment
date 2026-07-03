# CLAUDE.md — director's notes for `rum`

This file is **mine** (Claude). `AGENTS.md` is the operational first-stop guide
that any coding agent — including Codex — reads when executing work. This file
is about how *I* operate on this project: direction, diagnosis, delegation, and
the authoritative gate. Keep it short and high-signal; link to canonical docs
rather than duplicating them.

## Role split (the working mode)

- **I own**: overall direction, the performance/correctness diagnosis, picking
  the next target, writing detailed Codex task briefs, the authoritative gate,
  and the git commit. I do the *judgment*; the legwork I delegate.
- **Codex owns**: executing exactly one brief, self-verifying to green, and
  reporting with verbatim evidence. It does not commit, branch, or push — it
  leaves changes in the working tree for me to gate.
- The brief method lives in `docs/dev/codex/`: `TEMPLATE.md` is the format,
  `brief-NNN-<slug>.md` are the packages. Dispatch via the helper (commit the brief
  FIRST, as a separate step):

  ```
  scripts/dispatch-brief.sh NNN     # e.g. 081, or brief-081, or a slug fragment
  ```

  It resolves `docs/dev/codex/brief-NNN-*.md`, runs `cdx exec -s danger-full-access`
  with stdin from `/dev/null`, and logs the full report to `/tmp/cdx_brief_NNN.log`
  (`DRY_RUN=1` prints the resolution without dispatching). **Launch it via the Bash
  tool's `run_in_background:true`** so the run is harness-tracked — do NOT hand-type
  `cdx exec … &`; the `&`-detach is not tracked (I made that mistake twice, hence the
  script). The raw `cdx exec` line it wraps:

  ```
  cdx exec -C "$(git rev-parse --show-toplevel)" -s danger-full-access \
    "Read docs/dev/codex/brief-NNN-<slug>.md and execute it exactly. Follow its Hard rules." \
    < /dev/null > /tmp/cdx_brief_NNN.log 2>&1
  ```

  `-s danger-full-access` = Codex's sandbox level (needed for `cargo`,
  `rum-cli`, and `sample`); the brief's **Scope + Hard rules** are the real
  fence. My own dispatch/gate commands are allowlisted in
  `.claude/settings.local.json` (`cdx exec *`, `scripts/dispatch-brief.sh *`, the
  microbench script, the sample reducers) so I am not prompted each round.

## The gate I run before crediting any brief

1. **Scope check**: `git status` / `git diff` touched only the brief's
   MAY-modify list. Nothing under any `mma-hol` clone, no `Expr`/`Symbol`/serde
   shape change unless that was the Goal.
2. **Tests**: `cargo fmt --check` and `cargo test` green. Default to
   `--workspace`; use the narrower `-p rum-runtime` ONLY when the change is
   provably runtime-local. (Cost of skimping: brief-046's `-p rum-runtime`-only
   gate missed 7 PRE-EXISTING cross-crate test failures in rum-cli/host/jupyter/
   repl — stale `f[1 2]` syntax fixtures obsoleted by juxtaposition `67a3bdc` —
   which only surface under `--workspace`. A green narrow gate is not a green
   tree.)
3. **Performance claim**: I re-run the named microbench label myself and
   confirm the before/after ratio AND that same-tier neighbors did not regress.
   I verify Codex's profiling evidence with `scripts/sample-self-time.py` /
   `scripts/sample-attribute-callers.py` — I trust the numbers I reproduce, not
   its interpretation.
4. Only then do I commit and dispatch the next brief.

## Hard lines I enforce (do not let these drift)

- **Reply to the user in Chinese — always.** This project's working language is
  Chinese; the user cannot read any other language. Every user-facing message
  (status, diagnosis, recommendation) is in Chinese; only technical tokens (code
  identifiers, command/file names, WL/Rust terms like `Listable`/`HoldAll`) stay
  verbatim in English. Briefs/commits/docs are drafted in English (the codebase
  is English), but the conversation is Chinese. I have drifted to English **and
  once to Japanese** in long reports — watch for ANY non-Chinese drift,
  especially in multi-paragraph write-ups. (Corrected 3×; see
  `~/.claude/.../memory/working-language-chinese.md`.)
- **`mma-hol` is the unmodified acceptance standard.** Never accept a fix that
  edits it, forks `Simp.wl`, or patches a test harness to pass. If unmodified
  `mma-hol` runs under WolframKernel but fails/times-out under rum, the gap is
  rum's, full stop (`AGENTS.md` hard line). The acceptance standard is the
  persistent clone `/Users/fft/Developments/mma-hol-acceptance` (latest
  published HEAD `200f13a`, the completed project — past the old `698e3d0`
  probe snapshot); keep it pristine, diagnostic edits only in throwaway scratch
  copies. The old `/tmp` clone decayed under `/tmp` purging — do not rely on
  `/tmp` for the standard.
- **No default-path cost for diagnostics or off-by-default features.** Counters
  and instrumentation go behind the env-gated cold-load `AtomicU8` pattern; a
  `Mutex`/allocation/branch paid on every call when the flag is off is a
  regression (this already happened once — the `SharedArgs` stamp carrier cost
  ~12-14% and was reverted).
- **Measure honestly**: internal `AbsoluteTiming` vs WolframKernel internal
  timing only (startup ~3.2s contaminates wall); profiling env OFF for any
  timing number; one variable per A/B.

## Diagnosis method (why I redirect Codex so often)

Codex's recurring failure modes, watch for all three in review:
1. **optimizing fake sub-second hotspots** while the parent workload is
   hundreds of seconds — pick the target from the bottom-up microbench ratio,
   not from a sampler's loudest leaf;
2. **fixing at the downstream WL layer** (forking `Simp.wl`) instead of the rum
   kernel — the two-engine logical-work comparison settles it: if WK does the
   same logical work (same loop/dispatch counts) faster, the defect is rum's;
3. **masking symptoms** (reducing a dispatch *count* to hide a per-op
   slowness) instead of fixing the per-op cost.

My standing techniques: **count, don't sample, for multipliers** (instrument
the WL-level function in *both* engines — `simpFixpointConv`, `REWRCONV`,
`buildBaseConv` are WL, not builtins); **decompose a composite probe into its
factors before attributing its cost** (the interleaved-write probe's cost was
the Do-iterator `Block` wrapper, not the cache); **estimate the expected win
from the profile before keeping a change.**

## Current direction (read these for state)

- **ACTIVE LINE (2026-07-03): crate responsibility/size REBALANCING — route 2a (full
  builtin extraction). Public release is DEFERRED (not urgent; user chose code cleanliness
  first).** See memory `crate-rebalance-line.md` for the full plan + data. Summary: the
  post-long-tail codebase is crate-imbalanced — `rum-runtime` is a 116k-line/391-file
  MONOLITH (evaluator + ALL builtins) while `rum-stdlib` is a misnamed 21-line installer
  facade. The KEY constraint: builtins are DEEPLY woven into the kernel (528
  `pub(in crate::runtime)` symbols, builtins `use super::*`, dispatch is a CLOSED
  `BuiltinHandler` enum + big match), so a clean extraction is a real architecture change,
  not a move. **Route 2a plan:** (1) convert `BuiltinHandler` closed-enum dispatch → an
  OPEN registry (fn-pointer/trait handlers); (2) define a clean `pub` kernel API (promote
  the subset of the 528 internal symbols that builtins actually need); (3) move the builtin
  families (`strings/`, `structure_ops/`, `functional_ops/`, `associations/`, `polynomials/`,
  `logic/`, `predicates/`, `arithmetic`, `number_theory`, `combinatorics`, `calculus`,
  `integer_digits`, control-flow builtins, …) into a REAL stdlib crate (rename `rum-stdlib`
  honestly), leaving `rum-runtime` as the pure kernel (eval/patterns/scoping/definitions/
  model/cache/session). **DEFERRED within this line:** file-slimming (7 files >35k chars =
  architecture debt — `profiling.rs` 138KB, `evaluation_cache.rs` 62KB,
  `definition_application.rs` 55KB, `vm_rhs_template.rs` 51KB, `lexer.rs` 40KB,
  `arithmetic.rs` 39KB, `parser/tests.rs` 37KB — + 27 split-plan files; convention =
  `docs/code-organization-guidelines.md`, unit is CHARACTERS) is done AFTER the crate split
  (files move more easily once split; slimming a file about to move is wasted). **OPEN
  sub-decisions:** fate of `profiling.rs`/`vm_rhs_template.rs`/`debug_tools/` (diagnostic +
  opt-in-experimental) — stay-in-kernel vs feature-gate vs move — decide during 2a design.
  `release-staging/` was DELETED (will be re-created after cleanup via the committed
  brief-096 scrub method + the recorded whitelist); legal framing settled (independent
  sandboxed evaluator, NO WK-parity narrative — memory has the details).
- **Earlier lines CLOSED (2026-07-01), historical context below.**
  (1) **Pattern-matching line CLOSED** (briefs 058 + axis measurements): the adversarial
  `Orderless`+`BlankNullSequence` stack-overflow crash is fixed (matcher `MAX_MATCH_DEPTH`
  guard → graceful `TerminatedEvaluation`); WK was probed and is ALSO naive `k^n` (no
  pruning) so chasing pruning is a trap; axis (a) measured — no downvalue long tail in HOL
  (max 6/symbol), linear-scan dispatch is optimal. See memory
  `wk-ac-matching-is-naive-enumeration.md`. (2) **Host-embedding hardening arc DONE**
  (briefs 059-066): Tier-1 (interrupt/cancel + wall-clock timeout, `catch_unwind`+poison,
  structured `HostValue`, memory budget as a documented host-side RSS watchdog) + Tier-2
  core (`Session` is now `Send`, `reset()`, whole-session introspection, snapshot/
  `restore_from_bytes` = the WASM no-fs enabler) + the **rum-jupyter REAL interrupt**
  (execute-on-worker + interrupt→cancel + poison-restart, the payoff). Entry point +
  status: `docs/host-embedding-and-hardening.md` (all Tier-1/Tier-2-core items marked DONE)
  + memory `host-hardening-tier1.md`. **OPEN directions now:** a showcase project (sandbox +
  WASM-ready enablers, specific project undecided), the deferred perf line
  (hash-consing/value-memo ~26% on Pair, bytecode VM), lower-value jupyter stubs
  (history/comm_info/debug), and small conformance items (genuine-cycle →
  `TerminatedEvaluation[IterationLimit]`).
- **MEMORY LINE CONCLUDED for safe Rust (2026-06-30).** Phase 0 instruments + 0.5
  four-C panel + 1/1b gensym GC + Phase 3 per-node `Expr` compression (rungs 1-3:
  `size_of::<Expr>()` 64→32, dense-list factor 8.3x→~4.0x, kernel faster + alloc
  ~200x↓); Phase 2 interner residual kept as a bounded tradeoff. Headline Module leak
  (6 MB→bounded) + dense-list factor addressed. Further needs unsafe tagged-pointer or
  packed arrays (poor fit for symbolic rationals-only rum). Read
  `docs/memory-management-plan.md`. KEY measurement caveat: only the four-C
  `retained_ratio` is a clean cross-engine constant factor; peak/transient/RSS WK
  columns are confounded by WK's startup high-water — read them rum-local.
- **The correctness mainline is COMPLETE: rum reproduces the README's 3126/0 exactly
  (2026-06-30).** The 2855-vs-3126 gap was a REAL bug (brief-051: `Set[a,a]`
  self-fixed-point aborted `Get` mid-file in `arith_tests.wl`), now fixed; `run_fast`
  3055/0 + `kernel_tests` cold 71/0 = 3126/0, 0 failures. v8 snapshot is authoritative.
- The **perf/representation line is PAUSED at a clean stop** (not the active
  line): `docs/representation-rewrite-plan.md` (Layer A done+gated; flaws 2-3
  refuted; hash-consing validated SOUND+~26% on Pair but DEFERRED). Deferred SAFE
  perf work after correctness: hash-consing/value-memo + Module-gensym GC (~10GB
  live leak). Entry point for that line: `scripts/core-language-microbench.sh`.
- `PROGRESS.md` — full running log; the `2026-06-23` entries cover the correctness
  chain + the 5 parser/lexer/eval bugs; `2026-06-21`/`2026-06-22` cover the perf
  line (Layer A, the Pair re-diagnosis, the eliminated dead-ends).
- `docs/proptaut-performance-audit.md` — the profiling method and reducers.
- **Unified model (corrected this session):** the Pair `~34x` is NOT a separate
  failed-dispatch defect distinct from the ~3x constant — it is the SAME
  distributed per-call dispatch/eval constant the primitive microbench measures,
  compounded over the descent and amplified ~1.7x by the WL conversion-control
  combinators. The failing match is ~5% of a failing dispatch; the rest is the
  per-call machinery. Proven one-lever by A4 translating a per-call head-clone
  cut to the Pair raw failing-dispatch (`-5.8 to -7.9%`, clean A/B). Refuted as
  the Pair lever: the inline stamp (Task-3b), node-alloc compaction (A3 did not
  translate — only WORK cuts do), and Module frame setup (`1.58x`).

## Pointers

- `AGENTS.md` — operational first-stop (project snapshot, commands, trust
  boundary, reading path). I read it too, but it is written for any agent.
- `PROGRESS.md` / `TODO.md` — running log and near-term work.
- `~/.claude/.../memory/rum-project-context.md` — my cross-session memory of
  this project's arc and the recurring lessons.
