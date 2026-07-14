# Host-Embedding Line — Diagnosis & Inventory

Status: diagnosis landed 2026-07-14 (Claude, own work — not a Codex brief).
Inputs: `dun-protocol-host-requirements.md` (the authoritative dun-facing
ask list), the two earlier dun requirement docs, the `rum-host` and kernel
session sources, and fresh measurements (this file cites the evidence).

## Framing (user rulings, 2026-07-14)

The line has **two parallel concerns, overlapping but not conflated**:

- **(a) General embeddable-language API** — session management, in-kernel
  communication, typed invocation, rich result extraction. Judged on
  *general embedded-language merits* (the elisp/guile/lua family), NOT on
  dun's current needs. dun's pivot away from in-process embedding is not a
  reason to cut these.
- **(b) dun `pure-sandbox` + protocol conformance** — rum's strategic slot
  (Python/Lua hosts cannot clear the pure-sandbox bar; dun's default
  untrusted-plugin experience is gated on a pure-sandbox host existing).

**Sequencing (user, 2026-07-14): diagnosis first → concern (b) first
(requirements are explicit, boundary is clear) → concern (a) only after (b)
is complete, and (a) MUST open with scenario design and boundary drawing —
no feature bricklaying on instinct.**

Hard lines: rum does not become an editor framework (result schemas belong
to dun); the testing policy stands (fixtures test rum's own hosts only);
no new `unsafe`; no default-path cost for off-by-default features.

## Concern (b) part 1 — the five pure-sandbox gates

| # | Gate (dun's definition) | Status | Evidence |
|---|---|---|---|
| 1 | Pure-only evaluation profile (no fs/proc/net/clock/entropy/terminal) | **DONE** (kernel level) | `HostConfig::sandboxed()` = `{Pure}` policy + no FileIo + no entropy provider (`crates/rum-host/src/config.rs:171`). Capability gating is at builtin dispatch; stdlib has zero runtime capability checks (stateful↔pure line). Network never existed in-kernel; process is `Capability::Process`-reserved. |
| 2 | No interactive suspension | **DONE** (brief-228) | `Input`/`InputString`/`Dialog` all require `Capability::Interactive` (`builtin_catalog/scoping.rs:99-118`); `sandboxed()` denies at dispatch — tested (`rum-host/src/tests.rs:249`). The suspension machinery still exists for trusted embedders (correct: capability-gated, not deleted). |
| 3 | Per-request panic/error isolation | **DONE** | `catch_unwind` guard at every host entry point → session poisons; `reset()` recovers, or the host process exits and dun relaunches (`session.rs` `guard`/`poisoned`). |
| 4 | Request isolation (no state leak between requests by default) | **PARTIAL** | No first-class per-request mode. Achievable today: `reset()` per request (rebuild + reinstall builtins; full process cycle measured ~4.8 ms, so reset is well under that) or snapshot/restore (text image). Gap = a *decided, documented, tested* pattern for the shim (explicit state passes as protocol data anyway, per dun's log-filter design). |
| 5 | Resource-limit outcome classification | **PARTIAL — the real seam** | `HostFailureCode` is already fine-grained (`Cancelled`, `TimeConstraintExceeded`, `MemoryConstraintExceeded`, `GeneratedElementLimitExceeded`, `IntegerWork…`, `IterationWork…`; `outcome.rs:69-108`). BUT graceful cancel/timeout deliberately returns `HostEvalOutcome::Success` with in-band `TerminatedEvaluation["Cancelled"|"TimeConstraint"]` + a `General::cancel`/`timelim` message (`tests.rs:1150-1196`) — the shim must string-match. dun names this exact pain. Fix = a structured termination discriminator ON success (keep the in-band value; it is intentional partial-result semantics). |

## Concern (b) part 2 — the five protocol-born asks

| # | Ask | Status | Evidence |
|---|---|---|---|
| 1 | Stdout fully owned by the embedder; all plugin output captured | **DONE de facto, UNPINNED** | Kernel non-test code writes **zero** bytes to stdout (grep: the only `println!` in the three kernel crates is the codegen bin `generate-builtin-reference.rs`). `Print` → `push_print` → per-request `prints` buffer (`session/session_helpers.rs:176`), surfaced as `HostEvalSuccess.prints`. Env-gated profiling writes to **stderr** only. Gap = make it a mechanical hard line: a hygiene test banning stdout writes in kernel crates' non-test code, mirroring the `std::fs` hygiene test (`runtime/tests/hygiene.rs`). |
| 2 | Crash-only lifecycle (SIGKILL-safe, no residue) | **DONE by design, UNSTATED** | `std::fs` is hygiene-test-banned in all three kernel crates; sandboxed sessions have no providers at all; no temp files, no destructor-dependent state. Gap = a stated contract + a test (cheap). |
| 3 | Cold-start latency (2 s budget incl. handshake; ideal a few ms) | **MET, measured 2026-07-14** | Release `rum-cli`: 50 sequential full cycles (spawn + 613-builtin install + eval + exit) = 0.24 s, 0.24 s, 0.23 s over three rounds → **~4.8 ms per full process cycle**. Two orders of magnitude inside budget, already in the "ideal" band. Cross-check: brief-154 measured session construction 0.09 ms steady, eval 4.5 µs. Gap = none functionally; decide whether to pin (a doc'd measurement script beats a flaky CI timing assert). |
| 4 | Kernel-side memory ceiling with classified outcome | **PARTIAL — biggest real gap, mechanism already exists** | In-kernel: `current_memory_limit` + proxy-byte accounting (`debug_tools/resources/memory.rs`: `current_memory_proxy_bytes`, `expr_memory_proxy_bytes`; surfaced as `ByteCount`/`MemoryInUse`/`MaxMemoryUsed`), enforced in the evaluator core loop (`eval_core/mod.rs:61`) → `EvalError::MemoryConstraintExceeded` → `SessionRequestError` → `HostFailureCode::MemoryConstraintExceeded`. TODAY it activates only inside `MemoryConstrained[body, bytes]` (opt-in, in-band `$Aborted`/fallback on exceed — same in-band problem as timeout). MISSING: a **request-level knob** (analog of `set_eval_timeout`, `session_api.rs:71`) so the shim sets a ceiling per request and an exceed comes back as a *classified request failure*. Design constraints: (i) default path already costs only an `Option` check — keep it that way; (ii) when ACTIVE, `current_memory_proxy_bytes()` walks the whole session state per enforcement point — a per-request always-on ceiling needs an amortization decision (stride/incremental) measured honestly; (iii) proxy bytes ≠ RSS — document as a modeled proxy (good enough for balloon defense). |
| 5 | JSON at the boundary (results extractable without trusting plugin serialization) | **MOSTLY SHIM-INTERNAL** | `FromJson`/`ToJson` exist (stdlib, Pure) but the *shim* should extract via `HostTyped` and serialize in Rust — never eval plugin-controlled serialization code. `HostTyped` today: Integer/BigInteger/Rational/Symbol/Str/List/Other (`outcome.rs:121-146`) — sufficient for the syntax-highlight role (lists of integers/strings); Bool/Association/Rule collapse to `Other{head,text}`. Richer extraction is concern (a) territory; pull in a minimal slice only if the shim demonstrably needs it. |

## The missing artifact

No protocol shim exists. dun's dev kit is ready and confirmed on disk:
`~/Developments/dun/docs/plugin-protocol.md` (normative spec),
`~/Developments/dun/hosts/check-host.py` (language-agnostic conformance
checker), three reference hosts (`rust-syntect`, `python-pygments`, `lua`).
First role: `syntax-highlight` (bounded input, fixed style vocabulary,
easiest conformance). dun never gets touched (read-only clone).

## Proposed concern-(b) brief queue (pending user sign-off)

1. **b1 — structured termination discriminator. DONE (brief-231,
   74afadf, 2026-07-14).** `EvalTerminationReason` (9 reasons; `.tag()`
   preserves the legacy strings) threaded from the finalize seam through
   `EvalOutcome`/`SessionEvalResponse` to
   `HostEvalSuccess.termination: Option<HostTerminationReason>`. In-band
   values byte-identical; user-constructed `TerminatedEvaluation[...]`
   literals stay `None` (provenance tested). Gate: workspace 1478/0.
   Closes gate 5.
2. **b2 — request-level memory ceiling. DONE (brief-232, ee8b788,
   2026-07-15).** `ResourceLimits.eval_memory_limit: Option<usize>`
   (default `None`) arms `current_memory_limit` per top-level request at
   the same seam as the wall-clock deadline; an uncaught breach returns
   in-band `TerminatedEvaluation["MemoryLimit"]` + `General::memlim` +
   `termination == Some(MemoryLimit)` (b1's field). A top-level-only
   reason helper keeps the shared map untouched, so the
   `EvaluationData`/`TestReport` memory renderings are unchanged;
   `MemoryConstrained` frames min-combine and still catch in-band.
   Active-mode cost: full proxy walks strided every 64 armed checks
   (projected-byte prechecks unstrided). Gate-reproduced: armed workload
   3.825 s → 0.667 s (−82.6%, interleaved best-of-3); default path
   instruction-identical, microbench A/B against a calibrated noise
   floor (base-vs-base median 2.0%, max 10.4%) shows candidate median
   −3.4% and ZERO labels above the floor. Ceiling is modeled proxy
   bytes, not RSS — documented on the knob. Closes ask 4 (dun's stated
   single most valuable hardening item).
3. **b3 — stdout/stderr hygiene hard line.** Mechanical hygiene test
   banning stdout writes (and inventorying stderr writes as env-gated
   diagnostics only) in the three kernel crates' non-test code; a short
   embedder-facing contract note in the embedding guide. Pins ask 1.
4. **b4 — crash-only + per-request isolation contract.** State and test
   the crash-only property; decide + document + test the per-request
   isolation pattern (reset-per-request vs snapshot), including a
   no-state-leak test. Closes gate 4, pins ask 2.
5. **b5 — `rum` protocol host prototype (syntax-highlight role).** New
   non-default bin crate speaking Dun Plugin Protocol v0, sandboxed
   session, per-request isolation, shim-owned stdout, prints forwarded as
   bounded diagnostics, outcome→protocol-error mapping; validated against
   `check-host.py`; reports its own cold-start number (folds ask 3
   pinning). Location/naming decided in the brief (proposal:
   `crates/rum-dun-host`, not in `default-members`).

b1→b2→b3→b4 are kernel/host seams in rising size; b5 consumes them all.
b1/b2 are genuinely kernel-touching; b3/b4 are mostly tests + docs; b5 is
the deliverable dun can point its checker at.

## Concern (a) inventory — DEFERRED until (b) completes

Recorded so nothing is lost; **no work opens here without a
scenario-design pass first** (user directive: imagine the scenarios, draw
the boundary, do not add bricks on instinct).

Current session-management surface (already substantial):
`eval`/`eval_with_timeout`/`resume_with_input`, `cancel_handle` (cross-
thread), `reset`, `snapshot_to_bytes`/`restore_from_bytes`, `parse`,
`is_complete`, `complete`, `inspect`/`inspect_symbol`, `definitions`,
`contexts`, `with_builtin_installer`, capability policy, FileIo/Entropy
provider seams, resource limits.

Candidate gaps carried from the earlier inventory (each needs a named
scenario before it becomes a brief):

- `HostTyped` richness — Bool/Association/Rule collapse to `Other`
  (`outcome.rs:121`). Highest-leverage, cleanest.
- Typed `call(name, args)` — `HostEvalRequest` is source-string-only
  (`model/requests.rs:42`); a typed entry avoids escaping/injection.
- Named embedding profiles (sandboxed/interactive/scripting presets).
- Per-request rollback as first-class API (b4 will land the pattern;
  (a) decides whether it earns API surface).
- Session-communication strengths the user wants showcased: typed
  results, streaming outputs, definition inspection — scenario families
  to design against: REPL/notebook embedding (in-tree consumers exist:
  `rum-repl`, `rum-jupyter`, `rum-terminal`, `rum-cli`), rules/config
  engine inside a Rust application, batch scripting host. The in-tree
  consumers are real scenario material — survey what THEY had to work
  around before inventing needs.

## Measurement log

- Cold start (2026-07-14, M-series darwin, release build): 50-cycle
  amortized `rum-cli --eval "1+1"` = 0.24 s / 0.24 s → ~4.8 ms per cycle;
  third round with a String eval 0.23 s. Raw output:
  scratchpad `coldstart.txt` (session-local).
- `rum-cli` plain release binary today: 9.45 MB (was 8.53 MiB at
  brief-160; 6.19 MiB with strip+lto per the recorded A/B — size is not a
  dun gate anymore, single-user SSH deployment still favors small).
