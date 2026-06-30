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

**Tier 1 — required for untrusted / embedded use:**
- **Interrupt / cancel.** `HostSession` has no `cancel()`/`interrupt()`; `eval` is
  synchronous and uninterruptible from another thread. (`docs/v0.1-acceptance-boundary.md`
  already lists interrupt/cancel as deferred.) Needs a cancellation flag the eval
  loop checks periodically. #1 gap.
- **Session-level time + memory budget.** `HostResourceLimits` (config.rs) has
  recursion / replace_repeated / pass / generated_element / integer_work /
  iteration_work — but NO wall-clock time budget and NO memory budget.
  `TimeConstrained`/`MemoryConstrained` exist only at the WL level (guest code must
  wrap); `HostFailureCode` already surfaces both. The host cannot impose a per-eval
  time/memory cap. DoS surface.
- **Panic-safety boundary.** The host API should wrap eval in `catch_unwind` (a
  future FFI/pyo3 boundary must NOT let a panic cross). [The deep-nesting parser
  crash that motivated this is now fixed — see the limitation note below.]
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
- **Structured value extraction.** `HostValue` is `{ text: String }` only — the
  result comes back as a STRING. Embedders (Rust / Python / JS) need typed access
  (Integer / Rational / Symbol / List / Expr → `i64` / `Vec` / native), else the
  host re-parses.

**Tier 2 — for long-running, multi-client hosts:**
- **reset / clear session** to a clean state (multi-client reuse / isolation).
- **Session snapshot / serialize as host ops.** `DumpSave`/`Get` are WL-level AND
  file-based; a `restore_from_bytes(...)` (load a session image from an in-memory
  buffer) is needed — also the enabler for any no-fs host (WASM).
- **Concurrency / isolation contract.** Is `HostSession` `Send`/`Sync`? Can N
  sessions run in parallel threads? The global symbol interner is a
  `Mutex<HashSet>` (the memory line) shared process-wide — contention / cross-session
  leakage. Make the contract explicit and tested (ties to the `Rc`-vs-`Arc` tradeoff).
- **Whole-session introspection / enumeration.** `inspect` is per-symbol (you must
  name it); no host-level "enumerate all user definitions / contexts".

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
