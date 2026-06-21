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
  `brief-NNN-<slug>.md` are the packages. Dispatch:

  ```
  cdx exec -C "$(git rev-parse --show-toplevel)" -s danger-full-access \
    "Read docs/dev/codex/brief-NNN-<slug>.md and execute it exactly. Follow its Hard rules." \
    > /tmp/cdx_brief_NNN.log 2>&1
  ```

  `-s danger-full-access` = Codex's sandbox level (needed for `cargo`,
  `rum-cli`, and `sample`); the brief's **Scope + Hard rules** are the real
  fence. My own dispatch/gate commands are allowlisted in
  `.claude/settings.local.json` (`cdx exec *`, the microbench script, the
  sample reducers) so I am not prompted each round.

## The gate I run before crediting any brief

1. **Scope check**: `git status` / `git diff` touched only the brief's
   MAY-modify list. Nothing under any `mma-hol` clone, no `Expr`/`Symbol`/serde
   shape change unless that was the Goal.
2. **Tests**: `cargo fmt --check` and `cargo test -p rum-runtime` (or
   `--workspace`) green.
3. **Performance claim**: I re-run the named microbench label myself and
   confirm the before/after ratio AND that same-tier neighbors did not regress.
   I verify Codex's profiling evidence with `scripts/sample-self-time.py` /
   `scripts/sample-attribute-callers.py` — I trust the numbers I reproduce, not
   its interpretation.
4. Only then do I commit and dispatch the next brief.

## Hard lines I enforce (do not let these drift)

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

- `docs/performance-kernel-rewrite-plan.md` — the bottom-up "fix the primitive
  before the composite" plan (Stages 0-8) and `docs/performance-stage0-baseline.md`.
  The active entry point is `scripts/core-language-microbench.sh` (rum/WK ratio)
  plus the Lane D `core-language` suite (rum-local regression baseline).
- `docs/next-session-handoff.md` — the live task ledger (Task 2d: rum's ~34x
  failed-`REWRCONV` dispatch cost; Task 2e: rum `DumpSave`/`Get` `$Packages`
  restore). **The primitive ladder does NOT capture the Pair ~34x compounding**
  — that is a WL-level conversion-combinator path; see the Stage 4 bridge label.
- `docs/proptaut-performance-audit.md` — the profiling method and the reducer
  scripts.
- The persistent constant is ~3x across primitives; the Pair gate is a separate
  ~34x failed-dispatch path. Do not conflate them.

## Pointers

- `AGENTS.md` — operational first-stop (project snapshot, commands, trust
  boundary, reading path). I read it too, but it is written for any agent.
- `PROGRESS.md` / `TODO.md` — running log and near-term work.
- `~/.claude/.../memory/rum-project-context.md` — my cross-session memory of
  this project's arc and the recurring lessons.
