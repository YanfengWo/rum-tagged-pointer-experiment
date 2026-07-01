# Host embedding: capability boundary, hardening directions, showcase keepers

> Forward-looking notes (recorded 2026-06-30) for the eventual "rum as an
> embeddable engine other projects call" line. NOT active work — the active line is
> the `mma-hol` correctness/memory mainline; pick this up only after `mma-hol` is
> fully through. Grounded in a code survey of `rum-host` + `rum-runtime` +
> `builtin_catalog` + the support matrix, plus a 2026-06-30 cross-model design
> consult. The concrete SHOWCASE PROJECT is deliberately left undecided (browser
> CAS / mma-hol-in-WASM / desktop-AI-assistant plugin were all weighed and none
> chosen); only the two durable enablers are recorded below.

## 1. rum's capability boundary (what an embedding can rely on)

**Strong domains** (`builtin_catalog/*` + support-matrix CL-01..09): expression /
structure ops, definitions / attributes / scoping, **rules / patterns / tree
walkers** (the rewrite engine — the core strength), functional & list combinators,
**associations / keyed data**, procedural control / exceptions / failure flow,
**string language**, packages / namespaces / options, debug / verification surface.
Arithmetic is **exact rationals only** (`num-bigint`, no machine/arbitrary-precision
floats — a deliberate design choice).

**Deliberately excluded** (support-matrix CL-X): graphics / plotting / visualization,
notebook / front-end UI, `Dataset`, AND **external operations — `Import`/`Export`/
file/web**. The I/O exclusion is the key fact below: rum has **no ambient
file/network access baked into the language**.

**Net shape:** a faithful-WL **symbolic-rewrite / rule-engine / structured-data**
language — exact, deterministic, **sandboxed by design**, and embeddable. Two proofs
this is real, not aspirational: (a) it runs the unmodified `mma-hol` HOL development
(faithful WL semantics); (b) `rum-jupyter` is a working adapter layered on the
`rum-host` embedding boundary.

**Dependency surface (embedding / WASM relevance):** `rum-core` deps = `num-bigint`
+ `serde`; `rum-runtime` deps = `rum-syntax` + `thiserror`. All pure-Rust,
wasm-compatible; `mimalloc` is `rum-cli`-only; **no `std::fs` / threads / network in
the core dep tree** (because the language excludes Import/Export). So `wasm32` is a
near-trivial target (modulo a few loader file-path `cfg` guards).

## 2. Embedding / host hardening directions (the surveyed, concrete gaps)

The framework is good and proven (`rum-jupyter` on `rum-host`); these are the
"push it to the end" gaps for serving OTHER projects. Tiered by need.

**Tier 1 — STATUS 2026-07-01: 4 of 5 items DONE (briefs 059/060/061); memory budget is
documented as a host-side pattern (below). The KNOWN LIMITATION bullet still stands.**

- **Interrupt / cancel — DONE (brief-059, `e5b13b9`).** `HostSession::cancel_handle()`
  returns a `HostCancelHandle` (`Clone + Send + Sync`) the host obtains BEFORE `eval` and
  `.cancel()`s from another thread; the eval loop checks an `Arc<AtomicBool>` at an
  amortized poll point (`evaluate_once_*`, every 1024 armed steps) and returns
  `TerminatedEvaluation["Cancelled"]`. Only the Arc crosses threads — `Session`'s thread
  contract is unchanged.
- **Session-level TIME budget — DONE (brief-059).** `HostSession::eval_with_timeout(src,
  Duration)` sets a per-eval deadline checked at the same poll point → reuses
  `TimeConstraintExceeded`. (`HostResourceLimits` still also has recursion / replace_repeated
  / pass / generated_element / integer_work / iteration_work.)
