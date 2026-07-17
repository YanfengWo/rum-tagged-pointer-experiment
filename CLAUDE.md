# CLAUDE.md — director's notes for `rum`

This file is **mine** (Claude). `AGENTS.md` is the operational first-stop guide
that any coding agent — including Codex — reads when executing work. This file
is about how *I* operate on this project: direction, diagnosis, delegation, and
the authoritative gate. Keep it short and high-signal; link to canonical docs
rather than duplicating them.

## Role split (the working mode)

- **I own**: overall direction, the diagnosis, picking the next target,
  writing detailed Codex task briefs, the authoritative gate, and the git
  commit. I do the *judgment*; the legwork I delegate.
- **Codex owns**: executing exactly one brief, self-verifying to green, and
  reporting with verbatim evidence. It does not commit, branch, or push — it
  leaves changes in the working tree for me to gate.
- The brief method lives in `docs/dev/codex/`: `TEMPLATE.md` is the format,
  `brief-NNN-<slug>.md` are the packages. Dispatch via the helper (commit the
  brief FIRST, as a separate step):

  ```
  scripts/dispatch-brief.sh NNN     # e.g. 124, or brief-124, or a slug fragment
  ```

  It resolves `docs/dev/codex/brief-NNN-*.md`, runs `cdx exec -s
  danger-full-access` with stdin from `/dev/null`, and logs the full report to
  `/tmp/cdx_brief_NNN.log` (`DRY_RUN=1` prints the resolution without
  dispatching). **Launch it via the Bash tool's `run_in_background:true`** so
  the run is harness-tracked — do NOT hand-type `cdx exec … &`; the `&`-detach
  is not tracked. `-s danger-full-access` = Codex's sandbox level (needed for
  `cargo` and the CLI); the brief's **Scope + Hard rules** are the real fence.
- Do not mutate the repo while a brief is in flight (my commits queue until
  the gate).

## The gate I run before crediting any brief

1. **Scope check**: `git status` / `git diff` touched only the brief's
   MAY-modify list.
2. **Tests**: `cargo fmt --check` and `cargo test` green. Default to
   `--workspace`; use a narrower `-p` gate ONLY when the change is provably
   crate-local (a green narrow gate is not a green tree).
3. **Claims**: I reproduce the evidence myself — test totals, sweeps, and for
   perf work the microbench A/B (`scripts/core-language-microbench.sh`,
   rum-vs-rum, per-label `timings.tsv`, same-tier neighbors checked). I trust
   the numbers I reproduce, not the report's interpretation.
4. Only then do I commit and dispatch the next brief.

## Hard lines I enforce (do not let these drift)

- **Reply to the user in Chinese — always.** This project's working language
  is Chinese; the user cannot read any other language. Every user-facing
  message is in Chinese; only technical tokens (code identifiers,
  command/file names, WL/Rust terms) stay verbatim in English. Briefs,
  commits, and repo docs are drafted in English (the codebase is English),
  but the conversation is Chinese. Watch for ANY non-Chinese drift in long
  reports.
- **The testing policy is permanent** (see `AGENTS.md` §Testing Policy and
  `DISCLAIMER.md`): tests verify rum's own documented behavior plus
  rum-vs-rum perf A/B only. No black-box differential testing against other
  vendors' engines, no oracle harnesses, no compatibility matrices or
  reports, no committed third-party engine outputs — and I do not let a brief
  or a contributor reintroduce them.
- **No default-path cost for diagnostics or off-by-default features.**
  Counters and instrumentation go behind the env-gated cold-load `AtomicU8`
  pattern; a `Mutex`/allocation/branch paid on every call when the flag is
  off is a regression.
- **No new `unsafe` in `rum-core`/`rum-runtime`** (zero today).
- **Measure honestly**: profiling env OFF for any timing number; one variable
  per A/B; persist raw output to files before parsing; derive timeouts from
  data, not guesses.

## Diagnosis method (durable lessons)

- Pick targets from the bottom-up microbench ratio, not from a sampler's
  loudest leaf; decompose a composite probe into factors before attributing
  its cost; estimate the expected win from the profile before keeping a
  change.
