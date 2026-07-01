# Progress Log

This file records completed work, archived planning detail, and operational
lessons that are useful history but should not keep cluttering `TODO.md`.

`TODO.md` should stay focused on active or near-term work. When a completed
checklist stops helping prioritization, move or summarize it here.

## 2026-07-01 Host hardening line — Tier-1 (briefs 059/060/061; memory budget deferred)

New mainline after the pattern-matching line closed: host-embedding Tier-1 hardening
(`docs/host-embedding-and-hardening.md`). Sequenced as one-lever briefs:
059 interrupt/cancel + wall-clock time budget (DONE), 060 catch_unwind panic boundary,
061 structured HostValue, then the memory budget (deferred — has a real proxy-vs-RSS
design fork + WASM-purity tradeoff, and is lower-urgency since the time budget already
bounds most DoS; the structural `current_memory_proxy_bytes` is an O(state) full sweep,
too costly per-step, and is blind to the interner leak — RSS is the true total but is a
platform syscall that breaks the WASM-clean core).

brief-059 (`e5b13b9`): `HostSession::eval` was synchronous + uninterruptible and had no
wall-clock cap (the #1 + #2 Tier-1 gaps). BOTH now served by ONE amortized poll point in
the eval loop: at `evaluate_once_unstable_*` (eval_core.rs, where recursion_limit is
checked) a single `if self.interrupt_armed` branch gates a `#[cold] #[inline(never)]`
poll fn that, every 1024 armed steps (POLL_MASK=1023), checks the cancel `Arc<AtomicBool>`
(Relaxed) and the deadline. `interrupt_armed` is recomputed once per top-level eval from
`Arc::strong_count(interrupt_flag)>1` OR a timeout being set — so the `&self`
`cancel_handle()` API needs no `&mut`. New `EvalError::Cancelled` → own tag
(`TerminatedEvaluation["Cancelled"]`); timeout reuses `TimeConstraintExceeded`. Host API:
`HostCancelHandle` (Clone+Send+Sync over the Arc, `.cancel()`), `cancel_handle()`,
`eval_with_timeout()`, `HostFailureCode::Cancelled`. Only the `Arc<AtomicBool>` crosses
threads — Session's thread contract unchanged.

GATE (Claude): cargo fmt; cargo test --workspace 712/0 + rum-host 7/0 incl. a real
cross-thread cancel test (eval stopped ~60ms after cancel) + a 200ms timeout test
(stopped at 201ms) + a positive unused-handle test. Hot-path microbench A/B (base-vs-new
release binaries, scale 1/10/20, 3 interleaved rounds, median-of-medians): the
eval-loop-heaviest label loop-empty is flat-to-faster (0.95-1.01) at every scale; the
scattered 4-30% on sub-ms labels was run-to-run noise (uncorrelated with eval-loop
intensity, swung both ways). No per-step regression — the disarmed branch is free.

brief-060 (`4e532a8`): catch_unwind panic boundary + session poison. A Rust panic inside
eval unwound straight through the rum-host API (UB across a future FFI/pyo3 boundary,
takes the host down). HostSession gains `poisoned: bool` + a `guard<T>(&mut self, body,
on_panic)` helper running `catch_unwind(AssertUnwindSafe(|| body(self)))`; a caught panic
sets poisoned + returns a graceful `HostFailureCode::Panic` (message from the downcast
payload); subsequent calls return `Poisoned`. All entries uniformly guarded → the
read-only query methods became `&mut self` (ACCEPTED: uniform safety, host owns the
session mutably anyway; only ripple was trivial `let mut` + a borrow-split in the jupyter
adapter). Host-layer only, zero runtime change, zero hot-path (wraps the outer entry).
NOT caught: stack-overflow aborts (those are guarded by parser MAX_PARSE_DEPTH + matcher
MAX_MATCH_DEPTH). No global panic hook (a lib must not impose process-global state) — the
default hook still prints to stderr before catch_unwind returns; capturing that is a
documented follow-up. Gate: workspace 712/0 + rum-host panic/poison test 2/0.

brief-061 (`0423bb4`): structured (typed) HostValue. Was `{ text: String }` only; now
ALSO carries a typed view (`.text` byte-identical, additive). New `HostTyped` enum:
Integer(i64) | BigInteger(String) | Rational{num,den} | Symbol(String) | Str(String) |
List(Vec<HostTyped>) | Other{head,text}, built once from `response.value: Expr` in
`eval_success_from_runtime`. Rationals-only → no float variant; BigInteger/Rational use
decimal strings (2^200 doesn't fit i64). Depth-capped (MAX_TYPED_DEPTH=64) so a deep
List[List[...]] falls back to Other{text} instead of overflowing the converter (same
discipline as the parser/matcher guards). rum-host gained a direct workspace-internal
rum-core path dep to name Expr/Atom (runtime exposes the Expr publicly but doesn't
re-export the type; brief forbade runtime changes; no Cargo.lock change). Gate: workspace
712/0 + typed test 1/0 (Integer/BigInteger/Rational/Symbol/Str/List/Other, each asserting
.text unchanged).

**Memory budget — RESOLVED as a host-side pattern (user decision 2026-07-01); no new
core/host code.** The structural `current_memory_proxy_bytes` is an O(state) full sweep
(too costly to poll per step) AND blind to the interner `&'static` leak; RSS is the true
total but a platform syscall + threads that would break the WASM-clean core. Resolution:
impose the cap HOST-SIDE — a watchdog thread reads process RSS and reuses the brief-059
`cancel_handle()` to `.cancel()` on breach (eval returns `TerminatedEvaluation["Cancelled"]`).
This keeps `rum-core`/`rum-runtime` WASM-clean, gives the TRUE RSS bound (catches the
interner leak), is process-wide (right for single-session embedding), and is ALREADY
enabled by brief-059. Documented with a code example in
`docs/host-embedding-and-hardening.md` §2 Tier-1; a native-only `eval_with_memory_budget`
convenience wrapper in rum-host was considered and deliberately NOT added (keep rum-host
platform/thread-free). So the Tier-1 host-hardening line is essentially complete.

TIER-2 STARTED. brief-062 (`1f5b1b0`): make Session/HostSession `Send` + pin the
concurrency/isolation contract. Diagnosed HostSession was neither Send nor Sync; the only
Send blocker was the suspension-frame trait object `Box<dyn SuspensionFrame>` lacking
`+ Send` (Cell + FnMut additionally block Sync). Added `+ Send` to the frame trait object
+ stored-closure bounds (2 compiler-forced follow-on bounds in rewrite_rules.rs /
count_like.rs); ZERO `!Send` capture fixes (the runtime is Arc-based — every continuation
was already Send), no `unsafe impl Send`. Now Send (movable to a worker thread, proven by a
static assert + an 8-thread move/isolation test, i*i for i in 0..8 isolated despite the
shared interner Mutex), intentionally `!Sync` (single-threaded mutable state). Contract:
move between threads, never share; the interner is a shared Mutex (contention + shared name
table) but definitions are per-session. Gate: workspace 712/0, the 2 new tests, jupyter
stable 5/5 reruns (Codex's one transient ZeroMQ failure was environmental, not the Send
change — a zero-runtime marker-trait change, no microbench applies).

rum-jupyter upgrades UNLOCKED (surveyed; a dedicated post-Tier-2 brief). `interrupt` is
today a STUB (sets a flag, returns ok, does NOT stop a running eval) and the transport is a
single-threaded poll loop where a blocking `execute_request` prevents even RECEIVING the
interrupt. The hardening supplies the primitives: 059 `cancel_handle` (stop the eval), 062
`Send` (run execute on a worker so the loop stays responsive to Control), 060 panic
boundary (panic → error reply + restart-on-poison). Remaining = a jupyter restructure
(execute-on-worker + interrupt→cancel + poison-restart) → real interrupt + panic-resilient
kernel. Lower-value gaps: history/comm_info/debug stubs, typed display_data. See
`docs/host-embedding-and-hardening.md` §2 "Downstream".

brief-063 (`0bde8b7`): `HostSession::reset()` — rebuilds the inner session from the STORED
original HostConfig (a shared `build_inner_session` helper used by build() + reset()) and
clears `poisoned`. Not gated by poisoned — it is the RECOVERY path for a session poisoned
by a caught panic (brief-060). Drops all user state, invalidates prior cancel_handles
(new interrupt flag). Host-only. Gate: workspace 712/0 + rum-host 15/0; 3 tests (clear
state, poison→reset→usable, original-config restored not mutated).

brief-064 (`afad4de`): whole-session introspection. Runtime `definition_summaries()`
(iterate the definitions map → per-symbol name + has_own_value + down/up/sub counts, sorted)
+ `context_names()`; host `definitions()`/`contexts()` (pure `&self`, work on a poisoned
session). Native builtins excluded (only the definitions map is read). The enabler for a
session snapshot (065). Gate: workspace 712/0 + rum-host 17/0; test — `a=1; f[x_]:=; f[2]:=`
→ [Global`a own, Global`f down=2], builtin Plus absent, poisoned still introspectable.

brief-065 (`3d4a895`): session snapshot / restore through an in-memory buffer. Runtime
`snapshot_definitions_source()` renders all definition-map symbols (sorted) + attributes via
the existing `render_saved_symbols` (bumped to pub(super), signature unchanged) — no file;
host `snapshot_to_bytes()` (&self) → Vec<u8>, `restore_from_bytes()` → decode UTF-8 +
`reset()` + eval (faithful IMAGE load, replaces user state). NO filesystem touched — the
WASM/no-fs enabler. Captures definitions + attributes of defined symbols (not attribute-only
symbols / $-vars / options / messages / gensym-counter; TEXT image, binary .mx deferred).
Gate: workspace 712/0 + rum-host 18/0; round-trip composing with 064's definitions() —
snapshot A (a/f/g/h+Orderless) → 145 B → fresh B restore → definitions match, f[7]=7,
Attributes[h]=Orderless, pre-existing z=99 gone after restore.

**TIER-2 CORE COMPLETE** (062 Send / 063 reset / 064 introspection / 065 snapshot). Remaining
host work: the jupyter real-interrupt brief (execute-on-worker + interrupt→cancel +
poison-restart, all primitives now in place), or a showcase project. (Aside: brief-065
dispatch first hung on a codex self-update stdin prompt — see memory codex-update-hang.md;
killed + re-dispatched with the updated codex, clean.)

## 2026-07-01 Matcher backtracking stack overflow — depth guard shipped (brief-058)

A SECOND, distinct reachable crash (separate from the 2026-06-30 deep-nesting one):
a tiny `Orderless` + `BlankNullSequence` input stack-overflowed the host. E.g.
`SetAttributes[f,Orderless]; MatchQ[f@@Range[12], f[a___,b___,c___]/;False]` SIGABRT'd;
`Range[14]` with 2 sequence vars too. Found while picking up the deferred TODO
"pattern-matching axis (b)" (the user's question: how does WK handle the NP-hard
`Orderless`+sequence AC-matching blowup, and is it actually smart?).

DIAGNOSIS. The orderless backtracking (`pattern_matching/sequence/orderless/match_choices.rs`,
`collect.rs`) retries the next candidate via a recursive `index+1` call driven by
`drive_internal_value`, which in the normal path calls its continuation synchronously
on the same native stack (`suspension/core.rs:91`). Rust does not TCO it, so each
retry + each `rest` recursion adds an un-reclaimed clone-heavy frame; candidate count
per sequence-var position is ~combinatorial, so native stack DEPTH grows ~with the
WORK and ~`2^n` large frames overflow the 8 MB stack at n≈12 (3-seq) / 14 (2-seq).
The binding resource is stack depth, not a work count — `iteration_work_limit` (100k)
is far above the few-thousand frames that fit, so it never fires before the crash.
Non-Orderless (ordered) variable-length matching is polynomial and fine at n=30 — the
commutativity is the trigger.

WK PROBED (wolframscript, bounded n; the key half of the user's question). WK is NOT
smart here: it does naive `k^n` generate-and-test enumeration with NO pruning —
identical-element timing == distinct (no multiset dedup); a real var-dependent
subset-sum condition has the SAME exponential base as `/;False` (no condition-aware
pruning); `$RecursionLimit=50` doesn't trip its matcher (iterative / bounded native
stack). WK only beats rum because it degrades gracefully (slow, not crash) — an
implementation property, not an algorithm. (Caveat: WK `TimeConstrained` does NOT
reliably interrupt the C-level matcher.) See memory
`wk-ac-matching-is-naive-enumeration.md`.

FIX (brief-058, `a2d5ee6`). A `match_depth` counter + private `MAX_MATCH_DEPTH=512`
via a safe `with_match_depth_guard` wrapper (no unsafe; balanced inc/dec) at the
orderless choice/collect backtracking chokepoints. New
`EvalError::MatchRecursionLimitExceeded` wired with its own specific tag
(`General::matchreclim` + `TerminatedEvaluation["PatternMatchRecursionLimit"]`).
512 set empirically (adversarial depth 258 at n=8, 1026 at n=10 → 512 sits above the
small tractable cases, below the overflow). A depth guard is correct AND subsumes a
work counter here: depth≈work, so the first over-limit path returns an Err that
propagates and aborts the whole match fast (no `k^n` grind). NOT pursuing the perf
half (smart pruning / iterative rewrite): NP-hard, no general solution, WK lacks it,
HOL doesn't trigger it.

GATE (Claude): `cargo fmt --check`; `cargo test --workspace` 712/0; 5 CLI probes
(adversarial n=12/14/16 → TerminatedEvaluation not crash, normal matches → True);
`core-language-microbench` no regression (guard is OFF the common non-Orderless path);
mma-hol v8 cp42 heavy-matcher subset arith+num+rat+simp 818/0 + proptaut+meson 125/0
= 943/0, zero PatternMatchRecursionLimit leakage → 512 refuses no legitimate match.

AXIS (a) MEASURED + CLOSED (same line, no code). The TODO pattern-matching entry's
other half: rum's downvalue dispatch is a LINEAR scan O(total rules), not a
discrimination net — does the corpus have a long tail where indexing would pay?
Measured the per-symbol DownValues-count over the full cp42 tower (`Names["**"]` +
`Length[DownValues[#]]`): 1451 symbols carry downvalues, 1530 total, MAX 6 on any one
(`rnumMul`); histogram 1dv→1398 (96.3%), 2→41, 3→3, 4→5, 5→3, 6→1; only 12 symbols
have ≥3. NO long tail → a discrimination net / term index has nothing to optimize
(≤6 rules/symbol, cheap per-rule prefilter already); linear scan is optimal for HOL,
indexing would only add overhead. Caveat (general WL): thousands of rules on one
symbol would benefit — not rum's corpus. The whole pattern-matching line is now closed:
per-match cost already optimized (compiled matcher), axis (b) resilience fixed, axis
(a) a measured non-issue.

## 2026-06-30 Deep-nesting stack overflow — parser guard shipped; iterative Drop tried+rejected on perf (briefs 055-057)

A ~5000-deep nested literal crashed `rum-cli` with SIGABRT. Two unguarded recursers:
the recursive-descent parser and the default recursive `Drop` of the `Arc<Expr>` tree.

DONE (brief-055, `2a23d5e`): a `MAX_PARSE_DEPTH`=128 parser guard. Deep input now fails
gracefully (`parse nesting exceeded limit 128`) and the process survives; the mma-hol
corpus still loads under 128 (cp42 snapshot re-parse + num/rat/int/finite/fta 613/0,
arith 273/0). This closes every REACHABLE crash — WL paths are depth-capped by the
parser guard + eval `recursion_limit` 1024 + the resource limits (`Nest[ff,xx,60000]`
trips `General::reslim` before building a droppable deep tree, verified).

REJECTED (briefs 056/057): the iterative `Drop` — for a deep `Expr` built directly via
the rum-core Rust API, a host-only pathological path NOT reachable via WL — was tried
three ways: (1) unsafe `ManuallyDrop` (brief-055; also the FIRST `unsafe` in rum-core),
(2) safe `Option<Arc>` (brief-056, design B; `Option` branch on the hot `Deref` →
+8-44%), (3) safe `mem::replace`+sentinel (brief-057, design A; `Arc` field unchanged
so no per-access cost, but the per-drop drain → +22-49%, even with a `strong_count`
fast-path). Every design regressed the eval hot path 20-50% on list/dispatch labels
because every `Expr` drop pays the manual drain instead of the default `Arc` drop, and
no cheap trigger exists. Per the new standing microbench-gate rule (added to
TEMPLATE.md/AGENTS.md this session after brief-055's review), Codex measured + STOPPED
+ reported each time — none shipped. CONCLUSION: documented as a known host-API
limitation (`docs/host-embedding-and-hardening.md`) — a host must not build
pathologically-deep `Expr`s via the Rust API; the iterative drop's pathological-only
benefit does not justify a 20-50% eval-hot-path cost.

PROCESS: this is also where the two standing brief rules were added (no new `unsafe` in
kernel crates; hot-path changes need a microbench A/B) after brief-055 introduced unsafe
+ an unmeasured O(N) probe — root cause was an under-constrained brief, and the gate +
the new rules caught it.

## 2026-06-30 Memory line CONCLUDED for safe Rust — Phase 2 (interner) kept as a bounded tradeoff; resource-limit observability noted

Diagnosed Phase 2 (the per-gensym interned `&'static` name residual Phase 1 leaves).
MEASURED: Module×20000 (completing) ≈ 6.5 MB peak RSS vs 1.8 MB empty = ~235 B/gensym,
per-eval BOUNDED by `iteration_work_limit` (Module×200000 trips `General::reslim` →
`TerminatedEvaluation["ResourceLimit"]` gracefully), non-blocking (cold `run_all`
completes at 12 GB). Closing it needs either UNSAFE/perf-regressing interner reclamation
(A — `Symbol` is `Copy` + raw `&'static`, freeing = dangling; the safe fix adds
per-access/per-copy cost to the hottest type) or a prevent-at-source spike (B — Module
gensym → the existing `with_identity` mechanism; safe but with display + snapshot
`Names["*$*"]` + name-parse blast radius). Given a modest non-blocking residual vs real
risk on the foundational `Symbol`, **Phase 2 is concluded as a deliberate bounded
tradeoff** (the `&'static` interner buys an O(1) `Copy` `Symbol`). **The safe-Rust
memory line is CONCLUDED**: Phase 0/0.5/1/1b + Phase 3 rungs 1-3 done; the headline
Module-leak (6 MB → bounded) and dense-list factor (8.3x → ~4.0x) addressed; further
needs unsafe tagged-pointers or packed arrays (a poor fit for symbolic rationals-only
rum). See `docs/memory-management-plan.md`.

ALSO (an honest correction + a small finding): while measuring the above I first
mis-reported the `iteration_work_limit` abort as a "silent truncation" — it is NOT;
rum emits `General::reslim: evaluation exceeded iteration work limit (N)` AND returns a
graceful `TerminatedEvaluation`, the session continues. So the resource-limit failure
mode is already graceful + observable (ChatGPT's graceful-degradation dimension). The
one real refinement noted in `TODO.md`: the `TerminatedEvaluation` tag is generic
`"ResourceLimit"` for the work limit but specific `"IterationLimit"` for the pass limit
— make all limit tags specific so a user knows which budget to raise.

## 2026-06-30 Memory line Phase 3 rungs 2+3 — Integer small-int hybrid + Symbol niche-pack, `Expr` 40→32, dense-list 5.3x→4.0x; safe-Rust per-node compression CONCLUDED (brief-054)

Combined (neither moves the metric alone — `Integer` and `Symbol` are both 32 B and
co-cap the enum). `Atom::Integer(BigInt)` → `Atom::Integer(IntValue)` where
`IntValue = Small(i64) | Big(Box<BigInt>)` (INVARIANT: `Big` never holds an i64-fittable
value, so derived `Eq`/`Hash` are correct; custom `Ord` because variant order ≠ value
order; arithmetic still computes via `BigInt`, only STORAGE changes — low risk). Plus
`Symbol.identity: Option<u64>` → `Option<NonZeroU64>` (niche-packs 16→8 B; the gensym
counter never yields 0, audited).

RESULT: `size_of::<Expr>()` **40 → 32** (Atom 32, Symbol 32→24, IntValue 16). Dense-list
`retained` ratio **5.17–5.33x → 3.96–4.00x** (`rum_retained` 42370→32438 at N=1000).
`Range[100000]` **allocation count 201090 → 1087** (~200x — the per-element small-int
`BigInt` heap alloc is eliminated; bytes_total 5.99M→3.56M). TIME is a broad WIN, not
a cost: core microbench hot labels 8–18% faster (smaller Expr — `length-list` 0.82,
`if-true` 0.82, `sameq-integer` 0.83, `part-list` 0.84), integer probe ~13% faster,
rational neutral (+1.3%, within noise).

GATE (verbatim): `cargo fmt --check` clean; `cargo test --workspace` **710/0** + doc-tests;
Claude's mma-hol exact-arith corpus re-run on the v8 snapshot = **886/0** (num 198 / rat
177 / int 94 / finite 69 / fta 75 = 613, + arith 273 — all byte-identical to baseline
counts, so exact integer AND rational arithmetic is unchanged). 19-file change (the
`IntValue` type + custom serde/Ord in `expr.rs`, ~30 `Atom::Integer` match arms, the
`ByteCount` proxy, `Symbol.identity`).

**LINE CONCLUDED — safe-Rust per-node Expr compression is done.** Rungs 1+2+3 took
`size_of::<Expr>()` **64 → 32** and the dense-list factor **8.3x → ~4.0x**, while making
the kernel FASTER (smaller nodes) and slashing allocation traffic. Further per-node
shrink needs unsafe tagged-pointer / NaN-boxing (the unsafe deep water — out of scope).
The remaining safe lever, packed numeric arrays, was assessed as a poor fit (a
numeric-kernel technique vs rum's symbolic + rationals-only design; unpack perf-cliff on
the pattern-heavy path; highest blast radius) and DEFERRED. See
`docs/memory-management-plan.md` Phase 3.

## 2026-06-30 Memory line Phase 3 rung 1 — box `Atom::Rational`, `Expr` 64→40, dense-list ~8.3x→5.3x, FREE on time (brief-053)

First representation lever, gated as a measured design spike (brief-053). Boxing the
`Rational` variant (`Atom::Rational(Box<Rational>)`) drops `size_of::<Expr>()`
**64 → 40** — the `Atom` enum was sized by `Rational` (two inline 32-byte `BigInt`s);
boxing it leaves the cap at `BigInt`/`Symbol` = 32 + an 8-byte tag. Every `Vec<Expr>`
slot (the dominant memory cost of dense lists) shrinks accordingly.

GATE (verbatim, reproduced by me — the tradeoff measurement was the deliverable):
`cargo fmt --check` clean; `cargo test --workspace` **710/0** (rationals-only exact
arithmetic unchanged). MEMORY: `core-memory-microbench` `retain-range` `retained`
ratio **8.11/8.25/8.34x → 5.17/5.25/5.33x** (~1.57x; rum_retained 66538→42370 at
N=1000). TIME — the feared cost did NOT materialize: rum is rationals-only so boxing
adds a heap alloc + indirection per `Rational`, but the smaller `Expr` (cheaper
clone/move/`Vec`) offsets it — my own 8-run interleaved A/B on a rational-heavy probe
gave base median 2.105s vs new 2.069s (**ratio ~0.98, neutral/slightly faster**), and
the core microbench dispatch labels (`downvalue-*`, `matchq`, `replaceall`) were
neutral-to-faster (the new binary is even 358 KB smaller). 7-file mechanical change
(`expr.rs` enum + constructor, the `expr_memory_proxy_bytes` Rational arm, and 5
construction sites; matches auto-deref through the `Box`).

So: kept as Phase 3 rung 1. The ~8x is not closed — `Expr` is now tag-limited at 40;
the ladder forward is rung 2 (`Atom::Integer` small-int hybrid → caps Integer at 8 +
removes the per-element `BigInt` heap alloc), rung 3 (shrink `Symbol`'s 16-byte
`Option<u64> identity`) to reach `Expr` ~24→~16, then packed numeric arrays for the
real WK ~8 B/elem. See `docs/memory-management-plan.md` Phase 3.

## 2026-06-30 CORRECTNESS — the 2855-vs-3126 gap was a REAL bug: `Set[a,a]` self-fixed-point aborted `Get` mid-file (brief-051)

**Supersedes the "NOT failures, a counting reconciliation" framing in the
2026-06-29 cold-`run_all` entry below — that was WRONG.** The 271-assertion gap
between rum's cold `run_all` (2855) and the README's 3126 is (mostly) a genuine
correctness bug, not a counting artifact. Found by loading each silently-short
test file against the v8 snapshot and diffing per-file assertion counts vs WK.

ROOT CAUSE: `tests/arith_tests.wl` **silently aborted mid-file**. It ran only 41 of
140 `runTests` groups (50 of 271 assertions, no FAIL), then `Get` stopped — the
outer `Scan` over files continued, so the truncation was invisible. The abort line
is 399, `nnfConv = HOL\`Auto\`Arith\`nnfConv`: the runners add every PUBLIC `HOL`*`
context to `$ContextPath`, so the bare LHS `nnfConv` resolves to the SAME existing
symbol → `Set[s, s]`. rum raised a fatal `General::cycle` /
`EvaluationCycleDetected` on that benign self-fixed-point (ownvalue `a :> a`), where
WK returns the symbol unchanged. (The earlier `parseLin = …Private\`parseLin`
aliases are safe — Private contexts are off `$ContextPath`, so the LHS is a fresh
`Global` symbol, no self-reference.) Per-file `:::`-group-count vs static
`runTests`-count check across the WHOLE corpus: **arith is the ONLY truncated file**
(all 38 others run every group), so this one bug is **221 of the 271** gap.

FIX (`crates/rum-runtime/src/runtime/eval_core.rs` `evaluate_symbol`): when a
symbol's stored `own_value` / `delayed_own_value` RHS is structurally the symbol
itself, return it (fixed point) WITHOUT re-evaluating — the re-evaluation was what
re-entered the symbol on `evaluation_stack` and tripped the guard. Surgical; the
genuine-cycle path (`d := e; e := d`, RHS ≠ symbol) is untouched and still errors.
Three older tests (messages_part2 / context_and_loader / holding_and_control_part2)
that used `self := self` / `x := x` as a hard-error TRIGGER were re-pointed at a
genuine mutual cycle `loopA := loopB; loopB := loopA` (still `General::cycle`); new
regressions in definitions_and_rules lock the WK table (`a=a`→`a`, `b:=b`→`b`,
`s=s` keeps downvalues with `s[5]=6`).

GATE (verbatim): `cargo fmt --check` clean; `cargo test --workspace` green
(rum_runtime 710/0 incl. the 4 new regressions + 3 redirected cycle tests, all
other crates green); the 4 minimal rum-cli probes match WK; genuine cycle still
errors. **arith_tests.wl on v8 cp21: 50→271/0, 159 groups — byte-for-byte the WK
gold (WK measured 271/0/159).**

**GAP NOW FULLY CLOSED — rum reproduces the README's 3126/0 exactly.** Authoritative
fast confirmation (instead of the 113-min cold `run_all`): the real `tests/run_fast.wls`
(Stable mode) in a scratch acceptance clone with v8 `cp42` as `bootstrap.mx`, under
the brief-051 binary, gives **`passed: 3055, failed: 0`** over all 38 post-bootstrap
files (self-consistent: `$pass` 3055 == 3055 printed `ok` lines); `kernel_tests`
cold on the same binary is **71/0**. **3055 + 71 = 3126 = the README number, 0
failures.** Per printed-`ok` line, the ONLY file that changed vs the old cold run is
arith (50→271, +221). The earlier "residual ~50" was NOT separate content or
failures: the OLD cold `run_all` log itself printed 2905 `ok` lines but its summary
counted only 2855 (a 50 `$pass`-undercount in the old binary); the new run is
internally consistent, so that 50 evaporates. (run_all = cold/Strict from source,
run_fast = Stable snapshot restore — different load path, IDENTICAL test content, so
the run_fast number is the authoritative equivalent.) WK conformance note: a genuine
cycle in WK gives `TerminatedEvaluation[IterationLimit]`, not a hard error — rum's
`General::cycle` on real cycles is a separate, lower-priority conformance gap.

## 2026-06-30 v8 Stable snapshot tower rebuilt with the memory-fixed binary — byte-identical to v7

Rebuilt the full Stable-mode snapshot checkpoint chain with the post-memory-line
binary (briefs 048-050) →
`/Users/fft/Developments/rum-snapshot-checkpoints-v8/` (cp00 base Simp + 28 extend
steps to cp42 demos/Continuous.wl, all `ok`, ~68 min, cumulative 4076 s vs v7's
4112 s — GC adds no load-time cost). **v8 == v7 byte-for-byte across all 29
checkpoints** (size_bytes + num_contexts + num_names identical at every cpNN; e.g.
cp42 5,002,496 B / 57 contexts / 3773 names in both) — confirming the Module-gensym
GC is correctness-neutral at the snapshot level (same theorem construction, same
definitions). v8 is the new authoritative Stable snapshot; `rum-snapshot-checkpoints-latest`
now points at it.

NEW TOOLING: `scripts/rebuild-snapshot.sh` (`e95abd4`) — one-call HEAD-tagged
rebuild (builds the release binary, tags the output dir by git short HEAD, writes
`rum-snapshot-checkpoints-<HEAD>/`, repoints `…-latest`). Future rebuilds use it
(deterministic invoked tooling) instead of a remembered procedure; v8 was the last
hand-named chain.

## 2026-06-29 MILESTONE — authoritative cold `run_all` UNBLOCKED: completes 2855/0, bounded memory

The downstream validation of the memory line (Phases 0/1/1b) succeeded. The
single-process cold Strict `run_all.wls` from the acceptance-clone source
(`/Users/fft/Developments/mma-hol-acceptance`, HEAD `200f13a`), run under
`scripts/guarded-test-run.sh --rss-limit-gb 40`, **completed on its own (exit 0)**:
~113 min, **peak RSS 12.29 GB** (median 6.63 GB — load phase ~6.6 GB, post-bootstrap
heavy-prover test phase ~12.3 GB, plateaued, 100% CPU throughout, no swap-thrash).
Previously this was INFEASIBLE (pre-Phase-1 it OOM'd toward 50 GB; with Phase 1 only
it still hit 41 GB and was watchdog-killed at 25 min — the nested/recursive
definition leak). Phase 1b's mark-sweep bounded it.

All **39 test files** ran (1 pre-bootstrap `kernel_tests` + 38 post-bootstrap,
incl. the 4 heavy provers `meson`/`meson_proveprop`/`proptaut`/`simp` and the full
real-analysis tower) → **`passed: 2855, failed: 0`**. This EQUALS the pre-memory-line
clean count (run_fast 2784 + kernel_tests 71 = 2855), confirming the GC is
correctness-neutral while making the authoritative single-process gate feasible.
The watchdog also proved out (a 1 GB self-test killed cleanly in one poll interval).

OPEN (pre-existing, not a regression): the 2855-vs-README-3126 count. rum runs 2855
consistently (Stable run_fast AND cold Strict run_all); 3126 is a separate upstream
number already disputed in mma-hol `docs/dev/PLAN.md` (3124 vs 3126).
**CORRECTED 2026-06-30 (see the top entry): this was NOT just a counting
reconciliation — 221 of the 271 gap was a REAL correctness bug** (`arith_tests.wl`
silently aborted mid-file on the `Set[a,a]` self-fixed-point cycle error), now fixed
in brief-051. Only the remaining ~50 is the Stable-vs-Strict / disputed-count
reconciliation.

## 2026-06-29 Memory line Phase 1b DONE — root-set mark-sweep GC for nested/recursive modules, brief-050

Phase 1 (leaf-only) was too conservative for the real prover workload: cold
`run_all` STILL ballooned to 41 GB (watchdog-killed at 40 GB) during the cold tower
load, because that load is deeply nested/recursive Module/tactic code where Phase 1's
escape barrier marks the OUTER activation escaped on any inner-gensym write →
nested modules reclaim ~nothing. Measured: recursive `f[50]` grows `MemoryInUse`
linearly (~291 B/activation: 29 MB at 100k acts, 145 MB at 500k). RSS marginal
~1.3 KB/act (the proxy undercounts; the rest is real allocator cost of the SAME
definitions — so reclaiming the definition frees the bulk of RSS, interner residual
~100 B/act = Phase 2).

Implemented a threshold-triggered root-set mark-sweep (`scoping/gensym_gc.rs`):
escaped/retained gensyms go into a `gc_candidate_gensyms` registry at module exit;
when it crosses `RUM_GENSYM_GC_THRESHOLD` (default 200k), `collect_and_sweep_dead_gensyms`
marks every gensym reachable from the live root set and reclaims (via
`clear_symbol_state`) the unreachable ones. SOUND BY CONSTRUCTION (real reachability,
not a heuristic) → handles nesting/recursion. Three soundness pillars: (1) the MARK
is authoritative, the registry only bounds what's checked (so it is NOT in the
rollback snapshot — stale entry = no-op clear, missing entry = sound non-reclaim);
(2) ALL active-module gensyms (`module_activations`) are seeded live — this covers
gensyms captured in the un-walkable native suspend/resume continuation closures
(they belong to modules not yet finished → on the activation stack); (3) the root
walk `for_each_root_expr` covers every Expr-holding Session field (I audited it
against `current_memory_proxy_bytes` AND beyond — Codex correctly added
`dynamic_own_values`, which the proxy itself omits, and `ReapBucket.tag`; verified
`pending_eval_request` carries no Expr). `gensym_gc_extra_roots_for_outcome` also
roots Return/Throw payloads+tags. `in_gensym_sweep` guard stops the sweep's own
clears from re-tripping escape flags.

Gate (re-run by me): `cargo test --workspace` **1162/0** (+5 mark-sweep tests).
Bounded-memory PLATEAU confirmed (`RUM_GENSYM_GC_THRESHOLD=20000`): recursive
`f[50]` `MemoryInUse` delta = 5.3 MB / 3.0 MB / **498 B** at 100k / 500k / **2M**
activations — bounded by ~threshold, NOT linear (would be ~580 MB without the sweep).
All soundness smokes WK-correct incl. `RUM_GENSYM_GC_THRESHOLD=1` (sweep after EVERY
module exit, mid-recursion) → `f[50]=50` and Do-loop sums = 2870 identical to default
(the `Sum[i^2,…]` "unevaluated" result was a pre-existing rum Sum limitation,
threshold-independent — NOT a sweep bug). Comparator leaf/representation rows
unchanged (module-leak 293 B, retain-range 8.11x/8.25x). DOWNSTREAM VALIDATION IN
FLIGHT: re-running cold `run_all` under the watchdog with the mark-sweep to confirm
the cold load now stays bounded (was 41 GB-and-climbing); result to be logged.

## 2026-06-29 Memory line Phase 1 DONE — Module-gensym DEFINITION GC (the ~6 MB leak), brief-049

The dominant leak is FIXED. `Module` now reclaims its gensyms' definitions at
scope exit when provably safe, via a conservative per-activation escape barrier:
a `module_activations` stack (gensyms + `escaped` flag + a side-table size stamp);
the `escaped` flag is tripped by (a) `bump_evaluation_epoch_for_symbol` for any
write to a non-own-gensym symbol — the universal definition-write hook, covering
Set/SetDelayed/TagSet/UpSet/Unset/etc. — and (b) the options/syntax/message keyed
setters; the append-only channels (Once/Sow/Monitor/OptionValue/dynamic) are caught
by the side-table stamp comparison. At a clean (`Ok`, non-escaped) exit,
`finish_module_scope` retains the gensyms reachable from the result (transitive
closure over the activation's own gensyms via `definition_references_symbol`) and
reclaims the rest with `clear_symbol_state` (NOT `remove_symbol_state`, which would
just move the leak into `removed_symbols`). The activation lifetime is threaded
suspend-safely through `resume_module_initializers` mirroring `eval_block`
(`suspend_with_result_frame` + `resume_module_after_initializer` + `finish_module_scope`,
each handling `InputRequired`/`Err`). SOUND BY DESIGN: any escape (result, global
write, options/message/syntax, Sow/Once/Monitor, nesting, or an `Err`/`Throw`
outcome) → retain all = today's behavior. Only "leaf" modules using just their own
locals reclaim — exactly the dominant loop-local leak.

Two deviations from the brief, both accepted as correct refinements Codex found:
(1) `module_activations` IS added to the in-memory `EvaluationRollbackSnapshot`
(session_helpers.rs) — necessary so a `Check`/rollback keeps the activation stack
consistent with the rolled-back eval state (NOT the bootstrap.mx serde snapshot,
which is untouched); (2) Codex's stray `TODO.md` edit was reverted by me (out of
my brief's scope; TODO.md is mine).

Gate (verbatim, re-run by me): `cargo fmt --check` clean; `cargo test --workspace`
**1157/0** (+6 new soundness tests). Comparator headline: `module-leak-20000` rum
`MemoryInUse` delta **5,966,099 B → 293 B** (WK -80 B) — the ~6 MB definition leak
is gone. `retain-range-*` unchanged at 8.11x/8.25x (representation = Phase 3, not
touched). 7 soundness smokes + 5 complex HOL patterns (closures, recursion, nested,
global-def, fixpoint loop) all return the WK-correct value, never an `x$N`.
**Phase-2 residual IS visible and expected:** `module-leak-20000` rum peak-RSS
delta is still ~5 MB — the symbol-interner `&'static` name strings (`x$1..x$20000`)
are never freed (the proxy is blind to them; RSS sees them). That is exactly the
Phase 2 (interner reclamation) target. No hot-path regression (non-module
microbench labels — literal-integer/set-ownvalue/symbol-read — unaffected by
design via the `is_empty()` guard; the per-Module GC cost is the intended trade).
NEXT: Phase 2 interner reclamation (the ~5 MB RSS residual) or re-attempt the
authoritative cold `run_all` now that the definition leak is bounded.

## 2026-06-29 Memory line Phase 0 DONE — instruments (`MaxMemoryUsed[]` + cross-engine comparator), brief-048

Built the memory-line yardstick (the analog of `core-language-microbench.sh` for
the time line). (1) `MaxMemoryUsed[]` builtin: a `Cell<usize>`
`peak_memory_proxy_bytes` high-water mark on `Session` (non-serde, non-snapshot,
monotonic like WK's), bumped inside `current_memory_proxy_bytes` so every
`MemoryInUse`/`MaxMemoryUsed`/`MemoryConstrained` computation samples it — NO
default-path cost (the proxy is never on the hot eval path). 0-arg form only; the
1-arg peak-during-eval form is deferred (needs intra-eval sampling). (2)
`scripts/core-memory-microbench.sh`: runs each workload in a fresh process under
`/usr/bin/time -l`, same WL in rum + WK, reports per-label {`MemoryInUse` delta,
`MaxMemoryUsed`, peak-RSS delta} and the rum/WK ratio. Gate (verbatim): `cargo fmt
--check` clean, `cargo test --workspace` 1151/0 (incl. 4 new MaxMemoryUsed tests in
rum-runtime). Comparator REPRODUCES the plan's headline calibration (re-run by me,
deterministic proxy): `module-leak-20000` rum `MemoryInUse` delta **5,966,099 B
(~6 MB) vs WK -80 B**; `retain-range-1000`/`-10000` rum/WK ratio **8.11x / 8.25x**
(the ~8x dense boxed-Integer-vs-packed-array representation factor). RSS-delta is
noisy at this scale (OS page granularity) — proxy delta is the clean signal; RSS is
the secondary instrument that will see the interner `&'static` leak at large scale.
NEXT: Phase 1 (Module-gensym definition GC, the O(N)→O(1) win) now has a measured
before-baseline to gate against.

## 2026-06-29 Memory-management line opened (research + plan) — `docs/memory-management-plan.md`

User direction: replicate the time line's "WK parity to a constant factor" for
MEMORY. Research done (no code). Findings: (1) `Do[Module[{x},x=i^2;x],{i,1,20000}]`
grows `MemoryInUse` 6,002,561 B in rum vs 80 B in WK — the Module-gensym leak
(`eval_module` allocates fresh symbols, returns with NO scope-exit cleanup; cf.
`eval_block` which capture+restores). Two layers: definitions (reclaimable via
`definition_clear`) + interned name (`&'static`, append-only in core/expr.rs — needs
interner redesign). This is also the cold `run_all` 50 GB OOM cause. (2)
`ByteCount[Range[1000]]` 66 KB vs 8 KB (~8x dense boxed-Integer vs packed array).
(3) difftest Lane D has `--suite memory` but it is rum-LOCAL (no WK oracle);
`MemoryInUse`/`ByteCount` are deterministic proxies that see the definition leak but
are blind to the interner leak (RSS for the true total); `MaxMemoryUsed[]` is NOT
implemented. Plan: Phase 0 instruments (`MaxMemoryUsed[]` + cross-engine memory
comparator), Phase 1 sound Module-gensym definition GC, Phase 2 interner
reclamation, Phase 3 packed-array/hash-consing. Start at Phase 0.

## 2026-06-29 Observability + tooling: streaming Print (brief-047), resource watchdog, 7 stale-test fixes, gate tightened

Process work after the cold `run_all` OOM (above). All committed, workspace green.

- **`scripts/guarded-test-run.sh` (`9d460f7`)** — a deterministic resource watchdog:
  poll a guarded command's RSS / system-free / swap each interval, KILL it on
  breach (`--rss-limit-gb`, default 40). Built because the cold `run_all` thrashed
  to 50GB RSS for 2h undetected (only ~50% CPU was the tell). Reacts within one
  interval; an LLM poller is too slow for the OOM reflex. Tested both paths (clean
  completion no false-kill; low limit → BREACH+KILL in 1s). [[feedback-guard-heavy-test-runs]].
- **brief-047 streaming Print (`53ddba7`)** — a long `rum-cli --eval`/`--file` was a
  black box: `Print` is captured into `Session.prints` and emitted only after the
  top-level request returns, so the log is 0 bytes until exit (app-level capture,
  NOT OS buffering — `script`/`stdbuf` don't help; verified). Added an opt-in print
  sink on `Session`; `push_print` streams + skips accumulation when a sink is set
  (bounded memory); rum-cli registers a stdout-flushing sink. rum-host/jupyter set
  no sink → `outcome.prints` unchanged. Verified live: a 5-tick fib ticker showed 2
  ticks mid-run (was 0), 5 final. **`Pause` is NOT broken** — re-verified `Pause[3]`
  = 3.01s; the earlier "no-op" was a timing misread (a ticker had completed from
  elapsed wall-clock, not instant `Pause`).
- **7 stale syntax-error tests fixed (`c189b4d`)** — `f[1 2]`/`1 2` is valid
  juxtaposition (`Times`) since `67a3bdc`, so seven tests across rum-cli/host/
  jupyter/repl that used them as "syntax error"/"Invalid"/"Syntax failure"
  fixtures had been failing on `main`. Repointed to `f[1,]` (still invalid). These
  were PRE-EXISTING failures that `cargo test -p rum-runtime` (brief-046's gate)
  missed — only `--workspace` shows them.
- **Gate tightened (CLAUDE.md)** — the authoritative gate now DEFAULTS to
  `cargo test --workspace`; `-p rum-runtime` only when provably runtime-local. A
  green narrow gate is not a green tree.



The 50 open v7 batch-A failures (num 12 + rat 23 + set 15) were ONE rum bug, not
"shape rigidity". rum hard-holds the pattern argument of `MatchQ` (and the shared
`Cases`/`Count`/`Position`/`DeleteCases`/`ReplacePart`/`Catch`-tag resolver
`resolve_matcher_pattern_expr_owned`) and used an all-or-nothing gate: if the held
argument contained ANY pattern syntax anywhere, the WHOLE argument was returned
unevaluated. So ordinary subexpressions WK evaluates were skipped — the mma-hol
test helper `quantNestPat[q,n,body] := Nest[comb[const[q,_],abs[bvar[0,_],#,_]]&,
body,n]` never expanded (num/rat matched the literal, un-`Nest`ed
`quantNestPat[...]`), and the set tests' `const[pair[[2]], _]` kept `pair[[2]]` (a
`Part`) unevaluated so the const name never became `"IN"`.

- **Diagnosis (Claude)**: WK does NOT Hold the MatchQ pattern arg — it evaluates it
  normally and pattern leaves survive because they are inert. Confirmed against
  `wolframscript`: the minimal repros (`qq[2,_]` Nest helper, `f[{10,1}[[2]],_]`
  Part, `cc[pair[[2]],_]`) are all True in WK, False in rum. **[[diagnostic-tool-trust]]
  bit twice this session** — I used `MatchQ` (the buggy subject) as the probe and
  hit two ARTIFACTS that looked like a second bug: (a) `"\:2200"` through
  shell→`rum-cli` became literal `":2200"` not `∀`; (b) an `ab = HOL\`Terms\`abs`
  ownvalue indirection stayed unevaluated in the held pattern (the same bug firing
  on my probe). The user re-flagged the circular-instrument trap; calibrating every
  expected value against plain WK settled it. Also verified rum's `Pattern` is NOT
  HoldFirst (`x=5; FullForm[x_]` → `Pattern[5,_]`), so the fix must keep pattern
  constructs held, not blanket-evaluate.
- **Fix (Codex brief-046, gated+committed `7b0e4cd`)**: `resolve_matcher_pattern_expr_owned`
  recurses — pattern-syntax-free subtrees evaluate; `matches_pattern_head` nodes
  stay held (protect bound names / held tests); ordinary-headed nodes with pattern
  args are rebuilt and dispatched via `resume_call_from_prepared_args` so the head
  fires (expands `quantNestPat`) WITHOUT re-evaluating the resolved pattern leaves.
  One chokepoint fixes the whole matcher family. `_+_` now → `Times[2,Blank[]]`
  (WK-conformant; mma-hol uses inert ADT constructors, unaffected).
- **Gate (Claude, all verbatim)**: `cargo fmt --check` clean, `cargo test -p
  rum-runtime` 692/0; batch-A 50→0 — set 189/0 (all 10 defining-equation + 5 lambda
  now `ok`), num+rat 375/0; NO regression — real-tower 1122/0, remaining non-heavy
  batch-A 1063/0. Run against the existing v7 cp42 snapshot with the new binary
  (the fix is call-time MatchQ, not store-time theorem shape, so no snapshot
  rebuild needed for the test results).
- **batch-B heavy provers on v7+046 (first-ever heavy run on v7) — ALL GREEN, no
  pre-existing failures remain**: meson_tests 44/0 (122s), proptaut_tests 81/0
  (465s), simp_tests 46/0 (109s), **meson_proveprop_tests 69/0 (836s) — this had
  historically TIMED OUT at 300s without a summary; it now COMPLETES green**.
  batch-B total 240/0. Timing: proptaut /
  meson_proveprop are slow (no clean v7+045 baseline to A/B against — heavy provers
  were never run on v7 before; simp is FASTER than the old 208s, and
  meson_proveprop went from timeout→complete), so any 046-vs-045 perf delta is
  unmeasured and belongs to the deferred perf line, not a correctness concern.
- **CLEAN count reconciliation (corrects an over-claim — the batch-A/B subtotals
  above were OVERLAPPING pattern runs, they DO NOT sum to a corpus total).**
  Authoritative single `run_fast` (all 38 post-bootstrap files, Stable from the v7
  cp42 snapshot, no overlap) = **2784/0**; `kernel_tests` run cold (small bounded
  load, run_fast skips it) = **71/0**; total verified on v7+046 = **2855/0, zero
  failures**. The README's `3126/0` is a DIFFERENT runner — cold Strict
  `run_all.wls` from source, +kernel_tests — and is internally disputed in mma-hol's
  own `docs/dev/PLAN.md` (README 3126 vs graduation commit 3124, flagged "needs
  re-verification"). The ~271 residual (3126−2855) is Stable-vs-Strict mode
  difference + the un-reproduced cold count, NOT failures.
- **NEW FINDING — single-process cold `run_all.wls` is INFEASIBLE on this 64GB
  machine (explains why the cold Strict authoritative gate has been DEFERRED all
  along).** Launched it; at 2h20m it had ballooned to **50.7 GB RSS** (79% of 64GB),
  exhausted RAM + 27GB swap (931 pages free), and was swap-THRASHING (~120M
  swapins) — which is why it showed only ~50% CPU (half its time in page-fault I/O,
  not compute). Root cause: rum's unbounded Module-gensym live growth (the deferred
  GC; far worse than the recorded "~10GB" at full-corpus single-process scale).
  Killed it. The authoritative cold 3126-comparable number needs either the
  Module-gensym GC or a memory-bounded segmented cold pass (the checkpoint chain
  bounds the LOAD, but upper-tower TEST execution still needs the whole tower
  resident → same wall). [[snapshot-checkpointing-direction]].

## 2026-06-24 Power/Times WK-conformance line (Codex briefs 041-045) + attribute-axis fixes + v7 cold-rebuild baseline

All via Codex briefs, each gated by me (cargo test + WK probes + real-tower 1122/0).

- **brief-041** (`b7840a7`): attribute conformance — dropped FullForm's spurious
  `HoldAll` (`FullForm[1+1]` now `2`, not `Plus[1,1]`); added `Listable` to 12
  number-theory builtins (Abs/Round/Factorial/Factorial2/Denominator/Numerator/
  Mod/Quotient/PowerMod/GCD/LCM/CoprimeQ). KEY: the scan's "Evaluate is broken"
  was a MISDIAGNOSIS — a FullForm artifact (`Hold[Evaluate[1+1]]===Hold[2]` is
  True); Evaluate untouched. The ~32 EXTRA-Hold attrs on control symbols are
  BENIGN (rum hardcodes hold in the evaluator), verified behaviorally
  (`ReleaseHold[1+1]`=2, `Catch[Throw[1,1+1],1+1]`=1). [[diagnostic-tool-trust]].
- **brief-042** (`ca2dcff`): `Listable` on `Power`.
- **brief-043+044** (`56c860c`): symbolic Plus/Times like-term/like-base
  combination (numeric scope) — `x^2 x^3->x^5`, `x-x->0`, `x^3/x^2->x`,
  `2 a 3 a->6 a^2`. (A) `eval_power` nested-power flatten, (B) `combine_mult`
  like-base, (C) `combine_add` like-term. 044 = perf fast path (no-merge skip +
  direct `Times[coeff,...]` build, no recursive combine): symbolic `Plus`
  2.7x->1.96x, `Times` 2.0x->1.75x. Remaining O(n^2) like-term DETECTION is an
  ACCEPTED tradeoff — real tower (small term counts) shows NO macro regression
  (extend even slightly faster; combination cuts downstream term counts). Deferred
  lever in TODO "Performance And Representation".
- **brief-045** (`cec4bc7`): Power-over-Times distribution (numeric exponent) —
  `(2x)^2->4x^2`, `(a b)^2->a^2 b^2`, `(a/b)^2->a^2/b^2` (reuses (A)). Numeric `c`
  lifts for ANY numeric `n`; symbolic factors distribute only for INTEGER `n`
  (`(a b)^(1/2)` stays `Sqrt[a b]`); rationals-only keeps `4^(1/2)` un-rooted.
- **v7 cold rebuild + authoritative batch-A** (`b7e6eac`): full chain cold-built
  with the 045 binary -> `rum-snapshot-checkpoints-v7/` (cp00..cp42, ~68min, times
  ≈ v6 — no macro perf regression). v7 SUPERSEDES v6 (older binaries bake stale
  theorem shapes). Authoritative non-heavy batch-A = **2440/50**: num 12 + rat 23
  arithmetic SHAPE fails, set 15 (10 defining-equation + 5 lambda); real_arith now
  0. The original 2026-06-23 "50 all shape" was a DIFFERENT 50 — 045 changed shapes
  via the arith/rat decision procedures. LESSON: after any kernel/eval change,
  COLD-rebuild before trusting a recheck — a mixed v6 snapshot masked the arith/num
  fails. NEXT: diagnose the 50 (35 arith/rat shape via matcher inline-vs-prebound;
  15 set defeq/lambda).

## 2026-06-23 (LATE) Real tower 100% (41/41) + Continuous demo loads — 4 more bugs (#6-#9) via Codex briefs 032-035; two-axis conformance method

Continued the correctness mainline from the 88%/Real-Seq wall (block below) to the
BOTTOM. **The full ℤ→ℚ→ℝ tower now loads 41/41 = 100%** (`Seq, SeqAux, Compact,
Connected, Topology, CompactSet` all clean), and **`demos/Continuous.wl`** (the
next-chapter continuous-functions demo, OUTSIDE the tower — IVT/EVT/compact-image
capstones) **also loads and its theorems verify**. Driver: fast Stable
extend-on-snapshot from v5 `cp34` (re-extend RealArith with the current binary,
then extend each module) — seconds-to-minutes per module vs a full cold rebuild.

**4 bugs fixed (each a Codex brief I diagnosed + gated + committed):**
- **#6 `Numerator` (`33f1795`, brief-032):** rum had `Denominator` but not
  `Numerator`; bare `Numerator[1/2]` resolved to a non-System context and stayed
  inert, so RealArith's `positiveScalarTerm[1/2]` built `rnumTm[Private`Numerator[
  1/2]]` and `mkComb` raised `argument type mismatch` (fun/real) at Real/Seq.
  Root-caused by a two-engine diff (brief-031 diagnostic): WK had every
  intermediate `real`-typed, rum diverged at `Numerator`. Mirror of
  `eval_denominator`, Integer arm returns itself.
- **#7 `Max`/`Min` (`f2d3906`, brief-033):** missing variadic numeric builtins,
  found PROACTIVELY by the symbol-axis scan (used in `RealArith` `rnumNe` +
  `CompactSet`), not yet bitten. Mirror `eval_gcd`: flatten lists, compare exact
  Integer/Rational, empty→∓Infinity.
- **#8 2-arg `Sort[list, p]` (`dca37c4`, brief-034):** the ordering-function form
  was a no-op stub (`[expr, _ordering]` returned the call inert), so
  `varKeysSorted = Sort[Keys[vs], Order[#1,#2]===1 &]` stayed unevaluated, a
  downstream `Map` distributed over the inert `Sort`, the real linear form built
  malformed → `SPEC: term type does not match binder` / `Missing[KeyAbsent]` at
  Seq. Implemented the GENERAL ordering function (resumable per-pair comparator
  eval, mirror of `resume_sort_by_from_index`; stable), NOT a canonical-order
  special-case — mma-hol is the stress test, not the target ([[prefer-general-wl-semantics]]).
- **#9 `//` postfix operator (`49b081b`, brief-035):** rum's parser had the whole
  apply family (`@ @@ @@@ /@ @* /*`) but not `//`; `x // f` failed `expected
  expression, found /`. mma-hol's stdlib is explicit-`f[x]` style and never used
  `//`, so it slipped corpus-driven testing until `demos/Continuous.wl:516`
  (notebook-pipe style) hit it. Added the `//` token + `BinaryOp::PostfixApply`
  (precedence 15, between Set and ReplaceAll; left-assoc) + reversed lowering
  `x // f → Expr::call(rhs, [lhs])`.

**Method (record for reuse) — TWO conformance axes + demo stress test:**
- SYMBOL axis: mma-hol call-form System builtins (perl word-boundary extract, NOT
  `[A-Z]\w+\[` which catches camelCase suffixes like `tyArrow[`→"Arrow") ∩ WK
  `Names["System`*"]` − rum `Names["System`*"]` → complete missing-builtin set was
  exactly `{Max, Min}`.
- SYNTAX axis: WL operator inventory → rum parser (`FullForm[Hold[…]]`) → complete
  missing-operator inventory: `~f~`, `n!`/`!!`, `a..`/`...`, `x_.`, `^=`/`^:=`,
  `|->`, `~~`, `f'`, `a.b` Dot, `**`, `//@`, `%`/`>>`/`<<`. Functional forms of
  `! ' . %` exist; interactive ones irrelevant to loading. Fix on demand (no more
  surprises). (Watch the `|`-as-IFS-delimiter trap: `|` `||` `<|…|>` are supported,
  a harness artifact mis-flagged them.)
- Bug TAXONOMY: "missing builtin" (Nothing/Numerator/Max/Min) → symbol scan
  catches; "present-but-incomplete" (2-arg Sort) + "missing syntax" (`//`) → scan
  does NOT catch, need demo stress + patch-and-trace.

**Authoritative gate IN PROGRESS:** the frontier was driven with fast Stable
extend (correct iteration tool, NOT authoritative — Stable-vs-Strict gap + lower
cp00–cp34 built with the OLD binary). Launched the full-chain COLD rebuild →
`rum-snapshot-checkpoints-v6/` (`scripts/snapshot-checkpoint-chain.sh`, log
`/tmp/cold_rebuild_v6.log`) as the Strict authoritative gate (also settles
lower-checkpoint taint; risk LOW since the 4 bugs are call-time not store-time,
but unverified). v6 supersedes v5 once green.

## 2026-06-23 Correctness mainline: checkpoint chain drives the mma-hol load frontier — 3 parser/lexer bugs fixed, number-tower foundation now loads (stuck at Int)

Goal (user): push the unmodified-`mma-hol` load all the way to "the bottom" —
either the last theorem verifies, or a hard time/space wall on this machine.
Driver: `scripts/snapshot-checkpoint-chain.sh` (brief-025) builds a named
per-module snapshot (`cpNN-<module>.mx`) + profile + manifest, stopping at the
first failing module. Checkpoints in `rum-snapshot-checkpoints-v4/` (v4 = with
all three fixes below; cp00-cp20 are byte-identical to the pre-fix v1/v2 runs —
the early libraries were never affected).

**Harness blind-spot fixed first (`ca6312f`):** the chain classified 15
silently-failing libraries as "ok" and "completed all 27" because the real
errors are `! Get::syntax` / `! Get::load: uncaught throw: Failure[HOLError...]`
on STDERR while `extend_snapshot.wls` swallows them and exits 0 with "ok" on
stdout. `classify_step` now greps stderr for `uncaught throw|Get::syntax|
Get::load|holErrorTag|^! ` and stops at the first real failure (user flagged
this; "盯报错就停").

**Three parser/lexer bugs, each found by a rebuild and advancing the frontier
deeper into the arithmetic prover (`auto/Arith.wl`'s Cooper/Presburger +
Farkas/Fourier-Motzkin engine):**
1. **Juxtaposition (`67a3bdc`, brief-026):** rum had no implicit multiplication;
   `a b`/`2 x`/`g #` failed to parse. `Arith.wl`'s `Map[g # &, a]` threw
   `Get::syntax`, so Arith defined NOTHING, `arithProve` was undefined, and every
   `arithProve[...]` returned inert -> `concl: not a theorem`. Fix unblocked
   Arith + List.
2. **`#` slot adjacency (`df4b368`, brief-027):** `parse_slot` consumed a
   whitespace-separated token as the slot spec, so `# lcm` -> `Slot["lcm"]`
   (named) not `Times[#, lcm]`. `clearProvToInt`'s `Map[(# lcm) &, prov]` then
   produced a garbage Farkas certificate -> `farkasFM: certificate is not a
   positive constant` at Finite. Fix: require `leading_trivia.is_empty()`.
3. **`\[Name]` named characters (`9bcf231`, brief-028):** rum decoded no
   `\[Name]` escapes; `mkConst["\[Exists]", ...]` built a constant named
   `[Exists]` instead of `∃` -> `constType: unknown constant [Exists]` at Finite.
   Fix: a 128-entry table (authoritative from WK, incl. quirks `\[Phi]`=U+03D5,
   `\[Epsilon]`=U+03F5; Greek lower/upper + Curly variants + common logic/set/
   arrow symbols), decoded in both string and code contexts. (Side gap noted:
   `FromCharacterCode` is not a rum builtin yet — symbol-axis.)

**Frontier now (v4):** `core → … → Pair → Sum → Option → Set → Num → Arith →
List → Finite → FTA` all load clean; **stuck at `stdlib/Int.wl` (index 25)** with
`arith-farkas: no Farkas certificate` (then `concl: not a theorem`), 22s fast
fail in `proveConclUnderHyps`. Constants are CLEAN now (`const["≤"/"+"/"FST"/
"SND"/"SUC"]`), so this is NOT another surface bug — it is the first
algorithm-level frontier (the Farkas search finds no certificate for an ℕ
linear-arithmetic goal with SUC/FST/SND that WK proves). Per-library load times
(heavy): Set 395s, Finite 574s, Sum 277s, Pair 208s; the Real tower is downstream
of Int. Methodology lesson (user, blast-radius): to test a fix, restore the
checkpoint just before the EARLIEST file the fix affects and extend forward — the
full rebuild is only needed when the fix taints early checkpoints (see
[[feedback-test-fix-from-checkpoint]]); the systematic conformance/difftest
complement is [[coverage-conformance-direction]].

**4th bug — Function ignores extra args (`7d04aab`, brief-029), the FIRST
non-syntax / eval-semantics one, and the highest-leverage:** the Int
`no Farkas certificate` was NOT a farkasFM bug — the 7 `diffs` genuinely have no
rational certificate without ℕ non-negativity (verified by hand: `d0 + d6 +
(atom$3≥0) = 1 ≤ 0`). `auto/Arith.wl`'s `refuteWithNonneg = Catch[farkasRefute[
facts], holErrorTag, Function[err, <add non-negativity, retry>]]` is supposed to
catch the first failure and retry WITH non-negativity, but `try_apply_function`
(scoping/callable.rs) required EXACT arity and left a named-param `Function`
UNEVALUATED when given extra args — and WL's `Catch[e, tag, handler]` invokes
`handler[value, tag]` (2 args) on the 1-param handler. So the retry never ran.
Fix: error only when `args.len() < params.len()`; extras are ignored (WL
semantics). This breaks the standard `Catch[e, tag, handler]` idiom broadly, not
just here.

**Frontier now (v5, with all 4 fixes):** `core → … → Int → Rat → Real/{Cut,
RatAux, Field, Mul, Inv, Complete, Abs, MinMax} → auto/RealArith` ALL load clean —
the FULL ℤ→ℚ→ℝ number tower + real-number construction. **36 of 41 modules =
88%.** Int now loads in 526s (was a 22s fast-fail), Real/Mul 213s; the rest are
fast. **Stuck at `stdlib/Real/Seq.wl` (index 36)**, 1s fast fail:
`realarith-norm: term is not a real linear expression` from
`HOL`Auto`RealArith`parseLinRAux` — `typeOf[t] =!= realTy`. Diagnosed (patched
`parseLinRAux` on the cp35 checkpoint): the rejected `t` is an UNBOUND Module-local
gensym (`Head[t] = Symbol`, `leftTerm$NNN`), not a HOL term, so `typeOf[t]` stays
unevaluated. So this is a Module-local binding / scoping bug (the local isn't
carrying its value when passed to `realLinNormConv`) — the second scoping-area
frontier, murkier than the surface ones; needs a clean minimal repro (the
monkey-patch + dynamic-own-value scope interaction is unreliable). Remaining after
Seq: SeqAux, Compact, Connected, Topology, CompactSet (the real-analysis topology
layer). Authoritative checkpoints: `rum-snapshot-checkpoints-v5/`.

**Root of the Real/Seq frontier was a missing builtin — `Nothing` (5th bug,
`35a6a82`, brief-030).** `auto/RealArith.wl`'s `negativePart` maps unwanted
entries to `Nothing` and lets them vanish from the result list/association; rum
had no `Nothing` (it resolved to `Global``, was kept in lists), so a real linear
form came out malformed -> `rLinIsZero[negRec]` non-Boolean -> the
`If[rLinIsZero[negRec], ...]` took neither branch -> Module local `leftTerm`
UNBOUND -> `realarith-norm`. Fix: register `Nothing` as System + drop it from
direct `List` elements (List-only). BLAST-RADIUS lesson re-applied: `Nothing` is
used (stored) only in `auto/RealArith.wl` (1 use), so the incremental test had to
restore cp34 (Real/MinMax) and RE-extend RealArith with the new binary (the cp35
snapshot baked the OLD `Private`Nothing` into negativePart's stored DownValue) —
an old checkpoint that stores a definition using the changed symbol is tainted.
**That validated the fix and revealed the NEXT Real/Seq sub-frontier:** `mkComb:
argument type mismatch` (tag `term`) in `proveRealConclUnderFacts ->
realLinNormConv -> normLinRAux -> rMulComm -> buildLinRAny` (types `fun`/`real`) —
a deeper real-arithmetic proof-reconstruction issue. So Real/Seq is multi-layered;
fast iteration base = cp34 + re-extended RealArith. Total rum bugs fixed this
push: 5 (juxtaposition, slot-adjacency, `\[Name]`, Function extra-args, `Nothing`)
+ the harness/preamble infra; the load reaches the full ℤ→ℚ→ℝ tower + real
construction (88%).

## 2026-06-23 Snapshot reliability harness GREEN — BUG-MSG fixed + deterministic preamble; three round-trip bugs closed

Completed the snapshot-reliability line. The brief-023 harness
(`scripts/snapshot-reliability.sh`) cycles `restore -> re-dump` N times per fast
layer asserting byte-identity (guards BUG-RT growth), test stability, gensym=0
(guards #2), and cold-vs-incremental fidelity. It immediately earned its keep by
catching a THIRD round-trip bug:

**BUG-MSG (fixed, commit `2fa0749`, brief-024):** restore+redump silently dropped
~all theorem `::usage` strings. Root cause: `normalize_definition_lhs_expr`
(definitions.rs) special-cases Pattern/Blank/Condition/HoldPattern to preserve
their operands but had NO `MessageName` case, so the general Call arm resolved the
target symbol of `sym::tag = value` to its OwnValue — `MessageName[<value>, tag]`
can't form a key, so `define_message` was never called. Order-dependent: works
only if usage is set BEFORE the symbol's value. DumpSave emits value-before-usage,
so Get-restore (value first) hit the bug. Minimal repro: `a=5; a::usage="d";
Messages[a]` was `{}` (should be `{a::usage}`). Fix: one MessageName arm mirroring
HoldPattern. Diagnosed entirely from the harness failure + a value-vs-usage order
probe.

**Deterministic preamble (fix in package_tools.rs):** after BUG-MSG, the only
residual cold-vs-cyc1 diff was the auto-generated package-restore preamble: a cold
dump enumerates `loaded_packages` in LOAD order, but a restored session
reconstructs them SORTED (the preamble's own `Union`), so the first restore+redump
reordered the two `$ContextPath`/`$Packages` lines (same contexts, same size).
`dump_save_package_restore_contexts` now `contexts.sort()`s, making snapshots
byte-reproducible across the round-trip. Safe: restored worlds already used sorted
order (pair_tests green from restored snapshots).

**Harness now GREEN** (all 5 fast layers, `RUM_CLI_BIN` release): every layer
`gensym_count=0`, `cycle_byte_stable=yes` (Bool 177621, Drule 218422, Tactics
242758, Printer 262039, Parser 292656 — identical across 3 cycles), tests stable
(76/50/49/48/78, all /0), fidelity ok; pristine clone untouched.

**Net — three round-trip bugs found by reliability cycling, all fixed:** BUG-RT
(negative-literal growth, `20458b9`), #2 (gensym dump bloat 402MB->504KB,
`ae7e92d`), BUG-MSG (usage loss, `2fa0749`), plus deterministic preamble. The
DumpSave/Get snapshot round-trip is now byte-idempotent and fidelity-preserving on
the fast layers, with a standing regression guard. codex's old #1 ($Packages
restore) was already fixed; #2 was not (now is). The live Module-gensym leak
(locals never GC'd in-session) remains a separate, deferred evaluator issue.
This unblocks the resumable Pair-checkpoint workflow (pay Pair's ~190s once into a
trustworthy, backupable snapshot, then resume downstream).

## 2026-06-22 Snapshot reliability test (dimension A) — surfaced a real reader bug (BUG-RT) + confirmed #2 hygiene is unfixed

Per the user's direction (build a resumable snapshot-checkpoint workflow for
mma-hol loading; FIRST investigate codex's prior snapshot work + design a
reliability test), ran dimension-A reliability cycling (load→save→restore→
re-dump→test, repeated) on the FAST layers (scratch copy of `200f13a`,
pristine clone untouched).

**Investigation of codex's prior snapshot work (git + Task 2e):** two problems
were hit. **#1 `$Packages` restore** (after `Get`, `BeginPackage` deps didn't
see restored packages; was once worked around by a HARD-LINE-VIOLATING edit to
the standard's `extend_snapshot.wls`, reverted) — **really FIXED** (commit
`2d901ea` + 2026-06-13 runtime auto-registration; re-verified on 200f13a:
extend-proptaut green). **#2 Module-locals 402MB bloat** (dumping
`HOL`Auto`Simp`Private`` includes generated `baseConv$…`/`xEq$…`) — **NOT
fixed**: the only "fix" was a WL scratch experiment (`Scan[Remove,
Names["HOL`*$*"]]` before dump), absent from both rum and the unmodified
harness. Code (`symbols_in_context`, package_tools.rs:284 — context-level
filter only, no `$` exclusion) AND an empirical dump (`gen$1`/`gen$2` both
serialized) confirm #2 is a LIVE bug. Cycle data shows it is pervasive, not
Pair-only: even the Parser snapshot has 13277 generated `$`-locals of 13576
names (98%).

**Cycle results (Parser layer, 5× restore→redump):** context count (21) and
name count (13576) constant — no symbol-count leak; parser tests stay `78/0`
after the cycles — semantics preserved. BUT the snapshot grew **+80 bytes per
cycle, monotonically** (12193843→12194163). Logical state stable + bytes
growing = a non-idempotent round-trip.

**Root cause (BUG-RT, a reader/parser bug — minimal repro):**
`FullForm[Hold[-1]]` → `Hold[Times[-1, 1]]` (should be `Hold[-1]`);
`FullForm[Hold[Times[-1,b]]]` → `Hold[Times[Times[-1,1],b]]`;
`-3` → `Times[-1,3]`; positive `Times[2,b]` is immune. **rum's reader parses a
negative integer literal `-n` as `Times[-1, |n|]` (unary-minus-of-positive),
not an atomic negative integer.** Evaluation always collapses `Times[-1,1]→-1`,
so it is invisible in normal use; but in HELD contexts (SetDelayed RHS,
DumpSave/Get) it is preserved, and since the serialized `Times[-1, b]` re-parses
to `Times[Times[-1,1], b]`, every snapshot round-trip wraps the `-1` one level
deeper → unbounded growth + representation rot. DumpSave's serializer is
FAITHFUL (atomic `-1` from infix `x-b` dumps clean `Times[-1,b]`); the wrap is
introduced by the READER. Distinguished: `a1:=x-b` (infix, atomic -1) dumps
clean then rots after Get; `a2:=Times[-1,b]` (explicit literal) is wrapped on
the FIRST dump (parsed from input); `a3:=-b` (unary on a symbol) is clean.
**Latent correctness risk** beyond cosmetics: a pattern `Times[-1, x_]` will not
match the rotted `Times[Times[-1,1], x]`. Fix locus: the reader/lexer — a
negative numeric literal in operand position must tokenize as an atomic negative
number so FullForm/DumpSave round-trips are idempotent (WL round-trips). Care:
keep infix `a - 1` = `Plus[a, -1]` working; inside FullForm `Times[-1, b]` the
`-1` after `[`/`,` is unambiguously a literal.

**Verdict:** dimension-A cycling did exactly its job — the mechanism "looked
fine" (tests green) but is non-idempotent (BUG-RT) and dumps junk locals (#2).
Both must be fixed before the segment-by-segment checkpoint workflow is
trustworthy. Next: codex briefs for (a) BUG-RT reader fix, (b) #2 dump hygiene,
(c) the reliability-test harness (dimension A loop + dimension B assertion).

## 2026-06-22 Correctness-mainline pivot — first boundary on acceptance standard 200f13a IS the Pair.wl load wall (~5x rum defect, blocks the whole downstream stdlib)

Per the user's direction decision, pivoted from the (diminishing-returns)
performance line to the v0.1 correctness/stability mainline: run the UNMODIFIED
acceptance standard `/Users/fft/Developments/mma-hol-acceptance` (HEAD `200f13a`,
the full completed project — 61 smoke stages, far past the 2026-06-13 validation
that only reached Parser/Printer+proptaut on the older `51bbe9e`) through
`scripts/mma-hol-bounded-smoke.sh --mode loads` (release `rum-cli`, 180s/stage,
stop-at-first-break) to find the current first failing/timing-out boundary.

**Result — clean and decisive:** stages 1-14 (`ErrorUtil` … `auto/Simp`) each
load in ≤1s; **`load/stdlib/Pair` TIMES OUT at >180s (the first boundary, a
PERFORMANCE wall, NOT a correctness error)**. The cumulative prefix through
`auto/Simp` is ~1s, so the >180s is essentially Pair.wl load itself — i.e.
`Get["stdlib/Pair.wl"]` runs the heavy Simp descent (proving the Pair theory at
load time) and never finishes inside 180s. This BLOCKS the entire
never-validated downstream stdlib frontier (Sum, Option, Set, Num, Arith, List,
Finite, FTA, Int, Rat, Real/*) — we cannot even SEE whether those have
correctness issues because Pair never loads.

**Two-engine comparison on the SAME 200f13a clone (authoritative, current
standard):** `scripts/pair-simp-hotspot-probes.sh --only pair-load-root-phase
--engine wolfram` (WolframKernel `/Applications/Wolfram.app/Contents/MacOS/
WolframKernel`): internal `root-get-stdlib-pair = 38.56s`, preload-simp-stack
`0.22s`, and it PROVES the theory (`pair_mkPairInjThm_is_thm=True`,
`pair_pairInjThm_is_thm=True`). The matching rum run (`--engine rum
--timeout 400` on the SAME 200f13a clone) COMPLETES at `root-get-stdlib-pair =
190.12s` and proves the IDENTICAL theorems True (`pair_mkPairInjThm_is_thm=True`,
`pair_pairInjThm_is_thm=True`). So rum is **slow-but-CORRECT** here: not a
non-termination bug, not a correctness divergence — a pure performance wall.
**rum/WK = 190.12 / 38.56 ≈ 4.93x on the exact critical-path blocker** — WK does
the identical logical work (same unmodified Pair.wl, same proved theorems) in
38.6s. This is a genuine rum-specific ~5x defect, not inherent cost.

**Reframing (the important part):** the correctness pivot did NOT find new
correctness bugs and did NOT escape the Pair/Simp gate — it PROVED that the
Pair.wl-load performance wall is the single thing gating the entire downstream
correctness frontier. Performance and correctness are the SAME critical path
here. And the deferred hash-consing/value-memo project was validated SOUND +
~26% on exactly this Pair descent — so it is the one validated lever aimed
directly at the wall that blocks everything else. The rejected forked-`Simp.wl`
"fix" (85.5s) remains off the table (masks the rum hotspot, breaks snapshots).

## 2026-06-22 Hash-consing / value-memo probe: 57% dispatched-call structural repeat

After concluding flaws 1-3, asked WHY WolframKernel is fast (not LLVM/JIT — its
symbolic evaluator is a ~35-year-optimized C interpreter: compact rep, hashed
dispatch, expression SHARING/hash-consing, `Update[]` change-tracking). The one
WK-class architectural lever rum lacks is **hash-consing / structural value
memoization**. Probed its ceiling: a throwaway counter in `evaluate_call`
structural-hashes each DISPATCHED (unstable) call and counts repeats.

On the real bounded Pair descent (`runtime-profile-basic-only-pair4`,
`simpFixpointConv`): `total=432914 dispatched calls, distinct=186863,
repeat_rate=0.5684, max_rep=2440`. **~57% of dispatched calls are structurally
identical to one already dispatched.** This is the FIRST positive structural
signal for the Pair gap.

**Why this is a DIFFERENT lever than the refuted stamp:** the stamp / stability
cache only short-circuits STABLE (self-evaluating) exprs; the Pair repeats are
`REWRCONV[rule][term] → Failure` — NOT stable, so the stamp could never help
them. VALUE memoization (cache the evaluated RESULT, even a `Failure`, keyed by
structure) CAN. So the Task-3b stamp refutation does NOT rule this out.

**Caveats (the stamp also "looked promising" via 33M raw hits before the A/B
refuted it — do NOT over-read this):** (1) `0.57` is the whole-descent CEILING,
not an achievable bounded-cache hit rate (most structures recur only ~2.3×; a few
up to 2440×). (2) Measured PRE-arg-eval, so cheap repeats (the same
`REWRCONV[rule]` closure across the descent) may inflate it vs the expensive
failing-dispatch repeats. (3) SOUNDNESS is the real risk: value memo is only
valid if the result is a pure function of args (no hidden mutable state); within
one descent the rules/defs are fixed (memoizable) but it MUST invalidate on
definition/epoch change. Value memoization + hash-consing is the "WK-class"
effort the plan deferred (decision point 3) — a large, separate architectural
project, not an incremental cut. Next decisive validation BEFORE committing: a
shadow value-memo (structural key, bounded LRU, recompute-and-COMPARE to check
soundness empirically + measure the ACHIEVABLE hit rate and whether the repeats
are the EXPENSIVE dispatches) — the analog of the stamp prototype.

## 2026-06-22 Shadow value-memo VERDICT — sound for Pair, ~26% achievable, but needs the full hash-consing architecture

Built the shadow value-memo (measurement-only, never short-circuits): in
`evaluate_inner_unstable`'s `Call` arm, structural-hash the dispatched call key,
run the real `evaluate_call`, then on a repeat key recompute-and-COMPARE the new
result against the stored one (epoch-validated) to measure BOTH the achievable
hit rate AND empirical soundness (a mismatch = the memo would have returned a
stale/wrong value). Throwaway, now reverted.

**Decisive Pair result** (`runtime-profile-basic-only-pair4`): `total=397105,
hits=101851, hit_rate=0.2565, mismatches=0, distinct=160447`.

- **mismatches=0 → SOUND on the Pair descent.** The conversions act on concrete
  built terms (no free dynamic/loop vars), so an epoch-validated structural value
  memo would return correct results there.
- **~26% achievable hit rate** (below the 0.57 raw ceiling — epoch invalidation
  and the per-key first-miss eat the gap). This is the FIRST sound, positive
  structural lever found for the Pair gap, and it is a DIFFERENT lever than the
  refuted stamp: it short-circuits NON-stable failing conversions
  (`REWRCONV→Failure`) the stamp could never touch.

**But two hard caveats that make this the WK-class project, not a quick win:**

1. **General unsoundness — the epoch does not cover dynamic scopes.**
   `Do[Plus[i,1],{i,1,50000}]` → `total=50001 hits=49999 mismatches=49999`:
   rum binds Do/Table loop vars via a narrow single-symbol dynamic own-value
   scope that deliberately does NOT bump `evaluation_epoch` (for Do/Table perf),
   so a naive structural-key+epoch memo returns the first iteration's result for
   every iteration. `Block`/`Set` DO bump the epoch (`block-x-reassign`
   mismatches=0). So a production value-memo must either (a) make dynamic-scope
   bindings bump/extend the epoch key (a perf cost on the hottest loops), or
   (b) restrict memoization to exprs with no free dynamic vars. Pair is sound
   only because its path is concrete.

2. **Capturing the 26% REQUIRES cheap keys → hash-consing.** The shadow memo
   pays an O(expr) structural hash on 100% of dispatches to harvest the 26%
   hits. Without hash-consing (pointer-identity keys), that per-dispatch hash
   cost plausibly eats the 26% saving — exactly why WK hash-conses. And
   hash-consing has its OWN cost: interning every constructed expr (a global
   table probe on 100% of constructions). So the real project is
   hash-consing + value-memo + dynamic-scope-aware invalidation together — a
   large, multi-stage architectural effort with a now-MEASURED prize (~26% sound
   Pair dispatch short-circuit) and a now-MEASURED risk (the epoch model must be
   extended to dynamic scopes, and interning overhead must not eat the win).

**Net:** the hash-consing direction is VALIDATED as real and sound-for-Pair with
a moderate (~26%) prize — but confirmed to be the deferred WK-class architectural
project, not an incremental cut. Go/no-go is a scope decision, not a measurement
question anymore.

## 2026-06-22 ThinArc-args NO-GO — args-compaction line conclusively closed

Tested the one untried args layout: a THIN-pointer inline slice
(`triomphe::ThinArc<(), Expr>`) — thin pointer (no `Expr` growth, unlike B1b's
fat `Arc<[Expr]>`), elements inlined (no separate `Vec` buffer), variant shape
and Layer A sharing preserved. Correct (workspace + `DumpSave` round-trip green).

**Decisive machine-independent evidence (the alloc counter):** the alloc COUNT
did NOT drop — `pattern-hit` (which rebuilds `Plus[1,1]`, genuinely constructing
args) was `+2` (flat), `exact-hit` `+2`, `Range` flat; bytes and peak-live got
WORSE (`Range peak_live 7.46→13.86MB`, ~`2x`). Timing confirms: the unaffectable
control `if-true` moved `-4.5%` (binary/order artifact) while the construction
labels regressed ABOVE it (`exact-hit +8%`, `head-list +3.6%`).

**The mechanism, now fully understood (sharper than the earlier prediction):**
the construction pipeline (`prepare_arguments`, RHS substitution) always builds a
transient `Vec` first (incremental `push`; splicing makes a known-arity direct
build infeasible). Any inline-slice rep must then `from_iter` that `Vec` into a
new inline allocation and free the `Vec` — so it does the SAME 2 allocations per
construction as `Arc<Vec>` (`Vec` buffer + the node alloc), i.e. **it does not
even save an allocation**, it just adds an element-move + a transient
double-allocation (worse peak memory). `Arc<Vec>` wins because `Arc::new(vec)`
MOVES the `Vec` (zero element copy, the buffer stays put). And the only "live"
allocation an inline rep would save is the one Layer A's reuse already avoids
(the `Arc<CallData>` `-30` result).

**flaw-2 (multi-allocation node) is conclusively, mechanistically closed** — all
three single-allocation layouts tested and refuted with a consistent,
now-understood cause: `Arc<[Expr]>` (fat pointer grows `Expr`), `Arc<CallData>`
(access indirection, alloc reuse-minimized), `ThinArc` (no alloc-count saving +
element-move + worse peak). The current `Arc<Vec>` thin-pointer + zero-copy-move
construction + Layer-A reuse is genuinely well-matched to rum, not a flaw. The
representation-rewrite arc is closed with understanding, not just exhaustion:
Layer A (flaw 1) was the real and only profitable lever; flaws 2 and 3 are
reasonable designs whose obvious "fixes" each lose for an understood reason.

## 2026-06-22 flaw-3 churn REJECTED + measurement-methodology correction

The user rightly flagged that the flaw-3 (`miss_absent`) dismissal had leaned on
the now-distrusted `34x`. Re-examined on current code: 100% absent-miss confirmed,
and the call-stability cache THRASHES on fresh trees (`size_cap_clears` 2→32 —
fills with fresh entries, hits the cap, clears the whole cache, refills). The
`stability_check` phase is ~7% of the per-call cost (profiled ceiling). Two churn
experiments:

1. **Disable insertion entirely (blunt):** NET LOSS — the labels that benefit
   from legitimate hits (reused `{1,2,3}` list-literal stability, stable
   `pattern-miss`) regressed `+16-42%`, far outweighing the `2-11%` wins on
   churn-heavy labels. The cache's hits are valuable.
2. **Skip-insert-when-full instead of thrash-clear (targeted):** a first
   sequential back-to-back A/B looked like a broad WIN (`-3.7 to -7%`, fresh-tree
   `-8%`). **But it did not hold.** A 2×2 sequential A/B reversed it (`+3 to
   +7%`), and a drift-cancelled INTERLEAVED A/B (base/new alternated ×5, medians)
   settled it: the unaffectable control `if-true` showed `+2.7%` (the noise
   floor), and the cache-USING labels regressed ABOVE it (`exact-hit +4.9%`,
   `head-list +4.2%`) — a mild real regression. Mechanism: `thrash-clear` makes
   room so late-arriving hot entries can re-insert; `skip-when-full` starves them.
   The existing policy is actually better. REJECTED.

**MEASUREMENT METHODOLOGY CORRECTION (important, applies to all gating):** the
sequential back-to-back A/B (all-baseline-then-all-new, with a rebuild between
halves) has a systematic bias — the `new` half is always measured later, so
slow machine drift (thermal/background) lands entirely on it. For LARGE effects
(A4's ~6-11%, B1b's regression) the effect dominates the drift and the verdict is
safe, but for SMALL effects (~few %) the drift dominates and the result is
unreliable or sign-flipped (here the same change read `-7%` then `+7%`). The tell
is an unaffectable control label (e.g. `if-true` for a cache-policy change) moving
more than the claimed effect. The fix is an INTERLEAVED A/B (alternate base/new
each run, compare medians), which cancels slow drift. Use the machine-independent
counters first; reserve interleaved A/B for small-effect timing verdicts.

**Representation-rewrite arc — CLOSED (all three flaws data-exhausted):**
flaw 1 → Layer A (the real win, worst labels `-13-17%`); flaw 2 → single-alloc
node refuted (Arc<[Expr]> fat-pointer + copy; Arc<CallData> indirection-with-no-
alloc-saving — Layer A's reuse already neutralized flaw 2); flaw 3 → stamp
refuted + churn rejected (the cache earns its keep). Banked: Phase 0, **Layer A
(A1-A4)**, B0 (alloc tooling), B1a (immutable SharedArgs). The current `Arc<Vec>`
node + pointer-keyed stability cache are well-matched to rum's
construction-heavy, reuse-friendly workload.

## 2026-06-22 Arc<CallData> prototype: NO-GO; Layer A already neutralized flaw 2

To get a go/no-go on the ~572-site single-allocation-node migration WITHOUT doing
it, prototyped `Arc<CallData{head, args}>` via a trick (brief-019, throwaway):
`SharedExpr` and `SharedArgs` both wrap a SHARED `Arc<CallData>`, keeping the
`Expr::Call { head, args }` variant shape (no destructure-site migration). Thin
pointer (no `Expr` growth), move-construct (no copy) — it avoids B1b's two
failure modes and is a CONSERVATIVE lower bound (the real single-field node would
also shrink `Expr::Call` 16→8B). Correct: `cargo test --workspace` green, manual
round-tripping serde passes the `DumpSave` test.

**Result — decisive NO-GO:**
- **Allocations barely moved**: `count 3001487 → 3001457` (`-30`, not the
  expected ~`-1M`). The reason is the important finding: **Layer A's reuse
  (A2/A3/A4) already eliminated the node RE-allocations that flaw 2 would cost** —
  in the `fExact` hot loop the call nodes are reused, and the remaining ~3M
  allocations are NON-node (the `Do` machinery, arg-prep `Vec`s, evaluate
  temporaries) which a single-allocation node does not touch. So merging head+args
  into one Arc has essentially nothing left to save.
- **Hot tier regressed `+2.4%` to `+9.5%`** across the board (length `+9.5%`,
  part `+8.0%`, head `+7.4%`, sameq `+5%`, pattern-hit `+4.7%`): the `Arc<CallData>`
  projection adds an access indirection (`arc.args` is one more hop than
  `Arc<Vec>`) and construction complexity, with no alloc saving to offset it.

**Conclusion (data-gated, decisive):** the single-allocation node is NOT worth
the 572-site migration — it does not reduce allocations in practice (Layer A
already did, via reuse) and it regresses the hot tier even in the conservative
prototype. Combined with B1b (Arc<[Expr]> refuted: fat pointer + copy), **Layer
B's node-compaction line is fully data-refuted.** The deeper lesson: **Layer A
neutralized flaw 2** — by reusing nodes it avoids the re-allocation that the
multi-alloc node would otherwise cost, so the structural single-alloc fix is
unnecessary AND counterproductive. Kept from Layer B: B0 (alloc-count tooling)
and B1a (immutable `SharedArgs`, code quality). Remaining representation
candidate: flaw 3's cache-churn (~7% profiled ceiling), NOT the refuted stamp.

## 2026-06-22 Layer B B1b REFUTED: SharedArgs=Arc<[Expr]> is a net loss

Built the `SharedArgs(Arc<Vec<Expr>>)` → `Arc<[Expr]>` swap (brief-018). It is
CORRECT (workspace green, `DumpSave` round-trip unchanged) but a clean 2×2
back-to-back A/B (min-of-2 each half, idle machine) shows it **regresses the hot
tier `2-8%` across 6/9 labels** (`sameq +8.0%`, `pattern-hit +6.9%`, `if-true
+6.4%`, `length +5.0%`, `list-literal-small +4.8%`, `head-list +3.8%`). Reverted.

**Two mechanisms, both fundamental to `Arc<[Expr]>`:**
1. **Fat pointer grows `Expr`.** `Arc<[Expr]>` is a fat pointer (data + len = 16
   bytes); `Arc<Vec<Expr>>` is a thin pointer (8 bytes). So `SharedArgs` doubles
   `8 -> 16` bytes and the `Expr::Call` variant grows, enlarging the whole `Expr`
   enum — every `Expr` clone/move/cache-line gets more expensive, which is why
   even non-constructing labels (`if-true`, `sameq`) regress.
2. **`Arc::from(Vec)` copies.** Converting `Vec<Expr>` → `Arc<[Expr]>` allocates a
   new slice and COPIES the elements (vs `Arc::new(Vec)` which just moves the Vec
   — zero copy). rum is construction-heavy (fresh nodes everywhere), so this copy
   is paid constantly. The alloc counter confirmed it: peak-live actually ROSE on
   `Range[100000]` (`7.46 -> 13.86MB`, the transient Vec + new slice both live
   during `Arc::from`) and total allocs/bytes rose on the nested probe.

**Consequence for Layer B (data-gated decision):** the args-only compaction is
refuted. The full single-node option `Expr::Call(Arc<[Expr]>)` head-at-0 has the
SAME fat-pointer + copy problems, so it is refuted by the same data. The ONLY
non-regressing single-allocation layout is `Arc<CallData{head, args: Vec}>` (a
THIN pointer → no `Expr` growth; `Arc::new` → move-construct, no copy; `3 -> 2`
allocs/node) — but it requires the ~572-site `Expr::Call{head,args}` accessor
migration and folds head+args into one Arc (the independent-COW question). Its
prize is one fewer alloc/node, the same size of saving A3 made — and A3 did NOT
translate to Pair. So `Arc<CallData>` is a large migration for a likely-modest,
possibly-non-translating prize. Pattern repeats: the "obvious design flaw"
(multi-alloc node) has no cheap profitable fix; the current `Arc<Vec>` (thin
pointer, zero-copy move-construct) is actually well-matched to rum's
construction-heavy workload. Kept from Layer B: B0 (alloc-count tooling) and B1a
(immutable `SharedArgs`, a real code-quality win). Decision pending: pursue the
`Arc<CallData>` 572-site migration, or stop Layer B and bank B1a + Layer A.

## 2026-06-22 Layer B B1a (gated): SharedArgs mutators → build-then-freeze

`83938a0`. Removed the `DerefMut for SharedArgs` impl (kept the immutable
`Deref`) and rewrote the **40 compiler-exposed in-place mutation sites** across
12 files (`structure_ops/{sequence/insert,sequence/delete,replace/replace_part,
exact_paths,surface}`, `rewrite_builtins`, `definition_updates`,
`definition_catalog/traversal_continuation`, `parse_and_message_helpers`,
`suspension/{collections,rewrite_rules,traversal}`) to build-then-freeze: unwrap
to an owned `Vec<Expr>`, mutate, re-freeze into `SharedArgs` at the same point
the old code finished its in-place mutation. Most are continuation-payload
`pop`s; the structure-ops ones preserve the early-return-original no-mutation
branches and only unwrap when they actually mutate (spot-checked insert/delete).
Behavior-preserving, zero hot-path delta (cold builtin/suspension paths only).
`cargo test --workspace` green (1111 tests, reproduced by the director). This is
the step that makes B1b (the `Arc<[Expr]>` backing swap) a clean swap — nothing
mutates a `SharedArgs` in place anymore. NEXT = B1b.

## 2026-06-22 Layer B B0 (gated): allocation counter behind a cfg feature

`545b368`. The first step of the greenlit Layer B rewrite
(`docs/layer-b-node-rewrite-plan.md`): a counting allocator wrapping mimalloc
reports alloc count / total bytes / **peak live bytes** so B1 has an objective
before/after measure (the existing `RUM_PROFILE_REPR` counts `SharedArgs::new`
calls but not heap bytes / live memory, which is where the `Arc<Vec>` →
`Arc<[Expr]>` win actually lands). Baseline: `fExact[1]` 1M-call loop =
`count=3001487` (~3 allocs/call — directly confirms flaw 2's multi-allocation
node), `bytes_total=88.4MB`, `peak_live=7.47MB`; peak-live scales with held
structure size.

**Measurement/design lesson (cost-then-caught):** the brief specified an
env-gated (`RUM_PROFILE_ALLOC`) `AtomicU8` flag (the normal counter pattern). The
director gate (a clean back-to-back A/B, bare mimalloc vs the wrapper with its
flag OFF) caught a **4-19% regression on EVERY label** (if-true `+19%`,
pattern-hit `+18%`) — a `GlobalAlloc` WRAPPER blocks mimalloc's fast-path
inlining on every allocation, so even an off-by-default per-alloc branch is a
gross default-path cost (the no-default-path-cost hard line, but at the
allocator). Fix: gate the whole shim behind a Cargo `alloc-count` FEATURE, not an
env flag — the default build compiles in NO wrapper (bare mimalloc, verified: the
default binary emits no alloc line even with `RUM_PROFILE_ALLOC=1`); measurement
uses `cargo build -p rum-cli --features alloc-count`. Lesson for the rules: the
env-gated AtomicU8 counter pattern is fine for an ordinary instrumented function,
but NOT for a global allocator wrapper — there, only a compile-time `cfg` gate is
truly zero-cost. Next: B1a — migrate the ~70 in-place `SharedArgs` mutators to
build-then-freeze (remove `DerefMut`, fix the resulting compile errors), the step
that makes the `Arc<[Expr]>` backing swap (B1b) safe.

## 2026-06-22 A4 (gated, SYSTEMIC win): thread call head as SharedExpr

`9d6d639`. `evaluate_call` now carries the head as `SharedExpr` instead of
`head.into_owned()`: a stable-symbol/atom head is reused unchanged (no clone, no
re-evaluate), only unstable/computed heads are evaluated and re-wrapped, and the
no-change dispatch tail rebuilds with `Expr::call_shared` (reuse the head Arc, no
fresh `SharedExpr`). Stages keep borrowing `&Expr` via the `SharedExpr`.

Gate: `into_owned_clone` `1M/2M -> 0` across if-true/exact-hit/pattern-hit
(reproduced); if-true `shared_expr_alloc` `1M -> 27` (no-change reuse). Because
every call has a head, this is SYSTEMIC. A clean same-session back-to-back A/B
(run-to-run common-mode cancelled) shows EVERY label down: `block-local -11.7%`,
`downvalue-pattern-hit -9.6%`, `downvalue-exact-hit -7.2%`, `set-ownvalue -6.9%`,
`sameq -5.3%`, `if-true -3.6%`, `length`/`head-list -3.2%`, `part-list -2.1%`;
zero regressions.

Pair downstream — INDEPENDENTLY CONFIRMED by a clean back-to-back A/B on an idle
machine (pre-A4 commit `9d6d639^` vs A4, same machine state, nothing concurrent):
manual 19-rule loop `1.353 -> 1.247s` (`-7.9%`), raw-tryConv-d19 `1.786 ->
1.683s` (`-5.8%`), full TRYCONV[REPEATC] `2.248 -> 2.208s` (`-1.8%`). The raw
failing-dispatch (least combinator-diluted) moves most, confirming A4 reaches the
Pair per-failed-dispatch constant. The head clone is real per-call WORK the Pair
descent pays, which is why (unlike A3's pure candidate alloc) it translates.

**Measurement lesson (re-)learned:** cross-run microbench/Pair ratios are
unreliable when other work shares the CPU. The spurious if-true/block-local
"regressions" in a first comparison were a run-to-run confound — I had launched
a heavy Pair probe concurrently with that microbench run (and a MacBook Pro can
run nighttime maintenance: `mds`/`backupd`/etc.). It was NOT verified thermal
throttling (I cannot read CPU temperature; an earlier note that said "thermal"
was unsupported). The reliable gate is the clean same-session back-to-back A/B
run with nothing else competing; a verified-idle machine (maintenance daemons at
0% CPU) then shows ~1-2% run-to-run variance on most labels (if-true/block-local
stay intrinsically jittery ~6-9% — short loops — which is exactly why the A/B,
not a single run, is the gate for them). Cumulative from the frozen Phase-0
baseline: worst labels down ~13-17% (`downvalue-exact-hit 5.74x -> ~4.76x`,
`if-true 5.96 -> ~5.15`, `downvalue-pattern-hit 5.47 -> ~4.56`).

## 2026-06-21 Pair 34x ROOT CAUSE: distributed eval-loop constant (match is 5%)

Profiled the faithful failing-pattern dispatch (a pattern downvalue applied to
FRESH non-matching args each iteration, so the stability cache cannot
short-circuit — matching the Pair descent's fresh-subterm failing conversions).
The cost is DISTRIBUTED across the per-call evaluation machinery, with no single
hotspot and the failing MATCH itself only `5.3%`:

| phase | share | mult |
| --- | ---: | ---: |
| `evaluate.total` (fixed-point loop) | `21.1%` | 2/call |
| `evaluate.stability_check` | `7.4%` | 4/call |
| `evaluate.once_unstable` | `7.1%` | 2/call |
| `prepare_arguments.total` | `6.8%` | 1/call |
| `evaluate_call.dispatch` | `5.9%` | 1/call |
| `call.stage.down_values` (the actual failing match) | `5.3%` | 1/call |
| (long distributed tail) | … | |

**This is the final root cause of the Pair `34x`:** it is the persistent
distributed per-call evaluation constant (the fixed-point loop running ~2x/call,
~4 stability checks/call, argument prep, and dispatch drive glue) — the SAME
constant `if-true` measures (B0) — compounded over the deep descent and amplified
~1.7x by the WL control combinators. It is NOT the matcher (`5%`), NOT caching/
inline-stamp (refuted, Task-3b), NOT node allocation (A3 did not translate to
Pair), NOT Module frame setup (`module-local` `1.58x`). All single-hotspot
theories are eliminated; the gap is genuinely the distributed eval-loop constant.

**Biggest remaining lever:** the fixed-point evaluate-loop multiplicity — ~2
full `evaluate` passes and ~4 stability checks per call. A failing/stable call
(result structurally equal to input) is re-evaluated and re-checked more than
necessary. Reducing redundant evaluate passes / stability checks per call is the
next target, but it is eval-core surgery (the hottest loop), so it is
higher-risk and the wins are incremental (~5-15% per cut, per the brief-003/005/
007/008 history), and each must be downstream-validated (A3 showed a microbench
win need not translate to Pair).

## 2026-06-21 Downstream re-measure: A3 did NOT move Pair; lever corrected

Re-ran the bounded `basic-chain-orelsec-split` Pair unit post-A3 (the plan's
"re-measure downstream after each layer"). A3 did NOT move it: manual 19-rule
loop `3.76x -> 3.72x`, raw-tryConv-d19 `5.31x -> 5.40x`, full TRYCONV[REPEATC]
`6.55x -> 6.48x` — all within run noise. So the `downvalue-exact-hit`
candidate-allocation win (A3, ~10% on that label) did NOT translate to Pair.

Why (the correction): the Pair basic rules are pattern conversions that **fail**
on the target term, so the per-dispatch cost is the FAILING pattern-match work,
not the candidate allocation — the alloc only stood out in the trivial ground-
*hit* microbench (`exact-hit`). I picked the wrong proxy. The faithful Pair
proxy is `downvalue-pattern-MISS` (`3.31x`) ≈ the Pair manual-loop (`3.72x`),
not `exact-hit`. The real `34x` lever is the **failing-pattern-match dispatch
constant** (prefilter + match-attempt-then-reject over many rules per subterm),
compounded over the descent and amplified ~1.7x by the WL control combinators.

A3 remains a legitimate, gated win on `exact-hit` and removes a real per-dispatch
allocation, but it is not the Pair lever. Next: profile the failing-match path
(`downvalue-pattern-miss`, the fast faithful proxy) to localize the reducible
failing-dispatch cost before the next brief.

## 2026-06-21 Layer A3 (gated): reuse SharedArgs in candidate build

`868dea7`. The down/up/sub-value dispatch stages built their match candidate
with `Expr::call(head.clone(), args.to_vec())` — a deep arg copy + fresh
`SharedArgs` alloc per dispatch (the `call.stage.down_values` cost and the
exact/pattern-hit `shared_args_alloc` that the Pair re-diagnosis localized).
Post-A2 the state carries the args as `SharedArgs`, so the candidate now reuses
that Arc via `call_with_shared_args` (refcount bump). The three value stages take
`&SharedArgs`; the builtin/Listable/Function/special-form stages are byte-for-byte
unchanged.

Gate (counter authoritative, reproduced): `downvalue-exact-hit`
`shared_args_alloc` `1M -> 31`; `downvalue-pattern-hit` `2M -> 1M` (the RHS-rebuild
`Plus[1,1]` 1M remains — Layer B territory); `into_vec_clone` stays `0`.
`downvalue-exact-hit` ratio ~`5.5x -> ~5.0x` (2x2-run clean win); pattern-hit/miss
and the no-downvalue controls (if-true/sameq) unchanged. `length-list`/`head-list`
jitter is environmental noise — they are builtin calls whose dispatch path A3
does not touch.

**Layer A is now complete (A1+A2+A3).** Cumulative: flaw-1 arg churn eliminated
(`into_vec_clone -> 0`), the down/sub-value candidate arg-allocation eliminated,
the primitive tier down ~`5-11%` and `downvalue-exact-hit` from `5.74x` to
~`5.0x`. Remaining per-call flaw-2: the head allocation (`shared_expr_alloc`, the
candidate/RHS head `SharedExpr::new`) and the pattern RHS-rebuild node alloc;
plus the distributed dispatch/eval-loop constant (B0).

## 2026-06-21 Pair descent re-diagnosis (post-Layer-A), bounded two-engine

Per the corrected direction (the `34x` is the per-call dispatch/eval constant in
the conversion descent, not a cache/stamp problem), re-ran the bounded
`basic-chain-orelsec-split` unit (19 productive basic rules, 200 reps) in BOTH
rum and WK against the acceptance clone, no-profile. Logical work matches across
engines (both: 19/20 failed rules, same `is_thm`/`changed`), so the gap is a
pure per-op constant.

**No-profile decomposition (reliable):**

| segment | rum | WK | ratio | pre-Layer-A |
| --- | ---: | ---: | ---: | ---: |
| manual 19-rule loop (raw failed dispatch, no combinator) | `1.323s` | `0.352s` | `3.76x` | `4.7x` |
| + ORELSEC chain | `1.755s` | `0.331s` | `5.31x` | — |
| + REPEATC | `2.034s` | `0.349s` | `5.82x` | `5.9x` |
| + TRYCONV (full base-conv pass) | `2.298s` | `0.351s` | `6.55x` | `6.4x` |

So **Layer A cut the raw per-failed-dispatch constant `4.7x -> 3.76x` (~18%)**,
confirming the Pair chain is dominated by the same per-call constant the
primitive tier measures. The WL conversion-control combinators
(ORELSEC/REPEATC/TRYCONV — curried, Module+Catch+down/sub-value defined) amplify
`3.76x -> 6.55x` (~`1.7x`). The `34x` is this ~`6.5x` per-pass constant
compounded over the deep descent.

**Profile localization (SUGGESTIVE ONLY — profiling-inflated, needs no-profile
confirmation):** a profiling-ON run of the control-combinator path ranks
`sub_values.rhs_substitution`, `module.resume_initializers`, and
`down_values.rhs_substitution` far above the dispatch glue (which Layer A +
briefs 003-008 already cut). BUT those phases are densely instrumented, so their
absolute self-time is inflated; do not treat the ranking as the true cost split.
The substitution is ALREADY copy-on-write (`substitute_rule_bindings_changed`
returns `Option`, reusing unchanged subtree Arcs), so any real cost there is the
per-node WALK over large combinator bodies, not whole-body cloning. The
combinators are Module-wrapped, so Module frame setup
(`module.resume_initializers`) is paid per application.

**No-profile cross-check resolves the inflation (the unifying conclusion):** the
primitive microbench (no profiling, fast, no Pair load) shows `module-local` at
only `1.58x` — so Module frame setup is NOT the lever, and the profiled `65s`
`module.resume_initializers` was instrumentation inflation. The microbench's
worst labels are the DOWNVALUE/SUBVALUE dispatch ones (`downvalue-exact-hit`
`5.66x`, `downvalue-pattern-hit` `5.24x`). The Pair conversion combinators
(`ORELSEC`/`REPEATC`/`TRYCONV`/`simpRuleConv`) are curried down/sub-value-defined
WL functions, so the `34x` is the SAME per-call downvalue/subvalue dispatch
constant the microbench worst-labels measure, compounded over the descent (plus
a secondary RHS-walk cost only for the large-body combinators; `pattern-hit`'s
tiny `x+1` RHS is already `5.24x`, proving dispatch — not the RHS walk —
dominates the per-call constant).

**Implication for the plan:** there is no separate Pair-specific lever. The Pair
`34x` and the primitive tier's worst labels are ONE target: the
downvalue/subvalue dispatch + RHS-eval per-call constant. It can be worked on the
fast microbench (`downvalue-exact-hit` / `downvalue-pattern-hit`) as a faithful
proxy, with Pair as the compounding downstream beneficiary — no heavy Pair load
per iteration. Dead-ends eliminated this session: the inline stamp (refuted), the
node-alloc compaction (deferred, moderate), and Module frame setup (measured
cheap at `1.58x`). Next implementation target: the downvalue/subvalue dispatch
constant, diagnosed on `downvalue-exact-hit` (briefs 002/003 added the ground
fast path; it is still `5.66x`, so the residual is the shared dispatch +
matcher/binding + RHS-eval machinery — the same distributed per-call cost the B0
`if-true` self-timing showed).

## 2026-06-21 Inline stamp REFUTED (validation-first caught a known dead-end)

Direction was set to pivot Layer B to the inline normal-form stamp as the
`34x`/Pair lever. The validation-first step (reading the current cache mechanism
and the handoff before building) surfaced that this is a **already-run, negative
experiment**: Task-3b (2026-06-13) prototyped exactly this stamp behind
`RUM_CALL_STABILITY_INLINE_STAMP=1`; it carried the verdict correctly (unit
litmus passed) and **hit `33M+` times** on warm mkPair, **yet both variants
still timed out at the same `268M+` stability-check scale**. That refutes the
"redundant re-evaluation of shared stable subterms" hypothesis — if the cost
were re-checking shared subterms, the stamp would have moved it. The Pair `34x`
is therefore **genuine work in the conversion descent**, i.e. the per-failed-
dispatch CONSTANT (handoff Task 2d) = the same per-call dispatch/eval constant
the primitive tier measures, NOT a cache/sharing problem. The stamp carrier also
caused the reverted ~`12-14%` default-path regression (CLAUDE.md hard line).

Decision: do NOT build the inline stamp. `docs/representation-rewrite-plan.md`
B3 corrected to mark it refuted. The real `34x` lever is the per-call
dispatch/eval constant (which Layer A has been reducing, ~`5-11%` banked, and
which the B0 if-true self-timing localizes to the evaluate fixed-point loop +
stability-check passes + dispatch drive glue). Next target is that constant, not
a stamp/cache. This is the validation-first methodology working as intended:
it cost one diagnostic read and saved days of re-implementing a refuted lever.

## 2026-06-21 Brief-012 STOPPED: SharedArgs=Arc<[Expr]> hit hidden mutators

Attempted the low-risk Layer-B args compaction (`SharedArgs(Arc<Vec<Expr>>)` →
`Arc<[Expr]>`, one contiguous alloc instead of Arc-box + Vec-buffer, preserving
the `Expr::Call { head, args }` variant shape). The director audit estimated
~55 references, all slice-reads, no in-place mutation. **The audit was wrong:**
`cargo test --workspace` produced 70 compile errors, and removing `DerefMut`
exposed real in-place `SharedArgs` mutation (`pop`/`push`/`insert`/`remove`/
indexed-assign/`iter_mut`) via `Arc::make_mut` in ~12 files
(`structure_ops/{sequence/insert,sequence/delete,replace/replace_part,
exact_paths,surface}`, `rewrite_builtins`, `definition_updates`,
`definition_catalog/traversal_continuation`, `parse_and_message_helpers`,
`suspension/{collections,rewrite_rules,traversal}`). The audit grep filtered out
too aggressively. Codex's stop-loss (the brief's conservative ">~6 fallout
sites" limit) caught it via the compiler before any damage; the change was
reverted, tree clean.

Re-assessment: `Arc<[Expr]>` for args is a genuine ~70-site compiler-guided
migration (mechanical: ~58 `.as_slice()` read fixes + ~12 mutator sites
restructured to build-Vec-then-freeze), not a trivial swap. The prize is flaw-2
only (one fewer live allocation per *non-mutating* node construction +
contiguity); the mutating builtins (`Insert`/`Delete`/`ReplacePart`/...) do not
benefit (they clone+refreeze). It is NOT the `34x` headline (that is flaw 3, the
inline stamp). Per decision point 2 (a moderate prize behind a large migration
is a valid defer), the args/head alloc compaction is deferred; Layer B pivots to
the flaw-3 inline stamp (the `34x` lever), validation-first.

## 2026-06-21 B0 diagnostic: Layer B prize + the if-true residual fork

After Layer A, ran the B0 diagnostic (node-alloc share, miss_absent rate, what
now dominates if-true) to decide the Layer B go/no-go.

- **flaw 2 (node allocation):** the downvalue paths allocate 2-4 Arcs/call
  (`shared_args_alloc + shared_expr_alloc`: exact-hit `2/call`, pattern-hit
  `4/call` — candidate build + RHS rebuild + call node). `if-true` allocates
  ~`27` total (the result `1` is an atom; the body Arc is reused) — essentially
  zero per call.
- **flaw 3 (absent-miss):** on a fresh-RHS workload (`pattern-hit`),
  `call_stability_misses == call_stability_miss_absent` at every snapshot — 100%
  of stability-cache misses are absent-misses (the pointer key cannot hit a
  freshly rebuilt tree).
- **if-true residual is dispatch/eval-loop bound, NOT allocation bound.**
  Per-phase self-timing (profiling-inflated, read as proportions) spreads the
  cost across the evaluate fixed-point loop (`evaluate.total` called 2x/If,
  `stability_check` 3x/If), the COW `prepare_arguments` loop (still iterates
  plan/evaluate/compare per arg even with zero alloc), the `If` builtin itself,
  and the dispatch drive glue — no single dominant leaf. This is the persistent
  per-call constant (the brief-003/005/007/008 lever), separate from the
  allocation/absent-miss flaws.

**Consequence for Layer B:** B (single-alloc node + inline stamp) targets flaw 2
and flaw 3, so it helps the downvalue/subvalue paths and the downstream Pair
`34x` conversion chain (where there is real subterm sharing for a stamp to
catch). It will NOT move `if-true` (the current worst microbench label), because
`If[True,1,2]` evaluates to a different value (not a fixed point) and allocates
nothing — neither the cache nor a stability-stamp can short-circuit it; only
full value-memoization (deferred, out of scope) would. So Layer B should be
gated on the downvalue-path allocation counter halving and the downstream gates,
NOT on the if-true ratio.

## 2026-06-21 Layer A COMPLETE: shared-args dispatch + COW arg prep (gated)

Layer A of the representation rewrite (`docs/representation-rewrite-plan.md`)
landed in two gated commits and eliminated flaw-1 (owned arg round-trip churn)
on the hot paths.

- **A1 (`bfa7ba4`)** threaded `SharedArgs` (an `Arc<Vec<Expr>>`) through
  `evaluate_call` / `resume_call_after_head` / `CallDispatchState`, reading args
  by borrow in the stages. Net-neutral alone (it only *moved* the single
  `into_vec` from the dispatch entry into `PrepareArguments`) — the counter
  honestly stayed at ~`1M`, codex stopped at the gate, and I committed A1 only
  as the prerequisite carrier change. Lesson: the original brief-010 attached
  the `into_vec`-counter gate to the wrong layer; the materialization is in
  `prepare_arguments`, not the dispatch entry.
- **A2 (`6dd4fc7`)** made `prepare_arguments` copy-on-write: it reuses the input
  `SharedArgs` Arc when every prepared arg is structurally equal to its source,
  no `Sequence` splice changed the arity, and `Flat`/`Orderless` normalization
  is provably a no-op. Every arg is still evaluated once in order (safe by
  construction); suspension materializes the unchanged prefix and reuses the
  existing continuation. `evaluated_args` became `SharedArgs` so the COW result
  is not re-cloned on store.

**Measured result (counter is the authoritative proof, independently
reproduced):** `into_vec_clone` per 1M calls → `0` for `if-true`,
`downvalue-exact-hit` (was `1M`) and `downvalue-pattern-hit` (was `2M`).
Systemic tier ratio win, 2×2-run confirmed (A2 both runs below A1 both runs on
5/6 labels; `exact-hit` the 6th, mechanism-confirmed by the counter; zero
regressions):

| label | A1 (HEAD) | A2 | delta |
| --- | ---: | ---: | ---: |
| `if-true` | ~`5.65-5.89x` | ~`5.21-5.35x` | ~`-11%` |
| `downvalue-pattern-hit` | ~`5.46-5.70x` | ~`5.03-5.24x` | ~`-8%` |
| `length-list` | ~`3.64-3.68x` | ~`3.42-3.57x` | ~`-7%` |
| `head-list` | ~`3.53-3.56x` | ~`3.30-3.43x` | ~`-7%` |
| `sameq-integer` | ~`4.99-5.24x` | ~`4.67-4.89x` | ~`-6%` |
| `downvalue-exact-hit` | ~`5.61-5.74x` | ~`5.33-5.66x` | ~`-5%` |

**Re-rank for the next step:** the remaining per-call cost is now (a) the head
`into_owned` clone (~`1M`, but a cheap no-heap symbol clone — small) and (b) the
two-Arc node allocation `shared_args_alloc + shared_expr_alloc` ≈ `2/call`
(flaw 2). `if-true` at ~`5.2x` now allocates only ~`27` nodes (no rebuild), so
its residual is pure dispatch + evaluate-loop overhead, not allocation. The
downvalue shapes still allocate `2M`/`call` (RHS rebuild + call-node). Both flaw
2 and flaw 3 (pointer-cache `miss_absent`) are Layer B. Next: the B0 diagnostic
(node-alloc share + `miss_absent` rate + what dominates `if-true` now) decides
the Layer B node layout and whether the 572-site migration prize is worth it.

## 2026-06-21 Phase 0 Baseline FROZEN (representation rewrite reference)

Brief-009 (`044aa1e`) landed the env-gated (`RUM_PROFILE_REPR`) rum-core
representation-cost counters, zero-cost when off (verified by sampling that
`rum_core::profiling::*` / `record_into_*` never appear on the disabled hot
path; 672 rum-runtime + 6 rum-core tests green). This freezes the objective
Phase-0 baseline that every later rewrite step is gated against.

**Primitive tier (no-profile release, min-of-7, worst-first):**

| label | ratio vs WK |
| --- | ---: |
| `if-true` | `5.96x` |
| `downvalue-exact-hit` | `5.74x` |
| `downvalue-pattern-hit` | `5.47x` |
| `sameq-integer` | `5.38x` |
| `length-list` | `3.66x` |
| `head-list` | `3.38x` |

**Representation counters per 1M calls (the two flaws, now measured):**

| call shape | into_vec clone | into_owned clone | node allocs | reading |
| --- | ---: | ---: | ---: | --- |
| `if-true` | `1.00M` (100%) | `1.00M` (100%) | `~27` (≈0) | **pure flaw 1**: clones args, builds no new node |
| `downvalue-exact-hit` | `1.00M` (100%) | `1.00M` (100%) | `2.00M` (2/call) | flaw 1 + flaw 2 |
| `downvalue-pattern-hit` | `2.00M` (100%) | `2.00M` (100%) | `2.00M` | churn doubles (RHS rebuild) |

So 100% of owned conversions on these paths are deep clones (flaw 1), and an
ordinary call node costs **two** Arc allocations (flaw 2: SharedArgs + SharedExpr).

**Downstream gate:** Bool-stack load (acceptance clone `200f13a`) `0.70s`.
Pair `mkPair`/PropTaut are the heavy end-to-end gates, captured at Layer
boundaries, not every step.

**Per-layer success criteria derived from the baseline:**
- Layer A (borrowed/shared dispatch) wins ⇒ `into_vec_clone` / `into_owned_clone`
  fall toward 0 on builtin/held paths — `if-true` should go ~1M → ~0.
- Layer B (single-allocation node) wins ⇒ node allocs halve (2/call → 1/call):
  `shared_args_alloc + shared_expr_alloc` drops from ~2M to ~1M per 1M calls.

## 2026-06-21 Brief-008 Implemented: skip empty dispatch stages (gated)

Re-ranked after brief-007: `if-true` (~`6.4x`) was the worst again (shared
dispatch). `build_call_dispatch_plan` pushed the UpValues/DownValues/SubValues
stages unconditionally; `If[True,1,2]` (no such values) entered three no-op
stages costing ~`539ns` profiled. Gated each push on the value's possible
presence (`1212664`). Scope clean, `cargo test` 672 green (an existing
plan-structure test was correctly updated to the empty-stage plan, plus new
runtime regressions). Isolated proof: if-true's empty `call.stage.*` are gone,
while a downvalue symbol still enters `down_values` (control). A/B differential
confirms a real win — builtin labels `-10` to `-13%` (if-true `-13.4%`, sameq
`-13.0%`) while `downvalue-exact-hit` (keeps its stage) moved only `-1.2%`.

This largely exhausts the cheap, bounded per-call dispatch wins. The remaining
tier-wide cost is the owned-args churn (Arc->owned->Arc round-trip, ~13% of a
call) plus the two-allocation `Call` node and the pointer-keyed stability cache
that cannot hit freshly rebuilt trees. Closing more needs the representation
rework (Layer A: borrowed/shared dispatch — bounded, splittable; Layer B:
single-allocation call node + inline evaluated/stable stamp propagated through
substitution — larger). Scope Layer A with bounded briefs and Layer B from a
fresh allocation/absent-miss measurement before committing.

## 2026-06-21 Brief-007 Implemented: compiled-MatcherNode fast match (gated)

Added `fast_match_compiled` reading the precompiled `MatcherNode` instead of
re-probing the raw pattern `Expr`, swapped in at the 3 fast-path sites when the
matcher revision is current (`d4b50a6`). Scope clean, `cargo test` 671 green,
and the fall-through cases were spot-checked directly (Condition `/;`, sequence
`x__`, Alternatives `|`, PatternTest `?`, typed blank, multi-arg all evaluate
correctly — the compiled path returns `None` and the full matcher handles them).

Gate note on measurement: the whole-label microbench A/B was too noisy to
attribute (labels that cannot use the matcher — `if-true`, `sameq-integer` —
moved `-9` to `-12%` cross-run, same magnitude as the target's `-8.4%`). The
authoritative proof is the ISOLATED matcher phase: `down_values.fast_match`
self time dropped `583ns -> 163ns` (`-72%`) and the `pattern_*` predicate chain
vanished from a clean sample (`337 -> 0` self samples). Lesson: when whole-run
noise swamps the signal, measure the isolated mechanism directly. No real
same-tier regression.

## 2026-06-21 Brief-006 Diagnosis: pattern-matcher cost (gated, verified)

Re-ranked after brief-005, `downvalue-pattern-hit` (~`7.2x`) is the worst
primitive. Brief-006 split the ~`518-583 ns` `fast_match_definition_pattern`
cost. Probe clean (`symbol_invalidations = 1`). The matcher is dominated by
**re-probing the raw pattern `Expr`**: the `pattern_*` predicate chain is
`41.8%` (conservative) to `61.6%` of the matcher; `bind_pattern_name` /
`SymbolExprMap` is `13.9%`; structural clone/compare `7.1%`;
`attribute_set_for_head` `3.0%`; `match_blank` (the `_Integer` check) only
`1.0%`. Independently reproduced: in a clean sample the `pattern_*` probe chain
is `337` self samples vs `bind_pattern_name` `10` — the predicate chain
overwhelmingly dominates. This confirms the precompiled-`MatcherNode`
hypothesis: the matcher re-interprets the raw pattern every call even though a
compiled `MatcherNode` already sits on the `DownValue`. Fix → brief-007: drive
the fast match from the precompiled `MatcherNode`, not the raw-`Expr` probe
chain.

## 2026-06-21 Brief-005 Implemented: zero-cost dispatch timing (gated, systemic win)

Converted the 8 hot per-phase timing helpers to the `#[inline]` guard +
`#[cold] #[inline(never)]` slow-path pattern (`9bfec36`). Scope clean
(`profiling.rs` only), `cargo test` 664 green, enabled per-phase output
identical (46 phases), and a re-sampled disabled `if-true` no longer shows the
profiling helpers (were `12.3%`, now gone).

Because the instrumentation sat on the shared dispatch path, this is a SYSTEMIC
win, not a single-label one. Clean same-session microbench A/B, median rum-time
`-9.2%` across the primitive tier: `part-list -16.2%`, `length-list -13.7%`,
`set-ownvalue -11.5%`, `sameq-integer -10.5%`, `block-local`/`with-local` ~`-9%`,
`downvalue-exact-hit -8.2%`, `if-true -6.6%`; no same-tier regression. The
diagnostic scaffolding had been quietly taxing every call — exactly the
no-default-path-cost rule.

## 2026-06-21 Brief-004 Diagnosis: shared call-dispatch overhead (gated)

Diagnostic via `if-true` (the cleanest isolator of pure call overhead: literal
fast path, no downvalues, no matcher), with a `loop-empty` baseline subtracted.
Codex's ranked baseline-subtracted table puts the per-call shared cost in
owned-tree movement/cloning around call setup (`memmove`, `SharedArgs::into_vec`,
`Vec::clone`), argument preparation, the dispatch skeleton, and repeated
stability checks.

The actionable catch, independently verified: the per-phase timing
instrumentation costs **~12% of the per-call work with profiling DISABLED**.
`runtime_path_timing_start` / `record_profile_runtime_path_timing` (and the
builtin/dispatch timing variants) are sprinkled ~20x per dispatch and are NOT
`#[inline]`, so the disabled run still pays the call overhead. A clean unprofiled
`if-true` sample shows `230 / 1866` self samples (`12.3%`) in
`profiling::*` — matching Codex's `~11.8%`. This violates the no-default-path-cost
rule (template pitfall 3, the Mutex lesson). Fix → brief-005: apply the existing
`record_evaluation_cache_counter` pattern (`#[inline]` guard + `#[cold]
#[inline(never)]` slow path) so the timing hooks are zero-cost when off.

## 2026-06-21 Brief-003 Implemented: exact (ground) downvalue fast path (gated)

First implementation brief under the delegation model (`7748efd`). Codex added
`DownValue.lhs_is_ground` and a dispatch fast path that decides a pattern-free
LHS match with one `Expr` equality, skipping the shallow-signature prefilter and
the matcher. Scope was clean (the 4 in-brief files only), `cargo test -p
rum-runtime` is `664` green including new precedence/regression coverage.

Director gate measured the honest delta with a clean same-session A/B (stash the
change, rebuild, measure; restore, measure) rather than trusting the report's
cross-run numbers: `downvalue-exact-hit` rum time `-21.5%` (ratio ~`7.4x` ->
~`5.8x`), with `downvalue-exact-miss` / `downvalue-pattern-hit` /
`downvalue-pattern-miss` unchanged within noise. Codex's own report showed only
`7.76 -> 6.98` because it compared its run to the documented baseline (WK side
drifting); the gate's same-session A/B is why the true ~22% win was recovered.

After this, `downvalue-exact-hit` is no longer the worst primitive. The new
worst decision-grade targets are `if-true` (~`7.2x`) and `downvalue-pattern-hit`
(~`7.1x`); the next brief should target one of those.

## 2026-06-21 Brief-002 Diagnosis: `downvalue-exact-hit` (gated, verified)

Second Codex brief: diagnose the corrected worst primitive,
`downvalue-exact-hit` (~`7.76x`), a literal downvalue `fExact[1] = 2` looked up
by `fExact[1]`. The probe was clean this time (`symbol_invalidations = 1`,
pitfall 2b followed). The finding is verified by independent reproduction.

The hit-vs-miss asymmetry is the mechanism. Per application:

| metric | hit (`fExact[1]`) | miss (`fExact[2]`) |
| --- | ---: | ---: |
| `down_value_scans` / `down_value_applications` | `1` / `1` | `0` / `0` |
| `stability_checks` | `~3` | `~1` |
| `call_stability_hits` | `0` | `1` |

A downvalue MISS evaluates to itself, so the literal `fExact[2]` becomes a
steady-state call-stability HIT and short-circuits (cheap). A downvalue HIT
evaluates to a different value (`2`), so the call is never cached as stable and
**the full dispatch re-runs every iteration**: shallow-signature scan,
`normalize_definition_candidate_for_matching`, `fast_match_definition_pattern`,
`substitute_rule_bindings_for_rhs`, `Expr::clone`, RHS re-evaluation, and ~2
extra stability checks. `try_apply_down_values_from_symbol` is the top self-time
leaf with `evaluate_definition_rhs` fully under it.

Root cause and fix direction: down_values are a `Vec<DownValue>` dispatched by
scan + matcher; there is no exact-literal index (the `exact_down_value_rhs`
helper exists but is used only by the `Default` path). A literal, pattern-free
downvalue like `fExact[1] = 2` does not need the matcher / binding-substitution
/ candidate-normalization machinery at all — WolframKernel hash-indexes exact
downvalues and returns the stored RHS directly. The implementation brief
(brief-003) is an exact-downvalue fast path: when a call's args are all ground
and the symbol has an exact downvalue for them, return the stored RHS via a
keyed lookup, bypassing the matcher. It must preserve definition ordering /
specificity vs pattern downvalues.

## 2026-06-21 Brief-001 Diagnosis: `downvalue-pattern-hit` (gated, corrected)

First Codex task brief under the new delegation model (`docs/dev/codex/`): a
diagnostic profile of the worst bottom-up primitive, `downvalue-pattern-hit`
(~`5.18x` WK). Codex's directional finding is correct and is retained: a
pattern-downvalue hit does NOT repeat candidate scanning
(`down_value_scans / down_value_applications = 1.0`); the per-hit cost is the
match + binding substitution + evaluation of a fresh RHS tree, and the
pointer-keyed call-stability cache cannot help because each RHS instantiation
is a new tree (all `call_stability` misses are `miss_absent`). The hot path is
`resume_call_after_head → try_apply_down_values_from_symbol`, with
`fast_match_definition_pattern`, `substitute_rule_bindings_changed`, and
`SharedArgs::into_vec` / clone / memmove on the apply path.

Director gate correction: Codex's probe added `rumProfileSink = fPattern[1]`
per iteration — an ownvalue `Set` the target label does not have. Reproducing
with the faithful shape (`Do[fPattern[1], {n}]`, no sink) corrected the
contaminated numbers:

| metric | Codex probe (with sink Set) | faithful probe (no Set) |
| --- | ---: | ---: |
| time / iteration | `7.0us` | `4.1us` (label is ~`3.9us`) |
| `symbol_invalidations` | `2,500,005` | `1` |
| `stability_checks` / application | `9.0` | `4.0` |
| `down_value_scans` / application | `1.0` | `1.0` |

The per-iteration `Set` produced ALL `2.5M` invalidations and inflated both the
stability ratio (9→4) and per-iteration time (~70%); the "call-stability misses
every time" effect is the intrinsic fresh-RHS-tree behavior, not an invalidation
storm. The honest target for the implementation brief is ~`4` stability checks
plus fresh-tree RHS eval plus match/binding per hit (rum ~`4us` vs WK ~`0.76us`).
Lesson folded into `docs/dev/codex/TEMPLATE.md` pitfall 2b: a probe must be the
label's exact statement, never wrapped in a sink `Set`. This is exactly what the
director gate exists to catch — the raw report's headline counter was an artifact.

## 2026-06-21 Core-Language Microbench Performance Reset

Replaced the Pair/Simp-first performance entry route with a bottom-up
core-language microbench ladder. The active plan now lives in
[docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md),
and [docs/performance-stage0-baseline.md](./docs/performance-stage0-baseline.md)
documents the new `scripts/core-language-microbench.sh` runner. `AGENTS.md`
and `TODO.md` now say that Pair/Simp is downstream acceptance evidence, not
the default optimization entry point while lower-tier core labels still show
unexplained constant-factor gaps.

Initial runner validation exposed the first true bottom-up target:

- The first high-iteration harness attempt hit a rum stack overflow at
  `Do[..., {10000}]` scale.
- The first completed core table showed `loop-empty` at `0.510967s` in rum
  versus `0.001173s` in WolframKernel (`435.61x`), proving that larger labels
  were dominated by loop/evaluator plumbing rather than matcher or Pair/Simp
  mechanics.

Two retained runtime cuts followed from the lowest labels:

- `Do` count loops now advance with an iterative Rust loop instead of recursive
  `resume_do_iterations` calls. `Do[Null, {10000}]` no longer overflows the
  CLI stack.
- `Do` / `Table` iterator loops now use a narrow single-symbol dynamic
  own-value scope instead of constructing and evaluating
  `Block[{i = value}, body]` every iteration. The scope restores symbol state
  through suspension before the outer loop continuation resumes.

Representative no-profile release rows:

| label | before loop cuts | after loop cuts | WolframKernel | final ratio |
| --- | ---: | ---: | ---: | ---: |
| `loop-empty` | `0.510967s` | `0.004382s` | `0.001123s` | `3.90x` |
| `do-count-null` | stack/limit side lane | `0.001229s` | `0.000501s` | `2.45x` |
| `do-iterator-null` | `0.001503s` | `0.000339s` | `0.000052s` | `6.52x` |
| `table-count-null` | `0.000343s` | `0.000301s` | `0.000287s` | `1.05x` |
| `table-iterator-symbol` | `0.001723s` | `0.000781s` | `0.000096s` | `8.14x` |

The `--scale 5` rerun completed after lowering the `do-count-null` base count
to stay under the default iteration work limit. It shows the lowest count loop
labels in or near the target band (`loop-empty` `4.06x`,
`do-count-null` `2.19x`, `do-iterator-null` `4.30x`), while remaining larger
gaps now point to the next tiers: generic `Block`, `Table` iterator/list
collection, and definition/matcher dispatch labels such as
`downvalue-pattern-miss`.

Focused checks:

```bash
cargo test -p rum-runtime holding_and_control_part4 --lib
cargo test -p rum-runtime holding_and_control_part5 --lib
cargo test -p rum-runtime input_resume --lib
```

Added regression coverage for the large `Do` count loop and for `Do` iterator
scope preservation across `InputString` suspend/resume.

## 2026-06-20 Stage 1 Symbol-Key Runtime Cut

Started Stage 1 of the kernel rewrite with a conservative symbol-key cleanup:
core `Global`` / `System`` context strings are now canonical static texts,
`Symbol` has cheap context/name helpers, and the runtime builtin registry is
keyed by `Symbol` rather than `String`. Builtin default dispatch and default
attribute lookup now use the same symbol-key path, while display and completion
still render names from the symbol text.

Correctness checks:

```bash
cargo fmt --check
cargo test -p rum-runtime --lib
```

The first quick no-root baseline against
`/private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol` used:

```bash
scripts/perf-baseline-stage0.sh \
  --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol \
  --engine rum \
  --timeout 420 \
  --skip-root \
  --output-dir /tmp/rum-stage1-symbol-builtin-quick
```

Comparison with the Stage 0 rum rows:

| label | Stage 0 rum | after symbol-key builtin | result |
| --- | ---: | ---: | --- |
| `var-basic-baseConv-top-TRYCONV-REPEATC` | `0.013118075s` | `0.012318896s` | small positive |
| `basic-chain-full-TRYCONV-REPEATC-depth19` | `2.661301684s` | `2.487020894s` | positive |
| `control-TRYCONV-REPEATC-success` | `2.560320439s` | `2.493639789s` | small positive |
| `pair4-stock-repeat8` | `17.191801002s` | `16.569549047s` | small positive |

Retained interpretation: this is a low-risk constant-factor cleanup, but not
the main Pair/Simp gap. The current profile frontier after this cut remains
ordinary definition/subvalue dispatch and matcher/evaluator representation
costs, especially ``HOL`Drule`ORELSEC`` and
``HOL`Drule`Private`tryConv`` in the repeated conversion-failure chain.

Measured and rejected before commit:

- A symbol-head `call_dispatch_plan_cache` prototype. It moved
  `pair4-stock-repeat8` slightly (`16.57s -> 16.40s`) but regressed the
  depth-19 conversion chain (`2.49s -> 2.76s`), so it was removed.
- A `Vec`-backed small `SymbolExprMap` prototype for matcher/lexical bindings.
  It regressed the retained quick baseline (`depth19` `2.49s -> 2.69s`,
  `pair4` `16.57s -> 17.06s`), so the hash-map binding representation stays.
- A local-loop rewrite of downvalue/subvalue failed-candidate dispatch. It
  regressed the depth-19 chain (`2.49s -> 2.84s`) despite a noisy small pair4
  improvement (`16.57s -> 16.51s`), so the recursive continuation-shaped
  implementation stays until a deeper dispatch representation change is ready.
- A `LexicalPresenceCache` rewrite from `RefCell<HashMap<...>>` to direct
  mutable cache passing. It looked plausible from the `Module` body-substitution
  profile, but the no-root quick baseline regressed (`depth19` `2.49s ->
  2.56s`, `pair4` `16.57s -> 16.89s`), so the original interior-mutable
  presence cache stays.
- A compiled-`MatcherNode` fast-definition matcher prototype. It tried to avoid
  expression-pattern traversal on the fast path, but the retained quick
  baseline still regressed (`depth19` `2.49s -> 2.51s`, `control` `2.49s ->
  2.52s`, `pair4` `16.57s -> 16.79s`), so the current expression-based fast
  matcher stays until a larger matcher representation change is justified.

## 2026-06-20 Stage 0 Performance Baseline Runner

Started the deep performance rewrite at Stage 0. Added
`scripts/perf-baseline-stage0.sh` and
[docs/performance-stage0-baseline.md](./docs/performance-stage0-baseline.md).
The runner executes the fixed micro-proxy, mid-level, and root Pair/Simp probes
against an unmodified `mma-hol` checkout, disables rum profiling for baseline
timings, and writes `runs.tsv` plus decimal-normalized `measurements.tsv`.

Validated the full suite on
`/private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol` with:

```bash
scripts/perf-baseline-stage0.sh \
  --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol \
  --engine both \
  --timeout 420 \
  --root-timeout 900 \
  --include-wk-wrapper \
  --output-dir /tmp/rum-stage0-baseline-current
```

All rum and WolframKernel baseline probes completed, including the root
`pair-load-root-phase`; the optional WK wrapper probe also completed. Key
no-profile rows from `/tmp/rum-stage0-baseline-current/measurements.tsv`:

| label | rum | WolframKernel | ratio |
| --- | ---: | ---: | ---: |
| `var-basic-baseConv-top-TRYCONV-REPEATC` | `0.013118075s` | `0.001972s` | `6.65x` |
| `basic-chain-full-TRYCONV-REPEATC-depth19` | `2.661301684s` | `0.375481s` | `7.09x` |
| `control-TRYCONV-REPEATC-success` | `2.560320439s` | `0.442067s` | `5.79x` |
| `pair4-stock-repeat8` | `17.191801002s` | `2.587891s` | `6.64x` |
| `root-get-stdlib-pair` | `217.799748228s` | `39.357833s` | `5.53x` |

This confirms the rewrite plan's current target shape: the gap is consistent
from small conversion failure through the root Pair load, so Stage 1/2 work
must reduce fundamental runtime constants instead of chasing only one
microprobe.

## 2026-06-20 Performance Kernel Rewrite Plan

Added [docs/performance-kernel-rewrite-plan.md](./docs/performance-kernel-rewrite-plan.md)
as the active performance mainline. The plan sets the target at
WolframKernel parity or better on the supported `mma-hol` slice and sequences
the work through baseline freeze, symbol identity, expression representation,
borrowed evaluator frames, definition/matcher plans, conversion-chain plans,
memory strategy, and only then optional bytecode/JIT. `README.md`,
`ARCHITECTURE.md`, `TODO.md`, `AGENTS.md`, and the PropTaut performance audit
now link to that plan instead of treating Pair/Simp work as a loose sequence of
local probes.

## 2026-06-20 WolframKernel Conversion-Failure Probe

Added a WolframKernel-only wrapper probe at
`scripts/perf-probes/pair-simp/wolfram/wk-conversion-wrapper-profile.wls`.
It loads the unmodified clean `mma-hol` clone, builds the same Pair/Simp
basic-rule conversion chain, then temporarily wraps visible WL-level
conversion functions after setup. The reference clone at
`/private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol` stayed clean.

Clean non-instrumented WolframKernel baselines:

- `wk-profile-baseline-baseConv-tryConv-depth19`: `0.353249s` for `200`
  repetitions.
- `wk-profile-baseline-TRYCONV-REPEATC-depth19`: `0.372219s` for `200`
  repetitions.
- Existing `pair4-stock-repeat8` under WolframKernel:
  `preheat-basic` `7.63629s`, target `2.485002s` for `8` repetitions.
  The current rum target remains about `15-17s`, so the root-like small probe
  is still roughly `6x` slower in rum.

Instrumented WolframKernel counts:

- On `baseConv` through `tryConv`, WK still performs `3819` failed
  `REWRCONV` / `simpRuleConv` calls for `200` repetitions, plus `4020`
  failed `tryConv` calls. That is about `19` failed rewrite rules per
  iteration, not a WL-level rule skip.
- Wrapping the full `TRYCONV[REPEATC[baseConv]]` shape shows the same `3819`
  failed `REWRCONV` / `simpRuleConv` calls, with `201` `REPEATC` calls all
  taking the fail-to-`REFL` path and `201` outer `TRYCONV` calls returning
  success.
- On `pair4-stock-repeat2`, WK performs `6954` failed `REWRCONV` /
  `simpRuleConv` calls, `9336` failed top-level `tryTermMatch` calls, `366`
  `REPEATC` fail-to-`REFL` calls, and `366` successful outer `TRYCONV` calls.

Interpretation: WolframKernel is not avoiding the mma-hol conversion failure
chain at the WL level. It runs the same repeated failed rules, but its
per-failure constant is much lower: failed `tryTermMatch` is tens of
microseconds, and failed `REWRCONV` is roughly one hundred microseconds under
the intrusive wrapper. Therefore a production rum change that claims to copy
WK should focus on making the existing failure path cheap through evaluator
dispatch, representation, pattern dispatch, `Catch`/`Throw`, and structural
operation constants. A semantic failed-conversion/no-change cache remains a
possible rum-specific shortcut if proven sound and root-linked, but it should
not be justified as "what WK must be doing."

## 2026-06-20 Pair/Simp Repeated Failure-Chain Revalidation

Returned the Pair/Simp performance lane to the current unmodified
`Pair.wl` / first `mkPairInj` `CONVRULE` root after the evaluator
micro-splits showed that `prepare_arguments.evaluate_arg`, atom fast paths,
dispatch-plan construction, and cheap normalization were not root-scale hot
leaves.

Invalid probe history:

- `/tmp/rum-pair-repeat.current-var-native-trace-20260620` attempted
  `topRepeatcNativeTrace` on the known var leaf
  `9e642f187bbe77fcd524ebf5f1ca4aec59fe51d2e7d0d765595b9b378c8b9190`.
  The run failed after `34s` with a `rum-cli` stack overflow before emitting
  split rows. Treat it as probe-overhead / recursion evidence only, not as a
  timing result.

Valid current-worktree probes, all against the same clean `mma-hol` clone and
temporary instrumented copies only:

- `/tmp/rum-pair-repeat.current-var-toprepeatc-20260620`: same first
  `CONVRULE` root, `topRepeatc` split for the var leaf
  `9e642...9190`. Wall `279s`, markers true. Aggregate root rows show
  root `simpFixpointConv` about `226.35s` and root `descendOnce` about
  `217.13s`. The context-length `1` comb
  `736164760f6f457a67d70768cab89cfed0fd0966abfeddbcb0ad2bd1b9380bfc`
  accounts for about `64.55s` of `simpFixpointConv`; its var leaf accounts
  for about `26.04s`. Five `topRepeatc` samples on that var leaf averaged
  `23.46ms` (`19.79-27.98ms`), with the direct detail loop averaging
  `22.39ms`. The expanded base-conversion pass is a full failure scan over
  one context theorem, twenty basic rules, and beta.
- `/tmp/rum-pair-repeat.current-const-toprepeatc-20260620`: sibling const
  leaf
  `2821ed455384f83164f551f6b1682c1ce385679d7f8218a5bde44a4fc5cbf288`
  under the same comb. Wall `280s`, markers true. The same comb row is about
  `66.75s`; the const leaf is about `29.90s` and the var sibling about
  `25.20s`. Five `topRepeatc` samples on the const leaf averaged `21.10ms`
  (`17.00-25.10ms`), with the direct detail loop averaging `23.76ms`.
- `/tmp/rum-pair-repeat.current-failure-cache-first-convrule-20260620`:
  temporary `--prototype failure-cache` diagnostic, still on the first
  `CONVRULE` root and still only in generated probe copies. Wall `181s`;
  markers true. The prototype recorded `21` cache entries, `5283` hits,
  `21` stores, `2` changed cases, and `1832` skipped cases. Root
  `simpFixpointConv` fell to about `137.65s`; the hot comb
  `7361...0bfc` fell to about `29.18s`; the const and var leaves fell to
  about `8.13s` and `7.26s`.
- `/tmp/rum-pair-var-basic-baseconv-chain.call-result-cache-off-20260620`
  versus
  `/tmp/rum-pair-var-basic-baseconv-chain.call-result-cache-on-20260620`:
  a rum-side opt-in generic call-result cache prototype was measured and
  rejected before commit. The default-off baseline completed the
  `runtime-profile-var-basic-baseconv-chain` probe in `54s`; enabling the
  cache timed out at `240s` before producing useful phase output. The cause
  is the wrong granularity: constructing full evaluated-call keys around
  package load / preheat is far too broad for this hotspot. Do not revive a
  global "cache every successful call" shape for Pair/Simp without a new
  proof that candidate selection is narrow and root-linked.
- `/tmp/rum-var-basic-baseconv-chain.catch-failure-cache-off-20260620`,
  `/tmp/rum-var-basic-baseconv-chain.catch-failure-cache-on-20260620`, and
  `/tmp/rum-var-basic-baseconv-chain.catch-failure-cache-ignore-dirty-20260620`:
  a narrower opt-in held-`Catch` failure cache prototype was also measured
  and rejected before commit. On the tiny
  `runtime-profile-var-basic-baseconv-chain` probe, the safe version saw
  `40` lookups, `0` hits, and `18` dirty misses; ignoring dirty invalidation
  as an unsafe upper bound produced `18` hits and cut the manual rule loop
  from about `18-21ms` to `2.8ms`. The root-linked stock
  `pair4-stock-repeat8` check rejected it anyway: default-off target
  `16.94s`, safe cache `18.81s`, unsafe ignore-dirty `18.17s`. The prototype
  was too late and too narrow: caching the held `Catch` failure tail does not
  avoid the repeated `ORELSEC` / `REWRCONV` construction, dispatch, or
  no-change traversal that owns the Pair/Simp wall.
- `/tmp/rum-pair4-stock-repeat8.current-after-call-result-reject-noprofile`
  and
  `/tmp/rum-pair4-stock-repeat8.current-after-call-result-reject-evalprofile`:
  post-revert stock probe sanity checks. The no-profile `pair4-stock-repeat8`
  target measured `17.60s`; the evaluation-cache profiled run measured
  `24.75s` on the same target. The profiled run still concentrates
  call-stability invalidation under `HOL`Drule`ORELSEC`,
  `HOL`Drule`REWRCONV`, `HOL`Drule`Private`tryConv`, and Simp
  conversion builders, especially `HOL`Terms`comb` heads.
- `/tmp/rum-repeatc-tryconv-control-split.current-runtimepath`: narrow
  runtime-path probe with lexical timing enabled but shape/source details
  disabled. It completed in `101s`; `prepare_arguments.evaluate_arg` self
  time was only about `31ms`, dispatch-plan construction about `16ms`, and
  normalization under `8ms`, while profile wall was dominated by recursive
  conversion evaluation. This reconfirms that evaluator glue is side evidence,
  not the current Pair root optimization target.

Interpretation: the retained root-linked hotspot is repeated no-change
conversion failure over stable atomic leaves. The repeated cost is not owned
by Abs opening, RHS extraction, `TRANS`, `descendOnce` on the leaf itself, or
the earlier evaluator `prepare_arguments` split. A correctness-preserving
rum-side implementation should therefore target failed conversion dispatch /
no-change conversion caching with conservative term identity, rule-set/context
identity, and definition/version invalidation. The temporary `auto/Simp.wl`
cache prototype is only a diagnostic ceiling; do not carry it into downstream
`mma-hol` or present it as the acceptance path.

## 2026-06-20 Pair/Simp MatchQ Blank Fast Path

Kept the current Pair/Simp optimization pass small and measurable on the
unmodified clean `mma-hol` clone.

Rejected experiment:

- A rule-RHS presence prefilter keyed by shared call identity and alpha-scope
  ids was tested on the no-change basic-rule failure path and then reverted.
  It slightly improved one top-level `TRYCONV[REPEATC]` sample but slowed the
  manual rule loop and moved `pair4-stock-repeat8` from the current baseline
  `16.02s` to `16.81s`, so it was a net negative.

Retained cut:

- `MatchQ` now fast-paths already-resolved `Blank` / `Blank[head]` patterns
  through the same `match_blank` predicate used by the matcher. This covers
  the repeated `MatchQ[res, _Failure]` check in `HOL`Drule`Private`tryConv`
  after `Catch` converts conversion failures into `Failure[...]`.

Validation:

```bash
cargo test -p rum-runtime failure_symbolic_property_and_predicate_edges_cover_current_subset --lib
cargo test -p rum-runtime --lib
cargo build --release -p rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=0 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=0 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-var-basic-baseconv-chain --timeout 180 --output-dir /tmp/rum-pair-var-basic-baseconv-chain.matchq-fastpath-noprofile --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=0 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=0 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=0 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-pair4-stock-repeat8 --timeout 180 --output-dir /tmp/rum-pair4-stock-repeat8.matchq-fastpath-noprofile --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

Measured release result:

- `runtime-profile-var-basic-baseconv-chain` no-profile top
  `TRYCONV[REPEATC]`: old baseline about `13.17ms`, now `11.72ms`.
- `runtime-profile-pair4-stock-repeat8` no-profile target: old baseline
  `16.02s`, now `15.69s`.

This is a useful constant-factor cleanup of the failure tail, not the remaining
dominant Pair/Simp wall by itself.

Follow-up closed probe:

- Added
  `scripts/perf-probes/pair-simp/single-basic-failure-split.wls` to split one
  productive basic simplification rule failing on the variable target `x`.
  The probe keeps the clean `mma-hol` tree unmodified and validates that the
  single failure returns `$convFailed`, that the raw body catches a `Failure`,
  and that `tryTermMatch` returns `$matchFail`.
- Probe-design note: `Do[expr, {i, 1, 500}]` was rejected because the loop
  variable write polluted rum timings (`single-loop-empty` about `6.91s`).
  `Do[expr, {N}]` was also rejected for this purpose because the resulting
  timings were not interpretable. The retained probe uses
  `Module[{i = 0}, While[i < N, expr; i = i + 1]]` and prints a count check.

Validation commands:

```bash
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only single-basic-failure-split --timeout 180 --output-dir /tmp/rum-single-basic-failure-split.rum.while-loop --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only single-basic-failure-split --timeout 180 --output-dir /tmp/rum-single-basic-failure-split.wolfram.while-loop --engine wolfram
```

Selected `N=500` totals:

| segment | rum | WolframKernel | ratio |
| --- | ---: | ---: | ---: |
| full `tryConv[simpRuleConv[rule][x]]` | `0.1499s` | `0.0440s` | `3.4x` |
| `Catch[simpRuleConv[rule][x]]` | `0.1428s` | `0.0400s` | `3.6x` |
| direct `Catch[REWRCONV[rule][x]]` | `0.1089s` | `0.0274s` | `4.0x` |
| manual failed `REWRCONV` body | `0.0799s` | `0.0202s` | `4.0x` |
| `tryTermMatch[lhs, x, {<||>, <||>}, 0]` | `0.0349s` | `0.0079s` | `4.4x` |
| full `convFail` / `holError` catch | `0.0231s` | `0.0077s` | `3.0x` |
| construct `Failure[...]` with full extra payload | `0.0147s` | `0.0043s` | `3.4x` |
| `MatchQ[failure, _Failure]` after the fast path | `0.0045s` | `0.0005s` | small absolute |
| loop empty | `0.0023s` | `0.0004s` | small absolute |

Interpretation: the single basic-rule failure path is genuinely slower in rum,
but only around `3-4x` on the closed parent. It cannot alone explain the
remaining `6x`-class `pair4-stock-repeat8` gap. The next root-linked split
should move one level back up to the 19-rule `combineSimpConvs` /
right-associated `ORELSEC` chain and `TRYCONV[REPEATC[baseConv]]`, because the
extra multiplier likely appears in chain composition, repeated closure calls,
or parent-level conversion control rather than in one isolated `REWRCONV`
failure.

The next split did exactly that, again without changing the clean `mma-hol`
tree:

- Added
  `scripts/perf-probes/pair-simp/basic-chain-orelsec-split.wls`.
- The probe builds the 19 productive basic `simpRuleConv` conversions, the
  `combineSimpConvs` right-associated `ORELSEC` chain at depths `1`, `5`,
  `10`, and `19`, and prebuilt `REPEATC` / `TRYCONV` wrappers around the
  depth-19 base conversion.
- The retained repeat harness uses a local `While` count check and avoids
  storing every iteration result, so repeated local assignment does not dominate
  the measured segments.

Validation commands:

```bash
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only basic-chain-orelsec-split --timeout 180 --output-dir /tmp/rum-basic-chain-orelsec-split.rum --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only basic-chain-orelsec-split --timeout 180 --output-dir /tmp/rum-basic-chain-orelsec-split.wolfram --engine wolfram
```

Selected `N=200` totals:

| segment | rum | WolframKernel | ratio |
| --- | ---: | ---: | ---: |
| manual 19-rule `tryConv` loop | `1.621s` | `0.343s` | `4.7x` |
| manual 19-rule loop + final `BETACONV` fail | `1.663s` | `0.345s` | `4.8x` |
| raw `combineSimpConvs` chain, depth 19 | `1.861s` | `0.360s` | `5.2x` |
| `Catch[baseConv[x]]`, depth 19 | `1.934s` | `0.360s` | `5.4x` |
| prebuilt `REPEATC[baseConv][x]` | `2.199s` | `0.372s` | `5.9x` |
| prebuilt `TRYCONV[REPEATC[baseConv]][x]` | `2.425s` | `0.378s` | `6.4x` |
| fresh `TRYCONV[REPEATC[baseConv]][x]` | `2.431s` | `0.383s` | `6.4x` |

Depth scaling:

| segment | rum depth 1/5/10/19 | Wolfram depth 1/5/10/19 |
| --- | ---: | ---: |
| raw `tryConv[baseConv[x]]` | `0.101 / 0.470 / 0.958 / 1.861s` | `0.022 / 0.096 / 0.191 / 0.360s` |
| full `TRYCONV[REPEATC[baseConv]][x]` | `0.201 / 0.670 / 1.278 / 2.423s` | `0.038 / 0.116 / 0.212 / 0.374s` |

Interpretation: the extra multiplier appears as the probe moves up the
conversion-control stack. A single `REWRCONV` failure is only about `3-4x`.
The 19-rule loop is `4.7-4.8x`, right-associated `ORELSEC` is `5.2-5.4x`,
and `REPEATC` / `TRYCONV` pushes the same target to `6.4x`. Fresh wrapper
construction is not the issue: fresh and prebuilt `TRYCONV[REPEATC[baseConv]]`
are essentially equal. The next useful split is therefore inside rum's handling
of repeated nested conversion-control calls, especially the cost difference
between raw `baseConv` failure, `REPEATC` returning `REFL[x]` after failure,
and `TRYCONV` catching an already-successful `REPEATC`.

The next control-stack split narrowed that layer:

- Added
  `scripts/perf-probes/pair-simp/repeatc-tryconv-control-split.wls`.
- It keeps the same 19-rule `baseConv` and measures real `REPEATC`, hand-written
  `REPEATC` failure branches, real `TRYCONV`, hand-written `TRYCONV`, raw
  `Catch` over a successful `REPEATC`, `REFL`, `SameQ`, and `MatchQ` on a
  successful theorem.
- A standalone bare `Return[REFL[x]]` micro-segment was rejected because
  `Return` inside the HoldAll measurement wrapper did not produce reliable
  probe output. The retained comparison is inside realistic `REPEATC`-shaped
  branches: with `Return` versus without `Return`.

Validation commands:

```bash
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only repeatc-tryconv-control-split --timeout 180 --output-dir /tmp/rum-repeatc-tryconv-control-split.rum.clean --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only repeatc-tryconv-control-split --timeout 180 --output-dir /tmp/rum-repeatc-tryconv-control-split.wolfram.clean --engine wolfram
```

Selected `N=200` totals:

| segment | rum | WolframKernel | ratio |
| --- | ---: | ---: | ---: |
| raw `tryConv[baseConv[x]]` failure | `1.875s` | `0.348s` | `5.4x` |
| `TRYCONV[baseConv][x]` failure to `REFL[x]` | `2.105s` | `0.361s` | `5.8x` |
| `REPEATC[baseConv][x]` failure to `REFL[x]` | `2.133s` | `0.360s` | `5.9x` |
| manual `REPEATC` branch with `Return[REFL[x]]` | `2.316s` | `0.364s` | `6.4x` |
| manual `REPEATC` branch without `Return` | `1.902s` | `0.367s` | `5.2x` |
| direct `REFL[x]` | `0.0317s` | `0.0089s` | small absolute |
| actual `TRYCONV[REPEATC[baseConv]][x]` | `2.377s` | `0.372s` | `6.4x` |
| manual outer `TRYCONV` via `tryConv` | `2.237s` | `0.367s` | `6.1x` |
| manual outer `TRYCONV` inline `Catch`/`MatchQ` | `2.120s` | `0.366s` | `5.8x` |
| raw `Catch[REPEATC[baseConv][x]]` success | `2.117s` | `0.364s` | `5.8x` |
| `MatchQ[successThm, _Failure]` | `0.0010s` | `0.0002s` | negligible |
| `Catch[prebuiltSuccessThm]` | `0.0014s` | `0.0001s` | negligible |

Interpretation: the newly isolated small pieces are not the owner. `REFL[x]`,
`SameQ[$convFailed, $convFailed]`, `MatchQ` on a successful theorem, and
`Catch` returning a prebuilt successful theorem are all too small. The retained
hot layer is realistic nested conversion-control application: real
`REPEATC`/`TRYCONV` definitions around an already expensive baseConv failure,
plus `Return` in the failed `REPEATC` branch. The strongest next runtime-side
probe is no longer a WL-level split; it should instrument rum's definition
application / subvalue dispatch / nonlocal `Return` path for the concrete
symbols ``HOL`Drule`REPEATC`` and ``HOL`Drule`TRYCONV`` under this probe.

Runtime-path probe follow-up:

- Added
  `scripts/perf-probes/pair-simp/runtime-profile-repeatc-tryconv-control-split.wls`.
  It runs the same 19-rule `baseConv` control-stack target with
  `RumProfileReset[]` / `RumProfileSnapshot[]` around each label, so profile
  output is not mixed with preheat or sibling segments.
- Added profile-only rum runtime timing rows for call dispatch stages,
  definition RHS evaluate/finish, `Module` resume/body evaluation,
  `Catch` body/resume, `Return`/`Throw`, and `Function` evaluate/finish.
  These rows are emitted only under the existing runtime profile path and are
  reset by `RumProfileReset[]`.

Validation:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=1 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_DEFINITION_RULE_TIMING=1 RUM_PROFILE_BUILTIN_TIMING=1 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-repeatc-tryconv-control-split --timeout 240 --output-dir /tmp/rum-repeatc-tryconv-control-split.runtime-path --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

Selected `N=80` profile-target wall times under the new instrumentation:

| segment | rum |
| --- | ---: |
| raw `tryConv[baseConv[x]]` failure | `1.009s` |
| `TRYCONV[baseConv][x]` failure to `REFL[x]` | `1.127s` |
| `REPEATC[baseConv][x]` failure to `REFL[x]` | `1.167s` |
| manual `REPEATC` branch with `Return[REFL[x]]` | `1.019s` |
| manual `REPEATC` branch without `Return` | `1.039s` |
| actual `TRYCONV[REPEATC[baseConv]][x]` | `1.229s` |
| manual outer `TRYCONV` via `tryConv` | `1.166s` |
| manual outer `TRYCONV` inline `Catch`/`MatchQ` | `1.177s` |
| raw `Catch[REPEATC[baseConv][x]]` success | `1.460s` |

Runtime profile conclusions from the same run:

- Definition dispatch selection is not the owner. Across the control-stack
  labels, `definition-dispatch-time summary` stays around `0.06s` total
  (`prefilter` + `fast_match` + RHS substitution) while the measured label
  wall is about `1.0-1.5s`.
- Nonlocal `Return` itself is not the owner. The real `REPEATC` and manual
  `Return` labels record the expected `return_err` flow through
  `call.stage.special_form`, `definition.rhs.evaluate`, `module.body.evaluate`,
  and `module.resume_initializers`, but `Return` builtin self time is
  micro-scale and the manual-with-`Return` branch is not slower than the
  manual no-`Return` branch in this run (`1.019s` vs. `1.039s`).
- The realistic hot parent is still the failed conversion chain. The top RHS
  rows remain ``HOL`Drule`ORELSEC`` (1539 failing calls per snapshot),
  ``HOL`Drule`Private`tryConv`` (1620-1701 calls), then
  ``HOL`Auto`Simp`Private`simpRuleConv`` and ``HOL`Drule`REWRCONV``.
- The largest non-intrusive runtime self rows are not matcher dispatch but
  frame/preparation overhead: `module.resume_initializers` self is about
  `0.33-0.37s`, `call.stage.down_values` self about `0.17-0.26s`, and
  `call.stage.prepare_arguments` self about `0.13-0.16s` on the main labels.
  The lexical summary confirms that most `module.resume_initializers` self is
  body substitution: `module_body_substitution_total_ns` is about
  `0.32-0.36s` over roughly `9.3k-9.9k` module bodies.

Current interpretation: `REPEATC` / `TRYCONV` are not hiding a special rum
`Return` or subvalue-dispatch bug. They amplify the same repeated failure
chain by adding many ordinary `Module` frames, argument preparation, and
downvalue/subvalue calls. The next useful split is still runtime-side but
should drill below these new self rows: separate `prepare_arguments` evaluation
from cheap argument normalization, and split `Module` body substitution by
exact source shape/source symbol if profile disturbance remains tolerable.

Prepare-arguments split follow-up:

- Added profile-only runtime rows inside
  `Session::prepare_arguments_for_call` and
  `Session::normalize_call_args_by_attributes`: `prepare_arguments.total`,
  `prepare_arguments.plan_arg`, `prepare_arguments.evaluate_arg`,
  `prepare_arguments.append_arg`, `prepare_arguments.normalize.total`,
  `prepare_arguments.normalize.flat`, and
  `prepare_arguments.normalize.orderless`.
- The split is diagnostic only. It records one event per prepared argument and
  therefore adds visible observation overhead; use it to read same-run phase
  proportions, not as a no-profile timing baseline.

Validation:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=1 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_DEFINITION_RULE_TIMING=1 RUM_PROFILE_BUILTIN_TIMING=1 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-repeatc-tryconv-control-split --timeout 240 --output-dir /tmp/rum-repeatc-tryconv-control-split.prepare-args-20260620a --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

Selected runtime-path rows from the same `N=80` run:

| segment | calls | self time | average |
| --- | ---: | ---: | ---: |
| `prepare_arguments.evaluate_arg` | `161k-184k` | `0.117-0.150s` | `724-845ns` |
| `prepare_arguments.total` wrapper/loop self | `108k-121k` | `0.089-0.101s` | `818-863ns` |
| outer `call.stage.prepare_arguments` self | `108k-121k` | `0.040-0.045s` | `367-386ns` |
| `prepare_arguments.plan_arg` | `239k-265k` | `0.013-0.015s` | `55-59ns` |
| `prepare_arguments.append_arg` | `239k-265k` | `0.011-0.013s` | `47-49ns` |
| `prepare_arguments.normalize.total` self | `111k-130k` | `0.005-0.007s` | `46-56ns` |
| `prepare_arguments.normalize.flat` | `3k-9k` | `0.0005-0.0011s` | `118-157ns` |
| `prepare_arguments.normalize.orderless` | `3k-9k` | `0.0004-0.0009s` | `94-122ns` |

Interpretation: cheap `Flat` / `Orderless` normalization is not the owner.
`prepare_arguments` is mostly a multiplier around recursive argument
evaluation plus high-frequency loop/control overhead. The next closed split
should drill below `prepare_arguments.evaluate_arg`, especially
`evaluate_atom_fast_path` and the call paths it re-enters under the same
control-stack probe, before trying another optimization. If optimizing this
area later, the measurable candidates are reducing repeated argument-prep
wrapper/loop overhead or bypassing no-op preparation for common unheld calls,
but those should wait for a closed child phase tree.

Evaluator fast-path split follow-up:

- Added profile-only rows for `evaluate_atom_fast_path.total` and its
  branch buckets: `stable_symbol`, `unstable_symbol`, `atom`, and `call`.
- Added `evaluate.total`, `evaluate.stability_check`,
  `evaluate.once_unstable`, `evaluate.fixed_point_equal`, and
  `evaluate.trace_update` around the fixed-point evaluator loop.
- Added `evaluate_inner_unstable.total`, `evaluate_inner_unstable.call`,
  `evaluate_inner_unstable.symbol`, and `evaluate_inner_unstable.atom`.
- Added a second follow-up split for the call branch:
  `evaluate_call.total`, `evaluate_call.head_eval`,
  `evaluate_call.resume_after_head`, `evaluate_call.build_dispatch_plan`, and
  `evaluate_call.dispatch`.

Validation:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=1 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_DEFINITION_RULE_TIMING=1 RUM_PROFILE_BUILTIN_TIMING=1 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-repeatc-tryconv-control-split --timeout 240 --output-dir /tmp/rum-repeatc-tryconv-control-split.eval-fastpath-20260620a --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
env RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_LEXICAL_DEFER=0 RUM_PROFILE_LEXICAL_DETAIL=0 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 RUM_PROFILE_LEXICAL_SOURCES=1 RUM_PROFILE_EVALUATION_CACHE=0 RUM_PROFILE_EVALUATION_CACHE_DEFER=0 RUM_PROFILE_DEFINITION_RULE_TIMING=1 RUM_PROFILE_BUILTIN_TIMING=1 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-repeatc-tryconv-control-split --timeout 240 --output-dir /tmp/rum-repeatc-tryconv-control-split.eval-call-20260620a --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

The extra evaluator probes are high frequency. The two profiled runs completed
in `103s` and `112s`; use their same-run self/inclusive relationships, not
their absolute wall times, as the evidence.

Selected `N=80` self rows across the control-stack labels:

| segment | calls | self time | average |
| --- | ---: | ---: | ---: |
| `prepare_arguments.evaluate_arg` | `162k-184k` | `0.028-0.032s` | `172-181ns` |
| `evaluate_atom_fast_path.call` | `124k-144k` | `0.020-0.024s` | `162-172ns` |
| `evaluate_atom_fast_path.stable_symbol` | `151k-165k` | `0.0056-0.0063s` | `36-38ns` |
| `evaluate_atom_fast_path.unstable_symbol` | `21k-22k` | `0.0035-0.0038s` | `158-175ns` |
| `evaluate.stability_check` | `431k-468k` | `0.103-0.150s` | `239-325ns` |
| `evaluate_call.dispatch` | `151k-165k` | `0.093-0.106s` | `616-660ns` |
| `evaluate_call.head_eval` | `151k-165k` | `0.030-0.034s` | `199-210ns` |
| `evaluate_call.build_dispatch_plan` | `151k-165k` | `0.016-0.018s` | `105-112ns` |
| `evaluate.fixed_point_equal` | `161k-176k` | `0.012-0.014s` | `74-80ns` |

Interpretation: `prepare_arguments.evaluate_arg` itself is no longer a hot
leaf. Its inclusive time is mostly the ordinary call path reached through
`evaluate_atom_fast_path.call`. Stable-symbol and non-symbol atom fast paths
are cheap. Dispatch-plan construction is also not the owner. The remaining
runtime-side evaluator leaves worth splitting are:

- `evaluate_call.dispatch`, which is the wrapper around
  `resume_call_with_dispatch_plan` after child call-stage timing is subtracted;
  this likely captures dispatch recursion/skip/drive glue rather than rule
  matching itself.
- `evaluate.stability_check`, which is now a visible independent leaf at about
  `0.10-0.15s` per snapshot under this high-frequency probe.

Do not optimize either yet. The next probe should split
`resume_call_with_dispatch_plan` into skip, stage execution, `drive_internal`
continuation, and recursion overhead, or separately split
`cached_expr_is_evaluation_stable` if the goal is the stability-check leaf.

## 2026-06-19 Pair/Simp Lexical Presence Cache Cut

Turned the current root-level lexical absence-scan diagnosis into a rum-side
runtime cut. The clean `mma-hol` target stayed unmodified.

The retained implementation adds a per-substitution lexical presence cache for
`arg_prefilter` scans. The cache is scoped to a single
`substitute_lexical_bindings` call and is keyed by the immutable shared call
node identity `(SharedExpr head Arc, SharedArgs Arc)`, so it avoids a global
raw-pointer cache and cannot observe pointer reuse after the substitution
returns. Both single-binding and multi-binding prefilters use the cache.

Rejected experiments from the same A/B pass:

- A no-initializer `Module` fast path was too small and noisy. Keeping the
  fresh-symbol definition-table registration but skipping the initializer
  recursion moved `pair4-stock-repeat8` only from the same-machine baseline
  `18.24s` to about `17.98-18.09s`, while pair-depth did not consistently
  improve. Skipping the registration itself was worse.
- A shadowed-only lexical early exit did not hit the root path. The
  `pair4-stock-repeat8` probe stayed around `18.51-18.57s`, confirming that
  the hot false scans are ordinary closed conversion arguments, not subtrees
  already fully shadowed by nested scopes.

Validation:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
/Users/fft/Developments/rum/scripts/run-bounded.py --timeout 180 -- /Users/fft/Developments/rum/target/release/rum-cli --file /Users/fft/Developments/rum/scripts/perf-probes/pair-simp/runtime-profile-pair4-stock-repeat8.wls
/Users/fft/Developments/rum/scripts/run-bounded.py --timeout 160 -- /Users/fft/Developments/rum/target/release/rum-cli --file /Users/fft/Developments/rum/scripts/perf-probes/pair-simp/runtime-profile-pair-depth-chain-basic.wls
RUM_PROFILE_DEFINITION_RULE_TIMING=1 RUM_PROFILE_BUILTIN_TIMING=1 RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0 scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only runtime-profile-pair4-stock-repeat8 --timeout 240 --output-dir /tmp/rum-pair-repeat8-profile.presence-cache-multi --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

Measured release results on the same clean clone:

- Same-machine baseline immediately before the cut:
  `pair4-stock-repeat8` `18.24s`.
- Single-binding-only presence cache: `pair4-stock-repeat8` `16.91s` and
  `17.20s`; `presence_single_nodes` fell from `606.8M` to `75.1M`.
- Single + multi presence cache: `pair4-stock-repeat8` `15.22s`, `15.56s`,
  and `15.45s`; pair-depth `pair4` fell to about `1.93s`.
- Low-disturbance profile target:
  `runtime-profile-pair4-stock-repeat8` target segment fell from `31.92s`
  (`/tmp/rum-pair-repeat8-profile.baseline`) to `28.59s`
  (`/tmp/rum-pair-repeat8-profile.presence-cache-multi`).
- Lexical totals in that profile moved as expected:
  `lexical_substitution_total_ns` `10.01s -> 6.77s`,
  `presence_single_nodes` `606.8M -> 75.1M`,
  `presence_multi_nodes` `178.3M -> 13.5M`, and
  the `HOL`Drule`ORELSEC` single `arg_prefilter` node volume
  `400.7M -> 28.8M`.

This is a real constant-factor runtime cut, not a new semantic cache. It still
does not by itself prove the full `Pair.wl` gate is closed.

Root retest after the cut:

```bash
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstConv --timeout 600 --output-dir /tmp/rum-pair-firstconv.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only pair-load-root-phase --timeout 900 --output-dir /tmp/rum-pair-load-root.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only pair-load-root-phase --timeout 180 --output-dir /tmp/rum-pair-load-root.wolfram-current --engine wolfram
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstConv --timeout 180 --output-dir /tmp/rum-pair-firstconv.wolfram-current --engine wolfram
```

- rum first `mkPairInj` `CONVRULE` root: summary wall `200s`, closed Pair body
  phase sum `193.83s`; `mkPairInj-first-simp-apply-CONVRULE` is still the
  dominant child at `188.07s` (~97% of the Pair body phase sum). Theorem
  markers: `mkPairInjThm=True`, `pairInjThm=True`.
- WolframKernel first `mkPairInj` `CONVRULE` root: summary wall `41s`, Pair
  body phase sum `38.36s`, `mkPairInj-first-simp-apply-CONVRULE` `37.24s`.
  The same dominant child is now about `5.05x` slower in rum.
- rum dependency-preloaded `Pair.wl` root: summary wall `194s`,
  `root-preload-simp-stack` `0.64s`, `root-get-stdlib-pair` `188.27s`.
  Theorem markers: `mkPairInjThm=True`, `pairInjThm=True`.
- WolframKernel dependency-preloaded `Pair.wl` root: summary wall `40s`,
  `root-preload-simp-stack` `0.22s`, `root-get-stdlib-pair` `37.66s`.
  The current full-root gap is therefore about `5.0x`, not the old `14x+`
  hundreds-of-seconds wall.

The next Pair/Simp step is to rebuild the closed root phase tree from the
remaining dominant child, `mkPairInj-first-simp-apply-CONVRULE`, with
no-profile baselines preserved for every descent.

Follow-up `mkPairInj-first-simp-apply-CONVRULE` descent, same clean clone and
same release binary, kept the standard target unmodified and used only
temporary instrumented `Pair.wl` copies under `/tmp`:

```bash
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstConvRule --timeout 600 --output-dir /tmp/rum-pair-firstconv-rule.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstSimpFixpoint --timeout 600 --output-dir /tmp/rum-pair-first-simp-fixpoint.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstSimpStep --timeout 600 --output-dir /tmp/rum-pair-first-simp-step.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstDescendOnce --timeout 600 --output-dir /tmp/rum-pair-first-descend-once.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstDescendF --timeout 600 --output-dir /tmp/rum-pair-first-descend-f.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstDescendFX --timeout 600 --output-dir /tmp/rum-pair-first-descend-fx.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstDescendFXDescend --timeout 600 --output-dir /tmp/rum-pair-first-descend-fx-descend.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPathFXF --timeout 600 --output-dir /tmp/rum-pair-first-path-fxf.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path f/x/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-fxfb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path f/x/f/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-fxffb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path x/b --timeout 600 --output-dir /tmp/rum-pair-first-path-xb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path f/x/f/f/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-fxfffb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path x/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-xfb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path f/x/f/f/f/f/a/b --timeout 600 --output-dir /tmp/rum-pair-first-path-fxffffaB.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

Closed tree summary, rounded to two decimals where useful:

- `CONVRULE` expansion: `mkPairInj-first-simp-call-conv` is `189.77s`; target
  conclusion, `simpConv` construction, and `EQMP` are tiny.
- `simpConv` fixpoint: `first-step` is `159.84s`, `withBasic` `24.82s`,
  fixpoint loop `6.23s` with one iteration. The first step remains the owner.
- First `simpStepConv`: `descendOnce` is `164.26s`; top `REPEATC`,
  `TRANS`, and bookkeeping are tiny.
- First `descendOnce` on the top `Comb`: `comb-f` `95.63s` and `comb-x`
  `69.86s`; both are real hot siblings.
- Outer `comb-f`: its hot child is `f/x`, not `f/f`; `f/x` is `94.19s`,
  `f/f` is about `0.05s`, and outer `comb-x` remains about `73.70s`.
- `f/x`: its hot child is another `descendOnce` at `89.59s`; splitting that
  names `f/x/f` at `88.88s`, while `f/x/x` is tiny. The outer `x` sibling
  remains about `69.58s`.
- `f/x/f`: its unsplit `descendOnce` is `88.01s`. Splitting one level deeper
  gives `f/x/f/f` `73.61s`, `f/x/f/x` tiny, and the outer `x` sibling about
  `67.91s`.
- `f/x/f/f`: its hot child is `f/x/f/f/f` `61.26s`, with its own fixpoint
  loop about `18.89s`; the right sibling is tiny. At this depth the other hot
  top sibling is still outer `x` at about `71.79s`.
- Outer `x`: its hot child is `x/f`, specifically `x/f/f-descendOnce`
  `54.16s` plus a `8.78s` local loop; `x/x` is tiny.
- The left-chain `f/x/f/f/f/f` reaches an `Abs` shape. The expensive part is
  not `freesIn`, `pickFresh`, or `BETA-open` (all sub-millisecond here), but
  the opened inner body: nested `abs/abs-descendOnce` is `28.31s`, plus
  nested fixpoint loops of about `4.88s`, `9.93s`, and `18.69s`.

Current interpretation: the remaining `CONVRULE` wall is no longer one opaque
runtime primitive. It is a small number of repeated `simpFixpointConv` /
`descendOnce` traversals over structurally similar HOL `Comb` / nested `Abs`
subterms. The next probe should continue from the two retained hot leaves:
`x/f/f-descendOnce` and the nested `abs/abs-descendOnce` under
`f/x/f/f/f/f/a/b`, checking whether the same repeated conversion-tree rebuild
and failed-dispatch pattern remains dominant inside those leaves.

Additional split on the outer `x` sibling, still no downstream edits:

```bash
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path x/f/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-xffb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path x/f/f/f/b --timeout 600 --output-dir /tmp/rum-pair-first-path-xfffb.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
bash scripts/pair-simp-pair-body-phase-tree.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --detail mkPairInjFirstPath --path x/f/f/f/f/a/b --timeout 600 --output-dir /tmp/rum-pair-first-path-xffffab.presence-cache --rum-cli /Users/fft/Developments/rum/target/release/rum-cli
```

- `x/f/f/b` keeps the Pair body around `194.89s`. The target continues down a
  left comb chain: `x/f/f/f-descendOnce` is `35.71s`, its local fixpoint loop
  is `16.60s`, and the outer sibling is still about `91.34s`.
- `x/f/f/f/b` keeps the Pair body around `189.03s`. The hot child is an `Abs`
  at `x/f/f/f/f`: `descendOnce` is `16.39s` and its local fixpoint loop is
  `18.08s`; the right sibling is tiny.
- `x/f/f/f/f/a/b` confirms again that `Abs` housekeeping is not the owner:
  `freesIn`, fresh-name selection, `BETA-open`, and `ABS` are sub-millisecond
  to millisecond-scale. Opening exposes nested Abs bodies where
  `abs/abs-descendOnce` is `3.99s`, with enclosing fixpoint loops of about
  `4.03s`, `8.10s`, `16.13s`, and `17.93s`.

Stage-level profile snapshots then separated the main repeated conversion
path from cold theorem-preheat side evidence:

- Profiling `x/f/f/f-descendOnce` under
  `/tmp/rum-pair-first-path-xffb.presence-cache` confirmed the same
  conversion/failure amplifier: `ORELSEC`/`tryConv`/`REWRCONV`/
  `simpRuleConv` still produce many failed conversion calls; call-stability
  shows about `124.5M` checks and `42.7M` misses with no pointer misses, split
  between absent entries and dirty/generation invalidation; lexical presence
  scans are still material but no longer the only owner.
- Profiling the nested
  `f/x/f/f/f/f/a/b/a` `abs/abs/abs-descendOnce` leaf showed a different
  side lane: it mainly hit cold `andCongLemma[]` / `PropTaut` construction.
  That is valid evidence for local cost, but not the retained explanation for
  the repeated Pair root multiplier.
- Profiling the smaller
  `x/f/f/f/f/a/b` `abs/abs-descendOnce` leaf under
  `/tmp/rum-pair-first-path-xffffab.presence-cache` restored the main pattern
  at a smaller scale. The profiled target was `7.95s`; it reported
  `4384` failing `ORELSEC` sub-value RHS evaluations, `4280` failing
  `simpRuleConv` and `REWRCONV` evaluations, and builtin failure rows headed
  by `Module`/`If` under `HOL`Drule`ORELSEC``. Evaluation-cache counters show
  `14.7M` stability checks and `5.05M` misses, with dirty/generation misses
  dominating and no pointer misses. Rule-RHS timing concentrates in
  conversion call-argument rebuilding: `REWRCONV` call args `243,960` calls,
  `collectAppSpine` `88,164`, `tryTermMatch` `74,286`,
  `simpRuleConv` `68,480`, `ORELSEC` `39,456`, and `tryConv` `38,496`.
  This is the current strongest evidence that repeated failed conversion
  dispatch plus call-stability invalidation, not Abs opening, remains the
  rum-side multiplier inside the reduced leaf.

An attempted full shape/rebuild profile for `x/f/f/f-descendOnce` timed out at
`900s` without a completed snapshot, so full lexical shape profiling is too
disturbing at that granularity. Keep future rebuild probes narrower, or attach
shape/fingerprint accounting only after the no-profile phase has been reduced
below this leaf size.

## 2026-06-19 Pair/Simp Runtime Sharing Cut

Turned the first production runtime optimization from the recent Pair/Simp
diagnosis into code, without changing the unmodified `mma-hol` acceptance
target.

Retained runtime cuts:

- `Catch` now bypasses the general matcher for already-resolved atomic tag
  forms, while still evaluating the tag form before matching so visible side
  effects and pattern forms keep the previous semantics.
- `normalize_match_expr` now returns the original shared expression when
  recursive normalization and Flat/Orderless argument normalization make no
  semantic change. This preserves `SharedExpr` / `SharedArgs` identity for
  ordinary calls instead of rebuilding fresh call trees during failed
  conversion dispatch paths.
- Added
  `scripts/perf-probes/pair-simp/runtime-profile-basic-only-pair4.wls` as a
  focused low-disturbance runtime profile target around the basic-rule
  `pair4` no-op simplification.

Validation:

```bash
cargo fmt --check
cargo test -p rum-runtime
cargo build --release -p rum-cli
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only basic-only-pair4 --timeout 180 --output-dir /tmp/rum-pair-simp-basic-only.normalize.A1
scripts/pair-simp-hotspot-probes.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --only pair4-stock-repeat8 --timeout 180 --output-dir /tmp/rum-pair-simp-pair4-repeat8.normalize.A1
scripts/pair-simp-repetition-probe.sh --mma-hol /private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol --timeout 900 --output-dir /tmp/rum-pair-repeat.normalize.A1
```

Measured release results on the same clean clone:

- `basic-only-pair4-simpFixpointConv`: about `5.68-6.36s` before this cut,
  now `2.23s`; theorem marker `True`, changed marker `False`.
- `pair4-stock-repeat8`: target segment `60.76s` before this cut, now
  `18.30s`; theorem marker `True`, changed marker `False`.
- first `mkPairInj` `CONVRULE` repetition root: previous production baseline
  about `564s`, now `307s`.
- Low-disturbance profile target: about `6.36s` before, now `2.99s`.
  `module_body_substitution_total_ns` fell from about `1.59s` to `1.00s`;
  cumulative `eval_module_total_ns` fell from about `130.63s` to `65.49s`.
  Call-stability absent misses fell from `2,188,789` to `940,263`, confirming
  that preserving shared call trees lets existing cache paths hit more often.

This does not close the Pair gate, but it validates the current rum-side
frontier: avoid downstream `mma-hol` patches and avoid unsafe text-fingerprint
semantic caches; continue with conservative representation/sharing cuts or
closed root-level splits that further reduce repeated failed conversion
dispatch and lexical absence-scan churn.

## 2026-06-18 Pair/Simp Abs-And Cold-Cache Split

Continued the no-optimization Pair/Simp root phase tree against the unmodified
clean clone at `/private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol`, using the
diagnostic copy-based runner `scripts/pair-simp-pair-body-phase-tree.sh`.

The deep `mkPairInj` abs path now has a concrete correction:

- `f/x/f/f/f/f/a/a/a` showed the previous `abs-abs-descendOnce` continuing
  into `abs-abs-abs-descendOnce` at about `30.96s`, with primitive abs
  housekeeping (`freesIn`, fresh-name pick, `BETA`, `ABS`) still below
  millisecond scale.
- `f/x/f/f/f/f/a/a/a/a` showed that the next layer is no longer another abs
  opener. Four abs openings expose an `isAnd=True`, `isComb=True` body, and
  the large block became `abs-abs-abs-abs-descendOnce` at about `29.54s`.
- Adding a real `doAndCong` split for that body first produced an apparent
  `and-INST` block around `23.06s`. After splitting `and-vars`,
  `and-lemma`, and `and-INST`, the block was correctly attributed to the first
  lazy `andCongLemma[]` computation: `and-lemma` about `24.44s`, real
  `and-INST` about `0.00047s`, `and-MP` about `0.03s`, and p/q child
  simplification below `1s` each.

Interpretation: `andCongLemma[]` is a WL memoized cold-cache cost
(`andCongLemma[] := andCongLemma[] = computeAndCongLemma[]`), not a repeated
steady-state hotspot. Preheating or snapshotting it can improve warm-session
developer workflows, but it only moves time inside a full cold Pair load and
cannot explain a tenfold repeated Pair/Simp slowdown. The remaining Pair-root
frontier is still the repeated recursive `simpFixpointConv` / `descendOnce`
chain, especially the loop1 fixpoint labels that stay in the `~4-8s` range
after cold lemma cost is peeled away.

Follow-up loop splits kept the same rule: do not optimize until the hot child
is closed and repeatedly hot.

- `/tmp/rum-pair-body-phase.QR1nSF` split the previous
  `...abs-loop1-abs-loop1-abs` fixpoint label. The old `~7.9s` residual fell
  to about `0.002s`; its cost moved into two recursive children around
  `3.7s` each.
- `/tmp/rum-pair-body-phase.MXYNOV` split the visible `~3.4-3.8s` children.
  Most of them decomposed into And p/q recursive child labels around
  `0.7-1.0s`, with `descendOnce` and the immediate follow-up fixpoint loop
  carrying comparable halves. Repeated `and-lemma` calls after the first were
  micro/millisecond-scale, confirming that the earlier `~24s` block was
  one-time memoization.

Current interpretation: this branch is recursive fanout through repeated
`simpFixpointConv` / `descendOnce` traversal, not an atomic `INST`, `ABS`,
`MKCOMB`, `TRANS`, `freesIn`, or theorem-plumbing hotspot. The next useful
probe should move back up the same closed Pair root tree and split the largest
remaining repeated parent/sibling phase rather than chasing one-time lemma
construction or sub-second leaf fragments.

The matching top `x`-side probe
`/tmp/rum-pair-body-phase.KCHkCj` used
`--path x/f/f/f/f/a/a/a/a/b` and returned cleanly in `563s` with empty stderr
and both theorem markers true. It split the previous top sibling block and
showed the same recursive shape:

- The opposite top sibling remained large (`...-comb-f-sibling` about
  `258.97s`), as expected for a one-sided split.
- The traced `x` branch had repeated fixpoint parent labels around `61.48s`,
  `59.27s`, `29.79s`, `29.76s`, `14.96s`, `14.91s`, `7.41s`, `7.39s`, and
  `3.65s`.
- Its deepest And p/q children were again about `0.79-0.89s` each. The
  `and-lemma` call was about `55us`, so the earlier cold-cache lemma cost was
  not repeated on this branch.

This keeps the current root-level hypothesis narrow: both major `mkPairInj`
sides spend time in repeated `simpFixpointConv` loop fanout over similar
logical-congruence subtrees. The next probe should split the visible `x`-side
loop parents themselves and verify whether the `~60s` and `~30s` blocks are
just sums of the same `~0.8s` leaves, or whether there is still a larger
unaccounted repeated operation inside the loop machinery.

Two follow-up `x`-side loop probes clarified that frontier:

- `/tmp/rum-pair-body-phase.t8eJcG` split the exact `x`-side loop parents.
  The largest newly exposed blocks were not theorem primitives; they were
  loop-iteration `descendOnce` calls over terms that had changed shape from
  combs to abs terms: about `34.70s` and `31.48s` under
  `...x-f-f-f-loop{2,1}-descendOnce`, plus `17.38s` and `16.00s` under
  `...x-f-f-loop{1,2}-descendOnce`.
- That revealed a probe limitation. The loop splitter reused the parent path,
  but after a fixpoint iteration the next target could become an abs term
  before the old `f/...` path was reached. The diagnostic runner now has an
  explicit `--loop-path-override` option for exact loop labels; it logs
  `loop-path-override=...` when used.
- `/tmp/rum-pair-body-phase.k3hfNg` reran the same target with overrides for
  the affected loop labels. It returned cleanly in `592s`, with empty stderr
  and theorem markers true. The former large `loop*-descendOnce` blocks were
  replaced by abs-subtree fixpoint parents: about `32.93s`,
  `15.75s`, `15.61s`, `15.59s`, `15.57s`, then `~7.7-7.8s` and `~3.8s`
  descendants, with many And p/q leaf fixpoints around `0.8-1.0s`.

Current interpretation after the override run: `descendOnce` was not yet an
atomic runtime hotspot; it was an under-split loop step caused by path drift
after the term shape changed. The open frontier is now the loop-prefixed abs
fixpoint parents, especially
`...x-f-f-f-f-loop1-abs-fixpoint-loop` and the matching `~15s` children. The
next probe should split those exact loop-prefixed abs labels before considering
any algorithm or runtime change.

Additional exact-label splits pushed the same `x`-side branch down another
level:

- `/tmp/rum-pair-body-phase.wSqe4m` split
  `...x-f-f-f-f-loop1-abs` and nearby `~15-32s` abs parents. The old
  `~32.9s` parent disappeared; the largest traced descendants became
  `...x-f-f-f-f-loop1-abs-loop1-abs-fixpoint-loop` at about `16.77s`, then
  several `~7.6s` abs-fixpoint parents and `~0.8-1.0s` And p/q leaves.
- `/tmp/rum-pair-body-phase.tXypOP` split that `~16.77s` parent and its next
  abs layers. The same branch's largest traced parents fell to `~7.5-7.6s`,
  with one nested And p child around `2.33s` and most direct leaves around
  `0.8-1.0s`.
- `/tmp/rum-pair-body-phase.WLWnTh` split the visible `~7s` parents. The
  remaining traced branch was mostly `~3.7-4.1s` abs-fixpoint parents plus
  many `~0.8-1.0s` And p/q children.
- `/tmp/rum-pair-body-phase.c7cf9p` split the next visible `~7s` labels.
  It returned cleanly in `577s`, with empty stderr and theorem markers true.
  The traced `x` branch now has no remaining `~7s` label among the split
  focus; the biggest traced labels are `~3.5-4.0s` abs-fixpoint parents and
  `~0.8-1.0s` child fixpoints/descend-once blocks. The full root still shows
  expected one-sided sibling phases such as top `comb-f-sibling` and
  `loop1-comb-f-sibling`.

Interpretation: this traced branch is now well characterized as regular
recursive fanout rather than a single hidden primitive or one missed semantic
gate. The next root-level probe should stop chasing this branch's `~3s` leaves
and instead split the remaining larger parent/sibling phases that still exceed
ten seconds: the root-loop sibling under `...loop1-comb-f-sibling`, the second
`CONVRULE`, and any matching top-level sibling not already covered by the
earlier `f`-side run.

The first return-to-parent probe,
`/tmp/rum-pair-body-phase.lP4YmA`, used a root-loop path override
(`...xffffaaaab -> b/f/f/f/f/a/a/a/a/b`) to split the previous
`...loop1-comb-f-sibling` block. It returned cleanly in `578s`, with empty
stderr and theorem markers true. The `~15s` sibling broke into
`...loop1-f-fixpoint-loop` at about `7.73s` and
`...loop1-f-comb-x-sibling` at about `7.53s`, with small plumbing around it.
This means the root-loop sibling is also recursive fanout, not a hidden
single expensive primitive.

Current frontier after these probes:

- The heavily traced `x` branch is no longer the right place to look for a
  tenfold loss; it is down to mostly `~3-4s` parents and `~0.8-1.0s` leaves.
- The root-loop sibling likewise decomposes to `~7s` halves.
- Remaining larger root-level phases that still need separate closed probes
  are the second `CONVRULE` (`~16s`) and any top-level sibling not already
  matched by the earlier `f`-side run. Do not interpret the intentionally
  one-sided top `comb-f-sibling` from an `x`-path probe as new evidence by
  itself; it is the untraced opposite branch.

## 2026-06-18 Pair/Simp Repetition Fingerprint Probe

Added `scripts/pair-simp-repetition-probe.sh` as a copy-based diagnostic
runner. It leaves the clean `mma-hol` checkout untouched, loads a temporary
instrumented `auto/Simp.wl`, and by default exits a temporary `Pair.wl` after
the first `mkPairInj` `CONVRULE`. The probe groups `simpFixpointConv` and
`descendOnce` calls by kind, depth, rule count, context count, term shape, and
SHA256 term fingerprint. `--track-step` can also count `simpStepConv`, but is
off by default to keep probe overhead bounded.

First run:

- checkout: `/private/tmp/rum-mma-hol-rootprobe.e6tD3J/mma-hol`
- output: `/tmp/rum-pair-repeat.MCGLrS`
- target: first `mkPairInj` `CONVRULE`
- elapsed wall: `594s`
- result markers: `first_convrule_is_thm=True`,
  `first_convrule_changed=True`
- aggregate rows: `101`

The largest rows are expected inclusive parents, not atomized findings:

- root `simpFixpointConv` on an `eq` fingerprint: `1` call, about `551s`
- root `descendOnce` on the same `eq` fingerprint: `1` call, about `528s`

The useful signal is below those parents. Repeated fingerprints are real:

- `simpFixpointConv`: `35` repeated rows, `3552` calls, about `3202s`
  inclusive; `13` single-call rows, about `2758s` inclusive
- `descendOnce`: `42` repeated rows, `7127` calls, about `2965s`
  inclusive; `11` single-call rows, about `2487s` inclusive

Top repeated inclusive groups include:

- `abs` fingerprint `0e5a...4910`: `simpFixpointConv` `22` calls, about
  `373s`; matching `descendOnce` `44` calls, about `370s`
- `and` fingerprint `e901...caef`: `simpFixpointConv` `44` calls, about
  `370s`; matching `descendOnce` `88` calls, about `364s`
- context-length `1` `eq` fingerprint `98c7...405b`: `simpFixpointConv`
  `108` calls, about `210s`; matching `descendOnce` `216` calls, about
  `193s`
- context-length `1` `comb` fingerprint `7361...0bfc`: `simpFixpointConv`
  `240` calls, about `179s`; matching `descendOnce` `480` calls, about
  `142s`
- context-length `0` `comb` fingerprint `8f79...7dfa`: `simpFixpointConv`
  `224` calls, about `156s`; matching `descendOnce` `448` calls, about
  `124s`

These totals are intentionally inclusive and nested, so they exceed wall time
when summed. The conclusion is not "optimize this fingerprint" yet; it is that
the Pair gate is indeed dominated by repeated recursive `simpFixpointConv` /
`descendOnce` work over a small set of logical subtrees. The next probe should
select one or two of the top repeated fingerprints, record a parent/child
closed tree or self-time split inside their representative calls, and only then
decide whether the fix is memoization, normal-form caching, or a lower-level
runtime representation/call-dispatch issue.

Follow-up split probes stayed on the same unmodified checkout and continued
the no-optimization rule. The probe runner now has targeted split modes for
`simpFixpointConv`, `simpStepConv`, `descendOnce`, and `topRepeatc`.

Valid split runs:

- `/tmp/rum-pair-repeat.2dHawL`: context-length `0` `abs`
  `0e5a...4910` split at `simpFixpointConv`. Its `22` calls were all one
  loop terminating with `same`; `buildBaseConv` and RHS extraction were
  negligible, while the two step calls per fixpoint carried the cost.
- `/tmp/rum-pair-repeat.NapkUF`: the same `abs` split at `simpStepConv`.
  `descendOnce` dominated the `44` step calls; `top-repeatc`, `TRANS`, and
  RHS extraction were small.
- `/tmp/rum-pair-repeat.HfIdjG`: the same `abs` split at `descendOnce`.
  Abs housekeeping was not hot; `abs-inner-simpFixpoint` dominated and always
  opened to `and e901...caef`.
- `/tmp/rum-pair-repeat.xqfqRi`: `descendOnce` for `and e901...caef`.
  After the first cold `andCongLemma[]` call, theorem plumbing was small.
  The repeated cost came from two child equalities:
  `p` = `eq 2e4...7c35` at context length `0`, and
  `q` = `eq 98c...405b` at context length `1`.
- `/tmp/rum-pair-repeat.Qzh0Al`: `simpFixpointConv` for `eq 98c...405b`.
  `108` calls summed to about `201.08s`: `initial-step` about `99.62s`,
  `loop1-step` about `100.17s`, `buildBaseConv` about `1.28s`, and RHS
  extraction negligible.
- `/tmp/rum-pair-repeat.JJXw4V`: `simpStepConv` for the same `eq`.
  `216` calls summed to `top-repeatc` about `14.95s`, `descendOnce` about
  `183.24s`, and negligible `TRANS`/RHS work.
- `/tmp/rum-pair-repeat.Vgazhb`: `descendOnce` for the same `eq`. It is the
  generic `comb` branch: `comb-f-simpFixpoint` about `153.50s`,
  `comb-x-simpFixpoint` about `29.54s`, `MKCOMB` about `0.10s`.
  The hot child is `comb 7361...0bfc`; the right child is a fixed var.
- `/tmp/rum-pair-repeat.gZYe4n`: `simpFixpointConv` for
  `comb 7361...0bfc`. `240` calls summed to `buildBaseConv` about `3.78s`,
  `initial-step` about `83.23s`, and `loop1-step` about `84.99s`.
- `/tmp/rum-pair-repeat.eaVMmX`: `simpStepConv` for the same `comb`.
  `480` calls summed to `top-repeatc` about `31.93s`, `descendOnce` about
  `135.53s`, and negligible `TRANS`/RHS work.
- `/tmp/rum-pair-repeat.Mpl6iz`: `descendOnce` for the same `comb`.
  `comb-f-simpFixpoint` was about `68.53s` on a fixed const leaf
  `2821...f288`; `comb-x-simpFixpoint` was about `67.58s` on a fixed var
  leaf `9e64...9190`; `MKCOMB` was about `0.21s`.
- `/tmp/rum-pair-repeat.JXWmL3`: `simpFixpointConv` for the fixed const
  leaf. `528` calls summed to `buildBaseConv` about `6.55s`,
  `initial-step` about `34.03s`, and `loop1-step` about `34.72s`.
- `/tmp/rum-pair-repeat.sdZw6k`: `simpStepConv` for the same const leaf.
  `1056` step calls put the leaf cost in `top-repeatc` at about `68.95s`;
  `descendOnce` was about `3.56s`, with `TRANS`/RHS below `1s`.
- `/tmp/rum-pair-repeat.qjUT5u`: valid `topRepeatc` sample for the same const
  leaf, `20` calls with `440` base-conv piece rows and empty stderr. The
  original right-associated `TRYCONV[REPEATC[baseConv]]` averaged about
  `53.94ms` per leaf call; the direct expanded piece loop averaged about
  `31.44ms`. Individual rule failures were usually `0.4-0.6ms` each, with
  occasional spikes on rule positions `4`, `10`, `14`, and `19`; beta fallback
  was about `0.067ms`.

Invalid `topRepeatc` probe runs, retained only as probe-debugging history:

- `/tmp/rum-pair-repeat.lbOpgT` returned in `27s` because the first
  instrumented no-conversion path called ``HOL`Error`holError``, which throws
  under rum. This was a probe bug, not a performance result.
- `/tmp/rum-pair-repeat.QXHBxP` reached full wall time but treated the private
  failure sentinel as a successful piece result, so it only measured `ctx-1`.
  This was also a probe bug and should not be used for performance conclusions.

Current hot-tree interpretation:

```text
root first mkPairInj CONVRULE, root eq 2299...21d9 (~530s)
  descendOnce root eq (~507-513s)
    abs 0e5a...4910 repeated fixpoint (~358s)
      opened and e901...caef
        q child eq 98c...405b, ctxLen=1 (~200s)
          descendOnce eq (~183-190s)
            f child comb 7361...0bfc (~153s inside this eq)
              descendOnce comb (~135-137s)
                const leaf 2821...f288 fixpoint (~66-74s)
                  top-repeatc leaf failure scan (~69s over 1056 calls)
                var leaf 9e64...9190 fixpoint (~66-68s)
```

This is the first atomic-looking repeated hotspot in the current branch:
leaf-level `top-repeatc` repeatedly scans a 1-context + 20-rule + beta
converter chain and fails without changing a fixed const/var. The raw rule
matcher failures do not alone explain the full original `ORELSEC` cost; the
difference between original `top-repeatc` and the direct expanded loop points
at right-associated `ORELSEC` / repeated `tryConv` / failure propagation
overhead as the next probe target. The sibling var leaf should still be
checked with the same `simpFixpointConv`/`simpStepConv`/`topRepeatc` sequence
before changing algorithms.

The sibling var leaf check was then completed and matched the const-leaf
shape:

- `/tmp/rum-pair-repeat.wUt4Ex`: `simpFixpointConv` for fixed var leaf
  `9e64...9190`. `480` calls all terminated after one loop with `same`.
  `buildBaseConv` was about `6.64s`; `initial-step` about `31.22s`;
  `loop1-step` about `32.25s`; RHS extraction was negligible.
- `/tmp/rum-pair-repeat.T5BIi5`: `simpStepConv` for the same var leaf.
  `960` step calls put the cost in `top-repeatc` at about `61.39s`;
  `descendOnce` was about `3.45s`; `TRANS` and RHS extraction were small.
- `/tmp/rum-pair-repeat.fidk64`: `topRepeatc` sample for the same var leaf.
  `20` calls had `440` base-conv piece rows and empty stderr. The original
  right-associated `TRYCONV[REPEATC[baseConv]]` averaged about `54.13ms`; the
  direct expanded piece loop averaged about `28.83ms`. Individual rule
  failures were again mostly `0.4-0.6ms`, with occasional spikes on rule
  positions `11` and `15`; beta fallback was about `0.064ms`.

This confirms both sibling leaves share the same repeated leaf-level failure
scan shape.

Two internal propagation probes then isolated the original chain overhead more
directly:

- `/tmp/rum-pair-repeat.zYQrXN`: `topRepeatcTrace`, `5` samples on the var
  leaf. This emulates a right-associated `ORELSEC` recursion over the same
  pieces. The original unwrapped path averaged about `50.16ms`, the direct
  expanded loop averaged about `32.81ms`, and the right-associated emulation
  averaged about `56.65ms`. The emulated `ORELSEC_REST` inclusives decreased
  from about `51.52ms` at level `1` to about `1.90ms` at level `21`.
- `/tmp/rum-pair-repeat.egQmMP`: `topRepeatcNativeTrace`, `5` samples on the
  same var leaf. This temporarily wraps the public ``HOL`Drule`ORELSEC``,
  ``HOL`Drule`REPEATC``, and ``HOL`Drule`TRYCONV`` in the diagnostic copy
  only, with tracing active only inside the matched leaf step. The run
  completed with both theorem markers true and empty stderr. Because the
  wrapper itself adds overhead, the native traced top-repeatc averaged about
  `73.46ms`, but the shape is informative: `REPEATC`'s private `tryConv`
  around the whole baseConv averaged about `56.41ms`, the outer public
  `TRYCONV` averaged about `71.35ms`, and native `ORELSEC_REST` inclusives
  decreased from about `43.23ms` at level `1` to about `0.058ms` at level
  `21`.

Current interpretation after the native trace: the repeated leaf hotspot is a
failed right-associated converter chain. The matcher work is real but not the
whole cost; a large part is structural failure propagation through nested
`ORELSEC` plus `tryConv` catch boundaries around the whole chain. This is now
atomic enough to consider targeted changes, but they should be tested against
the same leaf probes and then the full Pair root. Plausible next experiments
are still probe-first: a flat `combineSimpConvs` diagnostic, a head/shape
prefilter before invoking rule conversions, or a failed-conversion cache for
stable atomic leaves. Do not change the downstream `mma-hol` standard.

Temporary prototype diagnostics then tested those two concrete mechanisms in
the same copy-based runner. These are not accepted fixes, because they rewrite
only the temporary instrumented `auto/Simp.wl` copy; the clean downstream clone
remained untouched.

- `/tmp/rum-pair-repeat.flat-baseconv.A1`: `--prototype flat-baseconv`,
  first `mkPairInj` `CONVRULE`, empty stderr, both first-convrule markers
  true. Wall time was `440s`, down from the earlier unprofiled/instrumented
  root around `594s`. This confirms right-associated `ORELSEC` failure
  propagation is a real root-level cost, but not enough by itself.
- `/tmp/rum-pair-repeat.failure-cache.A1`: `--prototype failure-cache`, same
  first-convrule target, empty stderr, markers true. Wall time was `253s`.
  The temporary cache stored `21` atomic leaf no-op top-repeatc entries and
  hit `5283` times, with `2` changed top-repeatc cases and `1832` non-atomic
  skips. This is the stronger signal: repeated no-op top-repeatc work over
  stable leaves is a major Pair/Simp multiplier.
- `/tmp/rum-pair-repeat.flat-failure-cache.A1`: combined temporary prototype,
  first-convrule target, empty stderr, markers true. Wall time was `176s`,
  with the same `21` entries and `5283` hits. The flat chain and leaf no-op
  cache are additive.
- `/tmp/rum-pair-repeat.flat-failure-cache-full.A1`: combined temporary
  prototype with `--target full-load`, empty stderr, and both
  `mkPairInjThm` / `pairInjThm` markers true. Full `Pair.wl` loaded in
  `183s`, with `30` cache entries and `5443` hits. This shows the first
  `mkPairInj` simplifier gate is still the dominant Pair root cost after the
  diagnostic cut.

The next real implementation should stay on the rum side: either a runtime
representation/dispatch cache that can skip provably stable repeated converter
failure/no-op calls, or an equivalent evaluator optimization for nested
`ORELSEC`/`TRYCONV` failure propagation. Do not upstream the temporary
`auto/Simp.wl` rewrites or treat them as the acceptance path. Since the cache
prototype produced the larger root win, a standalone `tryConv`/`Catch`
fast-path probe is lower priority than designing a correctness-preserving
rum-side cache key and invalidation rule.

## 2026-06-16 Pair/Simp Reproducible Hotspot Probe Assets

Stopped treating the current Pair/Simp wall as ready for optimization. The
working rule is now stricter: for every known hot shape, first create a stable
probe file that reproduces it against an unmodified `mma-hol`, then split the
called WL/runtime functions until the remaining hot spots are atomic enough to
name directly.

Added static probe assets under `scripts/perf-probes/pair-simp/` plus
`scripts/pair-simp-hotspot-probes.sh`. The runner uses
`scripts/run-bounded.py`, runs from the clean `mma-hol` checkout root, and
keeps probe code outside the downstream tree. The common prelude deliberately
uses fully qualified HOL symbols; an earlier draft accidentally parsed short
names before packages loaded, which produced a fake fast shape and was fixed
before retaining measurements.

Validation on `/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` with
`target/release/rum-cli`:

Completing outer reproducers:

| Probe | Target result |
| --- | ---: |
| `basic-only-pair4` | `5.63s`, theorem `True`, changed `False` |
| `pair4-stock-repeat8` | `51.64s` for 8 repeats, theorem `True`, changed `False` |
| `user-basic-pair2` | timeout at `120s` |

The timeout probe has empty stdout because `rum-cli` emits collected `Print`
output only after evaluation returns. It is useful as a bounded public-shape
non-completion reproducer, not as phase-level evidence. For phase evidence,
split into completing probes like the RHS body and RHS lambda files above.

Two probe correctness fixes matter for all later readings:

- `common.wls` must use fully qualified HOL symbols; short names parsed before
  package load bind to `Global`` and create fake fast probes.
- `impCongLemma`, `andCongLemma`, and `orCongLemma` are Private helpers in
  `auto/Simp.wl`, not public API. Congruence preheat is now an explicit
  `preheat-congruence` phase instead of being hidden in target timings.

Corrected opened-body split after separating one-time preheats:

| Probe / phase | Target result |
| --- | ---: |
| `preheat-basic` | about `22.3s` |
| `preheat-congruence` | about `64.7-65.2s` |
| logical left child `t[[1,2]]` | `1.25s`, changed `False` |
| logical right child `t[[2]]`, no context | `1.38s`, changed `False` |
| `doAndCong` left child | `1.29s`, changed `False` |
| `doAndCong` right child with left context | `1.46s`, changed `False` |
| `doAndCong` theorem plumbing | `0.10s`, changed `False` |
| original `doAndCong` / `descendOnce` | `3.19s`, changed `False` |
| full opened-body `simpFixpointConv` after preheat | `6.27s`, changed `False` |

Manual `simpFixpointConv` entry split for the same opened body closes over
that `6.27s`: `buildBaseConv` is about `0.01s`, the two top
`TRYCONV[REPEATC[baseConv]]` passes are about `0.04-0.05s` each, and the two
`descendOnce` calls are about `2.84s` and `3.09s`.

Corrected RHS-lambda depth split after preheating basic rules and congruence
lemmas:

| Remaining lambda shape | `descendOnce` time |
| --- | ---: |
| opened body | `2.89s` |
| one remaining abs | `6.64s` |
| two remaining abs | `14.42s` |
| three remaining abs | `29.59s` |
| four remaining abs | `60.43s` |

The abs-branch split shows the four-abs cost is not opening or theorem
plumbing: open/BETA is below `0.001s`, `ABS` is below `0.001s`, and the inner
`simpFixpointConv` on the three-abs term is `58.14s` versus `60.25s` for the
original `descendOnce`. Splitting that inner `simpFixpointConv` shows the same
pattern one level down: `buildBaseConv` about `0.009s`, top rewrite passes
about `0.04s`, and the two unchanged `descendOnce` calls about `28.00s` and
`29.81s`.

Current interpretation is still probe-only: the WL-level recursive multiplier
is now explicit. For unchanged lambda terms, `descendOnce` on an abs delegates
almost entirely to an inner `simpFixpointConv`; that fixpoint entry evaluates
two unchanged `descendOnce` passes to prove stability, producing an approximate
doubling at each binder level. The next probe-only work is to correlate the
remaining atomic-looking costs with rum runtime counters under these fixed
probes: lexical-substitution prefilter scans, module/closure construction, and
owned-tree churn inside those repeated unchanged `descendOnce` passes. Do not
implement optimizations until that runtime attribution is complete or a clear
semantic divergence is found.

## 2026-06-16 Pair/Simp Runtime-Layer Probe Baseline

Added runtime profile control for the fixed Pair/Simp probes without changing
normal evaluator behavior:

- `RumProfileReset[]` and `RumProfileSnapshot[label]` are inert unless a
  profile environment variable is enabled.
- `RUM_PROFILE_LEXICAL_DEFER=1` and
  `RUM_PROFILE_EVALUATION_CACHE_DEFER=1` keep profile code configured but
  inactive until `RumProfileReset[]`, so preheat phases do not dominate the
  measurement.
- The Pair/Simp `runtime-profile-*` runner path now enables lexical and
  evaluation-cache profile environments by default; set
  `RUM_PROFILE_LEXICAL_SHAPES=0 RUM_PROFILE_LEXICAL_BINDINGS=0` for the
  low-disturbance category/count mode. The runner disables evaluation-cache
  power-of-two incremental logs by default; set
  `RUM_PROFILE_EVALUATION_CACHE_INCREMENTAL=1` if those checkpoints are needed.

The first full lexical shape/binding profile for
`runtime-profile-opened-body-user-basic` timed out at `300s` before defer
control because profile mode covered package load and preheat. After adding
defer, the same full shape/binding profile completed in `130s`, but inflated
the target segment to `40.89s`. That run is useful for top shape/rebuild
attribution, not for wall-time comparison. Its largest rebuild groups were
large `rule_rhs_call` and `generic_call` rebuilds around
``HOL`Drule`ORELSEC``, ``HOL`Auto`Simp`Private`combineSimpConvs``, and
``HOL`Auto`Simp`Private`simpStepConv``.

Low-disturbance runtime profile mode keeps lexical detail timing but disables
shape/binding hash maps. Those runs stay close to the unprofiled WL-level
timings and are the comparable baseline:

| Probe target | Target time | RHS evals | Module evals | lexical substitutions | arg substitution attempts | presence-scan nodes |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| opened body `simpFixpointConv` | `6.36s` | `77,322` | `20,656` | `23,328` | `372,974` | `112,537,915` |
| three-abs `descendOnce` | `28.82s` | `302,101` | `81,609` | `91,911` | `1,486,709` | `455,967,571` |
| outer-abs inner `simpFixpointConv` | `59.91s` | `605,241` | `163,523` | `184,155` | `2,979,339` | `913,977,875` |

This closes the WL/runtime multiplier correlation: the same runtime categories
scale about `1x -> 4x -> 8x` across the opened-body, three-abs, and outer-inner
targets. The remaining question is therefore inside those repeated runtime
operations, not in a new larger input shape.

With evaluation-cache snapshot enabled on the opened-body target, the same
low-disturbance run measured `6.58s` for the target and reported:

- `973,755` evaluator entries, `70,252` down-value applications, `7,070`
  sub-value applications, `215,507` builtin applications, and `20,656`
  `Module` entries.
- `8,791,390` stability checks, `995,448` call-stability hits, `3,068,438`
  misses (`2,208,477` absent, `367,176` generation, `492,785` dirty), and
  `1,700,861` stable rewritten-dirty skips.
- `45,206` symbol invalidations, `311` dirty-bloom saturations, and `33`
  call-cache size-cap clears within the target segment.
- Top builtin applications: `Set` `39,402`, `CompoundExpression` `23,572`,
  `If` `21,436`, `Module` `20,656`, `MatchQ` `20,577`, and `Part` `20,445`.
- Top HOL down-values: ``HOL`Terms`stripOrigin`` `15,954`,
  ``HOL`Kernel`concl`` `7,859`, ``HOL`Terms`typeOf`` `4,814`,
  ``HOL`Auto`Simp`Private`isEquation`` `4,240`,
  ``HOL`Kernel`Private`tyMatch`` `3,308`,
  ``HOL`Drule`Private`tryTermMatch`` `2,791`, and
  ``HOL`Drule`Private`tryConv`` `2,488`.

The first source-level runtime attribution then added inclusive/exclusive
timing frames for RHS evaluation and `Module`, plus source counters for symbol
invalidations and source timing for `Module` lexical substitution. On the same
opened-body target (`6.57s` with this probe), the highest exclusive/self-time
items were:

| Source timing item | Count | Self time |
| --- | ---: | ---: |
| `module_body_substitution_by_source` / ``HOL`Drule`ORELSEC`` | `2,172` | `0.860s` |
| `eval_module_by_source` / ``HOL`Drule`ORELSEC`` | `2,172` | `0.757s` |
| `eval_module_by_source` / ``HOL`Drule`TRYCONV`` | `168` | `0.635s` |
| `eval_module_by_source` / ``HOL`Drule`REPEATC`` | `127` | `0.621s` |
| `eval_module_by_source` / ``HOL`Drule`Private`tryConv`` | `2,488` | `0.569s` |
| `module_body_substitution_by_source` / ``HOL`Drule`Private`tryConv`` | `2,488` | `0.384s` |
| `module_body_substitution_by_source` / ``HOL`Drule`REPEATC`` | `127` | `0.326s` |

Top source-owned symbol invalidations were
``HOL`Drule`Private`collectAppSpine`` `13,112`,
``HOL`Drule`REWRCONV`` `6,369`,
``HOL`Drule`Private`tryTermMatch`` `4,911`, and
``HOL`Auto`Simp`Private`productiveSimpRule`` `4,240`.

Current interpretation after this source-level pass was still
diagnostic-only: the smallest finishing target already showed massive repeated
evaluator, `Module`, down-value, builtin, and call-stability activity while
proving no rewrite occurred. Source-level timing moved the next split away
from broad `Module` suspicion and into module body substitution under Drule
conversion combinators, especially ``HOL`Drule`ORELSEC``,
``HOL`Drule`Private`tryConv``, ``HOL`Drule`REPEATC``, and
``HOL`Drule`TRYCONV``. The follow-up 2026-06-17 split below tests that
directly.

## 2026-06-17 Pair/Simp Runtime Internal Split

Extended the runtime profile probes one layer deeper:

- lexical source summaries now run in low-disturbance mode via
  `RUM_PROFILE_LEXICAL_SOURCES=1`, independent of the heavier shape/binding
  hash maps;
- lexical context guards now remain active for source summaries, so source
  output correctly attributes this lane to `arg_prefilter`;
- evaluation-cache snapshots now include `call-stability-source-head` counts,
  grouped by active definition source and call head.

On the same opened-body target, the low-disturbance probe with these summaries
measured `7.14s`. The repeated counts are still the same shape
(`77,322` RHS evals, `20,656` `Module` entries, `8.79M` stability checks), so
the source summaries are usable as attribution, not a new workload.

The module body substitution hotspot is now explained by lexical prefilter
presence scans:

| Source / shape | Scans | Nodes | Time |
| --- | ---: | ---: | ---: |
| ``HOL`Drule`ORELSEC`` / all single `arg_prefilter` scans | `19,548` | `59,926,272` | `0.872s` |
| ``HOL`Drule`ORELSEC`` / ``call-head-call:HOL`Drule`ORELSEC`` | `2,066` | `54,418,260` | `0.795s` |
| ``HOL`Drule`Private`tryConv`` / all single `arg_prefilter` scans | `22,392` | `16,436,594` | `0.374s` |
| ``HOL`Drule`Private`tryConv`` / `call-head:Catch` | `2,488` | `16,404,250` | `0.372s` |
| ``HOL`Drule`REPEATC`` / all multi `arg_prefilter` scans | `3,175` | `16,378,553` | `0.310s` |

These line up with the source-level self timings from the previous split:
``HOL`Drule`ORELSEC`` module body substitution was about `0.895s`, and its
source lexical prefilter scans account for about `0.872s`; ``tryConv`` was
about `0.394s` vs. `0.374s`; ``REPEATC`` was about `0.313s` vs. `0.310s`.
That makes the hot suboperation concrete: repeated `arg_prefilter` absence
scans over large Drule conversion trees inserted into `Module` bodies.

The call-stability split is a separate repeated-work lane, concentrated by
source and head:

- ``HOL`Drule`Private`collectAppSpine`` / `List`: `123,881` checks,
  `77,649` misses, mostly absent plus many rewritten-dirty skips.
- ``HOL`Drule`Private`tryConv`` / `List`: `123,546` checks,
  `116,899` misses, mostly absent.
- ``HOL`Drule`Private`tryConv`` / ``HOL`Types`tyApp``: `116,548` checks,
  `112,856` misses, mostly absent.
- ``HOL`Drule`ORELSEC`` / `List`: `105,139` checks, `77,349` misses, with
  dirty and generation misses prominent.
- ``HOL`Drule`ORELSEC`` / ``HOL`Types`tyApp``: `102,113` checks,
  `92,693` misses, again dirty/generation-heavy.

The next probe split added binding-target and opt-in source-fingerprint
summaries, plus a separate invalidation-source view for call-stability misses.
This was still diagnostic-only; normal evaluator behavior is unchanged, and
source fingerprints are behind
`RUM_PROFILE_LEXICAL_SOURCE_FINGERPRINTS=1`.

On `runtime-profile-opened-body-user-basic`, the source/target split names the
largest lexical absent scans directly:

| Source / target / shape | Scans | Nodes | Time |
| --- | ---: | ---: | ---: |
| ``HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res`` / ``call-head-call:HOL`Drule`ORELSEC`` | `2,066` | `54,418,260` | `0.86s` |
| ``HOL`Drule`Private`tryConv`` / ``HOL`Drule`Private`res`` / `call-head:Catch` | `2,488` | `16,404,250` | `0.38s` |
| ``HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res`` / ``call-head:HOL`Drule`Private`tryConv`` | `2,172` | `5,482,684` | `0.08s` |
| ``HOL`Drule`REPEATC`` / multi-binding / ``call-head:HOL`Drule`Private`tryConv`` | `127` | `5,458,036` | `0.16s` |
| ``HOL`Drule`REPEATC`` / multi-binding / ``call-head-call:HOL`Drule`REPEATC`` | `127` | `5,455,890` | `0.10s` |
| ``HOL`Drule`REPEATC`` / multi-binding / ``call-head:HOL`Drule`ORELSEC`` | `106` | `5,455,594` | `0.09s` |

The fingerprint follow-up is important: these hot scans are not one exact tree
being scanned hundreds of times. For the top ``ORELSEC`` /
``HOL`Drule`Private`res`` shape, text fingerprints group to `214` structures
with `max_scans_per_fingerprint=16`, but exact fingerprints split to `1,033`
structures with `max_scans_per_fingerprint=2`. The top ``tryConv`` /
`Catch` shape similarly has `242` text fingerprints vs. `1,166` exact
fingerprints, again `max exact=2`. That points at repeated text-equivalent
conversion trees with fresh identities, not a simple exact-tree memoization
miss.

The follow-up no-source generation split showed that the generation bumps were
not actually source-less; the earlier output was fragmented by fresh symbol
names and top-N reporting. Evaluation-cache profiling now records invalidation
file/line, active definition source, exact target text, and target family with
generated `$...` suffixes stripped. On the low-disturbance
`runtime-profile-opened-body-user-basic` run, the target segment was about
`8.59s` and still reported `367,176` generation misses, `492,785` dirty
misses, `311` dirty-bloom saturations, and `311` call-generation bumps. The
critical family view attributed those generation bumps to concrete sources:
`scoping/lexical.rs:316` under ``HOL`Kernel`Private`tyMatch`` writing family
``HOL`Kernel`Private`k`` (`29`), and `definition_tags.rs:272` under
``HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res`` (`24`),
``HOL`Drule`Private`tryConv`` / ``HOL`Drule`Private`res`` (`22`),
``HOL`Drule`REWRCONV`` / ``HOL`Drule`Private`c`` (`22`), and
``HOL`Auto`Simp`Private`simpRuleConv`` / ``HOL`Auto`Simp`Private`c`` (`21`).
Dirty writes remain a separate invalidation amplifier led by
``HOL`Drule`Private`collectAppSpine`` target families `args` and `cur`
(`6,556` each), ``HOL`Drule`Private`tryConv`` / `res` (`2,488`),
``HOL`Drule`Private`tryTermMatch`` / `args` and `head` (`2,430` each),
``HOL`Drule`ORELSEC`` / `res` (`2,172`), and ``HOL`Drule`REWRCONV`` / `c`
(`2,121`).

The Drule fresh-identity rebuild split is now attributed to rule-RHS
alpha-renaming, not to a later generic call alone. With lexical shape summaries
enabled, the profile-inflated same target finished in about `73.24s`.
Expression rebuild text/exact groups show text-equivalent but exact-unique
trees: ``HOL`Drule`ORELSEC`` rebuilds `Module` and `CompoundExpression`
with `225` text fingerprints but `2,172` exact fingerprints, `If` with `214`
text fingerprints but `2,066` exact fingerprints, and nested `ORELSEC` with
`214` text fingerprints but `1,033` exact fingerprints. ``HOL`Drule`Private`tryConv``
shows the same pattern for `Module` / `CompoundExpression` / `Set` (`242`
text vs. `2,332` exact), and `Catch` (`1,166` exact, max rebuilds `2`).
The explicit `rule-rhs-alpha-rename` summary names the source: `tryConv`
renames local ``HOL`Drule`Private`res`` `2,488` times, `collectAppSpine`
renames `cur` and `args` `2,430` times each, `tryTermMatch` renames `head`
and `args` `2,430` times each, and `ORELSEC` renames `res` `2,172` times.
All of those fresh locals carry symbol identities.

Current interpretation remains probe-only. Rule application alpha-renames
scoped RHS locals with fresh identities, rebuilds large Drule conversion
closures around them, and lexical `arg_prefilter` then repeatedly scans those
fresh trees to prove absence of newly introduced locals such as
``HOL`Drule`Private`res`` and ``HOL`Auto`Simp`Private`c``. This is still
probe-only evidence, not permission to optimize. The next step is to split
whether those absence scans can be explained by a precise provenance boundary
and to measure the remaining internal cost before any shortcut is implemented.

Added a small semantic guardrail for the provenance question. The internal unit
test
`rule_rhs_alpha_rename_does_not_rewrite_inserted_bound_values` proves that
rule RHS substitution can alpha-rename a scoped local to a fresh identity while
leaving an inserted bound value with the original plain symbol and shared call
nodes. The public evaluator test
`delayed_rhs_substitution_keeps_inserted_compound_values_outside_new_locals`
locks the same behavior through `Module`, `With`, and explicit `Function`:
compound held values inserted from the caller are not captured by locals that
are introduced by the delayed RHS.

The remaining probe work is to test the provenance hypothesis directly. Build
instrumentation that classifies each hot `arg_prefilter` absence scan by
target-local origin and scanned-subtree origin: direct caller-provided binding,
rule-RHS rebuilt parent, ordinary evaluated value, or unknown. The key question
is how much of the ``ORELSEC``/`tryConv` `res` node volume is over subtrees
created before the exact fresh local identity existed. Until that fraction is
measured and the residual unclassified scan cost is split, do not implement a
negative certificate, compact footprint, or shortcut.

The provenance classifier is now in place behind
`RUM_PROFILE_LEXICAL_PROVENANCE=1`. It records exact-fingerprint provenance for
direct rule binding values and rule-RHS rebuilds, classifies presence-scan
targets as rule-RHS alpha identities / generated `$` names / plain symbols, and
reports whether the scanned expression's maximum rule-RHS alpha id is older
than the exact target alpha id. This remains diagnostic-only.

Semantic consistency was rechecked before using the classifier: the internal
identity-level test and the public `Module` / `With` / `Function` compound
inserted-value tests pass in rum, and the same compound inserted-value cases
match WolframKernel modulo generated `$` suffixes.

The low-shape provenance run on
`runtime-profile-opened-body-user-basic`
(`/tmp/rum-pair-simp-probes.m4HXYh`) finished in `176s`; the target segment was
profile-inflated to about `81.51s` and still returned theorem `True`,
changed `False`. The main evidence:

- ``HOL`Drule`ORELSEC`` scanning for ``HOL`Drule`Private`res`` over nested
  ``HOL`Drule`ORELSEC``: `2,066` scans, `54,418,260` nodes, all
  `target_origin=rule-rhs-alpha`, `expr_origin=rule-rhs-rebuild`, and
  `alpha_relation=expr-older-than-target`;
- ``HOL`Drule`Private`tryConv`` scanning for `res` over `Catch`: `2,332`
  large scans, `16,395,778` nodes, same origin/relation;
- ``HOL`Drule`ORELSEC`` scanning for `res` over `tryConv`: `2,120` large
  scans, `5,479,280` nodes, same origin/relation;
- ``HOL`Drule`TRYCONV`` scanning for `res` over `tryConv`: `106` large scans,
  `5,457,136` nodes, same origin/relation;
- ``HOL`Auto`Simp`Private`simpRuleConv`` scanning for private `c` over
  `condRewrConv`: `2,120` scans, `5,477,160` nodes, same origin/relation.

So the hot single-binding absence scans are no longer unknown provenance: the
dominant node volume is over rule-RHS rebuilt parents whose contained RHS alpha
ids are older than the fresh local being searched for. However, the same run
also reports only about `0.824s` total timed `presence_single_scan_time` for
`88.26M` single-scan nodes. On this opened-body target, scan node volume is a
clear structural multiplier but not the full wall-time owner by itself. The
next probe should therefore time rule RHS substitution/rebuild phases directly
instead of moving to a skip/footprint optimization.

Follow-up RHS timing did that and rejected RHS substitution/rebuild as the
100s-class owner on the already-preheated opened-body target. With
`RUM_PROFILE_RULE_RHS_TIMING=1` and heavy shape/provenance summaries disabled,
`runtime-profile-opened-body-user-basic`
(`/tmp/rum-pair-simp-probes.QeWkZ0`) completed in `104s` total. The profiled
target segment itself was about `7.74s`; the largest RHS timing item was only
``HOL`Drule`REWRCONV`` `call_args` at about `0.42s`, followed by
``HOL`Kernel`Private`tyMatch`` `call_args` at about `0.16s`. This showed that
the run was mostly measuring the two preheats (`preheat-basic` and
`preheat-congruence`) plus the final target, while `RumProfileReset[]` after
preheat intentionally excluded the true slow setup window.

Two new runtime profile probe windows now isolate those preheats:
`runtime-profile-preheat-basic.wls` resets before `basicSimpset[]`, and
`runtime-profile-preheat-congruence.wls` resets after `basicSimpset[]` and
before the three lazy congruence lemmas. On the unmodified clean clone:

| Probe window | Target segment | Notes |
| --- | ---: | --- |
| `runtime-profile-preheat-basic` | `40.4-46.8s` under profile | `basicSimpset[]` via 19 `PropTaut` schemas |
| `runtime-profile-preheat-congruence` | `118.2-133.1s` under profile | three congruence lemmas, about `39-40s` each |

This supersedes the older interpretation that the final opened-body target was
the main wall. The post-preheat target can be single-digit seconds; the current
large Pair/Simp wall is dominated by `PropTaut` theorem construction during
Simp preheat.

DownValue-index timing, behind `RUM_PROFILE_DEFINITION_RULE_TIMING=1`, then
split the hot WL functions without editing `mma-hol`. In the basic preheat
window (`/tmp/rum-pair-simp-probes.yNYBD6`), the biggest branch-level costs
were:

| DownValue branch | Calls | Inclusive time |
| --- | ---: | ---: |
| ``HOL`Kernel`Private`tyMatch`` index `0` (`tyVar[...]`) | `190,074` | `15.6s` |
| ``HOL`Terms`stripOrigin`` index `4` (`abs[...]`) | `429,832` | `13.5s` |
| ``HOL`Kernel`Private`tyMatch`` index `1` (`tyApp[...]`) | `124,606` | `4.2s` |
| ``HOL`Terms`typeOf`` index `4` (`abs[...]`) | `272,286` | `4.1s` |
| ``HOL`Terms`Private`vsubstImpl`` index `4` (`abs[...]`) | `119,430` | `6.2s` |

The congruence window (`/tmp/rum-pair-simp-probes.2pOAfO`) has the same shape
at larger scale: ``stripOrigin`` `abs` branch `1,408,655` calls / `48.1s`,
``tyMatch`` `tyVar` branch `523,474` / `43.4s`, ``tyMatch`` `tyApp` branch
`343,578` / `11.6s`, and ``typeOf`` `abs` branch `759,580` / `11.8s`.

Builtin timing was added next behind `RUM_PROFILE_BUILTIN_TIMING=1`, with both
inclusive and child-subtracted self rankings. The self ranking from
`runtime-profile-preheat-basic`
(`/tmp/rum-pair-simp-probes.AQih8j`) makes the next runtime layer concrete:

| Builtin self item | Calls | Self time |
| --- | ---: | ---: |
| `Fold` in ``HOL`Kernel`Private`tyMatch`` | `190,074` | `4.24s` |
| `Module` in ``HOL`Kernel`Private`tyMatch`` | `124,606` | `3.45s` |
| `Set` in ``HOL`Equal`BETACONV`` | `51,800` | `2.61s` |
| `CompoundExpression` in ``HOL`Kernel`mkEq`` | `58,513` | `2.47s` |
| `Set` in ``HOL`Kernel`mkEq`` | `58,513` | `1.90s` |
| `Lookup` in ``HOL`Kernel`constType`` | `64,142` | `1.10s` |

This rules out some earlier guesses: `Part` in `tyMatch` is only about
`115ms` self and `KeyExistsQ` in `tyMatch` about `66ms` self in the same run.
The next probe-only step is therefore not a local `Part` or association lookup
micro-optimization. It should create smaller single-lemma or single-function
probes around `PropTaut`/`tyMatch`/abs-recursive term walkers and determine
whether the remaining cost is evaluator dispatch/control-flow overhead,
`Fold` callback overhead, repeated `Module` frame construction, or unavoidable
proof-construction work.

## 2026-06-17 Pair/Simp Probe-Discipline Correction

The single-lemma / micro-function split below exposed a process error rather
than a completed hotspot diagnosis. The probes are real, but the decision to
continue from `PropTaut` into one-case ``HOL`Bool`EXCLUDEDMIDDLE[p]`` was not
justified as the next Pair/Simp root task.

The point where the work drifted is now identifiable. Earlier root-level
evidence showed that dependency-preloaded `Get["stdlib/Pair.wl"]` spent about
`564s` in the Pair body in rum versus about `40s` in WolframKernel, and that
the warm public mkPair simplification still timed out after normal preheat.
That is the current acceptance root. Later runtime-profile probes showed that
an already-preheated opened-body target was only about `7.7s`, and that
`basicSimpset[]` / congruence preheat through `PropTaut` was measurable and
slow. The mistake was treating the profile-visible preheat and then the tiny
`EXCLUDEDMIDDLE[p]` descendant as "the current wall" without rebuilding a
closed no-profile phase tree for the actual Pair root. The small descendant
explains a real local constant factor, but it was not proven to account for
the hundreds-of-seconds Pair gate.

Root cause of the agent error:

- The current root target was not carried as an invariant. Once smaller
  probes produced readable rankings, the investigation followed the readable
  child instead of asking whether that child dominated `Pair.wl`.
- Phase closure was not required at every descent. A child probe was allowed
  to become the next focus without showing that child phases summed to the
  parent wall time and that the child owned a large percentage of the parent.
- Profiled timings and unprofiled timings were mixed too casually. Profiled
  `39-40s` congruence lemmas were useful for attribution but were not enough
  to replace no-profile root accounting.
- Inclusive rankings were over-trusted. Names such as `PropTautBoolVars`,
  `caseSplit`, `DISCH`, `mkEq`, `tyMatch`, and `stripOrigin` overlap heavily;
  following them without an exclusive parent/child tree can look like progress
  while failing to explain the root wall.
- The work optimized for momentum and locally measurable progress instead of
  enforcing the user's stated goal: find the dominant heat in the current
  hundreds-of-seconds Pair gate before considering fixes.

New hard rule for Pair/Simp performance work:

1. Name the current root target first: unmodified `Pair.wl` load or the warm
   public mkPair probe that reproduces the Pair gate.
2. Produce a no-profile timing baseline for that root.
3. Split that exact root into child phases whose measured times close over the
   parent wall time. If the child phases do not sum close to the parent, fix
   the probe before interpreting it.
4. Descend only into the largest closed child segment, recording its percentage
   of the parent. Do not descend into a merely interesting, slow, or
   cross-engine-different child that is not proven to dominate the parent.
5. Use profile-on runs only after the no-profile parent/child ownership is
   established, and record the profile overhead ratio.
6. If a smaller probe is useful but not root-dominant, label it as a side lane
   and do not let it redefine the next Pair/Simp frontier.

The immediate next diagnostic task is therefore not to split
``HOL`Bool`EXCLUDEDMIDDLE[p]`` further. It is to rebuild a closed phase tree
for the current unmodified Pair root or warm public mkPair root, then continue
recursing only down the largest closed branch.

## 2026-06-17 Pair/Simp Single-Lemma And Micro-Function Split

Kept the Pair/Simp lane in probe-only mode and added smaller reproducers for
the `PropTaut` preheat wall, `tyMatch`, abs-recursive term walkers, and
`Fold`/`Module` runtime overhead. These are retained as side-lane evidence, not
as the current Pair root hotspot. All rum probes ran against the unmodified
clean clone at `/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` through
`scripts/pair-simp-hotspot-probes.sh`.

Single congruence lemma probes show the Simp wrapper is not the owner. With
definition/builtin profiling enabled, the measured target windows were:

| Probe | Target segment | Shape |
| --- | ---: | --- |
| `runtime-profile-proptaut-imp-cong-schema` | `39.49s` | hand-built implication congruence schema through `propTaut` |
| `runtime-profile-imp-cong-lemma` | `39.28s` | `computeImpCongLemma[]` / `impCongLemma[]` |
| `runtime-profile-and-cong-lemma` | `39.55s` | `computeAndCongLemma[]` / `andCongLemma[]` |
| `runtime-profile-or-cong-lemma` | `40.68s` | `computeOrCongLemma[]` / `orCongLemma[]` |

The three lazy congruence lemmas are therefore essentially three same-shaped
`PropTaut` runs. A no-profile sanity pass measured `impCongLemma[]` at
`22.54s` in rum versus `7.08s` in WolframKernel, so profiling inflates the
absolute time but the unprofiled gap is still about `3.2x`.

The smallest currently retained `PropTaut` subshape is now one case split:

| Probe | Rum target | Wolfram target | Notes |
| --- | ---: | ---: | --- |
| `runtime-profile-excluded-middle-boolvar` with profiling off | `0.969s` | `0.322s` | bare ``HOL`Bool`EXCLUDEDMIDDLE[p]`` |
| same probe with profiling on | `1.75s` | n/a | names the internal rum hot functions |
| `runtime-profile-proptaut-boolvars-p-or-notp` with profiling on | `2.35s` | n/a | minimal `propTautBoolVars[p ∨ ¬p]`; `caseSplit` is `1.98s`, `EXCLUDEDMIDDLE` is `1.71s` |
| `runtime-profile-proptaut-free-bool-vars-schema` | `0.745s` for 1000 calls | n/a | `propFreeBoolVars` scan is only about `0.6ms` per call, so the full `PropTautBoolVars` rank is mostly inclusive recursion/case-split work |

Profiling the bare `EXCLUDEDMIDDLE[p]` still names the same runtime/language
suboperations seen in the larger lemmas: `mkEq`, `stripOrigin`, `tyMatch`,
`unfoldBinop`, `mkConst`, `BETACONV`, `MKCOMB`, `vsubstImpl`, `typeOf`, and
the theorem-plumbing rules. In builtin self timing, `Fold` inside `tyMatch`
is about `170ms` for `7,820` calls and `Module` inside `tyMatch` about
`132ms` for `5,134` calls on this one-case-split shape.

The `tyMatch` micro probes now separate branch and callback overhead:

| Probe | Target segment | Notable self timing |
| --- | ---: | --- |
| `runtime-profile-tymatch-var-repeat` | `0.178s` for 1000 calls | `tyMatch` RHS about `46.5us`/call; `Module` self about `34.5us`/call |
| `runtime-profile-tymatch-fold-repeat` | `0.383s` for 1000 outer calls | `5000` `tyMatch` RHS evals; `Fold` self `80.97ms`, `Module` self `100.56ms` |
| `runtime-profile-fold-callback-repeat` | `0.0287s` for 1000 trivial two-element folds | pure `Fold` self about `3.2us`/call |
| `runtime-profile-module-frame-repeat` | `0.0392s` for 1000 trivial modules | trivial `Module` self about `5.5us`/call |
| `runtime-profile-loop-null-repeat` | `0.0255s` for 1000 empty iterations | loop/iterator baseline; `Do` plus iterator `Block` is visible probe overhead |

The useful conclusion is not that `Fold` or `Module` are universally slow; it
is that `Fold` callbacks and `Module` frames become expensive when they carry
the real `tyMatch` body and recursive evaluator traffic. `Part`,
`KeyExistsQ`, `Transpose`, and `Append` remain small in the same probes.

The abs-recursive walker probes show real repeated tree-walk dispatch, but
also make the loop baseline explicit:

| Probe | Target segment for 1000 calls | Main observation |
| --- | ---: | --- |
| `runtime-profile-striporigin-abs-repeat` | `0.151s` | `17,000` `stripOrigin` down-value applications; `Block` iterator overhead is a large share |
| `runtime-profile-typeof-abs-repeat` | `0.167s` | `11,000` `typeOf` down-value applications plus `destFunType` |
| `runtime-profile-vsubst-abs-repeat` | `0.199s` | `17,000` `vsubstImpl` applications; empty substitution validation is small |

If a future closed Pair-root phase tree proves that congruence preheat or
`PropTaut` again dominates the current root, these retained probes provide a
ready side lane. In that case, useful next splits would be one-rule theorem
plumbing such as `DISCH`, `CONJUNCT1/2`, `DISJ1/2`, `DISJCASES`, `BETACONV`,
and `MKCOMB`, or a runtime source split for their repeated
`mkEq`/`tyMatch`/`stripOrigin` traffic. Until then, the main Pair/Simp frontier
returns to the root-level closed phase tree.

## 2026-06-16 Pair/Simp Arg-Prefilter Attribution Probe

Committed the hygienic delayed-RHS substitution fix as `e9989f1`
(`Fix hygienic delayed RHS substitution`) before continuing performance
diagnosis.

Added another probe-only attribution layer to the lexical profiler. It does not
change evaluator behavior. The new `context` / `context-shape` summaries showed
that essentially all presence-scan cost in the stock `basic-pair4` Pair/Simp
probe is under `generic_call`, not the `Module`/`With`/`Function` scoped
branches. A second split around `substitute_arg_list_changed` then showed the
cost is specifically in `arg_prefilter`: the scan that asks whether an argument
may contain the current lexical binding before recursively substituting it.
`arg_substitution` itself is not the owner of the presence-scan wall.

The stock clean-clone probe was still
`/private/tmp/rum-task2d-basic-pair4-stock.wl` against
`/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` (HEAD `51bbe9e`), run with
`RUM_PROFILE_LEXICAL_SUBSTITUTION=1 RUM_PROFILE_LEXICAL_DETAIL=1
RUM_PROFILE_LEXICAL_SHAPES=1 RUM_PROFILE_LEXICAL_BINDINGS=1`. With the heavier
attribution enabled, the target result was still theorem `True` and changed
`False`; absolute timing is profiler-inflated and should not be compared to the
default lane.

Arg-prefilter ownership:

- single-binding presence scans: `6,824,326` scans, `662,415,057` nodes,
  `13.96s`, all under `arg_prefilter`;
- multi-binding presence scans: `6,138,394` scans, `218,746,948` nodes,
  `5.15s`, all under `arg_prefilter`.

The largest concrete arg-prefilter sites are:

- parent `If`, argument index `1`, scanned shape
  ``call-head-call:HOL`Drule`ORELSEC``: `17,568` scans,
  `360,134,688` nodes, `7.52s`, all false;
- parent `Set`, argument index `1`, scanned shape `call-head:Catch`:
  `20,496` scans, `120,353,360` nodes, `3.68s`, all false;
- parent `Set`, argument index `1`, scanned shape
  ``call-head:HOL`Drule`Private`tryConv``: `19,520` scans,
  `80,319,984` nodes, `1.29s`, all false;
- parent `Which`, argument index `3`, scanned shape
  ``call-head-call:HOL`Auto`Simp`Private`condRewrConv``: `18,544` scans,
  `40,287,584` nodes, `0.56s`, all false.

Interpretation: the current Pair/Simp hot path is repeated prefilter absence
checking over large unchanged argument expressions, especially conversion
closures under `If`, `Set`, and `Which`. This is narrower than "lexical
substitution is slow": the recursive substitution body is not the owner.

The follow-up binding-name aggregation probe merged fresh `res#...` identities
by textual symbol name and confirmed the top rows are real repeated scans of
the same local-name families, not profile fragmentation:

- ``call-head-call:HOL`Drule`ORELSEC`` looking for
  ``HOL`Drule`Private`res``: `17,568` scans, `360,134,688` nodes,
  `6.63s`, all false;
- `call-head:Catch` looking for ``HOL`Drule`Private`res``: `20,496` scans,
  `120,353,360` nodes, `3.43s`, all false;
- ``call-head:HOL`Drule`Private`tryConv`` looking for
  ``HOL`Drule`Private`res``: `19,520` scans, `80,319,984` nodes, `1.15s`,
  all false;
- ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` looking for
  ``HOL`Auto`Simp`Private`c``: `18,544` scans, `40,287,584` nodes,
  `0.52s`, all false.

The next diagnostic layer should target arg-prefilter absence checking, not
rule matching or recursive substitution. Candidate explanations to test are:
repeated symbol-presence scans for large immutable argument subtrees, or a
lexical-substitution boundary that fails to recognize caller-inserted values
cannot contain the fresh local symbol introduced after insertion. Do not change
the downstream `mma-hol` standard to avoid this cost.

End-of-day probe status: added a follow-up presence-reuse profiler to ask
whether those large absent scans revisit the same shared expression node or
mostly scan newly allocated equivalent structures. The first reuse run printed
top rows with `scans=1` even for the largest `Catch` / `TRYCONV` nodes
(`~41k` nodes each), which is evidence that the hot path is not repeatedly
visiting one stable `SharedArgs` pointer. It is more likely rebuilding fresh
conversion closures that are structurally similar but pointer-distinct. A
`reuse-group` summary was then added to aggregate by `(scan kind, root shape,
target textual symbol)` and print `unique_ptrs`, `scans`, and
`max_scans_per_ptr`. Its final Pair run completed after the machine went quiet.
The top rows are decisive:

- ``call-head-call:HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res``:
  `unique_ptrs=17,568`, `scans=17,568`, `max_scans_per_ptr=1`;
- `call-head:Catch` / ``HOL`Drule`Private`res``: `unique_ptrs=20,496`,
  `scans=20,496`, `max_scans_per_ptr=1`;
- ``call-head:HOL`Drule`Private`tryConv`` / ``HOL`Drule`Private`res``:
  `unique_ptrs=19,520`, `scans=19,520`, `max_scans_per_ptr=1`;
- ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` /
  ``HOL`Auto`Simp`Private`c``: `unique_ptrs=18,544`, `scans=18,544`,
  `max_scans_per_ptr=1`.

This rules out a simple pointer-keyed presence cache as the primary fix for
the top absent-scan wall. The next probe should determine whether these are
structurally identical fresh trees or genuinely distinct trees by adding a
structural fingerprint aggregation for large scanned expressions.

Structural fingerprint profile completed. The fingerprint intentionally ignores
generated symbol identity and hashes only expression structure plus symbol
context/name text. The result shows that the hot absent scans are not unique
structures; they are repeated structures rebuilt with fresh pointers:

- ``call-head-call:HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res``:
  `unique_fingerprints=126`, `scans=17,568`,
  `max_scans_per_fingerprint=320`;
- `call-head:Catch` / ``HOL`Drule`Private`res``:
  `unique_fingerprints=147`, `scans=20,496`,
  `max_scans_per_fingerprint=320`;
- ``call-head:HOL`Drule`Private`tryConv`` / ``HOL`Drule`Private`res``:
  `unique_fingerprints=140`, `scans=19,520`,
  `max_scans_per_fingerprint=320`;
- ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` /
  ``HOL`Auto`Simp`Private`c``: `unique_fingerprints=133`,
  `scans=18,544`, `max_scans_per_fingerprint=320`.

Interpretation: a pointer-keyed cache will miss the top cases, but there is
real structural repetition. The next probe should find where these
pointer-fresh but structurally repeated conversion closures are rebuilt,
preferably without changing evaluator semantics.

Follow-up expression-rebuild attribution then extended the same gated profiler
from lexical substitution into rule RHS binding substitution. The full profile
was written to `/private/tmp/rum-expression-rebuild-profile.txt` to avoid
terminal truncation. It again proved the same target (`is_thm=True`,
`changed=False`); the absolute target time was profiler-inflated
(`270.893s`) and is not a performance baseline.

The new `expression-rebuild-group` rows line up with the earlier lexical
presence fingerprints:

- `rule_rhs_call` rebuilt ``call-head-call:HOL`Drule`ORELSEC`` with
  `unique_fingerprints=133`, `rebuilds=18,544`,
  `max_rebuilds_per_fingerprint=320`, matching the hot
  ``condRewrConv``/`c` and adjacent `ORELSEC` structural repetition;
- `rule_rhs_call` rebuilt `call-head:Catch` with
  `unique_fingerprints=147`, `rebuilds=20,496`,
  `max_rebuilds_per_fingerprint=320`, matching the hot lexical `Catch` scans
  for ``HOL`Drule`Private`res``;
- `rule_rhs_call` rebuilt ``call-head:HOL`Drule`Private`tryConv`` with
  `unique_fingerprints=147`, `rebuilds=20,496`,
  `max_rebuilds_per_fingerprint=320`, matching the hot `tryConv` scans;
- `rule_rhs_call` rebuilt
  ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` with
  `unique_fingerprints=133`, `rebuilds=18,544`,
  `max_rebuilds_per_fingerprint=320`, matching the hot `condRewrConv` scans.

Current interpretation: the Pair/Simp wall is now a concrete two-step runtime
path, not an unknown evaluator blob. Delayed rule RHS substitution repeatedly
rebuilds structurally repeated conversion-closure trees, and the later `Module`
lexical prefilter repeatedly scans those fresh trees to prove absence of
fresh locals such as ``HOL`Drule`Private`res`` and
``HOL`Auto`Simp`Private`c``. The next diagnostic split should identify the
definition/rule source of the largest `rule_rhs_call` rebuilds before changing
the algorithm. Plausible fix directions include preserving/propagating
symbol-presence summaries through rule RHS substitution outputs or avoiding
full absence scans on freshly inserted rule-RHS values that cannot contain a
newer `Module` local; do not implement either before a smaller probe validates
the exact source contract.

The source-symbol split completed next, again with the same theorem result
(`is_thm=True`, `changed=False`) and profiler-inflated target time
(`272.066s`). The largest `rule_rhs_call` sources are now concrete:

- ``HOL`Drule`ORELSEC`` rebuilds `Module` / `CompoundExpression` /
  `If` / nested `ORELSEC` shapes with `126-133` structural fingerprints and
  `17,568-18,544` rebuilds, `max_rebuilds_per_fingerprint=320`;
- ``HOL`Drule`Private`tryConv`` rebuilds `Module` / `CompoundExpression` /
  `Set` / `Catch` shapes with `147` structural fingerprints and `20,496`
  rebuilds, `max_rebuilds_per_fingerprint=320`;
- ``HOL`Auto`Simp`Private`combineSimpConvs`` rebuilds `ORELSEC`,
  recursive `combineSimpConvs`, and `List` shapes with `18-19` fingerprints
  and `8,784-9,272` rebuilds, `max_rebuilds_per_fingerprint=488`;
- ``HOL`Auto`Simp`Private`simpRuleConv`` rebuilds `Module` /
  `CompoundExpression` / `Which` shapes with `133` fingerprints and `18,544`
  rebuilds, `max_rebuilds_per_fingerprint=320`;
- ``HOL`Drule`REPEATC`` and
  ``HOL`Auto`Simp`Private`simpStepConv`` own the largest single-expression
  rebuild rows around `68k` nodes per rebuilt expression.

This confirms the next viable optimization frontier is not broad evaluator
rewrites. It is the interaction between rule RHS substitution of conversion
combinators and subsequent lexical absence scans. Any cache/provenance fix must
respect `Symbol` equality including generated identity; the diagnostic
fingerprint used above intentionally ignores identity and is not itself a safe
semantic cache key.

An exact-fingerprint follow-up then tested whether a safe structural absence
cache keyed on generated identities could still collapse the hot scans. It
again preserved the theorem result (`is_thm=True`, `changed=False`), with a
profiler-inflated target time of about `297.814s`. The answer is negative for
the main wall:

- text fingerprint for
  ``call-head-call:HOL`Drule`ORELSEC`` / ``HOL`Drule`Private`res``:
  `unique_fingerprints=126`, `scans=17,568`,
  `max_scans_per_fingerprint=320`;
  exact fingerprint: `unique_fingerprints=8,784`,
  `max_scans_per_fingerprint=2`;
- text fingerprint for `call-head:Catch` / ``HOL`Drule`Private`res``:
  `147` unique over `20,496` scans, `max=320`;
  exact fingerprint: `10,248` unique, `max=2`;
- text fingerprint for ``call-head:HOL`Drule`Private`tryConv`` /
  ``HOL`Drule`Private`res``: `140` unique over `19,520` scans, `max=320`;
  exact fingerprint: `9,760` unique, `max=2`;
- text fingerprint for
  ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` /
  ``HOL`Auto`Simp`Private`c``: `133` unique over `18,544` scans, `max=320`;
  exact fingerprint: `9,272` unique, `max=2`.

This rejects a simple exact structural cache as the next fix for the primary
single-binding absence scans. The repeated structures differ by generated
symbol identities, so the viable runtime directions are now narrower:
propagate exact symbol-presence summaries through rule RHS substitution and
expression construction, or preserve enough provenance to skip scanning
freshly inserted values for a newer lexical local without violating the
hygienic delayed-RHS semantics just fixed.

## 2026-06-15 Pair/Simp Hygienic RHS Substitution Probe

Reopened the Pair/Simp performance lane probe-first after rejecting further
trial-and-error optimizer changes. The stock clean-clone target remained
`/private/tmp/rum-task2d-basic-pair4-stock.wl` against the unmodified
`mma-hol` clone. The old clean clone path was empty by the time of the final
rerun, so the same probe file now points at the clean unmodified clone
`/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` (HEAD `51bbe9e`).

Added cold-gated lexical profiling only:

- `RUM_PROFILE_LEXICAL_SUBSTITUTION=1` for phase-level counters and summary
  timing;
- `RUM_PROFILE_LEXICAL_DETAIL=1` for presence-scan timings;
- `RUM_PROFILE_LEXICAL_SHAPES=1` for presence-scan root-shape attribution;
- `RUM_PROFILE_LEXICAL_BINDINGS=1` for single-binding target-symbol
  attribution.

With the probes enabled, the same 8-repeat target still proves the theorem and
does not change the target term. The measured lexical block is now clear:
`Module` initializer substitution is negligible, while final `Module` body
substitution accounts for essentially all lexical substitution time. In the
detail run, presence scans consume about `18.40s` of `23.01s` lexical
substitution time (`13.43s` single-binding plus `4.97s` multi-binding).

The dominant timed presence-scan shapes are all no-op scans:

- single ``call-head-call:HOL`Drule`ORELSEC`` looking for
  ``HOL`Drule`Private`res``: `17,568` scans, `360,134,688` nodes,
  `7.24s`, all false;
- single `call-head:Catch` looking for ``HOL`Drule`Private`res``: `20,496`
  scans, `120,353,360` nodes, `3.62s`, all false;
- single ``call-head:HOL`Drule`Private`tryConv`` looking for
  ``HOL`Drule`Private`res``: `19,520` scans, `80,319,984` nodes,
  `1.25s`, all false;
- single ``call-head-call:HOL`Auto`Simp`Private`condRewrConv`` looking for
  ``HOL`Auto`Simp`Private`c``: `18,544` scans, `40,287,584` nodes,
  `0.55s`, all false.

This changed the interpretation. Local WolframKernel checks show that values
substituted for pattern variables into a delayed RHS are not captured by an
inner `Module` local of the same name:

```wl
ClearAll[f, x, y];
SetAttributes[f, HoldFirst];
f[body_] := Module[{x}, Hold[body, x]];
f[x]
(* WolframKernel: Hold[x, x$...] *)
(* current rum:    Hold[x$1, x$1] *)
```

The same difference appears for ordinary downvalues, `HoldAll`, and
`RuleDelayed` replacement rules. Therefore the Pair/Simp `res`/`ORELSEC`
scanner wall is not just a low-level scanner constant: current rum substitutes
pattern bindings into the whole RHS before `Module` local renaming, so inserted
argument expressions are scanned and can be incorrectly captured by the
inner lexical binder.

Implemented the rum-side hygienic RHS fix in rule binding substitution:
delayed RHS substitution now treats `Module`, `With`, and explicit `Function`
locals as alpha-renamed boundaries before substituting pattern bindings into
their bodies. Pattern-bound values are inserted directly, so fresh local
alpha-renames do not recursively capture values that came from the caller; if
the local has the same symbol as the pattern variable itself, the same-name
binding behavior is preserved. Added regression coverage for downvalues,
`HoldAll` / `HoldFirst`, `RuleDelayed`, nested `Module`, `With`, and explicit
`Function`.

Validation:

- `cargo test -p rum-runtime delayed_rhs_substitution -- --nocapture`: green;
- `cargo test -p rum-runtime`: green (`654` tests);
- `cargo fmt --check`: green;
- release `basic-pair4-stock` probe on the unmodified clone: preheat
  `23.10s`, target `51.09s`, theorem `True`, changed `False`;
- release profile probe with lexical detail/shape/binding: preheat `28.31s`,
  target `53.26s`, theorem `True`, changed `False`.

The rerun fixes the semantic mismatch but does not remove the Pair/Simp
performance wall. Presence scans still consume about `16.59s` of `23.82s`
lexical substitution time (`12.04s` single-binding plus `4.55s`
multi-binding), and the dominant timed shapes are still no-op absence scans:

- single ``call-head-call:HOL`Drule`ORELSEC``: `17,568` scans,
  `360,134,688` nodes, `6.24s`, all false;
- single `call-head:Catch`: `20,496` scans, `120,353,360` nodes, `3.37s`,
  all false;
- single ``call-head:HOL`Drule`Private`tryConv``: `19,520` scans,
  `80,319,984` nodes, `1.12s`, all false;
- single ``call-head-call:HOL`Auto`Simp`Private`condRewrConv``: `18,544`
  scans, `40,287,584` nodes, `0.50s`, all false.

The binding-level profile now prints symbol identities and shows those rows as
fresh alpha locals such as ``HOL`Drule`Private`res#...`` rather than a single
textual `res` symbol. The next performance diagnosis should therefore stay on
the rum side and quantify how to avoid repeated full-tree absence scans over
caller-inserted values that cannot be captured by those fresh lexical locals.

## 2026-06-13 Task 2d Failed-Dispatch Kernel Probe

Started Task 2d against the unmodified clean `mma-hol` clone at
`/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` (HEAD `698e3d0`). The probe is
stock HOL code plus an external `/private/tmp` driver; no `mma-hol` source file
is edited.

Small reproducer: `basic`-only
``HOL`Auto`Simp`Private`simpFixpointConv`` on `pair4Same`. Calibration with
`/private/tmp/rum-task2d-basic-pair4-stock.wl`:

- 8 target repeats: preheat `22.616s`, target `79.261s`, theorem `True`,
  changed `False`;
- 20 target repeats during sampling: preheat `20.707s`, target `226.650s`,
  theorem `True`, changed `False`.

Sampled the 20-repeat target window with macOS `sample` for 40s and reduced
`/private/tmp/rum-task2d-basic-pair4.sample` with the repo profile tools. The
flat profile is dominated by lexical substitution rather than term matching:

- self time: `substitute_lexical_bindings_changed` `24.5%`,
  `Symbol::cmp` `7.1%`, allocation/drop/arg-list substitution following;
- inclusive path: `100%` through `evaluate_definition_rhs` and `eval_module`;
- attribution: `evaluate_definition_rhs` is `93.8%`
  `try_apply_sub_values_from_symbol`; `eval_module` is `62.4%` SubValue path;
  `substitute_arg_list_changed` is `100%` under generic call substitution;
  `substitute_function_call_changed` is `100%` under lexical substitution;
  `eval_set` is `50.6%` under `resume_module_initializers`.

Added cold-gated `RUM_PROFILE_EVALUATION_CACHE=1` counters for
`sub_value_applications` and `eval_module_entries`. A one-repeat profile run
(`/private/tmp/rum-task2d-basic-pair4-profile.log`) shows the target reaches at
least the following powers of two:

- ``sub_value_applications[HOL`Drule`REWRCONV] = 2048``;
- ``sub_value_applications[HOL`Auto`Simp`Private`simpRuleConv] = 2048``;
- ``sub_value_applications[HOL`Drule`ORELSEC] = 2048``;
- ``down_value_applications[HOL`Drule`Private`tryTermMatch] = 2048``;
- ``down_value_applications[HOL`Drule`Private`convFail] = 2048``;
- ``down_value_applications[HOL`Terms`stripOrigin] = 1048576``;
- `eval_module_entries = 262144` over the profiled process.

Current interpretation: the Task 2d hotspot is not `tryTermMatch` depth alone.
Each failed conversion dispatch repeatedly applies SubValues whose RHS bodies
are `Module`/`Function`-heavy WL code, then pays full lexical substitution and
local `Set` execution before failing. The next one-variable A/B should be a
rum-side optimization or diagnostic around Module/SubValue RHS execution
(for example, a cheaper lexical-substitution path or a compiled Module
renaming plan), measured against this same stock probe. Do not reintroduce a
`Simp.wl` fork.

Follow-up A/B work kept the same unmodified clone and external probe. Host
load made the first post-17:30 timings unreliable, so those intermediate
numbers were discarded and the lane was remeasured after the load cleared.
Retained 8-repeat target timings:

- first lexical-substitution cut, checking the binding map before the shadowed
  set for symbol atoms: `69.224s`;
- single-binding lookup view for lexical substitution: `62.692s` and
  `62.978s`;
- arg-list prefilter that skips full substitution for subtrees containing no
  bound symbol: `53.079s` and `53.070s`;
- single-binding-specific prefilter scan: `51.167s`, `55.363s`, `49.140s`,
  then `56.016s` and `51.672s` after rejecting the next experiment; median is
  about `51.7s`.

The final retained code keeps all four cuts above. A hand-written scan loop
experiment was rejected because it did not beat noise (`54.440s`, `50.073s`)
and was reverted. The newest sample after the retained prefilter work shows
the remaining self-time frontier has moved from full lexical substitution into
the prefilter scan itself: `expr_may_contain_symbol` about `17.4%`,
multi-binding `expr_may_contain_any_binding` about `5.2%`, then allocation,
drop, stability checks, and `normalize_match_expr`. Further work should not
blindly add another scanner tweak; the next evidence-backed direction is
either a real per-expression symbol-presence summary/cache with measured
overhead, or a larger Module/SubValue execution representation change.

Follow-up presence-cache experiments were rejected. A recursive thread-local
single-binding presence cache made the same 8-repeat target much worse
(`93.997s`), because every nested call paid cache lookup overhead. A narrower
top-level-only cache did improve the small target (`45.499s`, `45.381s`), but
did not move the real Pair gate: the split probe measured `0.620s` preload,
`565.747s` in `Get["stdlib/Pair.wl"]`, and `566.367s` total versus the prior
`564.032s` Pair-body baseline. A shadowed-aware prefilter/early-exit attempt
also failed to show stable benefit on the same target (`50.807s`, then
`54.730s`). None of these cache/shadowed variants are retained. The useful
lesson is that the remaining repeated lexical presence-scan cost is not fixed
by adding ordinary side caches around the current recursive scanner; it needs
either a cheaper expression-level summary carried by the representation, or a
larger reduction in how often Module/SubValue RHS substitution is invoked.

Higher-level validation after the same retained changes:

- focused PropTaut distribution probe through clean `mma-hol` returned
  `{True, True}` with internal timing `10.402s`;
- `scripts/mma-hol-bounded-smoke.sh --mode tests --only test/proptaut_tests`
  with release `rum-cli` exited `0` in `203s` under a `360s` stage timeout.
- `load/stdlib/Pair` through the same clean clone exited `0` in `571s` under a
  `900s` stage timeout, with stdout ending in `"ok"` and empty stderr. This is
  a material change from the earlier timeout status and would narrowly fit a
  `600s` budget on this run, but it is still far from the intended
  constant-bound target around `150s`; Pair/Simp remains the main performance
  gate.
- A follow-up rum split probe showed the same stage is almost entirely
  `Pair.wl` body time: preload through `auto/Simp.wl` was `0.727s`, while
  `Get["stdlib/Pair.wl"]` itself was `564.032s` (`564.759s` total). The matched
  WolframKernel split was `0.226s` preload, `40.249s` Pair body,
  `40.475s` total, so the remaining gap is Pair-body execution, not preload.
- A 40s `sample` window inside rum's `Pair.wl` body keeps the same kernel
  mechanism as the smaller `basic-pair4` probe: self time is led by
  `expr_may_contain_symbol` `15.8%`, allocation/drop, and
  `expr_may_contain_any_binding` `6.1%`; inclusive time is `100%` through
  `evaluate_definition_rhs`, `eval_module`, and `resume_module_initializers`.
  Caller attribution shows both symbol-presence scanners are `100%` under
  `substitute_arg_list_changed`; `eval_set` is `53.7%` under Module
  initializers; `normalize_match_expr` is `94.4%` under `eval_set`.

## 2026-06-13 Pair/Simp Secondary Split

Continued from the Task 2c verdict using the same scratch instrumented
`mma-hol` copy at `/private/tmp/rum-mma-hol-simp-loop.sSsJv1/mma-hol`. The
secondary split isolates one `mkPair` application term under progressively
different rule sets and term shapes. Timings below are release `rum-cli` unless
noted.

Rule-set / term-shape split:

| Probe | Time | Entries | While Iterations | Changed |
| --- | ---: | ---: | ---: | --- |
| `none` rules on `pair4Same` | `0.0909s` | `61` | `61` | `False` |
| `basic` rules on `pair4Same` | `8.739s` | `61` | `61` | `False` |
| `user` rule on `pair2Same` | `29.126s` | `1511` | `1513` | `True` |
| `user` rule on `pair3Same` | `29.674s` | `1619` | `1622` | `True` |
| `user` rule on `pair4Same` | `30.738s` | `1673` | `1677` | `True` |
| WolframKernel `userBasic` on `pair2Same` | `16.522s` | `1511` | `1513` | `True` |
| rum `userBasic` on `pair2Same` | timeout at `180s` and again at `360s` | n/a | n/a | n/a |

This immediately identified `userBasic` on the smallest partial application
`mkPair[x, y]` as the reduced long-tail shape. A first-step split for this
shape then showed:

- `preheatRules["userBasic"]`: `21.592s`;
- `buildBaseConv`: `0.0046s`;
- top `TRYCONV[REPEATC[baseConv]]` on `mkPair[x, y]`: `0.068s`, unchanged;
- `descendOnce` on the same term: timeout at `180s`.

Splitting the ordinary `comb` descent found that the argument positions are
not responsible:

- simplifying the argument `y`: `0.137s`, unchanged, `1` entry;
- simplifying the function position `mkPair[x]`: timeout at `180s`;
- within `mkPair[x]`, simplifying argument `x`: `0.129s`, unchanged, `1`
  entry;
- simplifying the head `mkPair` constant: timeout at `180s`.

The current smallest named hot shape is therefore the `mkPair` constant under
`userBasic`. Splitting that head shows:

| Probe | rum | WolframKernel | Changed |
| --- | ---: | ---: | --- |
| top rewrite of `mkPair` constant | `0.111s` | not measured | `True` |
| `descendOnce` on the rewritten RHS lambda, `user` rule only | `23.027s` | `7.854s` | `False` |
| `descendOnce` on the rewritten RHS lambda, `userBasic` rules | `112.766s` | `10.687s` | `False` |

The `RUM_STEP` cumulative top/descend timers are useful for direction but
double-count recursive nested calls, so the stage wall times above are the
authoritative comparison. The important semantic point is that these expensive
RHS-lambda descents are no-ops (`descend_changed=False`): rum spends over 100s
proving that the `mkPair` definition body is unchanged when the 19 basic rules
are present. This is not preheat, `buildBaseConv`, top rewrite, or argument
descent. It is the recursive descent over the definition RHS lambda/body under
the `userBasic` ruleset, where basic-rule failed dispatch multiplies an
otherwise already nontrivial user-rule-only descent.

RHS-lambda split, same scratch clone:

| Probe | rum | WolframKernel | Entries | Notes |
| --- | ---: | ---: | ---: | --- |
| body left branch, `userBasic` | `2.0616s` | not measured | `13` | unchanged |
| body right branch, `userBasic` | `2.0481s` | not measured | `13` | unchanged |
| full opened body, `userBasic` | `29.616s` | `7.640s` | `53` | unchanged |
| one remaining lambda, `userBasic` | `40.590s` | not measured | `107` | unchanged |
| two remaining lambdas, `userBasic` | `63.673s` | not measured | `215` | unchanged |
| three remaining lambdas, `userBasic` | `113.656s` | `10.535s` | `431` | unchanged |

The binder levels are the multiplier: each remaining lambda roughly doubles
the number of `simpFixpointConv` entries. Rule-set splits on the three-lambda
shape show that the extra wall time comes from basic-rule failed dispatch, not
from successful user rewrites:

| Probe | rum | WolframKernel | Entries |
| --- | ---: | ---: | ---: |
| `user` only on the three-lambda RHS | `22.492s` | `7.804s` | `431` |
| `basic` only, applied to the user-generated RHS | `94.288s` | `10.321s` | `431` |
| `userBasic` on the same RHS | `113.656s` | `10.535s` | `431` |

Scratch `Drule.wl` counters then showed that the `userBasic` three-lambda
probe performs `17240` `REWRCONV` calls and every one is a match failure
(`0` rewrite successes); the full body alone performs `2120` failed
`REWRCONV` calls. Dropping the large `Failure` payload and even replacing
conversion failure with a no-throw sentinel did not materially move the body
time (`~29.7s` -> `~29.3s`), so exception payload/throw mechanics are not the
primary gate.

A fast const-head mismatch check inside `REWRCONV` hit most body failures
(`1998/2120`) but also did not move wall time, which rejects "deep matcher
walk" as the main explanation. A stronger scratch experiment changed
`buildBaseConv` to index rules by LHS const head. Without caching, this cut
body `REWRCONV` calls from `2120` to `122` but regressed wall time to
`43.818s`; measuring the indexed builder separately showed `100` builds cost
`27.877s`. Adding an exact `buildBaseConv` cache fixed that: the body fell to
`21.293s` with `51` cache hits / `2` misses, and the three-lambda RHS fell to
`24.452s` with `429` hits / `2` misses and only `976` failed `REWRCONV` calls.

Verdict: the evidence-backed optimization direction is a compiled/cached
base-conversion dispatch structure, probably keyed by rules/context/depth and
using head-indexed rule candidates. The naive WL-layer indexed builder is too
expensive if rebuilt per recursive entry, but indexed dispatch plus reuse
removes almost all `basicSimpset` failed-dispatch overhead and returns the
three-lambda RHS to near the `user`-only no-op descent cost. The remaining
`~22-24s` is the general no-op lambda/body descent and theorem-construction
constant, not the public-rule multiplier.

Clean follow-up experiment, same date:

Re-ran the optimization direction on a fresh scratch clone at
`/private/tmp/rum-mma-hol-baseconv-exp.HtBwlQ/mma-hol` with only the minimal
`auto/Simp.wl` change: `buildBaseConv` produces a head-indexed conversion
dispatcher and exact-caches it by `{ruleThms, ctxThms, depth}` using a held
key. No `Drule.wl` failure-protocol diagnostics were present.

Results:

| Probe | Time | Cache Hits | Cache Misses | Result |
| --- | ---: | ---: | ---: | --- |
| RHS opened body, `userBasic` | `21.603s` | `52` | `2` | unchanged theorem |
| RHS three-lambda, `userBasic` | `27.618s` | `430` | `2` | unchanged theorem |
| `pair2Same`, public `userBasic` | `34.932s` | `1509` | `2` | changed theorem |
| `pair3Same`, public `userBasic` | `34.394s` | `1616` | `3` | changed theorem |
| `pair4Same`, public `userBasic` | `36.681s` | `1670` | `3` | changed theorem |
| `CONVRULE[simpConv[{mkPairDefThm}], REFL[pair2Same]]` | `53.047s` | `3887` | `2` | theorem |
| cold load through `stdlib/Pair.wl` | `85.537s` | `3670` | `8` | `pairInjThm` theorem |
| `pair_tests.wl` after cold load | `42/0`; test body `0.466s` | `3670` | `8` | green |
| `simp_tests.wl` | `46/0`; test body `208.655s` | `648` | `48` | green |

WolframKernel also runs the patched `pair_tests.wl` green (`42/0`). A
cache-only clone at `/private/tmp/rum-mma-hol-baseconv-cacheonly.65rVR2/mma-hol`
was a negative control: the RHS three-lambda probe still took `115.585s`
despite `430` cache hits / `2` misses, and `pair2Same` timed out at `180s`.
Therefore the important ingredient is not merely avoiding repeated builder
work; the dispatcher must also avoid walking the whole public `ORELSEC` rule
chain for obviously mismatched heads.

The experiment crosses the previous `Pair.wl` gate in source-load and tests.
The stable snapshot lane was then reduced and resolved experimentally. A
partial snapshot through `Pair` initially timed out at `300s`; loading through
Pair in Stable mode without `DumpSave` completes in `83.5s`, so the problem
was not proof load. Staged dumps showed `HOL`Auto`Simp`Private`` alone
serialized to about `402MB`. The cause is not the explicit base-conversion
cache: `Names["HOL`Auto`Simp`Private`*"]` contains many generated
`Module` locals such as `baseConv$...` and `xEq$...`; dumping a whole private
context saves those temporary symbols. `Remove["HOL`Auto`Simp`Private`*$*"]`
does not match in rum, but explicit enumeration works:

```wolfram
generated = Names["HOL`*$*"];
Scan[Remove, generated];
HOL`Auto`Simp`Private`$simpDepthLimit = 4;
HOL`Auto`Simp`Private`$rumBaseConvCache = <||>;
HOL`Auto`Simp`Private`$rumBaseConvCacheHits = 0;
HOL`Auto`Simp`Private`$rumBaseConvCacheMisses = 0;
```

After this cleanup, full HOL-context `DumpSave` through Pair completes: load
`86.9s`, remove `14.1s`, dump `0.0s`, total `114.3s`, snapshot size `500KB`.
Restoring `/private/tmp/rum-baseconv-exp-bootstrap-pair-filtered.mx` takes
`0.1s`, `pairInjThm` is still a theorem, and restored `pair_tests.wl` reports
`42/0`. A real snapshot patch should perform generated-symbol cleanup before
dumping, then restore intentional runtime globals/lazy caches explicitly.

## 2026-06-13 Task 2c Simp Fixpoint Loop Count Diagnosis

Ran the handoff's Task 2c two-engine loop-count check on a scratch copy of
`mma-hol` at `/private/tmp/rum-mma-hol-simp-loop.sSsJv1/mma-hol`. The scratch
`auto/Simp.wl` instruments the WL-level `simpFixpointConv` with counters for:

- total `simpFixpointConv` entries;
- total `While` iterations;
- maximum `seen` association size;
- fixpoint breaks (`newRhs === rhs`);
- cycle breaks (`KeyExistsQ[seen, newRhs]`).

The probe `/private/tmp/rum-simp-loop-user-rule-probe.wl` constructs the same
`mkPairDefThm` / `lhsAppXY` shape as the Pair/Simp probes and runs the
completing user-rule-only conversion:

```wolfram
HOL`Auto`Simp`Private`simpFixpointConv[
  {mkPairDefThm}, {}, HOL`Auto`Simp`Private`$simpDepthLimit][concl[lhsAppXY]]
```

The release `rum-cli` binary was rebuilt after reverting the Task 3b carrier.
Results:

| Engine | Time | Entries | While Iterations | Max Seen | Fixpoint Breaks | Cycle Breaks |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| rum release | `40.977489205s` | `3565` | `3573` | `3` | `3565` | `0` |
| WolframKernel | `12.837901s` | `3565` | `3573` | `3` | `3565` | `0` |

Verdict: the completing user-rule-only mkPair shape is not looping in rum.
The step counts and break reasons are identical to WolframKernel, so this
probe does not support the high-value `newRhs === rhs` / origin-annotation
non-convergence hypothesis. The observed ratio is about `3.2x`, squarely in
the current per-step constant range. The remaining Pair/Simp gate should now
be treated as either a downstream `Simp` algorithm issue or a general evaluator
constant-factor / representation issue; the next cheap split is still one
`mkPair` application term under user rule only, basic rules only, and smaller
partial applications. If the full public ruleset is suspected of a separate
non-convergence behavior, use an in-run snapshot/logging probe rather than
inferring from killed stdout.

Validation / commands:

- `scripts/run-bounded.py --timeout 180 -- cargo build --release -p rum-cli`
- `scripts/run-bounded.py --timeout 120 -- target/release/rum-cli --file /private/tmp/rum-simp-loop-user-rule-probe.wl`
- `scripts/run-bounded.py --timeout 120 -- /Applications/Wolfram.app/Contents/MacOS/WolframKernel -noprompt -script /private/tmp/rum-simp-loop-user-rule-probe.wl`

## 2026-06-13 Task 3b Shared-Subtree Stamp Prototype Reverted

A temporary opt-in call-stability stamp prototype was built for the
representation-work Task 3 lane, then reverted before commit after external
review found a default-path regression. The prototype stored a runtime
stability verdict on the existing shared argument-vector allocation so cloned
call subtrees could carry the verdict across rule RHS substitution.

Implementation shape tested, not retained:

- `SharedArgs` wrapped an internal argument-vector object with a
  non-serialized `CallStabilityStamp` slot. Equality, hashing, display, and
  serde still use only the argument vector, preserving the public expression
  shape and textual snapshot format.
- The stamp recorded the call head `SharedExpr`, call-stability generation,
  stable verdict, dependency bloom, and dirty snapshot. Keeping the head Arc in
  the stamp avoids unsafe pointer-reuse hits when the same argument vector is
  later paired with a different head.
- Mutable access to `SharedArgs` cleared the stamp. Cloning a shared argument
  vector preserves the stamp; clone-on-write copies start with no stamp.
- Runtime use was behind `RUM_CALL_STABILITY_INLINE_STAMP=1`, so default
  evaluation still consulted the existing pointer-keyed side cache. When
  enabled, `call_stability_with_bloom` checked the shared-args stamp first,
  fell back to the side cache on miss, and stored a stamp whenever it inserted
  or hit a stable side-cache entry.

Prototype-only profile counters under the existing
`RUM_PROFILE_EVALUATION_CACHE=1` switch were:

- `call_stability_stamp_hits`
- `call_stability_stamp_miss_absent`
- `call_stability_stamp_miss_head`
- `call_stability_stamp_miss_generation`
- `call_stability_stamp_miss_dirty`
- `call_stability_stamp_stores`

Prototype litmus coverage:

- a shared call subtree remains stable through its inline stamp after the
  side cache is manually cleared, proving that the verdict travels with the
  shared subtree rather than only living in the session map;
- a later write to the subtree's call head invalidates the stamp through the
  existing dirty-bloom logic, proving the prototype does not reuse stale stable
  verdicts across relevant definition writes.

Small WL-layer probe:

```bash
RUM_PROFILE_EVALUATION_CACHE=1 \
RUM_PROFILE_SUBSTITUTION_SHARING=1 \
RUM_CALL_STABILITY_CACHE_MAX_ENTRIES=1 \
RUM_CALL_STABILITY_INLINE_STAMP=1 \
  scripts/run-bounded.py --timeout 120 -- \
  cargo run -p rum-cli -- --eval \
  'f[x_] := q[x, x, x, x, x, x, x, x]; f[g[h[1]]]'
```

Observed output includes `rule_rhs_binding_call_hits=8`,
`call_stability_stamp_stores=2`, and `call_stability_stamp_hits=8`, confirming
that repeated rule-RHS insertion of one shared call subtree can be seen by the
stamp path. This proves the carrier mechanism, not the full Pair/Simp payoff.

Warm public mkPair A/B result: do not promote this path to the default runtime.
The release probe preloads through `auto/Simp.wl`, preheats
`HOL`Auto`Simp`basicSimpset[]` and the three congruence lemmas, then runs
`CONVRULE[HOL`Auto`Simp`simpConv[{mkPairDefThm}], lhsAppXY]`. With
`RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_SUBSTITUTION_SHARING=1`, both the
default runtime and `RUM_CALL_STABILITY_INLINE_STAMP=1` timed out at `240s`;
the inline-stamp run did show real stamp activity
(`call_stability_stamp_hits=33M+`, `call_stability_stamp_stores=16M+`), but it
reached the same `268M+` stability-check scale and did not move the gate.
No-profile `360s` runs with and without the env var also timed out.

Follow-up phase probes corrected one measurement trap: timeout logs can lose
earlier stdout `Print` output, so absence of `RUM_PROBE preheat_basic=` in the
full timeout log does not mean the preheat failed to finish. Dedicated probes
that exit after the phase show:

- standalone public `basicSimpset[]`: `24.40s`, length `19`, all theorems;
- mkPair setup plus `basicSimpset[]`: `22.68s`;
- mkPair setup plus `basicSimpset[]` plus all three congruence lemmas:
  `22.37s + 0.0018s`.

The retained value is diagnostic: the carrier mechanism works on small shared
subtree litmus tests, but it does not close the current public Simp gate. The
current gate remains the final public
`CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]` conversion after normal preheat,
not public `basicSimpset[]` construction.

External review then measured the uncommitted carrier against committed HEAD
and found a default-path regression: even with
`RUM_CALL_STABILITY_INLINE_STAMP` unset, every `SharedArgs` allocation paid for
the extra `Mutex<Option<CallStabilityStamp>>` slot. The PropTaut distribution
probe regressed from about `10.2s` without the carrier to about `11.6s` with
it. Because the prototype did not move the Pair/Simp gate, the carrier and its
runtime path were reverted. The current code intentionally has no
`CallStabilityStamp`, no `RUM_CALL_STABILITY_INLINE_STAMP` runtime path, and no
per-`SharedArgs` stamp slot; only the negative-result evidence and Task 3a
substitution-sharing counters remain.

Final-conversion split: the first corrected phase probes narrowed the timeout
inside `simpConv` itself. A false-start probe accidentally applied conversions
to the theorem object instead of `concl[lhsAppXY]`; the corrected probes use
`targetTm = concl[lhsAppXY]`.

- `buildBaseConv[Join[{mkPairDefThm}, basic], {}, $simpDepthLimit]` is tiny:
  `0.0048s` for 20 rules.
- `TRYCONV[REPEATC[baseConv]][targetTm]` is also tiny: `0.079s`, and leaves
  the term unchanged.
- `descendOnce[allRules, {}, $simpDepthLimit][targetTm]` times out at `240s`.
- Splitting the ordinary `comb` branch shows both recursive subcalls are
  already enough to reproduce the long tail:
  `simpFixpointConv[allRules, {}, depth][targetTm[[1]]]` times out at `240s`,
  and `simpFixpointConv[allRules, {}, depth][targetTm[[2]]]` also times out at
  `240s`.

That rules out rule preparation, `ORELSEC` chain construction, and the top
rewrite pass as the immediate bottleneck. The next split should target a
single `mkPair` application term under different rule sets: user rule only,
basic rules only, and progressively smaller partial applications, to decide
whether the multiplier is the user definition, interaction with the 19 basic
rules, or descent over the higher-order `mkPair` expansion shape.

While running workspace validation, the Lane D smoke-baseline test exposed a
test-thread stack limit: the same lane passed as a CLI command and with
`RUST_MIN_STACK=33554432`, but overflowed the default Rust test thread stack.
The test now runs its all-smoke lane in a 32 MiB thread so workspace validation
is not blocked by harness stack size.

Validation:

- `scripts/run-bounded.py --timeout 120 -- cargo test -p rum-core`
- `scripts/run-bounded.py --timeout 120 -- cargo test -p rum-runtime binding_substitution`
- `cargo fmt --check`
- `git diff --check`
- `scripts/run-bounded.py --timeout 420 -- cargo test -p rum-runtime`
- `scripts/run-bounded.py --timeout 180 -- cargo test -p rum-difftest --lib`
- `RUM_PROFILE_EVALUATION_CACHE=1 RUM_PROFILE_SUBSTITUTION_SHARING=1 RUM_CALL_STABILITY_CACHE_MAX_ENTRIES=1 RUM_CALL_STABILITY_INLINE_STAMP=1 scripts/run-bounded.py --timeout 120 -- cargo run -p rum-cli -- --eval 'f[x_] := q[x, x, x, x, x, x, x, x]; f[g[h[1]]]'`
- `scripts/run-bounded.py --timeout 240 -- target/release/rum-cli --file /private/tmp/rum-mkpair-warm-public-probe.wl` with and without `RUM_CALL_STABILITY_INLINE_STAMP=1` under profile counters; both timed out
- `scripts/run-bounded.py --timeout 360 -- target/release/rum-cli --file /private/tmp/rum-mkpair-warm-public-probe.wl` with and without `RUM_CALL_STABILITY_INLINE_STAMP=1` without profile counters; both timed out
- `scripts/run-bounded.py --timeout 120 -- target/release/rum-cli --file /private/tmp/rum-basic-simpset-public-probe.wl`
- `scripts/run-bounded.py --timeout 120 -- target/release/rum-cli --file /private/tmp/rum-mkpair-preheat-basic-only.wl`
- `scripts/run-bounded.py --timeout 180 -- target/release/rum-cli --file /private/tmp/rum-mkpair-preheat-cong-only.wl`
- `scripts/run-bounded.py --timeout 120 -- target/release/rum-cli --file /private/tmp/rum-mkpair-build-baseconv-only.wl`
- `scripts/run-bounded.py --timeout 180 -- target/release/rum-cli --file /private/tmp/rum-mkpair-top-baseconv-only.wl`
- `scripts/run-bounded.py --timeout 240 -- target/release/rum-cli --file /private/tmp/rum-mkpair-descend-only.wl`
- `scripts/run-bounded.py --timeout 240 -- target/release/rum-cli --file /private/tmp/rum-mkpair-descend-left-only.wl`
- `scripts/run-bounded.py --timeout 240 -- target/release/rum-cli --file /private/tmp/rum-mkpair-descend-right-only.wl`
- post-revert `rg` over `crates/` confirms no remaining
  `CallStabilityStamp`, `RUM_CALL_STABILITY_INLINE_STAMP`, or inline-stamp API
- `scripts/run-bounded.py --timeout 600 -- cargo test --workspace` in the
  ordinary sandbox failed only because native Jupyter ZeroMQ tests cannot bind
  loopback ports there; the same bounded workspace command passed outside the
  sandbox.

## 2026-06-13 Task 3a Substitution Sharing Audit

Started the representation-work Task 3a from
[docs/next-session-handoff.md](./docs/next-session-handoff.md): audit whether
rule RHS instantiation already preserves enough subtree sharing for a future
inline evaluated/stable stamp to travel with reused subexpressions.

Findings from the code read:

- `Expr` currently stores calls as `Call { head: SharedExpr, args:
  SharedArgs }`, so the current top-level call node is not itself a single
  shared allocation, but cloned call expressions do preserve the shared head
  and argument-vector arcs.
- `substitute_rule_bindings_for_rhs` preserves sharing in the important
  existing paths: a bound call value is inserted by `Expr::clone`, unchanged
  sibling call arguments are cloned into rebuilt argument vectors, and
  `Sequence` splices clone the spliced item expressions.
- Parent calls are still rebuilt whenever an RHS changes, so the current
  pointer-keyed call-stability cache still sees fresh parent pointers. This
  supports the Task 2b review constraint: a future stamp must live on / travel
  with shared subtrees, not just freshly allocated parent nodes.
- `lexical_substitution.rs` follows the same broad clone-preserving shape for
  unchanged children, but the Pair/Simp gate's direct pressure path is rule
  RHS instantiation through `binding_substitution.rs` and
  `definition_application.rs`.

Added opt-in substitution-sharing counters under
`RUM_PROFILE_SUBSTITUTION_SHARING=1`. They print lines like
`rum-profile substitution-sharing rule_rhs_binding_call_hits=N` at powers of
two, matching the existing evaluation-cache counter style. Initial counters:

- `rule_rhs_substitution_requests`
- `rule_rhs_fast_clone_empty_bindings`
- `rule_rhs_binding_hits`
- `rule_rhs_binding_call_hits`
- `rule_rhs_binding_atom_hits`
- `rule_rhs_rebuilt_reuse_shared_args`
- `rule_rhs_rebuilt_reuse_shared_head`
- `rule_rhs_rebuilt_fresh_head_args`
- `rule_rhs_preserved_existing_args`
- `rule_rhs_preserved_existing_call_args`
- `rule_rhs_splices_from_binding`
- `rule_rhs_splices_from_existing_rhs`

Added unit-level litmus coverage proving that rule RHS substitution preserves
shared nodes for:

- repeated insertion of the same bound call subtree;
- unchanged sibling call arguments copied into a rebuilt RHS parent;
- spliced `Sequence` items from a bound value.

Small WL-layer probe:

```bash
RUM_PROFILE_SUBSTITUTION_SHARING=1 \
  scripts/run-bounded.py --timeout 30 -- \
  cargo run -p rum-cli -- --eval 'f[x_] := q[x, x]; f[g[h[1]]]'
```

Observed output includes two `rule_rhs_binding_call_hits`, confirming that the
definition-application path sees both `x` occurrences as call-subtree binding
insertions. This is a necessary but not sufficient condition for Task 3: the
next design check is whether a prototype call-node stamp can exploit this
subtree reuse to reduce absent call-stability misses on a workload with real
shared subterms.

Validation:

- `scripts/run-bounded.py --timeout 120 -- cargo test -p rum-runtime binding_substitution`
- `RUM_PROFILE_SUBSTITUTION_SHARING=1 scripts/run-bounded.py --timeout 120 -- cargo test -p rum-runtime rule_rhs_substitution_reuses_bound_call_shared_nodes -- --nocapture`
- `cargo fmt --check`
- `git diff --check`
- `scripts/run-bounded.py --timeout 420 -- cargo test -p rum-runtime`
- `RUM_PROFILE_SUBSTITUTION_SHARING=1 scripts/run-bounded.py --timeout 120 -- cargo run -p rum-cli -- --eval 'f[x_] := q[x, x]; f[g[h[1]]]'`

## 2026-06-13 Remaining Helper Suspension Continuations

Closed the remaining public helper tranches from the 2026-06-13
suspension-safe helper audit.

Targeted mapping now preserves loop state across host input:

- `MapAt` saves the operator, remaining exact paths, current expression, and
  completed replacements;
- `SubsetMap` saves the operator result boundary and also frames the final
  post-replacement evaluation, so a replacement-triggered input request can
  resume to the full `SubsetMap` result.

Tensor and string callback helpers now use saved continuation state:

- `ArrayReduce` and `ArrayFilter` share a tensor callback loop over prepared
  operator arguments and output shape;
- `Inner` saves both the term-building `f` phase and the reduction `g` phase;
- `StringApply` preserves scalar/list output shape across callback input;
- delayed `StringReplace` and `StringCases` rule outputs save string-scan
  position, accumulated output, match count, and list-input progress.

Selected debug/tooling flows now resume through their public callback or
capture boundaries:

- `RepeatedTiming` resumes through repeated samples;
- `TimeConstrained` / `MemoryConstrained` resume through limit evaluation and
  fallback evaluation;
- `Reap` preserves the body value while handler rendering suspends;
- `Trace` / `TracePrint` preserve trace checkpoints and restore tracing state
  after resumed capture;
- `TestReport` preserves stream-capture checkpoints, test-case progress,
  `SameTest` evaluation, and report rendering state.

Intentional non-promotions remain documented in `TODO.md`: `Monitor`
display/callback refresh is best-effort, `ValueQ` remains a rollback probe, and
`Scan` remaining-part state plus exotic `Sow` pattern-test suspension should be
reviewed only if a concrete downstream workflow promotes those edges.

Validation:

- `cargo fmt --check`
- `cargo test -p rum-runtime resume_eval_request_with_input_can_finish`
- `cargo test -p rum-runtime`

## 2026-06-13 Sequence Fold Module Split

Split the post-continuation
`crates/rum-runtime/src/runtime/functional_ops/sequence/fold.rs` file into a
`fold/` submodule tree:

- `fold/mod.rs` keeps the public builtin entry points and shared fold parsing
  helpers;
- `fold/plain.rs` owns `Fold` / `FoldList`;
- `fold/pair.rs` owns `FoldPair`;
- `fold/while_loop.rs` owns `FoldWhile`;
- `fold/sequence_fold.rs` owns `SequenceFold` / `SequenceFoldList`.

All split files are under `10k` chars, closing the split-plan item introduced
after the fold/window suspension continuation pass.

Validation:

- `cargo fmt`
- `cargo test -p rum-runtime functional_ops::sequence`
- `cargo test -p rum-runtime functional_sequence_resume`

## 2026-06-13 Iteration Suspension Continuations

Closed the functional-iteration callback tranche from the suspension-safe
helper audit. The iteration helpers now save their state before evaluating
operator or test callbacks and resume from the saved point instead of
restarting the whole request:

- `Nest` / `NestList` preserve the current value, iteration index, and
  collected history for the list form;
- `FixedPoint` / `FixedPointList` preserve current value, seen values, iteration
  count, and list history, keeping the existing fixed-point, cycle, and
  max-iteration behavior;
- `NestWhile` / `NestWhileList` resume through both the test callback and the
  step callback while preserving history and max-iteration semantics.

Added host-level regression coverage that drives these callbacks through
`InputString` and `resume_eval_request_with_input`, including repeated
suspensions in `Nest`.

Validation:

- `cargo fmt`
- `cargo test -p rum-runtime functional_ops::iteration`
- `cargo test -p rum-runtime functional_iteration_resume`

## 2026-06-13 Fold/Window Suspension Continuations

Closed the fold/window callback tranche from the suspension-safe helper audit.
The sequence fold helpers now save their loop state before evaluating callback
expressions and resume from the saved point instead of restarting the whole
request:

- `Fold` / `FoldList` preserve current value, index, and collected history;
- `FoldPair` preserves pair state and also resumes through an optional
  projector callback;
- `FoldWhile` resumes through both the operator callback and the test callback,
  including positive and negative tail-adjustment handling;
- `SequenceFold` / `SequenceFoldList` preserve seed history and item offset;
- `TakeWhile` / `LengthWhile` preserve the accepted prefix count over both
  ordinary heads and associations;
- `Accumulate` preserves accumulated values while evaluating the internal
  `Plus` step;
- `BlockMap` preserves completed block results and the next block index.

Added host-level regression coverage that drives these callbacks through
`InputString` and `resume_eval_request_with_input`, including repeated
suspensions in the same helper loop.

Validation:

- `cargo fmt --check`
- `cargo test -p rum-runtime functional_ops::sequence`
- `cargo test -p rum-runtime functional_sequence_resume`
- `cargo test -p rum-runtime`

## 2026-06-13 Suspension-Safe Helper Audit

Completed the first audit pass over non-`Expr` and helper-level
suspension/resume paths after the host/Jupyter keepalive work. The main
evaluator, fixed-point resume stack, message-template evaluation, matcher,
traversal, replacement, association transforms, `Select` / `SelectFirst`, and
the `Array` / `Outer` generators already use ordinary suspension frames or
typed `ContinuationValue` drivers in the currently tested subset.

The remaining concrete gaps are narrower than the old TODO suggested. Public
callback loops that still call `evaluate(...)?` without saving loop state are
the next correctness tranche:

- functional iteration helpers in `functional_ops/iteration.rs`:
  `Nest`, `NestList`, `FixedPoint`, `FixedPointList`, `NestWhile`, and
  `NestWhileList`;
- fold/window helpers in `functional_ops/sequence/fold.rs` and
  `functional_ops/sequence/window.rs`: `Fold`, `FoldList`, `FoldPair`,
  `FoldWhile`, `SequenceFold`, `SequenceFoldList`, `TakeWhile`,
  `LengthWhile`, `Accumulate`, and `BlockMap`;
- targeted mapping helpers in `functional_ops/targeted_map.rs`:
  `MapAt` and `SubsetMap`;
- array/string callback helpers outside the already-fixed `Array` / `Outer`
  lane: `ArrayReduce`, `Inner`, `ArrayFilter`, `StringApply`, and delayed
  string-rule outputs for `StringReplace` / `StringCases`;
- debug/tooling helpers where the body is sometimes framed but option,
  fallback, repeated, filtered, or handler evaluation is not:
  `RepeatedTiming`, `Reap` / `Sow` handler paths, `Trace` capture,
  `TestReport` same-test/capture helpers, and `TimeConstrained` /
  `MemoryConstrained` limit and fallback expressions.

Some paths are intentionally not promoted by this audit. `Monitor` display and
callback refresh remain best-effort and clear suspension state if the display
expression asks for input; `ValueQ` retains rollback-style probing rather than
a resumable public continuation. `Scan` has per-part suspension frames but not
a saved remaining-parts loop state, so it should be reviewed with the broader
functional callback tranche rather than counted as fully closed.

## 2026-06-13 Runtime Namespace Test Split

Split the regrown
`crates/rum-runtime/src/runtime/tests/names_and_syntax_information.rs` file
along behavior boundaries and removed the mixed module. The coverage now lives
in:

- `namespace_names.rs` for `Names` pattern, context, and option behavior;
- `syntax_information_lifecycle.rs` for assignment, unset, protection, `Block`,
  `Clear`, and `ClearAll` behavior;
- `syntax_information_core_catalog.rs` for the current core builtin
  `SyntaxInformation` catalog;
- `syntax_information_debug_catalog.rs` for the CL-09 debugging/tooling catalog.

The largest split file is under `10k` chars, so this closes the immediate
organization debt without changing test assertions.

## 2026-06-13 Lightweight Correctness Debt Closure

Closed three non-performance cleanup items from the immediate debt list.

`Catch` tag-form side-effect scoping was rechecked against the local
WolframKernel oracle. Both the matching case
`Catch[Throw[1, "t"], (Print["tag"]; x = 9; "t")]` and a nonmatching nested
`Catch` case preserve visible `Print` output and leave the assignment visible
after the `Catch`. The earlier rollback-debt note was therefore wrong for this
path. rum keeps its ordinary matcher-pattern resolution and now has regression
coverage for the visible side effects.

The syntax lexer now reports `UnexpectedCharacter` spans over the offending
character's byte range instead of a zero-width `start..start` placeholder,
including multi-byte invalid characters.

REPL history persistence now stores multiline entries as a single encoded
physical line and decodes them back to one logical history entry on load. Plain
single-line history remains readable and backward-compatible.

Validation:

- `cargo fmt --check`
- `cargo test -p rum-runtime catch_tag_form_keeps_visible_side_effects`
- `cargo test -p rum-runtime holding_and_control_part6::cleanup_throw`
- `cargo test -p rum-syntax`
- `cargo test -p rum-repl`

## 2026-06-13 DumpSave Package Registration Restore

Closed the stable-snapshot `$Packages` workaround as a rum runtime conformance
bug. `mma-hol` maintainers verified that Wolfram Language 14.3 restores package
registration when `Get` reads a `DumpSave` `.mx`: public package contexts from
the snapshot reappear in both `$Packages` and `$ContextPath`, while
`` `Private` `` contexts do not. The earlier proposal to patch
`tests/extend_snapshot.wls` therefore masked a rum restore divergence and
should not be carried into upstream `mma-hol`.

rum's textual `DumpSave` now writes restore metadata for package contexts that
were present in the dumping session's `$Packages` and covered by the saved
symbol set. Reading the snapshot through ordinary `Get` evaluates that metadata
before the saved definitions, restoring the public package contexts to
`$Packages` and `$ContextPath` while excluding `` `Private` `` contexts.

Regression coverage updates
`dump_save_context_strings_roundtrip_public_and_private_contexts`: after
`Get[bootstrap.mx]`, `Select[$Packages, StringMatchQ[#, "Snap`*"] &]` and the
matching `$ContextPath` query both return only `"Snap`"`, short-name
`public[9]` resolves, and `Needs["Snap`"]` is a no-op.

Validation:

- `cargo test -p rum-runtime dump_save_context_strings_roundtrip_public_and_private_contexts`
- `cargo test -p rum-runtime package_tools`
- `scripts/mma-hol-bounded-smoke.sh --mode snapshot` with release `rum-cli` on
  a refreshed unmodified clone at
  `/private/tmp/rum-mma-hol-unmodified.itrrai/mma-hol` (HEAD `51bbe9e`):
  `snapshot/build-parser`, `snapshot/run-fast-parser`, and
  `snapshot/extend-proptaut` all exited `0`; `snapshot_extend-proptaut.stderr`
  was empty.

## 2026-06-13 Pair/Simp Stability-Check Amplifier Diagnosis

Completed Task 2b from
[docs/next-session-handoff.md](./docs/next-session-handoff.md). The unexplained
warm public mkPair signal is not call-stability cache size-cap thrash, and a
wider dirty-bloom saturation threshold is a negative optimization. The dominant
mechanism is fresh pointer-keyed call trees from the Simp/Drule conversion
chain, with dirty-overlap cache skips as a secondary cost.

The opt-in `RUM_PROFILE_EVALUATION_CACHE=1` counters now also report
call-stability cache size-cap clears, dirty-bloom saturations, stable insertions,
rewritten-dirty skips, and miss reasons (`absent`, `generation`, `dirty`,
`pointer`). Two diagnostic A/B knobs were added for bounded probes only:
`RUM_CALL_STABILITY_CACHE_MAX_ENTRIES` and
`RUM_STABILITY_DIRTY_BLOOM_MAX_BITS`.

Counter split on the clean-clone warm public mkPair shape
(`basicSimpset[]`, `impCongLemma[]`, `andCongLemma[]`, `orCongLemma[]` preheated,
then `CONVRULE[simpConv[{mkPairDefThm}], lhsAppXY]`):

- default 180s bounded window: timeout, `stability_checks=268M+`,
  `evaluate_entries=67M+`, `down_value_applications=8M+`,
  `call_generation_bumps=16k+`, `dirty_bloom_saturations=16k+`,
  `call_stability_size_cap_clears=1024+`, `call_stability_hits=67M+`,
  `call_stability_misses=134M+`, `call_stability_insertions=16M+`, and
  `call_stability_rewritten_dirty_skips=33M+`.
- 16x cache-cap A/B (`RUM_CALL_STABILITY_CACHE_MAX_ENTRIES=262144`) also timed
  out at 180s with the same major lower bounds; only
  `call_stability_size_cap_clears` fell to `32+`. This rejects size-cap thrash
  as the amplifier.
- dirty-bloom threshold A/B
  (`RUM_STABILITY_DIRTY_BLOOM_MAX_BITS=128`) also timed out and was worse:
  `call_generation_bumps`/`dirty_bloom_saturations` disappeared, but
  `stability_checks=4B+`, `call_stability_misses=1B+`,
  `call_stability_rewritten_dirty_skips=1B+`, `call_stability_hits=32k+`, and
  `call_stability_insertions=2k+`. The generation bump is therefore a
  protective reset against rewritten-dirty accumulation, not the primary
  amplifier.
- default 120s miss-reason window: timeout, `stability_checks=268M+`,
  `evaluate_entries=67M+`, `down_value_applications=8M+`,
  `call_stability_miss_absent=67M+`, `call_stability_miss_dirty=33M+`,
  `call_stability_miss_generation=4M+`, and no visible pointer-mismatch lower
  bound. The main misses are absent pointer keys, not stale generations.

Verdict: no cheap cache-cap or bloom-threshold fix closes the gate. The warm
public mkPair / `Pair.wl` gate should move either to a downstream `Simp`
algorithm experiment that avoids rebuilding the nested `ORELSEC` conversion
chain, or to the larger runtime representation lane (single-allocation call
nodes with inline stable/evaluated stamps and borrowed argument preparation).
Do not spend more time on call-cache knob tuning without a new probe signal.

## 2026-06-12 `Catch` Tag-Form Evaluation Fix

Closed the `mma-hol` Meson semantic bug identified in
[docs/next-session-handoff.md](./docs/next-session-handoff.md) Task 1.
`Catch` now resolves its optional tag form through the same matcher-pattern
normalization path used by `MatchQ` / `MemberQ`: explicit pattern syntax stays
held, while ordinary expressions are evaluated before compiling the exact
matcher. The implementation preserves body-first `Catch` behavior; the tag
form is resolved only when a `Throw` reaches the `Catch` frame.

The regression covers both sides of the symbol-valued tag case:
`tag = "$t$"; Catch[Throw[42, tag], tag]` now returns `42`, while
`tag = "$t$"; Catch[Throw[42, "$u$"], tag]` still escapes as a non-matching
throw with tag `"$u$"`.

Validation:

- `cargo test -p rum-runtime catch_evaluates_symbol_valued_tag_form_before_matching`
- `cargo test -p rum-runtime holding_and_control_part6::cleanup_throw`
- `cargo test -p rum-runtime`
- bounded clean-clone `test/meson_tests` smoke with the release CLI:
  `List[44, 0]` in `79s` on `mma-hol` HEAD `698e3d0`. The handoff expected
  `List[43, 0]`, but the current test tree reports 44 cases; the semantic gate
  is zero failures.
- bounded clean-clone `test/meson_proveprop_tests` was re-probed after the fix
  and timed out at `300s` without a stdout summary. The bounded runner killed
  the process group cleanly, so the remaining status is a performance/scale
  gate rather than the now-fixed `Catch` tag-form bug.

Follow-up oracle check: WolframKernel does not sandbox nontrivial `Catch`
tag-form side effects in the tested exact-tag path. A tag form like
`(Print["tag"]; x = 9; "$t$")` prints while matching, and `x = 9` remains
visible after both matching and nonmatching nested-`Catch` cases. rum's
ordinary matcher-pattern resolution is therefore retained for this boundary,
with regression coverage for visible `Print` and assignment effects.

## 2026-06-12 Pair/Simp Multiplier Diagnosis

Completed the Task 2 diagnosis from
[docs/next-session-handoff.md](./docs/next-session-handoff.md). The verdict is
that `stdlib/Pair.wl` is not currently blocked by another Pair correctness bug
or by a single cold PropTaut lemma. The multiplier is in the
`HOL`Auto`Simp`` simplifier path triggered by
`HOL`Drule`CONVRULE[HOL`Auto`Simp`simpConv[{mkPairDefThm}], lhsAppXY]`: adding
`withBasic` turns a constant-range user-rule simplification into a
failure-heavy nested `ORELSEC` / `TRYCONV` / `REPEATC` conversion chain over
the `mkPair` application term.

Reference timings, all bounded and using internal `AbsoluteTiming` where
available:

- WolframKernel dependency-preloaded `Get["stdlib/Pair.wl"]`: `37.697875s`.
- rum release dependency-preloaded `load/stdlib/Pair`: timeout at `300s`,
  with empty stdout and only the bounded-runner timeout in stderr.
- WolframKernel phase probe: `basicSimpset[]` `7.327259s`,
  `mkPairDefThm` construction `0.000538s`,
  first public `simpConv[{mkPairDefThm}]` `CONVRULE` `29.306584s`, and the
  later `simpConv[{xEqxTh, yEqyTh}]` collapse `0.76233s`.
- rum release `basicSimpset[]`: `27.070526269s`, about `3.7x` the
  WolframKernel time and in the current systemic constant-factor range.
- `andCongLemma[]` alone: WolframKernel `7.117217s`, rum
  `25.646827478s`, also about `3.6x`.
- Warm public mkPair simplification after precomputing `basicSimpset[]`,
  `impCongLemma[]`, `andCongLemma[]`, and `orCongLemma[]`: WolframKernel
  `23.10866s`, rum timed out at `360s`.
- User-rule-only mkPair simplification using
  `simpFixpointConv[{mkPairDefThm}, {}, $simpDepthLimit]`: WolframKernel
  `12.84882s`, rum `52.137050616s`, about `4.1x`.

The opt-in `RUM_PROFILE_EVALUATION_CACHE=1` counter was extended to report
`evaluate_entries`, `down_value_scans`, and `down_value_applications`, with
per-symbol power-of-two lower bounds. Counter probes show:

- `basicSimpset[]` completes with lower bounds of `134M` stability checks,
  `33M` evaluate entries, and `2M` downvalue applications.
- preheating `basicSimpset[]` plus the three congruence lemmas completes in
  `111.742s` with `268M` stability checks, `134M` evaluate entries, and
  `8M` downvalue applications.
- the warm public mkPair simplification still times out at `180s`; its lower
  bounds reach `536M` stability checks while downvalue/evaluate powers do not
  advance proportionally past the preheat background. The continuing tail is
  in the Simp/Drule conversion dispatcher: `isEquation`, `combineSimpConvs`,
  `tryConv`, `tryTermMatch`, `tryStructComb`, `collectAppSpine`, and
  `convFail`.
- The hottest successful downvalue symbols in the 180s warm run are term/type
  plumbing rather than Pair-specific definitions: `stripOrigin`, `typeOf`,
  `destFunType`, `tyMatch`, `vsubstImpl`, `freesWalk`, and `betaSubst`.

This identifies the next repair target more narrowly than broad evaluator or
representation work: validate and then fix the `Simp` base-conversion
construction/use pattern. The leading hypothesis is that recursive
`simpFixpointConv` / `descendOnce` rebuilds and re-evaluates a large
`withBasic` conversion chain for many subterms, and rum pays for the chain
through repeated stability checks and failure-driven conversion dispatch. A
targeted next experiment should hoist/cache prepared simplification rules or
the base conversion for the duration of a simplification pass, or otherwise
avoid repeatedly evaluating the same nested `ORELSEC` chain. Only if that
does not move the warm public mkPair probe should this return to the larger
single-allocation call-node / evaluated-stamp representation lane.

## 2026-06-12 Pair/Simp Runtime Stability Cuts

Followed up the Pair/Simp diagnosis on the `rum-runtime` side. The temporary
`mma-hol` source-level experiment that memoized `buildBaseConv` moved the
user-rule-only mkPair probe only from about `52.27s` to `51.10s`, and the
warm public mkPair probe still timed out at `240s`; that makes base-conversion
hoisting an insufficient standalone fix.

The retained runtime changes are conservative stability and dispatch cuts for
expressions that evaluate to themselves:

- stable atom evaluation now bypasses the full fixed-point evaluator for
  already-stable symbols and non-symbol atoms in call heads and ordinary
  argument preparation;
- `Association`, `Rule`, and `RuleDelayed` participate in call-stability
  caching when their data shape is already normalized and dependency-safe;
- `Part[expr, i, j, ...]` has a successful exact-integer fast path that
  reuses the existing exact structural selector and falls back to the old path
  for associations, selector errors, and diagnostics;
- `HoldComplete` / `HoldAllComplete` and ordinary hold-literal forms
  (`Hold`, `HoldAll`, `HoldPattern`, `Function`) participate in stability
  caching when no direct `Evaluate[...]` or `Unevaluated[...]` escape would
  change argument preparation;
- normalized status/data objects (`Failure`, `Missing`, `Success`, and
  `TerminatedEvaluation`) participate in stability caching, while
  `Failure["tag", rule-list]` shapes that still need details normalization
  stay on the old evaluator path.

Release timing on the local clean-clone Pair/Simp probes:

- user-rule-only mkPair simplification:
  `52.14s` before these cuts, `41.77s` after data-constructor stability,
  `41.44s` after the `Part` fast path, and `40.61s` after status/data
  stability.
- `basicSimpset[]`: `27.07s` before these cuts, `21.02s` after
  data-constructor stability, `20.80s` after the `Part` fast path, and
  `20.27s` after status/data stability.
- warm public mkPair simplification still timed out at `240s` after the
  retained runtime stability cuts.

The warm public mkPair / `stdlib/Pair.wl` gate is not closed by these cuts.
The remaining profile-visible work is mostly ordinary evaluator/dispatch cost
over HOL term/type plumbing (`stripOrigin`, `typeOf`, `destFunType`,
`tyMatch`, `vsubstImpl`, `freesWalk`, `betaSubst`) plus high-frequency
ordinary builtins such as `Part`, `If`, `Function`, `Module`, and `Length`.
Further improvement now points either to a real `Simp` algorithmic change in
the downstream package, or to the larger call representation / argument
preparation rework rather than another narrow stability tweak.

## 2026-06-12 Block Scope Invalidation Fix (External)

Re-examined the same-bit follow-up below with a decomposed re-access probe
and corrected the earlier diagnosis: the slow interleaved-write shape was
never about the dirty-bloom same-bit fallback. `Do[e2;, {100}]` re-accessed
a cached 100k-node expression at `2.3us` per iteration while
`Do[e2;, {i, 100}]` cost `428us` per iteration with no writes at all — the
cost was the iterator binding, not the write invalidation. Do/Table
iterator bindings wrap each iteration in `Block[{i = value}, body]`, and
Block entry and exit each called `bump_attribute_revision()`
unconditionally, which clears all three stability caches and invalidates
every compiled definition matcher twice per iteration; the scope-exit path
also rebuilt the message-symbol counts once per restored symbol.

The retained fix (`6df7bfe`) grades Block entry/exit invalidation by what
actually changed: definition shadow/restore invalidates just the affected
symbol, the attribute revision bumps only when an attribute set really
changed, and message-symbol counts rebuild at most once per scope exit.
The re-access probe drops from `332us` to `5.8us` per iteration and stays
flat in expression size. The PropTaut distribution probe is unchanged
(`13.2-13.4s`; its hot path holds no Blocks), `bool_tests` stays green, and
`cargo test -p rum-runtime` passes.

Consequences of this correction for the planning notes below: both the
same-bit rewritten-bloom scheme and the rejected exact-version experiments
were optimizing a path that the motivating probe never exercised; their
small effect sizes are explained. The probe-visible invalidation lane is
now genuinely closed. `meson_tests.wl` now completes inside a `300s`
budget and reports `List[34, 9]` — the 9 failures reproduce identically
on the pre-fix commit, so the Meson lane has crossed from a performance
gate to a semantic-triage gate. `stdlib/Pair.wl` still exceeds a `600s`
load budget with the fix in place, so the Pair gate is not a Block-scope
artifact; the next step there is a logical-work comparison of the
`Simp` simpset construction path against WolframKernel (multiplier hunt),
not further constant-factor tuning.

## 2026-06-12 v0.1 Acceptance Boundary

Added [docs/v0.1-acceptance-boundary.md](./docs/v0.1-acceptance-boundary.md)
as the canonical stabilization boundary for the first usable `v0.1` cut.

The document consolidates decisions that were previously spread across
`ARCHITECTURE.md`, the Jupyter host migration audit, and the `mma-hol` smoke
and snapshot notes:

- `v0.1` is a kernel and host-contract stabilization cut, not a full CAS or
  notebook product;
- `rum-host` is the primary public embedding boundary;
- `mma-hol` cold and stable-snapshot lanes are the main downstream pressure
  tests;
- Jupyter is scoped as a `rum-host`-backed basic Notebook tech preview with
  one documented deploy path;
- speculative showcase packages, Python adapter revival, broad frontend
  compatibility, graphics, and `WLBuiltin/` cleanup remain outside the cut.

## 2026-06-12 `rum-jupyter` Host Migration

Migrated `rum-jupyter` request handling onto `rum-host`.

- `rum-jupyter` now depends on `rum-host` instead of direct `rum-runtime` and
  `rum-stdlib`.
- `KernelAdapter`, bootstrap, socket, ZeroMQ, launcher, and tests now pass
  `HostSession`.
- Public Jupyter model outcomes use host-facing success, input-required,
  failure, and inspect response types instead of runtime response/failure
  types.
- Jupyter rendering now consumes host messages, prints, traces, failures, and
  text results; `Print` output maps to stdout streams and runtime messages map
  to stderr streams.
- The private Jupyter cursor helper was removed because inspect and completion
  cursor handling now belongs to `rum-host`.
- A lightweight manifest guard now fails if `rum-jupyter` reintroduces direct
  `rum-runtime` or `rum-stdlib` dependencies.

Validation: `cargo test -p rum-jupyter` passed locally, including native
ZeroMQ loopback tests.

Remaining Jupyter lane work: document one real Notebook deploy path and run
the host-backed adapter through an actual Notebook frontend.

## 2026-06-12 `mma-hol` Stable Snapshot Validation

Ran the stable snapshot lane against the fixed clean clone at
`/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol`, bounded through
`scripts/run-bounded.py`.

Follow-up: `scripts/mma-hol-bounded-smoke.sh --mode snapshot` now owns the
bounded stable snapshot lane. It runs three stages in order:
`snapshot/build-parser`, `snapshot/run-fast-parser`, and
`snapshot/extend-proptaut`. With `RUM_CLI_BIN=target/release/rum-cli`, the lane
is green on the fixed probe clone:

```text
snapshot/build-parser      ok
snapshot/run-fast-parser   ok
snapshot/extend-proptaut   ok
```

This run originally used a fixed probe clone whose `tests/extend_snapshot.wls`
restored `$Packages` alongside `$ContextPath`, making
`auto/PropTaut.wl` extension quiet. That workaround was later superseded by
the 2026-06-13 rum runtime fix: textual `DumpSave` restore now auto-registers
public package contexts, matching the WolframKernel behavior observed by
`mma-hol` maintainers.

The CLI now renders accumulated prints/messages before honoring `Exit[n]`, so
`tests/run_fast.wls parser` produces visible progress and parser test logs
while still exiting with the test status code.

- Added the missing `SelectFirst` sequence subset needed by
  `tests/build_snapshot.wls` partial layer selection. The supported forms now
  cover unary operator form, two-argument search, three-argument default, and
  the snapshot shape
  `SelectFirst[Range[Length[libs]], StringContainsQ[Last[libs[[#]]], pat] &, $Failed]`.
- Fixed `Get` to propagate `Print` output from loaded files, so snapshot
  scripts keep useful progress logs when they are invoked through `Get`.
- `tests/build_snapshot.wls Parser` now succeeds through 11 of 32 libraries in
  Stable mode, dumps 21 `HOL` contexts, and writes
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol/bootstrap.mx` at about
  `11904 KB`.
- `tests/run_fast.wls parser` exits with status `0` against that snapshot and
  prints the parser test log; the parser summary is `passed: 78`, `failed: 0`.
- `tests/extend_snapshot.wls auto/PropTaut.wl` exits with status `0`, restores
  the Parser snapshot, loads `auto/PropTaut.wl`, and re-dumps 23 `HOL`
  contexts at about `11940 KB`.

Remaining snapshot boundaries: full `build_snapshot.wls` still runs into the
known downstream `Pair.wl` / `Simp` performance gate.

## 2026-06-12 Same-Bit Stability Invalidation Follow-Up

Followed the external pass recommendation to test the same-symbol rewrite
hole in the call-stability dirty bloom. The retained change is a conservative
rewritten-dirty-bloom scheme in `evaluation_cache.rs`: repeated writes whose
bloom bit is already dirty no longer bump the whole call-stability generation,
but cache hits still reject entries touched by newly dirty bits or by bits
rewritten after the entry snapshot. New entries touching rewritten dirty bits
are not cached. This preserves the cheap 128-bit dependency filter without
pretending it is an exact symbol dependency set.

Measured and rejected alternatives:

- Exact per-symbol and per-bloom-bit version snapshots fixed the black-box
  re-access shape but regressed the PropTaut distribution probe to about
  `24.9s` and `24.6s`, respectively.
- Skipping call-stability entries related to dirty bits regressed the same
  probe to about `20.5s`.

Retained result: the PropTaut distribution probe now measures about
`12.73-12.88s` release internal timing on the reference machine, compared with
the external pass frontier of about `13.1s` and the local WolframKernel
internal timing of about `3.80s`. The bounded `proptaut_tests.wl` smoke is
green in about `239s`.

The original expert black-box script cannot currently run unchanged through
the default CLI path: `Range[10^6]` exceeds the generated-element limit and
`Do[..., {10000}]` overflows the current CLI/runtime stack. Scaled diagnostic
probes are therefore only partial reproductions: with `Range[10000]` and
`Do[100]`, repeated same-write re-access measured about `37-39us` per access;
with `Range[50000]` and `Do[100]`, about `164us` per access.

Checks after the retained change: `cargo fmt --check`, `git diff --check`,
`cargo test -p rum-runtime`, unsandboxed `cargo test --workspace`, and the
bounded `proptaut_tests.wl` smoke passed. The workspace-wide Rust test needs
unsandboxed execution because the `rum-jupyter` native ZeroMQ tests bind
loopback sockets.

Next evidence-backed representation work is not another exact-version cache
attempt. The remaining plausible cuts are the larger single-allocation call
node / inline evaluated-stamp representation, borrowed argument preparation,
and only if new profiling justifies it, a non-allocating exact dependency
representation for stability invalidation.

## 2026-06-12 Representation Optimization Pass (External)

A second external optimization pass reopened the performance lane after the
pause below and landed five retained structural changes. All numbers are the
PropTaut distribution probe (dependency-preloaded, release, internal
`AbsoluteTiming`) on the reference machine; WolframKernel internal reference
is `3.80s`.

- Symbol interning with dense pair ids (`7c0ff8a`): `Symbol` equality,
  hashing, and hot map lookups became integer operations; `definitions` and
  `attributes` moved from `BTreeMap` to id-hashed `HashMap`s. `28.6s -> 20.4s`.
- Dependency-aware stability-cache invalidation (`f1806fb`): definition
  writes invalidate exactly the written symbol plus a 128-bit dirty bloom
  over call-stability entries instead of clearing all three caches per
  assignment; only stable verdicts are cached. Full clears remain for
  package/context/attribute changes. `20.4s -> 19.9s`, and cache wipes per
  proof dropped from ~65k to ~4-8k generation bumps.
- mimalloc as the `rum-cli` global allocator (`cfd3680`): expression-tree
  churn made small-object allocation the largest single block (~34% self
  time). `19.9s -> 14.1s`.
- Message-symbol index plus pre-sized argument preparation (`9920aff`):
  `symbol_exists` stopped scanning every message definition per name lookup.
  `14.4s -> 13.9s`.
- Id-hashed pattern/lexical binding maps and a 32-byte `Symbol`
  (text-pointer + inline id): `13.9s -> 13.1s`.

End state: distribution probe `13.1s` (3.4x vs WolframKernel, down from
12.8x), Bool-stack load `0.78s` vs `0.19s` (4.1x, down from 11.5x).
Downstream gates moved materially: `proptaut_tests.wl` is green for the
first time (`238s` full suite, previously a hard timeout at `300s`),
`bool_tests` `6s`, `drule/tactics/printer/parser/kernel` tests `~1s` each.
`cargo test --workspace` (24 suites) and `rum-difftest check` pass. The
`stdlib/Pair.wl` load still exceeds a `600s` budget; the earlier `335s`
observation was an interrupted run, not a completion, so Pair remains the
next downstream gate after `proptaut_tests`.

The original `<2x` target was not reached. The remaining ~3.4x is spread
flat across the evaluator (no single item above ~7% self time): stability
scanning ~10%, allocator/drop/memmove ~20%, substitution ~7%, definition
matching ~8%, dispatch skeleton ~9%, long tail ~45%. Closing it requires a
representation rework, in order of expected leverage: a single-allocation
call node (head+args in one `Arc`, halving per-node allocations and
refcount traffic, and enabling an inline per-node evaluated/stable stamp in
place of the pointer-keyed side cache), borrowed argument preparation so
shared arg vectors are not cloned per evaluation pass, and a non-recursive
dispatch loop. Each is multi-day, cross-cutting work; per-probe estimates
suggest together they plausibly reach ~2x but not below without a JIT-class
overhaul. Rejected this pass: depth-gated cycle tracking (broke trace-depth
and `Stack[]` semantics for ~1.5%), caching unstable call verdicts (pins
dead trees, regressed to 25s), text-pointer-only `Symbol` equality without
the inline id (hot-path pointer chase, ~3% regression).

A black-box evaluation-state probe (re-accessing a large inert expression
stored in a symbol, with interleaved definition writes) shows where the
invalidation scheme still trails WolframKernel. Both engines re-access a
100x larger expression at unchanged cost and survive a write to an
unrelated symbol, but with a write to one symbol interleaved between
accesses WolframKernel stays at ~0.35us per access while rum pays ~394us.
The follow-up above tested this same-bit rewrite path. A cheap rewritten dirty
bloom was retained; exact version snapshots were rejected because their entry
snapshot overhead regressed the real PropTaut workload. This means the next
high-leverage frontier has moved back to the larger call-node representation
work rather than more blind stability-cache micro-optimization.

## 2026-06-12 Performance Lane Pause And Cleanup

- Cancelled the long-lived PropTaut optimization goal and stopped automatic
  work toward the earlier `<2x` cross-engine target. That target was not met:
  the retained release frontier is about `27.40-27.64s` for the PropTaut
  distribution probe versus the local WolframKernel internal timing of about
  `3.80s`.
- Verified the repository was clean before this documentation pass and found no
  lingering agent-owned `rum-cli`, `run-bounded.py`, `sample`, or `cargo`
  processes. No generated profiler or bounded-smoke artifacts were present in
  the workspace outside normal `.git/hooks/*.sample` templates.
- Reframed the current `mma-hol` status for future agents: `PropTaut` / `Simp`
  remains the first stdlib/auto gate beyond the green Parser/Printer cold lane,
  but the next action is external review of the accumulated evidence rather
  than continuing speculative evaluator/representation micro-optimization.
- The useful artifacts for that review are
  [docs/evaluator-performance-audit.md](./docs/evaluator-performance-audit.md),
  [docs/proptaut-performance-audit.md](./docs/proptaut-performance-audit.md),
  `scripts/sample-self-time.py`, `scripts/sample-attribute-callers.py`, and
  the retained/rejected probe history below.

## 2026-06-11 PropTaut Performance Attribution

- Added `docs/proptaut-performance-audit.md` and the local macOS `sample`
  reduction helpers `scripts/sample-self-time.py` and
  `scripts/sample-attribute-callers.py`. The audit corrected the Bool-lane
  oracle comparison by separating Wolfram startup from kernel-side
  `AbsoluteTiming`, and reframed the current `PropTaut` cost as the same
  systemic evaluator factor rather than a special proof-script cliff.
- Kept the positive evaluator fast-path follow-up on the `PropTaut`
  distribution theorem: delayed suspension-frame cloning in hot synchronous
  paths and reduced definition/rewrite-rule cloning moved the release timing
  from roughly `48.8s` to roughly `46.5s`.
- Gated `DefinitionStored` trace rendering so ordinary non-tracing execution
  no longer formats stored-definition RHS strings. The same `PropTaut` probe
  then measured about `43.55-43.78s`.
- Added an opt-in cache-invalidation counter probe behind
  `RUM_PROFILE_EVALUATION_CACHE=1`. It reports power-of-two checkpoints for
  `epoch_bumps`, `cache_clears`, `stability_checks`,
  `call_stability_hits`, and `call_stability_misses` to stderr. With the env
  var unset, the same `PropTaut` probe stayed in the `43.7-43.8s` range.
- Made definition writes skip global evaluation-cache invalidation when the
  written symbol has not been observed by the current stability scan. This
  kept the same `PropTaut` result and moved the release probe to about
  `40.07s`; the cache counter run showed `epoch_bumps/cache_clears` reduced
  from the previous `262144+` range to the `65536+` range, with
  `262144+` skipped unobserved-symbol writes. A regression test covers the
  prior-read case (`Module[{x}, x; x = 1; x]`) where invalidation must still
  happen.
- Changed `Symbol` storage from two owned `String`s to shared `Arc<str>`
  context/name fields while preserving the public `context()`/`name()` API,
  ordering/hash semantics, display behavior, and serialized field shape. This
  is a low-risk representation step short of full numeric symbol interning;
  the same release `PropTaut` probe then measured about `32.60-32.82s`.
- Delayed association key-evaluation suspension-frame cloning until an input
  request actually occurs. This removed synchronous-path cloning of the parsed
  entry list and evaluated prefix for ordinary `Association[...]` construction;
  the same release `PropTaut` probe then measured about `30.06s`. A similar
  `Lookup` lazy-frame probe was neutral to slightly worse and was not kept.
- Re-profiled current HEAD `ec372d6`: the remaining self-time is again led by
  `memcmp`, allocator/free traffic, and `cached_expr_is_evaluation_stable`.
  Leaf attribution now points `memcmp` mostly at stability scanning,
  attribute lookup, definition matching, and symbol existence checks; allocator
  traffic is led by `Expr::call`, rule-binding substitution, `Symbol::new`,
  sequence splicing, and `SharedArgs` construction/unwrapping. Association key
  evaluation is no longer the dominant clone caller after the lazy-frame
  change.
- Added session-local caches for hot attribute-set and builtin-dispatch
  resolution, invalidated on attribute revision changes and builtin
  registration. The dispatch planner and stability scanner now use these
  caches while colder inspection paths keep the direct lookup API. The same
  release `PropTaut` probe then measured about `28.10-28.32s`. A hand-written
  `Symbol` comparison probe using `Arc<str>` pointer equality measured worse
  (`31.55-32.51s`) and was reverted.
- Reused shared `Arc<str>` storage for Global/System contexts and
  canonical core System symbol names through the hot `Symbol::from_name` /
  `Symbol::system` constructors, and routed obvious System-symbol runtime
  constructors through that path. The same release `PropTaut` probe then
  measured about `27.40-27.64s`. Two nearby probes were rejected: removing the
  recursive definition shape prefilter regressed to about `29.16s`, and
  skipping unchanged literal-`Sequence` checks during RHS substitution stayed
  neutral to slightly worse (`28.35-28.70s`).
- Re-profiled after the shared System-symbol text change: `Symbol::new`
  allocation is no longer a leading allocator caller, and the remaining
  frontier is `Expr::call`, `SharedArgs::{new,into_vec}`, rule-binding
  substitution, stability scanning, and downvalue matching. Two representation
  probes were rejected: reusing a static empty `SharedArgs` regressed to about
  `28.50s`, and skipping evaluation-cache clearing on epoch bumps regressed to
  about `37.00s` because stale cache-table growth outweighed avoided clears.

## 2026-06-10 Scope Reset

The project is moving from broad core-language expansion toward a `v0.1`
kernel-first stabilization pass.

Current direction:

- stop treating Wolfram same-name surface growth as the default next action
- use concrete downstream pressure, especially `mma-hol`, to promote missing
  semantics back into active work
- keep `rum-host` as the main embedding boundary
- freeze project-owned Python adapter work until a real downstream need returns
- keep Jupyter as an optional capability-validation adapter, not a roadmap
  driver

Detailed future showcase plans for a Python AI policy router and a Rust Wasm
prototype were removed. Those remain possible application directions, but they
are not active project-owned examples.

## 2026-06-10 Correctness Fixes

- Definition insertion now applies the same specificity ordering to
  `UpValues` and `SubValues` that `DownValues` already used, so later typed or
  literal rules can outrank earlier generic patterns while equivalent pattern
  definitions still replace in place.
- Added runtime regressions plus curated Lane A `definitions_precedence`
  coverage for generic-before-typed `UpValue` and `SubValue` definitions.

## 2026-06-10 CL-Y Cold Lane First Pass

- Added first-pass `mma-hol` cold-lane coverage for exact `Total`,
  `Denominator`, `Merge[..., Total]`, `Apply[LCM|GCD, list]`, `/@` map syntax,
  and ordinary Unicode letter identifiers.
- `Total` now reduces list totals through the existing exact `Plus` semantics,
  so grouped association coefficients in `Merge[..., Total]` no longer remain
  inert.
- `Denominator` now covers exact integers and rationals, which is enough for the
  current Farkas-certificate denominator extraction shape.

## 2026-06-10 CL-Y Clean Source Smoke

- Used a clean archive of sibling `mma-hol` HEAD `83d768f` instead of the dirty
  working tree for the cold source smoke.
- Cleared the next loader/source blockers exposed by that smoke: initial
  shebang trivia, multiline comments in the script splitter, logical and
  comparison infix operators, `:` named-pattern syntax, `Function` postfix
  followed by `/@`, string-join `<>`, package usage-message public-symbol
  visibility from private contexts, and string-key `Part` selectors for
  association-backed `Failure` payloads.
- That pass loaded `ErrorUtil.wl`, `Types.wl`, `Terms.wl`, `Kernel.wl`,
  and `tests/harness.wl` in sequence. Narrowed test summaries were:
  `errorutil_tests.wl` `List[10, 0]`, `types_tests.wl` `List[10, 6]`,
  `terms_tests.wl` `List[16, 13]`, and `kernel_tests.wl` `List[8, 15]`.
- At that point, the CL-Y work was no longer parser/loader triage in the
  abstract. The next step was to expose first-failure details from the harness
  and promote only the concrete semantic gaps behind the remaining type, term,
  and kernel failures.

## 2026-06-10 CL-Y Core Assertion Triage

- Cleared the first core semantic failures exposed by the clean `mma-hol`
  snapshot: `Lookup` now holds and evaluates association/default/wrapper
  arguments in the needed order; `Association[...]` evaluates keys before
  canonical entry storage; `BeginPackage[..., deps]` keeps dependency contexts
  available on `$ContextPath`; association key lvalues and list destructuring
  `Set` now cover the current `Types.wl` / `Terms.wl` mutation shapes.
- Definition installation now evaluates call heads in definition lhs positions
  before selecting the definition owner. This matches Wolfram behavior for
  forms such as `h = f; g[h[x_]] := ...` and fixes the strict kernel theorem
  tag patterns generated through `Module` / `Unique`.
- Definition RHS evaluation now consumes `Return[...]` at definition
  application boundaries, matching Wolfram's `g[] := Return[1]; f[] := (g[]; 2)`
  behavior. This removed the immediate `"P"` short-circuit in `HOL`Bool`GEN`.
- Current clean-snapshot summaries after these fixes:
  `errorutil_tests.wl` `List[10, 0]`, `types_tests.wl` `List[36, 0]`,
  `terms_tests.wl` `List[67, 0]`, `kernel_tests.wl` `List[71, 0]`,
  `bootstrap_tests.wl` `List[45, 0]`, and `equal_tests.wl` `List[17, 0]`.
- The next concrete blocker is the Bool layer. Valid probes must preload the
  lower HOL files before `Bool.wl`; a fresh direct `Get["Bool.wl"]` still hits
  package-dependency/autoload mismatches and is not the representative proof
  benchmark. After the first Bool optimization pass, full dependency-preloaded
  `Bool.wl` completes in release, but debug loading is still too slow for a
  routine smoke and `bool_tests.wl` has not yet been promoted into a bounded
  check.

## 2026-06-10 Bool Block Profiling

- Re-ran the Bool blocker against the fixed clean `mma-hol` snapshot. The issue
  is no longer a parser/loader failure or a semantic short-circuit: with the
  definition-boundary `Return` fix in place, the Bool proofs enter their real
  beta/derived-rule path.
- Isolated full `Bool.wl` loading to the first heavyweight top-level theorem:
  a temporary slice through line 580, before `condTThm`, loads; adding
  `condTThm` makes loading exceed an interactive window. A direct `condTThm`
  body probe still timed out under a 60s internal `TimeConstrained` window.
- Ruled out `Part` as the primary hotspot. `rhsOfConcl[APTHM[condDefThm, T]]`
  returns a large redex successfully; the cost is in repeated `BETACONV`,
  `BETA`/`INST`, `TRANS`, `SYM`, `CONJ`, and `DISCH` over large HOL terms.
- `GEN[x, REFL[x]]` is not a dead loop in isolation: under the Bool pre-theorem
  slice it completes, but still costs several seconds beyond package load.
- WolframKernel loads the same full `Bool.wl` in roughly 2.4s on this machine,
  while rum still exceeded 150s for full Bool loading before interruption. This
  makes the remaining blocker a rum evaluator performance problem, not just an
  expensive downstream proof script.
- Kept targeted evaluator optimizations after probe/measure cycles:
  non-held definition matching skips recursive candidate normalization; simple
  fixed-arity patterns use a conservative fast matcher and shape prefilter;
  trace strings are materialized only when tracing is enabled; only small
  expressions keep full fixed-point/call-stack cycle histories; clearly stable
  expression trees can return without another dispatch pass; suspend/resume
  clones are delayed until an input request actually occurs.
- Local debug microbenchmarks improved materially: the large Bool `BETACONV`
  redex dropped from roughly 1.8s to roughly 0.4s, `CONJ[ASSUME[p], ASSUME[q]]`
  from roughly 7.6s to roughly 1.8s, and a `DISCH[T=T, REFL[a]]` subpath from
  roughly 25s to roughly 8s. The dependency-preloaded prefix before `condTThm`
  is now around 4.5s in debug instead of the earlier mid-teens.
- Full dependency-preloaded `Bool.wl` now completes, but is still expensive:
  around 231s in debug and around 60s in release on the measured machine. This
  is progress, not closure. The remaining hot path is dominated by repeated
  derived-rule work in `MP`, `GEN`, `SPEC`, `BETACONV`, and witness/equality
  construction over large HOL terms.
- Dead-end or low-yield probes so far: treating `Part` as the primary hotspot,
  a broad top-level stable-evaluation shortcut, and optimizing direct fresh
  `Get["Bool.wl"]` before resolving the dependency/autoload mismatch.
- The next likely performance cut is deeper than surface dispatch: reduce
  owned-expression cloning in recursive pure-function and derived-rule
  workloads, preferably with borrowed traversal/matcher paths or profiler-guided
  clone accounting before adding more special cases.

## 2026-06-10 Evaluator Performance Audit Follow-up

- Added `docs/evaluator-performance-audit.md` as a historical external review
  of the Bool proof-load bottleneck. The audit was captured for rum commit
  `fddf4fd` and sibling `mma-hol` HEAD `96cf163`, and confirmed that allocator
  traffic, owned expression cloning, fixed-point loop cloning, catalog scans,
  definition dispatch cloning, and matcher binding snapshots dominate the
  release profile.
- Implemented the first quick-win series from that audit: hash-backed builtin
  catalog lookup; delayed fixed-point resume-state cloning; symbol+index
  definition dispatch that avoids cloning whole `SymbolDefinition` values;
  cached stored-definition matchers guarded by an attribute revision; and
  matcher failure rollback using a small binding undo trail instead of cloning
  the entire binding map at every matcher entry.
- Fixed the CLI tooling side issue from the audit: `rum-cli --eval` and
  `--file` now render `Print` output before the final value, so load-time
  bisection markers are visible.
- Re-ran the dependency-preloaded Bool release smoke after these changes:
  `AbsoluteTiming[Get["Bool.wl"]]` was about 46.84s and external wall time was
  48.26s, down from the previous comparable `real 62.82s`. This is a real
  improvement, but still far from WolframKernel's roughly 2.4s reference; the
  structural representation work remains open.

## 2026-06-10 Evaluation Epoch Cache Follow-up

- Rebuilt the post-quick-win Bool baseline before making deeper changes:
  dependency-preloaded `AbsoluteTiming[Get["Bool.wl"]]` reported about 46.21s,
  with external wall time `real 47.64`.
- Collected a fresh macOS `sample` profile from that build. The profile still
  showed allocator traffic, owned `Expr` cloning/dropping, structural compares,
  `run_call_dispatch_stage`, `try_apply_down_values_from_symbol`, and
  `expr_is_evaluation_stable` on the hot path. Parser and loader work remained
  absent from the hot stacks.
- Added a session-level evaluation epoch and two exact symbol-keyed caches for
  stability scanning: atom-symbol evaluation stability and symbol-head
  call-effect decisions. The epoch is bumped and the caches are cleared when
  definitions, attributes, registered builtins, declared package stubs, loaded
  packages, removed symbols, or dynamic-scope definition state changes.
- Tested and rejected a whole-expression exact normal-form cache. With the
  current owned-tree `Expr`, hashing and equality on large expressions became a
  visible new profile cost; keeping only symbol-level caches was faster on the
  Bool smoke.
- Final dependency-preloaded Bool release smoke after this pass:
  `AbsoluteTiming[Get["Bool.wl"]]` about 43.09s and external wall time
  `real 44.49`, with earlier confirmation runs at `real 44.72` and `43.79`.
  This is a modest but repeatable improvement, not a substitute for the shared
  expression / symbol interning work.

## 2026-06-10 Shared Call Tree Follow-up

- Replaced the owned `Expr::Call { head: Box<Expr>, args: Vec<Expr> }`
  representation with shared call nodes:
  `Expr::Call { head: SharedExpr, args: SharedArgs }`, backed by `Arc`.
  `SharedArgs` preserves borrowed iteration and copy-on-write mutation for the
  existing runtime code shape; old owned-Vec boundaries now explicitly call
  `to_vec`/`into_vec`, while pure expression rebuilds can reuse shared args.
- Kept `Expr::call(head, Vec<Expr>)` as the normal source-compatible
  constructor and added shared constructors for paths that can preserve an
  existing argument spine. The expression type remains `Send`/`Sync` friendly by
  using `Arc`, not `Rc`.
- Re-ran the same dependency-preloaded Bool release smoke after this change.
  External wall times were `real 8.09`, `8.07`, and `8.13`; internal
  `AbsoluteTiming[Get["Bool.wl"]]` was about 7.68-7.74s. This is the first
  large structural win on the Bool blocker, down from the prior 43.8-44.7s
  range. Remaining work should now re-profile before choosing between symbol
  interning, node-level evaluated-state bits, and derived-rule workload
  specialization.

## 2026-06-11 Bool Performance Retest

- Re-ran the post-23:30 Bool timing work after a separate system-heavy task
  finished; the late 2026-06-10 timing samples were treated as contaminated
  and not used for decisions.
- Kept the evaluator-side wins that survived clean retesting: call-stability
  caching over shared call trees; the global no-upvalue fast path; planned
  builtin handler dispatch; early stable-expression return from the fixed-point
  evaluator; and copy-on-write binding substitution for rule RHS/template
  substitution.
- Rechecked the late-session decisions that affected the retained diff:
  rule-RHS binding-substitution copy-on-write survived clean A/B timing, while
  skipping call-stability cache entries for leaf calls was effectively neutral
  and was not kept. Other low-yield probes remain reverted: lazy epoch cache
  clearing, fast/default hasher experiments, fixed-array dispatch stages, a
  registered-builtin fallback shortcut, and slot-substitution copy-on-write.
- Final dependency-preloaded Bool release smoke on the clean retest:
  first run `real 3.99`; five warmed runs `real 3.57`, `3.51`, `3.52`,
  `3.53`, and `3.51`. Internal
  `AbsoluteTiming[Get["Bool.wl"]]` for those warmed runs was about
  `3.17-3.22s`, with median about `3.19s`. This is down from the shared
  call-tree baseline of about `8.1s` external wall time, but still does not
  reach the local WolframKernel reference of roughly `2.4s`.
- Clean A/B checks: disabling rule-RHS binding-substitution copy-on-write
  regressed warmed Bool to about `3.59-3.61s` external wall and `3.24-3.26s`
  internal timing; disabling the leaf-call cache skip was effectively neutral,
  so the skip was not kept.

## 2026-06-11 Bool Optimization Direction Sweep

- Re-ran the next five optimization directions in order after the clean
  retest baseline: clean profile, symbol fingerprint/ID, definition matcher
  metadata, expression construction/allocation, and release flags.
- The clean profile confirmed that parser/loader work is no longer the blocker.
  The visible hot costs were allocator/free traffic, string/structural compare,
  SipHash on symbol-keyed maps, stability checks, expression clone/drop/call,
  lexical substitution, rule binding substitution, and definition matcher work.
- Tested a conservative `Symbol` fingerprint field that preserved lexicographic
  `Ord` while hashing only a precomputed deterministic `u64`. It removed the
  visible SipHash string-write hotspot, but warmed Bool wall time stayed in the
  same `3.50-3.57s` range and the larger `Symbol` plus custom serde complexity
  was not kept.
- Tested per-definition matcher metadata for root arity and syntactic fast-path
  eligibility. Runtime tests passed, but warmed Bool stayed around
  `3.53-3.57s`; the additional definition size and branching were not kept.
- Kept the expression-construction win: lexical substitution now uses an
  internal copy-on-write traversal, returning unchanged subtrees instead of
  eagerly rebuilding every visited call. This directly addressed the allocator
  and clone-heavy `Module`/`With` path called out in the external evaluator
  audit.
- Final dependency-preloaded Bool release smoke after lexical-substitution
  COW: first run `real 3.13`; warmed runs `real 2.45`, `2.51`, `2.46`, and
  `2.45`. Internal `AbsoluteTiming[Get["Bool.wl"]]` on those warmed runs was
  about `2.30-2.36s`. This reaches the same order as the local WolframKernel
  reference, previously measured at roughly `2.4s`.
- Release-flag probes: `RUSTFLAGS="-C target-cpu=native"` improved warmed wall
  time further to about `2.29-2.35s` on this machine. `codegen-units=1` and
  thin LTO did not produce a clear additional runtime win over native-only, and
  increased clean build/link time substantially. Treat native CPU builds as an
  optional local benchmark/deployment knob, not the portable default profile.

## 2026-06-11 Bool Evaluator/Representation Follow-Up

- Committed the lexical-substitution COW sweep as `bfadc4b` and continued from
  a clean default-release baseline of about `real 2.45` on the dependency-
  preloaded Bool smoke.
- Kept a lightweight runtime cache hasher for epoch-scoped symbol/effect caches.
  This removed the remaining default SipHash cost from those hot caches without
  changing cache keys or semantics. Warmed default-release Bool smoke moved to
  roughly `real 2.41-2.45`.
- Kept owned call-argument preparation: the evaluator now consumes the owned
  remaining argument vector during normal call preparation instead of cloning
  each argument before planning. Warmed Bool smoke improved to about
  `real 2.38-2.42`.
- Kept an empty-binding rule-RHS fast path. Definition and rewrite-rule storage
  now precomputes whether a RHS may need literal `Sequence` splicing even when
  matcher bindings are empty; rule application can reuse the shared RHS directly
  when it is safe. Warmed Bool smoke improved to about `real 2.36-2.40`.
- Rejected a one-entry call-stability MRU cache after measurement. Runtime tests
  passed, but warmed Bool smoke regressed to about `real 2.41-2.44`, so the
  probe was reverted.
- Deferred deeper performance directions until a future profiling pass:
  borrowed fast paths for compiled matcher application, delayed materialization
  of definition candidates before rule traces/options need an owned expression,
  narrower pointer/shape fast paths for exact expression comparisons, and
  head-specific investigation of repeated `cached_expr_is_evaluation_stable`
  calls. A whole-expression DAG/arena remains a last-resort architecture change
  rather than the next default step.

## 2026-06-11 Latest `mma-hol` Cold Probe

- Cloned sibling `mma-hol` clean HEAD `698e3d0` (`M7-7 (Real)
  Complete.wl: Dedekind completeness + Archimedean + Q-density`) to
  `/private/tmp/rum-mma-hol-latest.FtARYw/mma-hol` and used that fixed clone as
  the probe target so live downstream development can continue independently.
- Baseline low layers still pass on the latest tree: dependency-preloaded
  `kernel_tests.wl` reported `List[71, 0]`; dependency-preloaded
  `bool_tests.wl` reported `List[73, 0]` with external wall time about
  `20.69s` for cold package load plus tests.
- Temporary source normalizations in the `/private/tmp` clone let the probe
  move past syntax sugar without changing `rum`: pattern `|` was rewritten to
  explicit `Alternatives[...]`; several assignment RHS `& /@` forms were
  rewritten to `Map[...]`; `_?pred` was rewritten to
  `PatternTest[_, pred]`; one `Table[heldLhs -> rhs, ...]` mapping was
  rewritten to `MapThread[Rule, ...]`; and a Meson recognizer using
  `$boolEqTy` in a definition lhs was rewritten to compare a captured `ty_` on
  the RHS.
- With those probe-only normalizations, `drule_tests.wl` reported
  `List[43, 0]` and `tactics_tests.wl` reported `List[49, 0]`. This suggests
  the Bool/Drule/Tactics semantic base is now mostly healthy once the promoted
  source/evaluator compatibility gaps below are fixed.
- The promoted compatibility gaps from this latest probe are:
  Wolfram pattern infix `|` lowering to `Alternatives`, `_?` lowering to
  `PatternTest`, correct precedence for assignment RHS forms like
  `ks = #[[1]] & /@ args`, `Table` iterator substitution inside held `Rule`
  lhs positions, evaluation of non-pattern own-value constants in definition
  lhs forms such as `p = a; f[p] := ...`, and a less eager evaluation-cycle
  guard for state-progressing recursive definitions such as
  `parseTypeExpr[]`.
- Additional follow-up observations from this pre-fix probe: `Printer` had a
  low-risk display mismatch for a naked registered infix constant
  (`formatTerm[=]` rendered `"="` instead of `"(=)"`); `Parser` tests stopped
  at the same recursive parser cycle issue; `PropTaut` and `Meson` could load
  and execute small non-parser probes after the syntax/lhs normalizations;
  `Simp` loaded but emitted many recursion-limit messages during tests; and
  the first stdlib type packages (`Pair`, `Sum`, `Option`) then exposed
  derived-theorem failures that should be revisited after the evaluator/source
  fixes, not before.

## 2026-06-11 CL-Y Promoted Compatibility Fixes

- Folded the latest `mma-hol` probe-only normalizations back into `rum`:
  source `|` now lowers to `Alternatives`, `_?pred` lowers to `PatternTest`,
  and pure-function postfix parsing now lets rule bodies such as
  `Map[f[#[[1]]] -> #[[2]] &, pairs]` become `Function[Rule[...]]` while
  preserving assignment RHS map forms such as `ks = #[[1]] & /@ args`.
- Aligned rule/definition evaluation for the observed HOL workloads:
  `Rule` / `RuleDelayed` evaluate their lhs enough for substitution into
  `Table`-generated held rules; pattern names remain protected while blank
  heads and non-pattern constants are normalized; and definition lhs constants
  with ownvalues now store rules under the evaluated form (`p = a; f[p] := ...`
  defines `f[a]`).
- Replaced the same-expression cycle guard with an epoch-aware guard so
  state-progressing recursive definitions can continue after mutations while
  unchanged direct and mutual recursion still fail fast.
- Fixed ordinary-expression pattern resolution for `MatchQ` / `MemberQ` style
  calls: explicit pattern syntax stays held, but non-pattern expressions such
  as `spec["kind"]` are evaluated before compiling an exact pattern. This
  cleared the `Printer` registered-infix-constant failure.
- Current clean-clone `mma-hol` HEAD `698e3d0` smoke results, using
  `/private/tmp/rum-mma-hol-clean.PKeAHp/mma-hol` without source
  normalizations:
  `bool_tests.wl` reports `List[73, 0]` with cold release wall time about
  `22.95s`; `drule_tests.wl` reports `List[43, 0]` in about `4.18s`;
  `tactics_tests.wl` reports `List[49, 0]` in about `5.18s`; and
  `printer_tests.wl` plus `parser_tests.wl` report `List[126, 0]` in about
  `4.62s`.
- Verification: `cargo fmt --check`, `cargo test -p rum-syntax`,
  `cargo test -p rum-runtime`, and `cargo test --workspace` pass. The next
  CL-Y step is a bounded cold `run_all.wls` driver and triage of the first
  stdlib/auto failure beyond Parser/Printer.

## 2026-06-11 Bounded `mma-hol` `run_all` Probe

- Added a temporary out-of-tree probe script at
  `/private/tmp/rum_mma_hol_bounded_run_all_probe.wl` and then switched to
  one-layer `rum-cli --eval` probes because `--file` did not surface enough
  intermediate progress for long runs.
- First post-Parser/Printer test frontier is performance, not correctness:
  `proptaut_tests.wl` reaches
  `propTaut: distribution ⊢ p ∨ (q ∧ r) = (p ∨ q) ∧ (p ∨ r)` and exceeds a
  90s suite budget. The isolated distribution formula eventually succeeds
  (`{True, True}` for theorem status and conclusion alpha-equivalence) but
  takes about `real 60.68s`. With a 300s suite budget, `proptaut_tests.wl`
  still times out after reaching `nnfThm: literal unchanged`, so `PropTaut`
  is the first new performance gate.
- Package-only loading through `Parser`, `PropTaut`, `Meson`, and `Simp`
  succeeds under the current probe. The first stdlib package gate is
  `stdlib/Pair.wl`: with a 60s per-package load budget it times out; with a
  180s budget it reaches an uncaught HOL error after about `real 119.66s`.
- The `Pair.wl` failure is semantic/correctness-oriented rather than a parser
  failure. Non-quiet loading reports `concl: not a theorem` while constructing
  `pairInjThm`: `HOL`Bool`MP[...]` tries to use `mkPairInjThm`, but
  `mkPairInjThm` is not a theorem at that point. Running `pair_tests.wl` after
  the failed load cascades into missing constants/types (`mkPair`, `prod`,
  `ABS_prod`, `REP_prod`, `FST`, `SND`) because `Get` rolls back the failed
  package load.
- Current triage order after Parser/Printer:
  1. profile/optimize the `PropTaut` proof-construction path enough that the
     propositional suite becomes a useful signal;
  2. debug why `Pair.wl` fails to construct `mkPairInjThm`, likely around the
     `Simp`simpConv` / `CONVRULE` path used to simplify the unfolded
     `mkPair` equality;
  3. only after `Pair` loads cleanly, revisit `Sum`, `Option`, `Set`, `Num`,
     and later Real packages, because their current probe status is polluted by
     the failed Pair dependency.

## 2026-06-11 `Pair.wl` Recursion-Limit Triage

- Isolated the first `Pair.wl` hard failure to the `mkPairInjThm` construction:
  the first simplifier conversion over `mkPairDefThm` exceeded rum's old
  default recursion limit of `256` after about `real 121.13s`, producing
  `TerminatedEvaluation["RecursionLimit"]` instead of a kernel theorem. The
  later `pairInjThm` construction then consumed that value as `mkPairInjThm`,
  leading to the observed `concl: not a theorem` failure.
- Checked the local Wolfram kernel baseline: `$RecursionLimit` is `1024`.
  Raised rum's default recursion limit from `256` to `1024` and added a runtime
  regression test for the default. Explicitly lowered test sessions still
  produce `TerminatedEvaluation["RecursionLimit"]`, so the resource-limit
  behavior remains covered.
- Stage probes with the raised limit show that direct `mkPairDefThm` rewriting
  is not the bottleneck: `REWRCONV` / `baseUser` / top-level `TRYCONV` are
  millisecond-to-0.1s operations. The remaining cost is `PropTaut`-backed
  simplifier schema construction: `basicSimpset[]` takes about `106-110s`, the
  first `andCongLemma[]` use via `stepAndXX` takes about `99s`, and later
  `mkPair`-term fixpoints still take tens of seconds.
- A real `Pair.wl` load with the raised default passed the old `real 119.66s`
  hard-failure point without reporting the old non-theorem error, but was
  intentionally terminated after `real 335.35s`. Therefore the correctness
  blocker is explained and the default-limit fix is in place, but `Pair.wl` is
  not yet a green smoke. This identified `PropTaut` / `Simp` performance, not
  more ad hoc Pair-source debugging, as the remaining Pair lane blocker.

## 2026-06-11 Loader Resource-Failure Contract

- Split top-level `TerminatedEvaluation` handling from file-load handling.
  Interactive `eval_text`, `eval_script`, and ordinary eval-request paths still
  convert recursion, iteration, and current resource-budget errors into
  `TerminatedEvaluation[...]` values with the existing `General::*` message.
- `eval_file` / `Get` now evaluate loaded forms with resource termination
  propagated as a real load error. A resource-limit hit during `Get` therefore
  returns `$Failed`, emits the outer `Get::load` message, hides the internal
  failing-form message just like other failed loads, and rolls back definitions
  and package-loader state.
- Added a runtime regression that lowers `$RecursionLimit`, triggers recursion
  inside a loaded file, and verifies that pre-failure assignments, post-failure
  assignments, and the recursive definition are all absent after the failed
  load.

## 2026-06-11 `mma-hol` Harness And Snapshot Audit

- Added `scripts/mma-hol-bounded-smoke.sh`, a reusable bounded diagnostic
  driver for the `mma-hol` lane. It supports package-load probes, independent
  test-file probes, per-stage wall timeouts, captured stdout/stderr, and a
  tab-separated summary. The driver intentionally replays load prefixes in
  fresh `rum-cli` sessions, so it localizes failures but is not a timing
  benchmark.
- Added `docs/mma-hol-smoke-and-snapshot.md` as the canonical note for the
  driver contract and stable snapshot audit. The snapshot lane remains below
  the cold semantic smoke. At audit time, its concrete missing support was:
  `Contexts[]`, `$ContextPath` / `$Packages` session-field assignment,
  `FileExistsQ`, `FileDate`, `FileByteCount`, `AbsoluteTime`, `Round`,
  `FileNames[..., ..., Infinity]`, and a focused `DumpSave` / `Get` context
  round-trip regression.

## 2026-06-11 Stable Snapshot Helper Builtins

- Added the first small-helper tranche for `mma-hol` stable snapshot scripts:
  `Contexts[]`, `FileExistsQ`, `FileByteCount`, `FileDate[..., "Modification"]`,
  `AbsoluteTime`, `Round`, and `FileNames[pattern, dir, Infinity]`.
- Kept the timing-literal support narrow: simple `digits.digits` source forms
  lower to exact rationals, enough for `Round[..., 0.1]` / `Round[..., 0.01]`
  without introducing Wolfram machine-real or precision semantics.
- Added focused runtime regressions for context enumeration, recursive file
  discovery, file metadata helpers, exact/decimal `Round`, and decimal
  lowering. Remaining stable snapshot work is `$ContextPath` / `$Packages`
  session-field assignment plus a focused `DumpSave` / `Get` context
  round-trip regression.

## 2026-06-11 Stable Snapshot Restore And Lint Helpers

- Added session-field assignment for `$ContextPath` and `$Packages`, so stable
  snapshot restore code can mutate runtime lookup/package state instead of
  creating inert own values.
- Extended textual `DumpSave` object handling so context strings from
  `Evaluate[holContexts]` save all persisted symbols in those contexts. Added a
  tiny public/private context round-trip regression that restores via `Get`,
  updates `$ContextPath` / `$Packages`, and calls the restored public symbol.
- Added text-only `Row`, lint-only `Import[..., "Text"]`, and string-pattern
  wildcards such as `"Snap`*"` for the stable snapshot context filter shape.
  The remaining boundary is full validation of `mma-hol` stable scripts and
  private-context lint, not the small helper availability itself.

## 2026-06-11 `PropTaut` Performance Harness Cleanup

- Re-established the current `PropTaut` performance frontier on clean
  `mma-hol` HEAD `698e3d0`: package loading through `auto/PropTaut.wl` is about
  `2.49s`, while the single distribution theorem
  `(p || (q && r)) = ((p || q) && (p || r))` still takes about `49.7-50.1s`
  internally in release. The full `proptaut_tests.wl` suite remains a
  performance gate, not a package-loading gate.
- Captured a macOS `sample` profile for a repeated distribution probe. The hot
  stacks are still dominated by allocation/free traffic, structural compare,
  `cached_expr_is_evaluation_stable`, evaluator dispatch, binding/lexical
  substitution, and builtin/evaluator lookup. Parser and loader work are not
  the bottleneck.
- Rejected two low-yield or unsafe probes: a broad literal `ReplaceAll`
  shortcut broke `Rule` evaluation boundaries inside `PropTaut`, and a borrowed
  rewrite-matcher fast path was effectively neutral on the distribution probe.
  A lightweight hasher for the builtin-name map also regressed the same probe
  and was reverted.
- Fixed the bounded `mma-hol` smoke driver so test stages preload their needed
  package prefix and added `--only LABEL` for single-stage probes. With
  `RUM_CLI_BIN=target/release/rum-cli`, `test/parser_tests` now runs as a
  single green stage, while `test/proptaut_tests --timeout 20` reports the
  intended timeout after preloading through `auto/PropTaut.wl`.

## Completed Architecture Work

- The workspace was split into responsibility-oriented crates:
  `rum-syntax`, `rum-core`, `rum-runtime`, `rum-stdlib`, `rum-terminal`,
  `rum-repl`, `rum-cli`, `rum-host`, `rum-jupyter`, and `rum-difftest`.
- Major `include!`-style and monolithic runtime, syntax, Jupyter, and
  difftest files were replaced with normal Rust module trees.
- The 2026 source-size cleanup pass removed Rust files over the `35k`
  architecture-debt threshold; the remaining work is mostly warning-range
  opportunistic cleanup.
- `rum-host` now provides the first-class Rust embedding facade over runtime
  sessions, requests, structured outcomes, failures, inspection, completion,
  completeness checks, and input suspension/resume.
- `rum-jupyter` was kept optional and layered as an adapter instead of becoming
  evaluator infrastructure.

## Completed Core-Language Mainlines

The `CL-01` through `CL-09` mainlines are considered closed for the current
kernel-first phase. Remaining gaps are demand-driven parity tails, not default
implementation work.

Broad completed areas include:

- expression model, lowering, parser diagnostics, formatter, exact integers
- definitions, attributes, values, scoping, contexts, package loading
- matcher IR, rules, patterns, replacement, traversal, and tree walkers
- functional/list combinators across the current first-pass supported subset
- associations, keyed data, missing values, keyed mutation, and grouping
- procedural control, nonlocal flow, cleanup, status objects, host input
- string/text surface, string patterns, formatting, templates, byte arrays
- package/options/protection/name-inspection tooling
- tracing, messages, resource limits, timing, memory proxies, test reports

The useful lesson from this phase is that every new Wolfram same-name builtin
is expensive: it tends to touch evaluator semantics, messages, resource
budgets, suspension/resume, host surfaces, docs, and conformance fixtures.
Future additions should therefore be promoted by a concrete blocker rather
than by inventory coverage alone.

## Archived Roadmap Detail

The old `TODO.md` carried long completed checklists for individual CL waves,
external-review fixes, difftest lane construction, string-language expansion,
and math-surface footholds. Those details were compressed on 2026-06-10 so the
roadmap can show active decisions instead of completed work.

The archived completed waves include:

- `CL-08` package, namespace, options, protection, persistence, and name
  tooling
- `CL-09` tracing, timing, resource-limit proxies, monitors, messages,
  evaluation data, test reports, and dialog support
- post-mainline closure passes for `CL-01` through `CL-09`
- first practical math footholds for exact arithmetic, polynomial expansion,
  coefficient extraction, factorization, `Together`, and symbolic `D`
- the first large string-language surface, including string construction,
  exact search, string patterns, templates, conversions, byte arrays,
  hashing/compression/encryption, nearest-neighbor helpers, and pluralization
- resolved external-review P0/P1 items for definition replacement, Jupyter
  signing/codec/frame hardening, conformance fixtures, and package/context
  coverage

Canonical current status should be read from the support matrix and tests, not
from the archived checklist wording.

## Verification And Harness Work

The `rum-difftest` workstream now has:

- Lane A for stateless Wolfram-oracle comparisons
- Lane B for stateful session comparisons
- Lane C for checked known-difference ledger validation
- Lane D for local performance smoke/baseline runs
- non-oracle inventory and aggregate `check` gates
- artifact-hygiene checks to keep generated oracle material out of the repo
- summary JSON output with stable top-level schema versions

The practical default gate remains:

```bash
cargo fmt --check
cargo test --workspace
cargo run -p rum-difftest -- check
```

Wolfram oracle runs are intentionally environment-gated and should remain
optional for normal development.

The Jupyter harness now has an isolated `jupyter_home/` deploy path with real
`jupyter_client` and JupyterLab smoke commands. `kernel-smoke` covers kernel
start/ready/execute/restart/shutdown, stdin suspend/resume, completion,
inspection, is-complete, history, stdout/stderr stream mapping, runtime
messages, failure-to-error mapping, and unsupported-message keepalive over real
ZMQ channels. Unsupported-message handling now returns an empty
`comm_info_reply`, ignores comm open/msg/close, returns an explicit unsupported
`debug_reply`, and ignores unknown frontend probes so the kernel stays alive.
`browser-smoke` covers notebook execution through JupyterLab and headless
Chromium. On 2026-06-13, `cargo test -p rum-host`, `cargo test -p
rum-jupyter`, `scripts/rum-jupyter-home.sh kernel-smoke`, and
`scripts/rum-jupyter-home.sh browser-smoke` all passed locally. The Codex
sandbox blocks loopback-port binding, so the native ZeroMQ and real Jupyter
smokes need unsandboxed execution, but their filesystem state stays under
`jupyter_home/`. Robust interrupt/cancel, widgets/comm state, and the debug
protocol remain outside the current contract.

## Operational Lessons

- Do not use the full Wolfram builtin inventory as a commitment list.
- Prefer local WolframKernel spot checks for same-name semantics, but do not
  commit generated oracle transcripts.
- Keep resource limits cooperative and explicit. Generated-output functions
  need budget checks before allocation.
- Keep support-matrix statuses conservative. `partial` means only the
  documented subset is compatibility-targeted.
- Prefer one real downstream driver over multiple speculative examples.
- Move completed roadmap detail here once it no longer changes decisions.

## Frozen Or Archived Directions

### Python Adapter

Python remains a plausible future host, especially for AI/data/tooling
workflows, but project-owned Python binding work is frozen for now.

If revived later, the rules remain:

- build on `rum-host`, not broad `rum-runtime`
- keep `pyo3` and packaging concerns outside kernel crates
- start with eval/resume/inspect/complete/is-complete only
- add a real downstream use case before committing to public Python API shape

### Standalone Showcase Plans

The detailed Python AI policy and Rust Wasm prototype plans were removed to
avoid turning speculative examples into roadmap obligations.

The underlying application directions remain valid as short examples of where a
symbolic rule-engine kernel could be useful:

- policy / validation / rewrite layers in AI or data tooling
- semantics and validation prototypes for compilers, runtimes, or DSLs
