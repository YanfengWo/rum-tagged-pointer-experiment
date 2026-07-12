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
- **USABILITY PASS — ACTIVE MAINLINE (opened 2026-07-07).** The "works
  well" layer over every installed builtin (three axes:
  flexibility/convenience/safety), audit-then-fix per sweep unit. Tracker
  `docs/dev/usability-pass.md`; memory `usability-pass-line.md`. **Units 1-8
  CLOSED — 501 builtins swept** (units 1-7 = 446 stdlib, briefs 161-207;
  unit 8 = 55 kernel A, briefs 208-213).
  **Unit 8 (kernel A — definitions/values/attributes/protection) was the
  richest unit so far**; its arc is worth remembering:
  - The audit found the `Protected` **enforcement surface was 129/612** —
    `Plus[a_,b_] := 42` succeeded and `1+2` then returned `42`.
  - **A REAL protection-bypass bug** surfaced at the gate (209): `ApplyTo[x, f]`
    where `f` protects `x` mid-update landed the write on a visibly `Protected`
    symbol, because `store_held_update` skipped the recheck and `ApplyTo` is the
    one compound assignment whose update runs user code in that window.
  - Sweep (210): **129/612 → 608/612** (only `$AssertFunction`/`$ContextPath`/
    `$Packages`/`$Path` stay open). Three enforcement holes closed (211).
    Value tables made round-trippable (212). Name patterns + `Information`
    hold/usage (213).
  - **Kernel list derived exactly**: reference 612 − stdlib catalog 453 = 159
    kernel names; unit 8 took 55, **104 remain for units 9-10**.
  - Standing boundary: **`Protected` is a mistake-guard + WL conformance, NOT a
    sandbox boundary** (a program may always `Unprotect[Plus]`). Real plugin
    isolation = per-request rollback = the dun host line. Do not conflate.
  - DEFERRED from unit 8: the **builtin message-tag registry** (`Messages[Set]`
    is empty even after `Set::argx`; ~108 ad-hoc `push_message` sites, no static
    catalog). Its own project; pairs with docgen. `Information`'s new `Messages`
    field lights up for free once it exists.
  **NEXT: unit 9** (kernel B — scoping/eval control/patterns/forms), then unit
  10 (the 104 kernel names split across the two), THEN the dun host-embedding
  API line (user sequencing ruling 2026-07-08).
  Standing user directives: high coverage incl. the WL-inherited surface;
  compat enhancement only (default behavior stays WL-conformant, new powers via
  options / rum-native siblings). Standing design rulings: arithmetic
  singularities stay INERT (no special-value arithmetic — user);
  Orderless/Flat/OneIdentity on folk numeric reducers + Equivalent + the Implies
  short-circuit all REJECTED; ConformSchema/SchemaErrors operator forms DEFERRED
  (HoldRest hold-semantics).
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
  audit done; lone mild kernel candidate `TestReport` flagged for units
  8-10. Detail: `usability-pass-line.md` + tracker.
- **DUN HOST-EMBEDDING LINE — QUEUED (after usability 8-10; user ruling
  2026-07-08).** The downstream `dun` editor filed two plugin requirement
  docs (`docs/dev/dun-*-requirements.md`) — the first real downstream pull,
  squarely on the post-deferral direction (embedded scripting + sandbox
  safety). Grounded gap analysis (rum-host is ~70% there): the genuine
  new work is a rum-host line — richer `HostTyped` (Association/Rule/Bool
  currently collapse to `Other`), a typed `call(name, args)` entrypoint,
  structured termination classification (cancel/timeout return `Success` +
  `TerminatedEvaluation[...]` text today), a named non-interactive plugin
  profile, per-request isolation. Builtin-ergonomics asks (capture spans,
  span-units, offset-preserving record parsers) fold into a later
  editor-support batch; result-schema conventions stay dun-owned (don't
  make rum an editor framework). Line opens with a diagnostic/inventory
  brief. Memory: `dun-plugin-requirements-line.md`.
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
