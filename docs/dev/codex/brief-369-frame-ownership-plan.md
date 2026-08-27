# brief-369 — how a rooted frame resumes without deep-copying (design only)

## Goal

Step 4 of plan-364 failed on a real tension, not on a brief contradiction.
Design the resolution and produce the step-by-step plan plus drafted
implementation briefs.

**Scope: NONE — no source change.** Reading, grepping, and throwaway probes
under `/tmp` are in scope. Leave nothing in the repository, and feed every
`rum-cli` probe `stdin < /dev/null`.

## What failed, and why it is not a scope problem

brief-368 required two things that currently cannot both hold:

1. The executing frame stays rooted in `Session` for the whole resume call, so
   a sweep during that call can see it. That forces `SuspensionFrame::resume`
   to take `&self` or `self: Arc<Self>` — it cannot consume the frame.
2. No deep clones of bulk captured state.

The codemod satisfied (1) by extracting 390 frames with
`self.as_ref().clone()`, which fails outright for non-`Clone` state and
violates (2) categorically — the inventory has roughly 230 `Vec<Expr>`
captures plus maps and other bulk containers. That mechanism was tried twice
and stopped correctly under the brief's stop-loss rule. Its output is
discarded; the tree is back at `7e598612`.

The tension is real: continuations resume into functions that take **owned**
state, while a rooted frame can only lend `&self`.

## Two facts the design should start from

**Fact 1 — the bulk data is already `Arc`-backed upstream, and gets degraded
at the builtin boundary.** `Expr::Call { head: SharedExpr, args: SharedArgs }`
(`crates/rum-core/src/expr.rs:752`), and `SharedArgs` is `Arc<Vec<Expr>>`
(`:638`) whose `Clone` is an `Arc` bump. But builtins receive `args: &[Expr]`,
so a capture like `let call_args = args.to_vec();`
(`crates/rum-stdlib/src/associations/mutation.rs:184`) allocates a fresh `Vec`
from a slice whose data was already shared.

**Fact 2 — that copy is paid on every call, not only when suspending.** A
closure captures at construction, and the closure is built at the call site
before the `Ok` / `Err(InputRequired)` split (`suspension/core.rs:64-72`). So
the current code deep-copies argument lists on the ordinary path too. Confirm
this with a counter probe rather than taking my word for it: `SharedArgs`
records `record_into_vec(cloned)` and `record_shared_args_alloc()` behind the
env-gated `repr_profile_enabled()` path (`crates/rum-core/src/profiling.rs:41`),
with `INTO_VEC_TOTAL` / `INTO_VEC_CLONE` counters. **Quantify the current cost
before designing anything** — if it is negligible, say so; that changes the
argument.

If Fact 2 holds, this line is not only closing a correctness defect; the
cutover can remove work from the ordinary path. That is a finding worth
stating precisely, with counter numbers.

## What the plan must contain

1. **The ownership resolution, argued.** How does a `&self` frame hand owned
   state to a continuation without a deep copy? Give each candidate a real
   hearing and recommend one, with the cost of each on the ordinary
   (non-suspending) path:
   - Arc-backed frame fields (`SharedArgs`, `Arc<[T]>`, `Arc<Map>`) so the
     clone is a refcount bump, with downstream signatures changed to match;
   - downstream functions changed to borrow (`&[Expr]`) instead of owning;
   - `self: Arc<Self>` plus `Arc::try_unwrap`, and whether rooting makes that
     always fail;
   - anything else you find. Interior mutability behind a `Mutex` is
     **rejected up front**: a lock on the default path violates a standing
     hard line.

2. **Frame construction confined to the suspending arm.** Fact 2 says the
   current API forces eager capture. What API shape lets the `Ok` arm call the
   continuation with zero frame, zero `Arc`, zero allocation, while the
   `Err(InputRequired)` arm builds the named frame? plan-364 sketched a
   declarative `resume_or_suspend!` macro that expands to a match and cannot
   store a Rust closure. Assess that concretely — including what it does to
   readability across 399 sites — and say whether an explicit match at each
   site is better.