- **Re-probe an audit finding before briefing a fix for it — an earlier batch
  may have already changed it.** Unit 8's audit recorded a "silent false
  round-trip" for `DownValues[g] = DownValues[f]`; by the time I got to that
  batch, the `Protected` sweep (two batches earlier) had already turned it into
  a loud failure. The premise had shifted from "fix a safety hole" to "build a
  WL feature" — a different decision, which belonged to the user, not to me
  silently. Findings age; verify, then brief.
- Codex's recurring failure modes to watch in review: optimizing sub-second
  hotspots while the parent workload is elsewhere; fixing at the wrong layer;
  masking a symptom (reducing an operation count to hide a per-op slowness)
  instead of fixing the per-op cost; **over-generalized scope** — a correct
  root-cause observation attributed too widely (brief-208 correctly saw that
  `store_held_update` skips a protection recheck, then attributed the resulting
  bypass to all 11 compound assignments when only `ApplyTo` — whose update runs
  user code in the store window — can actually trigger it).
- **A dangling evidence citation may be a truncated write, not a fabrication —
  and my own falsifying probe may be too weak to trust.** brief-208 cited
  "Evidence E15/E16" that were absent from the file; the run was truncated at
  the end (an upstream `cybersecurity risk` content filter fired in the dispatch
  log's tail, so the protection-bypass section was likely cut off mid-write). I
  gated it "false" from ONE counter-probe (`AddTo` with a protecting downvalue)
  that returned `wrsym` — but that probe could not reach the code window (the
  second arg is not held, so protection fires before the guard). Reading the
  code showed the window is real and `ApplyTo` triggers it (`x = 101` on a
  `{Protected}` symbol, no message). Lesson: **for a diagnostic-level safety
  claim, a single green counter-probe does NOT falsify it — read the code path
  and confirm the mechanism.** A weak probe's NOT-REPRODUCED is a false
  negative. When a report cites evidence that isn't in the file, suspect
  truncation (check the dispatch log tail for a filter/quota cutoff) before
  suspecting invention, and settle the claim from the source, not from one
  probe.
- **Calibrate the instrument before believing it — three ways it lied to me in
  one session (unit 8):**
  - *A timing harness that measured nothing.* An A/B over `Do[…, {300000}]`
    showed both arms identical — because the big `N` silently hit the iteration
    ceiling and bailed out, so I was timing process startup. **The tell: `N=1e6`
    came back FASTER than `N=1e5`.** Always check that time SCALES with `N`
    before trusting a single timing number.
  - *A parser that under-reported.* `docs/reference/builtins.md` backticks each
    attribute separately (`` `Listable`, `Protected` ``); a regex capturing only
    the first backticked token said "527 unprotected" when the truth was 483.
    Cross-check any script-derived count against a few live probes.
  - *A hybrid test session.* `session_with_test_control_builtins()` resolves SOME
    symbols to test-registered builtins and others to the real catalog (a test
    registration with EMPTY attributes falls through to the catalog). My
    "obvious" uniform fix to four attribute assertions was wrong for two of them.
    Probe ground truth per symbol; never assume a uniform rule across a test
    harness.
- **Noise discipline for an A/B**: if the UNAFFECTED control labels move as much
  as (or more than) the affected one, that IS the noise floor — report no
  regression, and do not "fix" noise. Say plainly when you did NOT re-run a
  gate and why (e.g. no eval-hot-path code changed and the panel never calls the
  changed symbol) rather than performing the ceremony.
- Profiling toolkit: `scripts/sample-self-time.py`,
  `scripts/sample-attribute-callers.py`, and the env-gated runtime counters;
  self-time + caller attribution + counters beat call trees for the recursive
  evaluator.

## Current direction (read these for state)

- **ACTIVE: CLEANUP + MESSAGE-REGISTRY LINE (user ruling 2026-07-17;
  queue in TODO.md §ACTIVE PLAN).** In order: (1) builtin message-tag
  registry + docgen pairing (brief-243 inventory LANDED; mechanism +
  fill + docgen briefs next); (2) `vm_rhs_template/` — DONE, DELETED
  (brief-244: ON path crashed the guard-adjacent recursion test; A/B
  clean); (3) rum-jupyter `signature.rs` hand-rolled
  sha256 → workspace RustCrypto (+ ZeroMQ flake if it recurs);
  (4) rum-cli feature forwarding; (5) then the wl15/wfr design pass
  (structured exceptions, SubValuesHoldAll, ReplaceAllUnheld,
  ConformSchema operator forms). **Same ruling: the parked pools are
  CLOSED WON'T-DO** — stdlib parked candidates (ByteArray audit,
  compression, unicode normalization, ipaddress folk, fuzzing, codec
  follow-ons) and the embedding parked tail (background-eval helper,
  per-request rollback API, lighter checkpoint). Standing Engineering
  Debt stays a record of known limitations, not a queue. fs/os/net
  stay deferred until after publication. Memory:
  `message-registry-cleanup-line.md`.
