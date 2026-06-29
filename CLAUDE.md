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

- **ACTIVE LINE (2026-06-23): the correctness mainline** — drive the unmodified
  `mma-hol-acceptance` (`200f13a`) load to "the bottom" (last theorem verifies,
  or a hard time/space wall). Read `docs/next-session-handoff.md`'s **`2026-06-23`
  top block** for the live state: the full ℤ→ℚ→ℝ tower + real construction loads
  (88%, 36/41); frontier is `stdlib/Real/Seq.wl` (multi-layered; `Nothing` fixed,
  next is a `mkComb` type mismatch). Driver: `scripts/snapshot-checkpoint-chain.sh`;
  checkpoints in `rum-snapshot-checkpoints-v5/`. Method: rebuild fixes from the
  checkpoint just before the earliest tainted file; patch-and-trace
  `HOL`Auto`RealArith` functions on a restored checkpoint.
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