3. **The downstream signature inventory.** Every function a continuation
   resumes into that takes owned bulk state, with `path:line` and its
   parameter types. This is the number that decides how the work splits, so
   ground it in a grep, not an estimate.

4. **Whether this splits into landable steps.** brief-368 said the cutover
   must be atomic because splitting runtime from stdlib would leave a legacy
   opaque path alive. Does an ownership-first step avoid that? A step that
   only changes captured state and downstream signatures introduces no second
   suspension path, so I believe it can land on its own — **verify or refute
   that**, and if it holds, say exactly where the seam is.

5. **The file-size consequence.** The failed run pushed seven files past the
   35k architecture-debt line, `pattern_matching/sequence.rs` reaching 53,626
   bytes from 21,841. Say what the named-frame step does to file sizes under
   your design and where the `resume_frames.rs` siblings go, so this is
   planned rather than discovered.

6. **How the ordinary path is proved unchanged.** Name the counters and labels
   that settle it. `INTO_VEC_CLONE` and `shared_expr_alloc` are load-immune
   and beat timings; this project's rule is to report counters first and not
   to quote a time when the counters do not move.

7. **A step-by-step plan**, smallest safe steps first, each naming
   files/functions, the invariant it preserves, its gate tests, and its ripple
   inventory.

8. **Drafted follow-up implementation briefs** in `TEMPLATE.md` shape.

9. **Open questions for the director**, each with your recommendation.

## Scope

- Files you MAY modify: **NONE**. Design only. Probes go in `/tmp`.
- Files/areas you MUST NOT touch: everything, including `AGENTS.md`,
  `PROGRESS.md`, `TODO.md`, `docs/**`, `.git`, `Cargo.toml`, `Cargo.lock`.

## rum / Rust pitfalls (read twice)

1. **Quantify before designing.** Fact 2 is my reading of the code, not a
   measurement. If the counters say the ordinary-path copy is negligible, the
   argument for an ownership-first step weakens and you should say so.
2. **Every probe needs a liveness assertion** — a number that proves the code
   path ran, not merely an absence of error.
3. **No deferred sweep, no pinning, no `Mutex` on the default path.** All
   three were considered and rejected earlier in this line; do not
   re-propose them as the ownership answer.
4. **Do not design a second suspension mechanism.** One path, one traversal.
5. **Size is not an argument** (user ruling 2026-08-27). A design that touches
   400 call sites is not worse than one that touches 40 for that reason. What
   counts: correctness, semantic loss, whether it leaves an open-ended
   obligation, enforceability, measurability.
6. **Stop-loss is real.** If the ownership question does not resolve after two
   serious attempts, report what you established and where the trail ends.

## Verification (MANDATORY)

No source change, so no test gate. The evidence IS the deliverable:

- Counter numbers for Fact 2, with the exact commands and the env you set.
- The downstream signature inventory, grep-derived, with counts.
- Verbatim `path:line` source for every mechanism you name.
- For each ownership candidate, the concrete reason it works or fails.

## Hard rules

- Do NOT `git commit`, branch, push, or modify any file in the repository.
- Touch nothing outside this repo except `/tmp`; no network.
- **Testing Policy is a hard line** (`AGENTS.md`): rum's own behavior only.
- No new `unsafe` in any drafted plan; no new dependencies.
- Bounded probes: `scripts/run-bounded.py --timeout SECONDS -- <command>`;
  state the bound and where you derived it.

## Report format (your final message)

1. "Design only, no source change."
2. The Fact 2 measurement — counters, commands, verdict.
3. The ownership resolution — candidates, costs, recommendation.
4. The API shape for confining frame construction to the suspending arm.
5. The downstream inventory and whether the work splits, with the seam.
6. The plan, and the drafted briefs.
7. Open questions with recommendations, and any stop-loss.