- **RELEASE TRANSFORMATION LINE — COMPLETE (2026-07-04 → 07-06).** `main`
  itself was transformed into release shape with full history kept (no
  remote; endpoint = clean tree + bundle cold backup; publication mechanics
  are a later user decision). Landed: briefs 124-131 + M0/M1/M1.5/M2 —
  difftest and external-engine tooling removed, docs archived, narrative and
  policy rewritten, wording scan, generated builtin reference + drift test,
  nine-chapter guide, rustdoc + metadata, native stack guard +
  ReplaceRepeated limit-as-value. Removed material lives in the private
  archive `/Users/fft/Developments/Archive/rum-history/` (MANIFEST.md maps
  each item to its removal commit; recycling candidates flagged). Memory:
  `release-transformation-line.md`.
- **PUBLICATION DEFERRED INDEFINITELY (user directive 2026-07-06) — do not
  steer toward release.** rum publishes only after sufficient divergence.
  Direction: embedded scripting + sandbox safety + rapid script development
  (elisp/guile/lua/python family). The FS-purity line (briefs 132-134)
  landed the three mechanical kernel hard lines: zero unsafe, zero std::fs
  (non-test), Pure = deterministic. The stdlib-expansion line (briefs
  135-156) is COMPLETE.
- **USABILITY PASS — COMPLETE (2026-07-07 → 2026-07-14, briefs 161-230).**
  The "works well" layer over every installed builtin (three axes:
  flexibility/convenience/safety), audit-then-fix per sweep unit. Tracker
  `docs/dev/usability-pass.md` (per-unit ledgers + close-outs); memory
  `usability-pass-line.md`. **All 10 units closed — 605 builtins swept**
  (units 1-7 = 446 stdlib, briefs 161-207; unit 8 = 55 kernel A, 208-213;
  unit 9 = 47 kernel B, 214-222; unit 10 = 57 kernel C, 223-230), reference
  613 after the FileNames restore. The kernel units carried the highest-value
  finds:
  - **Unit 8 (kernel A)**: `Protected` enforcement surface lifted 129/612 →
    608/612; a REAL `ApplyTo` protection-bypass bug caught at the gate
    (`store_held_update` skipped the recheck). Standing boundary: **`Protected`
    is a mistake-guard + WL conformance, NOT a sandbox boundary** — real
    isolation = per-request rollback = the dun host line; do not conflate.
  - **Unit 9 (kernel B)**: evaluator hold correctness across three user
    rulings — `Unevaluated` no-consumer restore, `HoldAll` Sequence splice,
    Module error-path gensym reclamation (WL semantics; mma-hol Strict gate),
    staged `With`, replacement-produced Sequence splices into held rebuilds.
  - **Unit 10 (kernel C)**: a REAL loader capability bypass caught at the gate
    (`BeginPackage`/`DeclarePackage` → `eval_needs` skipped the `FileRead`
    dispatch check; fixed by gating at the `eval_get`/`eval_needs` seam) and a
    REAL `Pause[10^100]` host-process panic (Duration overflow). Three
    capability-decision candidates ruled: `Input`/`InputString`/`Dialog` →
    new `Capability::Interactive` (sandbox denies, trusted allows); `TestReport`
    → `Clock` KEPT (a test report is intrinsically multi-dimensional — timing
    is its job, not an impure contaminant; **not every impure label is a
    mislabel to purify**); `FileNames` → restored as a `FileRead`-gated folk
    glob through a new `list_dir` provider method.
  - DEFERRED (stated, not dropped): the **builtin message-tag registry**
    (`Messages[Set]` empty even after `Set::argx`; ~108 ad-hoc `push_message`
    sites, no static catalog). Its own project; pairs with docgen.
  Standing user directives (still in force for any future builtin work): high
  coverage incl. the WL-inherited surface; compat enhancement only (default
  behavior stays WL-conformant, new powers via options / rum-native siblings).
  Standing design rulings: arithmetic singularities stay INERT (no
  special-value arithmetic — user); Orderless/Flat/OneIdentity on folk numeric
  reducers + Equivalent + the Implies short-circuit all REJECTED;
  ConformSchema/SchemaErrors operator forms DEFERRED (HoldRest hold-semantics).
