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
  instead of fixing the per-op cost.
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
  `release-transformation-line.md`. Next mainline candidates: post-cut queue
  in `TODO.md` (coverage rebuild first; dun is the first downstream
  embedder).
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
