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
- Codex's recurring failure modes to watch in review: optimizing sub-second
  hotspots while the parent workload is elsewhere; fixing at the wrong layer;
  masking a symptom (reducing an operation count to hide a per-op slowness)
  instead of fixing the per-op cost; **fabricated evidence citations** —
  brief-208 cited "Evidence E15/E16" in twelve findings rows when the appendix
  ended at E14 and the claimed behavior was false (its own E4 said the
  opposite). A confident report with a citation number is not evidence.
  **Check that every cited evidence section EXISTS, and reproduce the
  load-bearing claims yourself** — especially the ones a fix batch is scoped
  around. The gate is where fabrication dies; downstream it becomes code.
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
  `docs/dev/usability-pass.md`; memory `usability-pass-line.md`. **Units
  1-7 CLOSED** (strings, structure+functional, predicates+associations,
  control+logic+rewrite, math, codecs+bytes+datetime, character_classes+
  random — briefs 161-207, 446 builtins swept, all fixes compat; reference
  612). Unit 7 (203 audit + 204-207): both FRESH arcs held (157-159
  RandomPassword, 200-202 separation); `Protected` on all 11 random heads,
  nested `{dims}` for RandomInteger/RandomChoice, weighted RandomChoice
  (exact rational, no floats), WhitespaceCharacters constant + RandomPassword
  option discovery. Deferrals: RandomString `{dims}`/`[]` (WL surface
  uncertain), string-keyed option-discovery wart (cross-cutting → the dun
  host line), `Dimensions` absent (later structure touch). **NEXT: unit 8**
  (kernel A — definitions/values/attributes/protection, ~50), then units
  9-10, THEN the dun host-embedding API line (user sequencing ruling
  2026-07-08). Standing user directives: high coverage incl. the
  WL-inherited surface; compat enhancement only (default behavior stays
  WL-conformant, new powers via options / rum-native siblings). Standing
  design rulings: arithmetic singularities stay INERT (no special-value
  arithmetic — user); Orderless/Flat/OneIdentity on folk numeric reducers +
  Equivalent + the Implies short-circuit all REJECTED; ConformSchema/
  SchemaErrors operator forms DEFERRED (HoldRest hold-semantics).
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