- **STATEFUL↔PURE SEPARATION LINE — COMPLETE (2026-07-08, briefs 200-202).**
  User rulings, spun out of unit-6 datetime. Killed the pure/impure
  DUAL-FORM anti-pattern (one symbol mixing a pure + a stateful form).
  Final: `AbsoluteTime[]` = the sole numeric wall-clock reader (`Clock`,
  spec-level, reader-only, kernel); `FromDateList` = pure converter (`Pure`,
  stdlib, inverse of DateList); `DateList`/`DateString` genuinely pure;
  `Now` REMOVED (reserved for downstream rum-os); `SeedRandom` split →
  `ReseedRandom[]` (`Random`) + `SeedRandom[seed]` (`Pure`). **Standing
  rule: stateful reads are single-purpose symbols gated by a SPEC-level
  capability — never crammed into a pure symbol, never gated by a runtime
  check (stdlib now has ZERO runtime capability checks).** Whole-surface
  audit done; the lone mild kernel candidate `TestReport` was reviewed and
  CLOSED with NO CHANGE (unit 10, user ruling 2026-07-14): a test report is
  intrinsically multi-dimensional (correctness + timing + memory), so `Clock`
  is the honest label, not a mislabel to purify. Detail:
  `usability-pass-line.md` + tracker.