- **Session-level MEMORY budget — host-side pattern (no new core/host code needed).**
  The structural memory proxy (`current_memory_proxy_bytes`) is an O(state) full sweep
  (too costly to poll per eval step) AND blind to the interner `&'static` leak — so it is
  NOT the right signal. RSS is the true total but is a platform syscall + threads that
  would break the WASM-clean core. RESOLUTION (decided 2026-07-01): impose a memory cap
  **host-side**, reusing the brief-059 cancel handle — a watchdog thread reads process RSS
  and `.cancel()`s on breach. This keeps `rum-core`/`rum-runtime` WASM-clean, gives the
  TRUE RSS bound (catches the interner leak the proxy misses), and is already enabled by
  brief-059. It is process-wide (right for a single-session embedding; coarse for
  multi-session). Pattern:
  ```rust
  let handle = session.cancel_handle();
  let cap_bytes = 2 * 1024 * 1024 * 1024;
  std::thread::spawn(move || loop {
      if process_rss_bytes() >= cap_bytes { handle.cancel(); break; }
      std::thread::sleep(std::time::Duration::from_millis(10));
  });
  let outcome = session.eval(src); // returns TerminatedEvaluation["Cancelled"] on breach
  ```
  `process_rss_bytes()` is the host app's platform code (macOS `task_info` /
  Linux `/proc/self/statm` / Windows `GetProcessMemoryInfo`; WASM has no RSS and uses a
  different mechanism). rum-host stays clean and just exposes the cancel handle. (A
  native-only `eval_with_memory_budget` convenience wrapper in rum-host was considered and
  deliberately NOT added, to keep rum-host platform/thread-free.)
- **Panic-safety boundary — DONE (brief-060, `4e532a8`).** Every public `HostSession`
  entry runs guest code under `catch_unwind(AssertUnwindSafe(...))`; a caught panic →
  `HostFailureCode::Panic` (message from the payload) and the session is POISONED (a
  later call returns `HostFailureCode::Poisoned`). Does NOT catch stack-overflow aborts —
  those are the parser `MAX_PARSE_DEPTH` + matcher `MAX_MATCH_DEPTH` guards.
- **KNOWN LIMITATION — do not build pathologically-deep `Expr`s via the Rust API.**
  rum-core's `Expr` `Drop` is the default RECURSIVE drop; dropping a `>~thousands`-deep
  `Arc<Expr>` chain overflows the native stack (SIGABRT) and takes the host process
  with it. The reachable crash (deep PARSE input) is closed (a `MAX_PARSE_DEPTH`=128
  parser guard, brief-055) and every WL path is depth-capped (parser guard + eval
  `recursion_limit` 1024 + resource limits), so this CANNOT happen via evaluating WL.
  It is only reachable by a host constructing a pathologically-deep `Expr` directly
  via `Expr::call(...)` in a Rust loop. An iterative `Drop` was tried three ways
  (briefs 056/057: unsafe `ManuallyDrop`, safe `Option<Arc>`, safe sentinel) and ALL
  regressed the eval hot path 20-50% (every drop pays the manual drain) — rejected per
  the no-default-path-cost rule. So: **a host must not build deeper than ~a few
  thousand levels of `Expr` nesting via the Rust API.** If a real host needs it, the
  options are a depth-limited builder, a dedicated iterative-drop helper the host calls
  explicitly for known-deep trees, or accepting the per-drop cost — revisit then.
- **Structured value extraction — DONE (brief-061, `0423bb4`).** `HostValue` now carries
  `typed: HostTyped` alongside the unchanged `.text`: `Integer(i64)` | `BigInteger(String)`
  | `Rational{numerator,denominator}` | `Symbol(String)` | `Str(String)` |
  `List(Vec<HostTyped>)` | `Other{head,text}`, built once from the result `Expr`,
  depth-capped at 64. (rationals-only → no float variant; big values are decimal strings.)

**Tier 2 — for long-running, multi-client hosts — CORE ITEMS DONE (briefs 062/063/064/065).**
- **reset / clear session — DONE (brief-063, `0bde8b7`).** `HostSession::reset()` rebuilds
  the inner session from the STORED original `HostConfig` and clears `poisoned` — it is
  also the RECOVERY path for a session poisoned by a caught panic (brief-060). Drops all
  user state; invalidates prior `cancel_handle()`s (new interrupt flag).
- **Session snapshot / restore from a buffer — DONE (brief-065, `3d4a895`).**
  `snapshot_to_bytes()` (&self) renders all definition-map symbols (+ attributes) to a
  WL-source `Vec<u8>` via the existing `render_saved_symbols` (no file); `restore_from_bytes()`
  decodes UTF-8, `reset()`s, and evals — a faithful IMAGE load, NO filesystem touched (the
  WASM point). Captures definitions + attributes of defined symbols; NOT attribute-only
  symbols / `$`-vars / options / messages / gensym-counter; TEXT image (binary `.mx` stays
  a separate deferred perf item).