- **HOST-EMBEDDING LINE — COMPLETE. Concern (b) closed at b4 (2026-07-15);
  concern (a) = the EMBEDDING API LINE, COMPLETE (2026-07-16, briefs
  236-242, opened and closed in one day; canonical doc
  `docs/dev/embedding-api-line.md`, memory `embedding-line-a-scenarios`).**
  (a) endstate: scenario design (three user-supplied abandoned downstream
  concepts: Python/AI-infra adapter, Rust/wasm tooling, tauri typst IDE =
  dun's ancestor) + in-tree consumer survey (findings F1-F8) drove five
  gated increments — a1 contract honesty (exit_status, requires_reset, b1
  termination-field adoption), a2 dogfood arc (contract surface + rum-cli +
  rum-repl ports; **in-tree hosts are pure rum-host consumers — cli/repl
  have ZERO rum-runtime/rum-stdlib dependencies; user ruling: dogfood
  stance A**), a3 typed bridge out (Bool/Association/Rule + jupyter
  application/json), a4 typed bridge in (expr_from_typed + injection-safe
  `HostSession::call`; kernel seam `eval_expr_request`), a5 named profiles
  (hermetic/scripting; all-8-capability pins, a ninth Capability variant
  fails compilation until every profile takes a stance). Parked pending
  scenario pull: background-eval helper, per-request rollback API, lighter
  checkpoint. Cold backup `rum-2026-07-16-3d31e09.bundle`. Watch: a
  rum-jupyter native ZeroMQ ordering flake has surfaced twice
  (brief-165, brief-242; green in isolation both times) — brief it on a
  third occurrence. **NEXT DIRECTION decided 2026-07-17: the cleanup +
  message-registry line (top of this section); the parked tail above is
  closed WON'T-DO by the same ruling.**
  Original sequencing (user 2026-07-14): concern (b) dun pure-sandbox +
  protocol conformance FIRST (explicit requirements, clear boundary),
  concern (a) general embedding API only AFTER (b) completes, and (a) MUST
  open with scenario design + boundary drawing — no feature bricklaying on
  instinct. That is how it ran.
  **b1–b4 DONE and reviewed boundary-clean** (b1 structured termination
  discriminator, b2 request-level memory ceiling, b3 stdout-ownership hygiene
  hard line, b4 crash-only + per-request isolation contract — all generic
  sandbox-hardening primitives, zero dun/editor/protocol naming in the
  kernel). **b5 (dun protocol host prototype) CANCELLED (user boundary ruling
  2026-07-15): building a dun plugin is downstream work, not rum's job — rum
  is a language project, dun an editor project; rum's obligation was to harden
  the sandbox to downstream-usable strength, achieved at b4, and hosting one
  editor's reference plugin crosses the "don't make rum an editor framework"
  line. Part 1 (brief-235, the `rum-dun-host` codec crate) had landed and was
  reverted (`6062c03`+`ed01e33`, revert commit `93a93d2`).** **fs/os/net
  downstream capabilities are DEFERRED until after publication (user
  2026-07-15) — explicitly NOT the next target.** (Historical diagnosis
  detail below; the concern-(a) gaps it lists were all closed by briefs
  236-242.) Diagnosis headline: gates 1-3 done (sandboxed
  profile, Interactive capability, panic isolation), stdout/crash-only/
  cold-start (~4.8 ms full cycle) already satisfied but unpinned; the two
  real seams = structured termination discriminator (graceful cancel/timeout
  is in-band `TerminatedEvaluation[...]` Success) + request-level memory
  ceiling (mechanism exists via `MemoryConstrained`/`current_memory_limit`,
  missing the `set_eval_timeout`-analog knob; active-path walk needs
  amortization). The downstream `dun` editor is the first real pull, but the
  framing was RESET twice, both recorded in `dun-plugin-requirements-line.md`:
  1. **dun went protocol-first (2026-07-13, third doc
     `docs/dev/dun-protocol-host-requirements.md`).** dun shipped a
     protocol-first plugin system: plugins run in EXTERNAL host processes
     speaking a framed-stdio JSON protocol; the editor links no runtime. As
     dun-facing surface, the in-process embedding asks (`session.call`,
     `HostTyped` as dun's result surface) are DEAD — they demote to internal
     needs of a small `rum-host` shim (protocol on one side, kernel on the
     other). What SURVIVES and is now load-bearing for dun's `pure-sandbox`
     trust class (rum's exclusive slot — Python/Lua hosts can't clear it):
     pure-only + non-interactive + per-request isolation + crash-only +
     resource-outcome classification, PLUS new protocol-born asks (stdout
     fully owned by the embedder so plugin `Print` can't corrupt framing;
     kernel-side memory ceiling with a classified outcome — dun has no RSS
     watchdog; cold-start latency; JSON boundary).
  2. **User strategic reframe (2026-07-14): do NOT narrow the line to dun's
     protocol shim.** dun began as a showcase of rum's EMBEDDABILITY; its
     pivot (rum kernel too big for a lightweight editor) means dun now shows
     off only the sandbox, not the kernel's session-management / in-kernel-
     communication strengths. Those strengths are still wanted by FUTURE
     tighter-embedding applications, so evaluate the general embedding API
     (rich `HostTyped`, typed `call`, session management) on **general
     embeddable-language merits, not dun's current needs**. → the line has
     **two parallel concerns**: (a) the general embeddable-language API,
     judged on general value; (b) dun's `pure-sandbox` + protocol conformance.
     Overlapping but not conflated.
  How it actually ran: the diagnosis landed (2026-07-14), then concern (b)
  executed as briefs b1–b4 (231–234) and CLOSED at b4 with b5 cut (above).
  The `syntax-highlight` `rum-host` prototype that the diagnosis proposed was
  the cancelled b5 — that idea is retired (downstream work). Hard line held:
  don't make rum an editor framework (the protocol boundary made the
  separation physical). Memory: `dun-plugin-requirements-line.md`.
- Earlier arcs (correctness mainline, memory line, host-hardening Tier-1/2,
  pattern-matching line, crate rebalance route 2a, file slimming) are all
  CLOSED; their plans/audits are in the archive and in git history. Cross-
  session state lives in my memory directory, not in `PROGRESS.md` (user ban
  on reading it for context; appending entries is fine).

## Pointers

- `AGENTS.md` — operational first-stop (snapshot, commands, testing policy,
  reading path).
- `TODO.md` — active roadmap. `PROGRESS.md` — append-only log.
- `~/.claude/.../memory/` — my cross-session memory of this project's arc and
  the recurring lessons (`rum-project-context.md`,
  `release-transformation-line.md`, …).