- **Concurrency / isolation contract — DONE (brief-062, `1f5b1b0`).** `HostSession` (and
  inner `Session`) is now **`Send`** (movable to a worker thread; only blocker was the
  suspension-frame trait object, fixed with a `+ Send` bound, no `unsafe`, no `!Send`
  capture) but intentionally **`!Sync`** (single-threaded mutable state — `Cell`/`FnMut`;
  run one per thread, don't share). Proven by a static assert + an 8-thread move/isolation
  test. The process-global symbol interner is a shared `Mutex` (contention point + shared
  interned-NAME table) but definitions/ownvalues/contexts are per-session and isolated.
- **Whole-session introspection / enumeration — DONE (brief-064, `afad4de`).** Host
  `definitions()` (per symbol: name + has_own_value + down/up/sub counts, sorted) +
  `contexts()`, pure `&self` reads (work on a poisoned session). Native builtins excluded
  (only the definitions map is read).

**Downstream — rum-jupyter upgrades UNLOCKED by this hardening round (a dedicated
post-Tier-2 brief).** Surveyed 2026-07-01:
- **Real interrupt.** `KernelAdapter::interrupt` is today a STUB — it only sets
  `lifecycle_state = InterruptRequested` and returns ok; it does NOT stop a running eval.
  Worse, the transport (`zeromq.rs`) is a SINGLE-THREADED poll loop and `execute_request`
  is a synchronous blocking call, so while execute runs the loop can't even RECEIVE the
  interrupt message. The hardening supplies the missing primitives: brief-059
  `cancel_handle()` (stop a running eval), brief-062 `Send` (run execute on a worker thread
  holding the session so the loop stays responsive to the Control channel), brief-060
  panic boundary (a kernel panic → graceful error reply + poison → restart the session,
  not crash the process). The remaining work is a jupyter-side restructure: run execute on
  a worker, keep polling Control, wire `interrupt_request` → `cancel_handle.cancel()`, and
  auto-rebuild on `Poisoned`. This turns the stub interrupt into a real one + makes the
  kernel panic-resilient.
- Lower value: `history` (returns empty — a ring buffer could fill it), `comm_info` (empty
  — rum has no comm/widget protocol), `debug` (returns a clean "not implemented"); typed
  `display_data` from brief-061's `HostTyped` (marginal for a text kernel).

**Tier 3 — nice-to-have:**
- **Deeper debug.** The structured `trace` (EvaluationPass / DefinitionStored /
  RuleApplied) is post-hoc; no step / breakpoint / "why didn't this pattern match"
  diagnostics.

## 3. Showcase keepers (only the two durable enablers; project undecided)

- **Sandbox selling point.** rum is **sandboxed by design** (no Import/Export/file/
  network in the language). It is therefore safe to feed UNTRUSTED expressions —
  e.g. LLM-generated ones — the worst case is a wrong answer or a (configurable)
  resource-limit trip, NOT privilege escalation. A genuine differentiator vs
  embedding Python/SymPy (generated code can do I/O / `os.system`) or a cloud API
  (network + data egress). Pairs with the Tier-1 resource budgets for DoS bounding.
- **WASM-friendliness.** Minimal pure-Rust deps (`num-bigint`/`serde`/`thiserror`),
  no fs/threads/net in the core tree, `mimalloc` CLI-only → `wasm32` is low-friction.
  Enables shipping rum as a single lightweight module embeddable in Tauri (native
  Rust crate — no WASM even needed), Electron (WASM or napi addon), or VS Code (WASM)
  hosts. The architectural mapping when embedded: the host app is the embedding
  program, a thin adapter (e.g. a `rum-wasm` / Tauri-command layer, peer to
  `rum-jupyter`) bridges the host's calls ↔ the `rum-host` API; `rum-host` stays the
  single transport-agnostic boundary.

> The specific showcase project (and whether to optimize for "impression" like
> `mma-hol` vs "utility" like an embedded reasoning co-processor) is left open on
> purpose — revisit after `mma-hol` is fully through.
